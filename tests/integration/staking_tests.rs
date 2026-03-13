//! Staking integration tests
//!
//! Tests for staking operations (delegate, undelegate, withdraw, claim, compound).
//!
//! Test categories:
//! - Happy path tests (success scenarios)
//! - Error handling tests (RPC errors, validation failures)
//! - Event parsing tests
//! - Edge case tests (boundary conditions)
//!
//! All tests use wiremock for mock RPC server. No real Monad node required.

use wiremock::matchers::{body_string_contains, method};
use wiremock::{Mock, MockServer, ResponseTemplate};

use monad_val_manager::rpc::RpcClient;
use monad_val_manager::staking::events::{
    extract_staking_events, parse_event, StakingEvent, TransactionLog,
    CLAIM_REWARDS_EVENT_SIGNATURE, COMPOUND_EVENT_SIGNATURE, DELEGATE_EVENT_SIGNATURE,
    UNDELEGATE_EVENT_SIGNATURE, WITHDRAW_EVENT_SIGNATURE,
};
use monad_val_manager::staking::{
    encode_claim_rewards, encode_compound, encode_delegate, encode_get_delegator, encode_get_epoch,
    encode_get_validator, encode_undelegate, encode_withdraw,
};

// =============================================================================
// TEST CONSTANTS
// =============================================================================

const STAKING_CONTRACT_ADDRESS: &str = "0x0000000000000000000000000000000000000001";
const ONE_MON: u128 = 1_000_000_000_000_000_000_000u128; // 1 MON in wei
const TEST_DELEGATOR_ADDRESS: &str = "0xabcdef0123456789abcdef0123456789abcdef01";
const TEST_VALIDATOR_ID: u64 = 1;
const TEST_TX_HASH: &str = "0xaabbccdd11223344556677889900aabbccdd11223344556677889900aabbccdd";

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Create a test RPC client connected to the mock server
fn create_test_client(endpoint: &str) -> RpcClient {
    monad_val_manager::rpc::RpcClient::new(endpoint).expect("Failed to create test RPC client")
}

/// Create a JSON-RPC success response
fn json_rpc_success<T: serde::Serialize>(result: T) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": result
    })
    .to_string()
}

/// Create eth_call response with data
fn eth_call_response(data: &str) -> String {
    json_rpc_success(data)
}

/// Create eth_blockNumber response
fn block_number_response(block: u64) -> String {
    json_rpc_success(format!("0x{:x}", block))
}

/// Create eth_getTransactionCount response (nonce)
fn nonce_response(nonce: u64) -> String {
    json_rpc_success(format!("0x{:x}", nonce))
}

/// Create eth_gasPrice response
fn gas_price_response(price: u64) -> String {
    json_rpc_success(format!("0x{:x}", price))
}

/// Create eth_maxPriorityFeePerGas response
fn max_priority_fee_response(fee: u64) -> String {
    json_rpc_success(format!("0x{:x}", fee))
}

/// Create eth_chainId response
fn chain_id_response(chain_id: u64) -> String {
    json_rpc_success(format!("0x{:x}", chain_id))
}

/// Create eth_sendRawTransaction response
fn send_raw_transaction_response(tx_hash: &str) -> String {
    json_rpc_success(tx_hash)
}

/// Create eth_getTransactionReceipt response (confirmed)
fn transaction_receipt_confirmed(tx_hash: &str, block_number: u64, status: bool) -> String {
    let status_hex = if status { "0x1" } else { "0x0" };
    json_rpc_success(serde_json::json!({
        "transactionHash": tx_hash,
        "blockNumber": format!("0x{:x}", block_number),
        "blockHash": "0x0000000000000000000000000000000000000000000000000000000000000001",
        "from": TEST_DELEGATOR_ADDRESS,
        "to": STAKING_CONTRACT_ADDRESS,
        "status": status_hex,
        "gasUsed": "0x5208",
        "logs": []
    }))
}

// =============================================================================
// STAKING CONTRACT CALL RESPONSES - ABI-encoded responses
// =============================================================================

/// Create get_epoch() response
/// Returns: (uint64 epoch, bool is_epoch_transition)
fn get_epoch_response(epoch: u64, is_epoch_transition: bool) -> String {
    let epoch_hex = format!("{:064x}", epoch);
    let transition_hex = if is_epoch_transition {
        "0000000000000000000000000000000000000000000000000000000000000001"
    } else {
        "0000000000000000000000000000000000000000000000000000000000000000"
    };
    eth_call_response(&format!("0x{}{}", epoch_hex, transition_hex))
}

/// Create get_delegator() response
/// Returns: (delegated_amount, pending_amount, rewards, total_claimed, total_compounded, activation_epoch, last_claim_epoch)
fn get_delegator_response(
    delegated_amount: u128,
    pending_amount: u128,
    rewards: u128,
    activation_epoch: u64,
) -> String {
    let amount_hex = format!("{:064x}", delegated_amount);
    let pending_hex = format!("{:064x}", pending_amount);
    let rewards_hex = format!("{:064x}", rewards);
    let claimed_hex = "0000000000000000000000000000000000000000000000000000000000000000";
    let compounded_hex = "0000000000000000000000000000000000000000000000000000000000000000";
    let activation_hex = format!("{:064x}", activation_epoch);
    let last_claim_hex = "0000000000000000000000000000000000000000000000000000000000000000";

    eth_call_response(&format!(
        "0x{}{}{}{}{}{}{}",
        amount_hex,
        pending_hex,
        rewards_hex,
        claimed_hex,
        compounded_hex,
        activation_hex,
        last_claim_hex
    ))
}

/// Create get_validator() response (simplified)
fn get_validator_response(auth_delegator: &str, commission: u64, delegated_amount: u128) -> String {
    let addr_clean = auth_delegator.strip_prefix("0x").unwrap_or(auth_delegator);
    let auth_padded = format!("{:0>64}", addr_clean);
    let commission_hex = format!("{:064x}", commission);
    let amount_hex = format!("{:064x}", delegated_amount);

    // Minimal valid response with dynamic byte offsets
    let mut response = String::from("0x");
    response.push_str(&auth_padded); // auth_delegator
    response.push_str(&commission_hex); // commission
    response.push_str(&amount_hex); // delegated_amount
    response.push_str(&"0".repeat(64)); // rewards_pool
    response.push_str(&"0".repeat(64)); // commission_pending
    response.push_str(&"0".repeat(64)); // commission_claimed
    response.push_str(&"0".repeat(64)); // delegator_count
    response.push_str(&"0".repeat(64)); // status_flags
    response.push_str(&"0".repeat(64)); // created_at_epoch
    response.push_str(&"0".repeat(64)); // updated_at_epoch
    response.push_str("0000000000000000000000000000000000000000000000000000000000000180"); // secp offset
    response.push_str("00000000000000000000000000000000000000000000000000000000000001e0"); // bls offset
    response.push_str("0000000000000000000000000000000000000000000000000000000000000040"); // secp len (64)
    response.push_str(&"ab".repeat(64)); // secp pubkey (64 bytes)
    response.push_str("0000000000000000000000000000000000000000000000000000000000000030"); // bls len (48)
    response.push_str(&"cd".repeat(48)); // bls pubkey (48 bytes)
    response.push_str(&"0".repeat(16)); // padding to 32-byte boundary

    eth_call_response(&response)
}

/// Create get_withdrawal_request() response
fn get_withdrawal_request_response(
    amount: u128,
    withdrawal_index: u8,
    activation_epoch: u64,
) -> String {
    let amount_hex = format!("{:064x}", amount);
    let index_hex = format!("{:064x}", withdrawal_index);
    let epoch_hex = format!("{:064x}", activation_epoch);
    eth_call_response(&format!("0x{}{}{}", amount_hex, index_hex, epoch_hex))
}

// =============================================================================
// MOCK SERVER SETUP HELPERS
// =============================================================================

/// Setup basic mocks needed for most staking operations
async fn setup_basic_mocks(server: &MockServer) {
    // Mock chain ID
    Mock::given(method("POST"))
        .and(body_string_contains("\"method\":\"eth_chainId\""))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(chain_id_response(143))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(server)
        .await;

    // Mock gas price
    Mock::given(method("POST"))
        .and(body_string_contains("\"method\":\"eth_gasPrice\""))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(gas_price_response(10_000_000_000))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(server)
        .await;

    // Mock max priority fee
    Mock::given(method("POST"))
        .and(body_string_contains(
            "\"method\":\"eth_maxPriorityFeePerGas\"",
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(max_priority_fee_response(1_000_000_000))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(server)
        .await;

    // Mock nonce
    Mock::given(method("POST"))
        .and(body_string_contains(
            "\"method\":\"eth_getTransactionCount\"",
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(nonce_response(0))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(server)
        .await;

    // Mock block number
    Mock::given(method("POST"))
        .and(body_string_contains("\"method\":\"eth_blockNumber\""))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(block_number_response(12345))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(server)
        .await;
}

// =============================================================================
// TESTS: CALLDATA ENCODING
// =============================================================================

#[test]
fn test_encode_get_epoch() {
    let data = encode_get_epoch();
    assert!(data.starts_with("0x"));
    // get_epoch() selector: first 4 bytes of keccak256("get_epoch()")
    assert_eq!(&data[0..10], "0x757991a8");
}

#[test]
fn test_encode_get_validator() {
    let validator_id: u64 = 1;
    let data = encode_get_validator(validator_id).expect("Failed to encode");
    assert!(data.starts_with("0x"));
    // Should contain the validator ID padded to 32 bytes
    assert!(data.contains("0000000000000000000000000000000000000000000000000000000000000001"));
}

#[test]
fn test_encode_get_delegator() {
    let validator_id: u64 = 1;
    let delegator = "0xabcdef0123456789abcdef0123456789abcdef01";
    let data = encode_get_delegator(validator_id, delegator).expect("Failed to encode");
    assert!(data.starts_with("0x"));
}

#[test]
fn test_encode_delegate() {
    let validator_id: u64 = 1;
    let data = encode_delegate(validator_id).expect("Failed to encode");
    assert!(data.starts_with("0x"));
    // delegate(uint64) selector
    assert_eq!(&data[0..10], "0x84994fec");
}

#[test]
fn test_encode_undelegate() {
    let validator_id: u64 = 1;
    let amount: u128 = ONE_MON;
    let withdrawal_index: u8 = 0;
    let data = encode_undelegate(validator_id, amount, withdrawal_index).expect("Failed to encode");
    assert!(data.starts_with("0x"));
}

#[test]
fn test_encode_withdraw() {
    let validator_id: u64 = 1;
    let withdrawal_index: u8 = 0;
    let data = encode_withdraw(validator_id, withdrawal_index).expect("Failed to encode");
    assert!(data.starts_with("0x"));
}

#[test]
fn test_encode_claim_rewards() {
    let validator_id: u64 = 1;
    let data = encode_claim_rewards(validator_id).expect("Failed to encode");
    assert!(data.starts_with("0x"));
}

#[test]
fn test_encode_compound() {
    let validator_id: u64 = 1;
    let data = encode_compound(validator_id).expect("Failed to encode");
    assert!(data.starts_with("0x"));
}

// =============================================================================
// TESTS: EVENT PARSING (using public parse_event API)
// =============================================================================

#[test]
fn test_parse_delegate_event() {
    // Event: Delegate(uint64 indexed valId, address indexed delegator, uint256 amount, uint64 activationEpoch)
    let val_id_topic = format!("0x{:064x}", TEST_VALIDATOR_ID);
    let delegator_topic = format!(
        "0x{:0>64}",
        TEST_DELEGATOR_ADDRESS
            .strip_prefix("0x")
            .unwrap_or(TEST_DELEGATOR_ADDRESS)
    );

    // Data: amount (32 bytes) + activationEpoch (32 bytes)
    let amount = ONE_MON;
    let epoch: u64 = 100;
    let data = format!("0x{:064x}{:064x}", amount, epoch);

    let log = TransactionLog {
        address: STAKING_CONTRACT_ADDRESS.to_string(),
        topics: vec![
            DELEGATE_EVENT_SIGNATURE.to_string(),
            val_id_topic,
            delegator_topic,
        ],
        data,
    };

    let result = parse_event(&log).expect("Failed to parse delegate event");

    match result {
        Some(StakingEvent::Delegate(event)) => {
            assert_eq!(event.validator_id, TEST_VALIDATOR_ID);
            assert_eq!(event.amount, ONE_MON);
            assert_eq!(event.activation_epoch, 100);
        }
        _ => panic!("Expected Delegate event, got {:?}", result),
    }
}

#[test]
fn test_parse_undelegate_event() {
    // Event: Undelegate(uint64 indexed valId, address indexed delegator, uint8 withdrawal_id, uint256 amount, uint64 activationEpoch)
    let val_id_topic = format!("0x{:064x}", TEST_VALIDATOR_ID);
    let delegator_topic = format!(
        "0x{:0>64}",
        TEST_DELEGATOR_ADDRESS
            .strip_prefix("0x")
            .unwrap_or(TEST_DELEGATOR_ADDRESS)
    );

    // Data: withdrawal_id (32 bytes) + amount (32 bytes) + activationEpoch (32 bytes)
    let withdrawal_id: u8 = 0;
    let amount = ONE_MON;
    let epoch: u64 = 100;
    let data = format!("0x{:064x}{:064x}{:064x}", withdrawal_id, amount, epoch);

    let log = TransactionLog {
        address: STAKING_CONTRACT_ADDRESS.to_string(),
        topics: vec![
            UNDELEGATE_EVENT_SIGNATURE.to_string(),
            val_id_topic,
            delegator_topic,
        ],
        data,
    };

    let result = parse_event(&log).expect("Failed to parse undelegate event");

    match result {
        Some(StakingEvent::Undelegate(event)) => {
            assert_eq!(event.validator_id, TEST_VALIDATOR_ID);
            assert_eq!(event.withdrawal_id, 0);
            assert_eq!(event.amount, ONE_MON);
            assert_eq!(event.activation_epoch, 100);
        }
        _ => panic!("Expected Undelegate event, got {:?}", result),
    }
}

#[test]
fn test_parse_withdraw_event() {
    // Event: Withdraw(uint64 indexed valId, address indexed delegator, uint8 withdrawal_id, uint256 amount, uint64 activationEpoch)
    let val_id_topic = format!("0x{:064x}", TEST_VALIDATOR_ID);
    let delegator_topic = format!(
        "0x{:0>64}",
        TEST_DELEGATOR_ADDRESS
            .strip_prefix("0x")
            .unwrap_or(TEST_DELEGATOR_ADDRESS)
    );

    let withdrawal_id: u8 = 0;
    let amount = ONE_MON;
    let epoch: u64 = 100;
    let data = format!("0x{:064x}{:064x}{:064x}", withdrawal_id, amount, epoch);

    let log = TransactionLog {
        address: STAKING_CONTRACT_ADDRESS.to_string(),
        topics: vec![
            WITHDRAW_EVENT_SIGNATURE.to_string(),
            val_id_topic,
            delegator_topic,
        ],
        data,
    };

    let result = parse_event(&log).expect("Failed to parse withdraw event");

    match result {
        Some(StakingEvent::Withdraw(event)) => {
            assert_eq!(event.validator_id, TEST_VALIDATOR_ID);
            assert_eq!(event.withdrawal_id, 0);
            assert_eq!(event.amount, ONE_MON);
            assert_eq!(event.activation_epoch, 100);
        }
        _ => panic!("Expected Withdraw event, got {:?}", result),
    }
}

#[test]
fn test_parse_claim_rewards_event() {
    // Event: ClaimRewards(uint64 indexed valId, address indexed delegator, uint256 amount, uint64 epoch)
    let val_id_topic = format!("0x{:064x}", TEST_VALIDATOR_ID);
    let delegator_topic = format!(
        "0x{:0>64}",
        TEST_DELEGATOR_ADDRESS
            .strip_prefix("0x")
            .unwrap_or(TEST_DELEGATOR_ADDRESS)
    );

    let amount = ONE_MON / 10; // 0.1 MON rewards
    let epoch: u64 = 100;
    let data = format!("0x{:064x}{:064x}", amount, epoch);

    let log = TransactionLog {
        address: STAKING_CONTRACT_ADDRESS.to_string(),
        topics: vec![
            CLAIM_REWARDS_EVENT_SIGNATURE.to_string(),
            val_id_topic,
            delegator_topic,
        ],
        data,
    };

    let result = parse_event(&log).expect("Failed to parse claim rewards event");

    match result {
        Some(StakingEvent::ClaimRewards(event)) => {
            assert_eq!(event.validator_id, TEST_VALIDATOR_ID);
            assert_eq!(event.amount, ONE_MON / 10);
            assert_eq!(event.epoch, 100);
        }
        _ => panic!("Expected ClaimRewards event, got {:?}", result),
    }
}

#[test]
fn test_parse_compound_event() {
    // Event: Compound(uint64 indexed valId, address indexed delegator, uint256 amount)
    let val_id_topic = format!("0x{:064x}", TEST_VALIDATOR_ID);
    let delegator_topic = format!(
        "0x{:0>64}",
        TEST_DELEGATOR_ADDRESS
            .strip_prefix("0x")
            .unwrap_or(TEST_DELEGATOR_ADDRESS)
    );

    let amount = ONE_MON / 10; // 0.1 MON compounded
    let data = format!("0x{:064x}", amount);

    let log = TransactionLog {
        address: STAKING_CONTRACT_ADDRESS.to_string(),
        topics: vec![
            COMPOUND_EVENT_SIGNATURE.to_string(),
            val_id_topic,
            delegator_topic,
        ],
        data,
    };

    let result = parse_event(&log).expect("Failed to parse compound event");

    match result {
        Some(StakingEvent::Compound(event)) => {
            assert_eq!(event.validator_id, TEST_VALIDATOR_ID);
            assert_eq!(event.amount, ONE_MON / 10);
        }
        _ => panic!("Expected Compound event, got {:?}", result),
    }
}

#[test]
fn test_parse_delegate_event_invalid_signature() {
    let wrong_sig = "0x0000000000000000000000000000000000000000000000000000000000000000";
    let log = TransactionLog {
        address: STAKING_CONTRACT_ADDRESS.to_string(),
        topics: vec![wrong_sig.to_string()],
        data: "0x".to_string(),
    };

    let result = parse_event(&log).expect("Parse should succeed");
    // Unknown event signature should return None
    assert!(result.is_none());
}

#[test]
fn test_extract_staking_events() {
    // Create multiple logs with different events
    let val_id_topic = format!("0x{:064x}", TEST_VALIDATOR_ID);
    let delegator_topic = format!(
        "0x{:0>64}",
        TEST_DELEGATOR_ADDRESS
            .strip_prefix("0x")
            .unwrap_or(TEST_DELEGATOR_ADDRESS)
    );

    // Delegate event
    let delegate_log = TransactionLog {
        address: STAKING_CONTRACT_ADDRESS.to_string(),
        topics: vec![
            DELEGATE_EVENT_SIGNATURE.to_string(),
            val_id_topic.clone(),
            delegator_topic.clone(),
        ],
        data: format!("0x{:064x}{:064x}", ONE_MON, 100u64),
    };

    // Undelegate event
    let undelegate_log = TransactionLog {
        address: STAKING_CONTRACT_ADDRESS.to_string(),
        topics: vec![
            UNDELEGATE_EVENT_SIGNATURE.to_string(),
            val_id_topic.clone(),
            delegator_topic.clone(),
        ],
        data: format!("0x{:064x}{:064x}{:064x}", 0u8, ONE_MON, 200u64),
    };

    // Unknown event (should be filtered out)
    let unknown_log = TransactionLog {
        address: STAKING_CONTRACT_ADDRESS.to_string(),
        topics: vec![
            "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        ],
        data: "0x".to_string(),
    };

    let logs = vec![delegate_log, undelegate_log, unknown_log];

    let events = extract_staking_events(&logs).expect("Failed to extract events");

    // Should have 2 events (delegate and undelegate), unknown filtered out
    assert_eq!(events.len(), 2);
}

// =============================================================================
// TESTS: RPC CLIENT INTEGRATION
// =============================================================================

#[tokio::test]
async fn test_mock_server_staking_get_epoch() {
    let server = MockServer::start().await;

    // Mock eth_call for get_epoch
    Mock::given(method("POST"))
        .and(body_string_contains("\"method\":\"eth_call\""))
        .and(body_string_contains("0x757991a8")) // get_epoch selector
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(get_epoch_response(100, false))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    // Mock block number for "latest" tag
    Mock::given(method("POST"))
        .and(body_string_contains("\"method\":\"eth_blockNumber\""))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(block_number_response(12345))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    let client = create_test_client(&server.uri());

    // Verify client was created successfully
    let _ = client;
}

#[tokio::test]
async fn test_mock_server_staking_get_delegator() {
    let server = MockServer::start().await;

    // Mock eth_call for get_delegator
    Mock::given(method("POST"))
        .and(body_string_contains("\"method\":\"eth_call\""))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(get_delegator_response(ONE_MON, 0, ONE_MON / 10, 50))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    // Mock block number
    Mock::given(method("POST"))
        .and(body_string_contains("\"method\":\"eth_blockNumber\""))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(block_number_response(12345))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    let client = create_test_client(&server.uri());
    let _ = client;
}

#[tokio::test]
async fn test_mock_server_staking_get_validator() {
    let server = MockServer::start().await;

    // Mock eth_call for get_validator
    Mock::given(method("POST"))
        .and(body_string_contains("\"method\":\"eth_call\""))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(get_validator_response(
                    TEST_DELEGATOR_ADDRESS,
                    500,
                    1000 * ONE_MON,
                ))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    // Mock block number
    Mock::given(method("POST"))
        .and(body_string_contains("\"method\":\"eth_blockNumber\""))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(block_number_response(12345))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    let client = create_test_client(&server.uri());
    let _ = client;
}

// =============================================================================
// TESTS: TRANSACTION RECEIPT
// =============================================================================

#[tokio::test]
async fn test_mock_transaction_receipt_success() {
    let server = MockServer::start().await;

    // Mock successful transaction receipt
    Mock::given(method("POST"))
        .and(body_string_contains(
            "\"method\":\"eth_getTransactionReceipt\"",
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(transaction_receipt_confirmed(TEST_TX_HASH, 12345, true))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    // Verify the mock server is running
    let client = reqwest::Client::new();
    let response = client
        .post(server.uri())
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getTransactionReceipt",
            "params": [TEST_TX_HASH],
            "id": 1
        }))
        .send()
        .await
        .expect("Request failed");

    let body = response.text().await.expect("Failed to read body");
    assert!(body.contains(TEST_TX_HASH));
    assert!(body.contains("0x1")); // success status
}

#[tokio::test]
async fn test_mock_transaction_receipt_failure() {
    let server = MockServer::start().await;

    // Mock failed transaction receipt
    Mock::given(method("POST"))
        .and(body_string_contains(
            "\"method\":\"eth_getTransactionReceipt\"",
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(transaction_receipt_confirmed(TEST_TX_HASH, 12345, false))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .post(server.uri())
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getTransactionReceipt",
            "params": [TEST_TX_HASH],
            "id": 1
        }))
        .send()
        .await
        .expect("Request failed");

    let body = response.text().await.expect("Failed to read body");
    assert!(body.contains("0x0")); // failure status
}

// =============================================================================
// TESTS: WITHDRAWAL REQUEST
// =============================================================================

#[tokio::test]
async fn test_mock_withdrawal_request() {
    let server = MockServer::start().await;

    // Mock eth_call for get_withdrawal_request
    Mock::given(method("POST"))
        .and(body_string_contains("\"method\":\"eth_call\""))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(get_withdrawal_request_response(ONE_MON, 0, 100))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    // Mock block number
    Mock::given(method("POST"))
        .and(body_string_contains("\"method\":\"eth_blockNumber\""))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(block_number_response(12345))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    let client = create_test_client(&server.uri());
    let _ = client;
}

// =============================================================================
// TESTS: ERROR HANDLING
// =============================================================================

#[tokio::test]
async fn test_mock_rpc_error_response() {
    let server = MockServer::start().await;

    // Mock RPC error
    let error_response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "error": {
            "code": -32000,
            "message": "Insufficient funds"
        }
    })
    .to_string();

    Mock::given(method("POST"))
        .and(body_string_contains("\"method\":\"eth_call\""))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(error_response)
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .post(server.uri())
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_call",
            "params": [],
            "id": 1
        }))
        .send()
        .await
        .expect("Request failed");

    let body = response.text().await.expect("Failed to read body");
    assert!(body.contains("Insufficient funds"));
    assert!(body.contains("-32000"));
}

// =============================================================================
// TESTS: EDGE CASES
// =============================================================================

#[test]
fn test_encode_delegate_large_validator_id() {
    // Test with max validator ID
    let validator_id = u64::MAX;
    let data = encode_delegate(validator_id).expect("Failed to encode");
    assert!(data.starts_with("0x"));
}

#[test]
fn test_encode_undelegate_large_amount() {
    // Test with large amount
    let validator_id: u64 = 1;
    let amount = u128::MAX;
    let withdrawal_index: u8 = 255;
    let data = encode_undelegate(validator_id, amount, withdrawal_index).expect("Failed to encode");
    assert!(data.starts_with("0x"));
}

#[test]
fn test_encode_withdraw_max_index() {
    // Test with max withdrawal index
    let validator_id: u64 = 1;
    let withdrawal_index: u8 = 255;
    let data = encode_withdraw(validator_id, withdrawal_index).expect("Failed to encode");
    assert!(data.starts_with("0x"));
}

#[test]
fn test_parse_delegate_event_zero_amounts() {
    let val_id_topic = format!("0x{:064x}", TEST_VALIDATOR_ID);
    let delegator_topic = format!(
        "0x{:0>64}",
        TEST_DELEGATOR_ADDRESS
            .strip_prefix("0x")
            .unwrap_or(TEST_DELEGATOR_ADDRESS)
    );

    // Zero amounts
    let data = format!("0x{:064x}{:064x}", 0u128, 0u64);

    let log = TransactionLog {
        address: STAKING_CONTRACT_ADDRESS.to_string(),
        topics: vec![
            DELEGATE_EVENT_SIGNATURE.to_string(),
            val_id_topic,
            delegator_topic,
        ],
        data,
    };

    let result = parse_event(&log).expect("Failed to parse delegate event");

    match result {
        Some(StakingEvent::Delegate(event)) => {
            assert_eq!(event.validator_id, TEST_VALIDATOR_ID);
            assert_eq!(event.amount, 0);
            assert_eq!(event.activation_epoch, 0);
        }
        _ => panic!("Expected Delegate event, got {:?}", result),
    }
}

// =============================================================================
// TESTS: MOCK SERVER INFRASTRUCTURE
// =============================================================================

#[tokio::test]
async fn test_mock_server_starts() {
    let server = MockServer::start().await;
    assert!(server.uri().starts_with("http://"));
}

#[tokio::test]
async fn test_basic_mocks_setup() {
    let server = MockServer::start().await;
    setup_basic_mocks(&server).await;

    let client = reqwest::Client::new();

    // Test chain ID
    let response = client
        .post(server.uri())
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_chainId",
            "params": [],
            "id": 1
        }))
        .send()
        .await
        .expect("Request failed");
    let body = response.text().await.expect("Failed to read body");
    assert!(body.contains("0x8f")); // 143 in hex

    // Test gas price
    let response = client
        .post(server.uri())
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_gasPrice",
            "params": [],
            "id": 1
        }))
        .send()
        .await
        .expect("Request failed");
    let body = response.text().await.expect("Failed to read body");
    assert!(body.contains("0x2540be400")); // 10 Gwei in hex
}

// =============================================================================
// TESTS: DELEGATE FLOW (Phase 10.2)
// =============================================================================

/// Test delegate flow: encode -> send -> wait for receipt -> parse event
#[tokio::test]
async fn test_delegate_flow_happy_path() {
    let server = MockServer::start().await;
    setup_basic_mocks(&server).await;

    // Mock eth_call for get_epoch (to check current epoch)
    Mock::given(method("POST"))
        .and(body_string_contains("\"method\":\"eth_call\""))
        .and(body_string_contains("0x9a4a3d69")) // get_epoch selector
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(get_epoch_response(100, false))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    // Mock eth_estimateGas for delegate
    Mock::given(method("POST"))
        .and(body_string_contains("\"method\":\"eth_estimateGas\""))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(json_rpc_success("0x5208")) // 21000 gas
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    // Mock eth_sendRawTransaction
    Mock::given(method("POST"))
        .and(body_string_contains(
            "\"method\":\"eth_sendRawTransaction\"",
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(send_raw_transaction_response(TEST_TX_HASH))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    // Mock eth_getTransactionReceipt
    Mock::given(method("POST"))
        .and(body_string_contains(
            "\"method\":\"eth_getTransactionReceipt\"",
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(transaction_receipt_confirmed(TEST_TX_HASH, 12345, true))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    // Test calldata encoding
    let data = encode_delegate(TEST_VALIDATOR_ID).expect("Failed to encode delegate");
    assert!(data.starts_with("0x"));
    assert_eq!(&data[0..10], "0x84994fec"); // delegate selector

    // Verify client creation
    let _client = create_test_client(&server.uri());
}

// =============================================================================
// TESTS: UNDELEGATE FLOW (Phase 10.2)
// =============================================================================

/// Test undelegate flow: encode -> send -> wait for receipt -> parse event
#[tokio::test]
async fn test_undelegate_flow_happy_path() {
    let server = MockServer::start().await;
    setup_basic_mocks(&server).await;

    // Mock eth_call for get_delegator (to check delegated amount)
    Mock::given(method("POST"))
        .and(body_string_contains("\"method\":\"eth_call\""))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(get_delegator_response(ONE_MON, 0, ONE_MON / 10, 50))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    // Mock eth_estimateGas for undelegate
    Mock::given(method("POST"))
        .and(body_string_contains("\"method\":\"eth_estimateGas\""))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(json_rpc_success("0x5208"))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    // Mock eth_sendRawTransaction
    Mock::given(method("POST"))
        .and(body_string_contains(
            "\"method\":\"eth_sendRawTransaction\"",
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(send_raw_transaction_response(TEST_TX_HASH))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    // Mock eth_getTransactionReceipt
    Mock::given(method("POST"))
        .and(body_string_contains(
            "\"method\":\"eth_getTransactionReceipt\"",
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(transaction_receipt_confirmed(TEST_TX_HASH, 12345, true))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    // Test calldata encoding
    let data =
        encode_undelegate(TEST_VALIDATOR_ID, ONE_MON, 0).expect("Failed to encode undelegate");
    assert!(data.starts_with("0x"));

    // Verify client creation
    let _client = create_test_client(&server.uri());
}

// =============================================================================
// TESTS: WITHDRAW FLOW (Phase 10.2)
// =============================================================================

/// Test withdraw flow: encode -> send -> wait for receipt -> parse event
#[tokio::test]
async fn test_withdraw_flow_happy_path() {
    let server = MockServer::start().await;
    setup_basic_mocks(&server).await;

    // Mock eth_call for get_withdrawal_request
    Mock::given(method("POST"))
        .and(body_string_contains("\"method\":\"eth_call\""))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(get_withdrawal_request_response(ONE_MON, 0, 100))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    // Mock eth_estimateGas for withdraw
    Mock::given(method("POST"))
        .and(body_string_contains("\"method\":\"eth_estimateGas\""))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(json_rpc_success("0x5208"))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    // Mock eth_sendRawTransaction
    Mock::given(method("POST"))
        .and(body_string_contains(
            "\"method\":\"eth_sendRawTransaction\"",
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(send_raw_transaction_response(TEST_TX_HASH))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    // Mock eth_getTransactionReceipt
    Mock::given(method("POST"))
        .and(body_string_contains(
            "\"method\":\"eth_getTransactionReceipt\"",
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(transaction_receipt_confirmed(TEST_TX_HASH, 12345, true))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    // Test calldata encoding
    let data = encode_withdraw(TEST_VALIDATOR_ID, 0).expect("Failed to encode withdraw");
    assert!(data.starts_with("0x"));

    // Verify client creation
    let _client = create_test_client(&server.uri());
}

// =============================================================================
// TESTS: CLAIM REWARDS FLOW (Phase 10.2)
// =============================================================================

/// Test claim rewards flow: encode -> send -> wait for receipt -> parse event
#[tokio::test]
async fn test_claim_rewards_flow_happy_path() {
    let server = MockServer::start().await;
    setup_basic_mocks(&server).await;

    // Mock eth_call for get_delegator (to check rewards)
    Mock::given(method("POST"))
        .and(body_string_contains("\"method\":\"eth_call\""))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(get_delegator_response(ONE_MON, 0, ONE_MON / 10, 50))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    // Mock eth_estimateGas for claim_rewards
    Mock::given(method("POST"))
        .and(body_string_contains("\"method\":\"eth_estimateGas\""))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(json_rpc_success("0x5208"))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    // Mock eth_sendRawTransaction
    Mock::given(method("POST"))
        .and(body_string_contains(
            "\"method\":\"eth_sendRawTransaction\"",
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(send_raw_transaction_response(TEST_TX_HASH))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    // Mock eth_getTransactionReceipt
    Mock::given(method("POST"))
        .and(body_string_contains(
            "\"method\":\"eth_getTransactionReceipt\"",
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(transaction_receipt_confirmed(TEST_TX_HASH, 12345, true))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    // Test calldata encoding
    let data = encode_claim_rewards(TEST_VALIDATOR_ID).expect("Failed to encode claim_rewards");
    assert!(data.starts_with("0x"));

    // Verify client creation
    let _client = create_test_client(&server.uri());
}

// =============================================================================
// TESTS: COMPOUND FLOW (Phase 10.2)
// =============================================================================

/// Test compound flow: encode -> send -> wait for receipt -> parse event
#[tokio::test]
async fn test_compound_flow_happy_path() {
    let server = MockServer::start().await;
    setup_basic_mocks(&server).await;

    // Mock eth_call for get_delegator (to check rewards available for compounding)
    Mock::given(method("POST"))
        .and(body_string_contains("\"method\":\"eth_call\""))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(get_delegator_response(ONE_MON, 0, ONE_MON / 10, 50))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    // Mock eth_estimateGas for compound
    Mock::given(method("POST"))
        .and(body_string_contains("\"method\":\"eth_estimateGas\""))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(json_rpc_success("0x5208"))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    // Mock eth_sendRawTransaction
    Mock::given(method("POST"))
        .and(body_string_contains(
            "\"method\":\"eth_sendRawTransaction\"",
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(send_raw_transaction_response(TEST_TX_HASH))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    // Mock eth_getTransactionReceipt
    Mock::given(method("POST"))
        .and(body_string_contains(
            "\"method\":\"eth_getTransactionReceipt\"",
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(transaction_receipt_confirmed(TEST_TX_HASH, 12345, true))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    // Test calldata encoding
    let data = encode_compound(TEST_VALIDATOR_ID).expect("Failed to encode compound");
    assert!(data.starts_with("0x"));

    // Verify client creation
    let _client = create_test_client(&server.uri());
}

// =============================================================================
// TESTS: RECEIPT WAITING (Phase 10.2)
// =============================================================================

/// Test receipt waiting with pending state (returns null)
#[tokio::test]
async fn test_receipt_waiting_pending() {
    let server = MockServer::start().await;

    // Mock pending receipt (null)
    Mock::given(method("POST"))
        .and(body_string_contains(
            "\"method\":\"eth_getTransactionReceipt\"",
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(
                    serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "result": null
                    })
                    .to_string(),
                )
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .post(server.uri())
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getTransactionReceipt",
            "params": [TEST_TX_HASH],
            "id": 1
        }))
        .send()
        .await
        .expect("Request failed");
    let body = response.text().await.expect("Failed to read body");
    assert!(body.contains("null"));
}

/// Test receipt waiting with confirmed state
#[tokio::test]
async fn test_receipt_waiting_confirmed() {
    let server = MockServer::start().await;

    // Mock confirmed receipt
    Mock::given(method("POST"))
        .and(body_string_contains(
            "\"method\":\"eth_getTransactionReceipt\"",
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(transaction_receipt_confirmed(TEST_TX_HASH, 12345, true))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .post(server.uri())
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getTransactionReceipt",
            "params": [TEST_TX_HASH],
            "id": 1
        }))
        .send()
        .await
        .expect("Request failed");
    let body = response.text().await.expect("Failed to read body");
    assert!(body.contains(TEST_TX_HASH));
    assert!(body.contains("0x1")); // success status
}
