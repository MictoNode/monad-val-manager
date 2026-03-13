//! Test fixtures for integration tests
//!
//! Contains test addresses, transaction hashes, and mock responses.

/// Test validator address (40 hex chars, no 0x prefix for display)
#[allow(dead_code)]
pub const TEST_VALIDATOR_ADDRESS: &str = "0x1234567890123456789012345678901234567890";

/// Test delegator address
pub const TEST_DELEGATOR_ADDRESS: &str = "0xabcdef0123456789abcdef0123456789abcdef01";

/// Test validator address 2 (for multi-validator tests)
#[allow(dead_code)]
pub const TEST_VALIDATOR_ADDRESS_2: &str = "0x9876543210987654321098765432109876543210";

/// Test delegator address 2
#[allow(dead_code)]
pub const TEST_DELEGATOR_ADDRESS_2: &str = "0xfedcba9876543210fedcba9876543210fedcba98";

/// Staking contract address on Monad testnet
pub const STAKING_CONTRACT_ADDRESS: &str = "0x0000000000000000000000000000000000000001";

/// Test transaction hash
pub const TEST_TX_HASH: &str = "0xaabbccdd11223344556677889900aabbccdd11223344556677889900aabbccdd";

/// Test transaction hash 2
#[allow(dead_code)]
pub const TEST_TX_HASH_2: &str =
    "0x11223344aabbccdd556677889900aabbccdd11223344556677889900aabbccdd";

/// Test block number (decimal 12345678)
#[allow(dead_code)]
pub const TEST_BLOCK_NUMBER: u64 = 0xBC614E;

/// Test peer count (decimal 25)
#[allow(dead_code)]
pub const TEST_PEER_COUNT: u64 = 0x19;

/// Test chain ID for testnet (10143)
#[allow(dead_code)]
pub const TEST_CHAIN_ID: u64 = 0x279F;

/// Test gas price (10 Gwei = 10_000_000_000 wei)
#[allow(dead_code)]
pub const TEST_GAS_PRICE: u64 = 0x2540BE400;

/// Create a JSON-RPC success response
pub fn json_rpc_success<T: serde::Serialize>(id: u64, result: T) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
    .to_string()
}

/// Create a JSON-RPC error response
pub fn json_rpc_error(id: u64, code: i32, message: &str) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
    .to_string()
}

/// Create eth_blockNumber response
pub fn block_number_response(block: u64) -> String {
    json_rpc_success(1, format!("0x{:x}", block))
}

/// Create eth_syncing response (synced = false)
pub fn syncing_response_synced() -> String {
    json_rpc_success(1, false)
}

/// Create eth_syncing response (syncing in progress)
pub fn syncing_response_syncing(starting: u64, current: u64, highest: u64) -> String {
    json_rpc_success(
        1,
        serde_json::json!({
            "startingBlock": format!("0x{:x}", starting),
            "currentBlock": format!("0x{:x}", current),
            "highestBlock": format!("0x{:x}", highest)
        }),
    )
}

/// Create net_peerCount response
pub fn peer_count_response(count: u64) -> String {
    json_rpc_success(1, format!("0x{:x}", count))
}

/// Create eth_chainId response
pub fn chain_id_response(chain_id: u64) -> String {
    json_rpc_success(1, format!("0x{:x}", chain_id))
}

/// Create eth_gasPrice response
pub fn gas_price_response(price: u64) -> String {
    json_rpc_success(1, format!("0x{:x}", price))
}

/// Create eth_getTransactionReceipt response (pending/null)
#[allow(dead_code)]
pub fn transaction_receipt_pending() -> String {
    json_rpc_success(1, serde_json::Value::Null)
}

/// Create eth_getTransactionReceipt response (confirmed)
pub fn transaction_receipt_confirmed(tx_hash: &str, block_number: u64, status: bool) -> String {
    let status_hex = if status { "0x1" } else { "0x0" };
    json_rpc_success(
        1,
        serde_json::json!({
            "transactionHash": tx_hash,
            "blockNumber": format!("0x{:x}", block_number),
            "blockHash": "0x0000000000000000000000000000000000000000000000000000000000000001",
            "from": TEST_DELEGATOR_ADDRESS,
            "to": STAKING_CONTRACT_ADDRESS,
            "status": status_hex,
            "gasUsed": "0x5208"
        }),
    )
}

/// Create eth_call response (empty bytes)
#[allow(dead_code)]
pub fn eth_call_empty() -> String {
    json_rpc_success(1, "0x")
}

/// Create eth_call response with data
#[allow(dead_code)]
pub fn eth_call_response(data: &str) -> String {
    json_rpc_success(1, data)
}

// =============================================================================
// STAKING FIXTURES - Mock responses for staking contract calls
// =============================================================================

/// Create eth_getTransactionCount response (nonce)
#[allow(dead_code)]
pub fn nonce_response(nonce: u64) -> String {
    json_rpc_success(1, format!("0x{:x}", nonce))
}

/// Create eth_sendRawTransaction response (transaction hash)
#[allow(dead_code)]
pub fn send_raw_transaction_response(tx_hash: &str) -> String {
    json_rpc_success(1, tx_hash)
}

/// Create eth_estimateGas response
#[allow(dead_code)]
pub fn estimate_gas_response(gas: u64) -> String {
    json_rpc_success(1, format!("0x{:x}", gas))
}

/// Create eth_getBalance response
#[allow(dead_code)]
pub fn balance_response(balance: u128) -> String {
    json_rpc_success(1, format!("0x{:x}", balance))
}

/// Create eth_maxPriorityFeePerGas response
#[allow(dead_code)]
pub fn max_priority_fee_response(fee: u64) -> String {
    json_rpc_success(1, format!("0x{:x}", fee))
}

/// Create eth_feeHistory response
#[allow(dead_code)]
pub fn fee_history_response(base_fee: u64, priority_fee: u64) -> String {
    json_rpc_success(
        1,
        serde_json::json!({
            "baseFeePerGas": [format!("0x{:x}", base_fee)],
            "gasUsedRatio": [0.5],
            "reward": [[format!("0x{:x}", priority_fee)]]
        }),
    )
}

// =============================================================================
// STAKING CONTRACT CALL RESPONSES - ABI-encoded responses
// =============================================================================

/// Create get_epoch() response
/// Returns: (uint64 epoch, bool is_epoch_transition)
/// ABI encoding: 2 x 32 bytes = 64 hex chars
#[allow(dead_code)]
pub fn get_epoch_response(epoch: u64, is_epoch_transition: bool) -> String {
    let epoch_hex = format!("{:064x}", epoch);
    let transition_hex = if is_epoch_transition {
        "0000000000000000000000000000000000000000000000000000000000000001"
    } else {
        "0000000000000000000000000000000000000000000000000000000000000000"
    };
    eth_call_response(&format!("0x{}{}", epoch_hex, transition_hex))
}

/// Create get_validator() response (simplified for testing)
/// Returns minimal validator data for testing purposes
#[allow(dead_code)]
pub fn get_validator_response(
    auth_delegator: &str,
    commission: u64,
    delegated_amount: u128,
) -> String {
    // Simplified response - auth_delegator (32 bytes) + commission (32 bytes) + delegated_amount (32 bytes)
    // Full response would be much larger with pubkey bytes
    let addr_clean = auth_delegator.strip_prefix("0x").unwrap_or(auth_delegator);
    let auth_padded = format!("{:0>64}", addr_clean);
    let commission_hex = format!("{:064x}", commission);
    let amount_hex = format!("{:064x}", delegated_amount);

    // Minimal valid response with dynamic byte offsets
    // 10 static params + 2 dynamic offsets + minimal pubkeys
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
    response.push_str(&"0".repeat(32)); // padding to 32-byte boundary

    eth_call_response(&response)
}

/// Create get_delegator() response
/// Returns: (delegated_amount, pending_amount, rewards, total_claimed, total_compounded, activation_epoch, last_claim_epoch)
#[allow(dead_code)]
pub fn get_delegator_response(
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

/// Create get_withdrawal_request() response
/// Returns: (amount, withdrawal_index, activation_epoch)
#[allow(dead_code)]
pub fn get_withdrawal_request_response(
    amount: u128,
    withdrawal_index: u8,
    activation_epoch: u64,
) -> String {
    let amount_hex = format!("{:064x}", amount);
    let index_hex = format!("{:064x}", withdrawal_index);
    let epoch_hex = format!("{:064x}", activation_epoch);

    eth_call_response(&format!("0x{}{}{}", amount_hex, index_hex, epoch_hex))
}

/// Create get_delegations() response (paginated)
/// Returns: (has_more, total_count, validator_ids[])
#[allow(dead_code)]
pub fn get_delegations_response(has_more: bool, validator_ids: &[u64]) -> String {
    let has_more_hex = if has_more {
        "0000000000000000000000000000000000000000000000000000000000000001"
    } else {
        "0000000000000000000000000000000000000000000000000000000000000000"
    };
    let count_hex = format!("{:064x}", validator_ids.len() as u64);
    let offset_hex = "0000000000000000000000000000000000000000000000000000000000000060"; // offset to array
    let array_len_hex = format!("{:064x}", validator_ids.len() as u64);

    let mut ids_hex = String::new();
    for id in validator_ids {
        ids_hex.push_str(&format!("{:064x}", id));
    }

    eth_call_response(&format!(
        "0x{}{}{}{}{}",
        has_more_hex, count_hex, offset_hex, array_len_hex, ids_hex
    ))
}

/// Create get_proposer_val_id() response
#[allow(dead_code)]
pub fn get_proposer_val_id_response(validator_id: u64) -> String {
    eth_call_response(&format!("0x{:064x}", validator_id))
}

/// Create get_consensus_valset() response
#[allow(dead_code)]
pub fn get_validator_set_response(has_more: bool, validator_ids: &[u64]) -> String {
    // Same format as delegations
    get_delegations_response(has_more, validator_ids)
}

// =============================================================================
// TRANSACTION LOG FIXTURES - For event parsing tests
// =============================================================================

/// Create a mock transaction log for testing event parsing
#[allow(dead_code)]
pub fn mock_transaction_log(address: &str, topics: &[&str], data: &str) -> serde_json::Value {
    serde_json::json!({
        "address": address,
        "topics": topics,
        "data": data
    })
}

/// Create Delegate event log
/// Event: Delegate(uint64 indexed valId, address indexed delegator, uint256 amount, uint64 activationEpoch)
#[allow(dead_code)]
pub fn delegate_event_log(
    val_id: u64,
    delegator: &str,
    amount: u128,
    activation_epoch: u64,
) -> serde_json::Value {
    // Event signature hash (keccak256 of event signature)
    let event_sig = "0x774921ca02705390dc9a54eca50012716ef2aa7ab4265b3f5005294f163bbce8";

    // Topics: [signature, valId (padded to 32 bytes), delegator (padded to 32 bytes)]
    let val_id_topic = format!("0x{:064x}", val_id);
    let delegator_clean = delegator.strip_prefix("0x").unwrap_or(delegator);
    let delegator_topic = format!("0x{:0>64}", delegator_clean);

    // Data: amount (32 bytes) + activationEpoch (32 bytes)
    let data = format!("0x{:064x}{:064x}", amount, activation_epoch);

    mock_transaction_log(
        STAKING_CONTRACT_ADDRESS,
        &[event_sig, &val_id_topic, &delegator_topic],
        &data,
    )
}

/// Create Undelegate event log
/// Event: Undelegate(uint64 indexed valId, address indexed delegator, uint8 withdrawal_id, uint256 amount, uint64 activationEpoch)
#[allow(dead_code)]
pub fn undelegate_event_log(
    val_id: u64,
    delegator: &str,
    withdrawal_id: u8,
    amount: u128,
    activation_epoch: u64,
) -> serde_json::Value {
    let event_sig = "0xb5fe097b373241e83675cffa172c83dffb6cdeea4878e4725b2bfbfc8817c58d";

    let val_id_topic = format!("0x{:064x}", val_id);
    let delegator_clean = delegator.strip_prefix("0x").unwrap_or(delegator);
    let delegator_topic = format!("0x{:0>64}", delegator_clean);

    // Data: withdrawal_id (32 bytes) + amount (32 bytes) + activationEpoch (32 bytes)
    let data = format!(
        "0x{:064x}{:064x}{:064x}",
        withdrawal_id, amount, activation_epoch
    );

    mock_transaction_log(
        STAKING_CONTRACT_ADDRESS,
        &[event_sig, &val_id_topic, &delegator_topic],
        &data,
    )
}

/// Create Withdraw event log
/// Event: Withdraw(uint64 indexed valId, address indexed delegator, uint8 withdrawal_id, uint256 amount, uint64 activationEpoch)
#[allow(dead_code)]
pub fn withdraw_event_log(
    val_id: u64,
    delegator: &str,
    withdrawal_id: u8,
    amount: u128,
    activation_epoch: u64,
) -> serde_json::Value {
    let event_sig = "0xb5b6939823da72d47af76b05c23a7f5ccdef9e1e367aef1880a7c5b12cbdce9f";

    let val_id_topic = format!("0x{:064x}", val_id);
    let delegator_clean = delegator.strip_prefix("0x").unwrap_or(delegator);
    let delegator_topic = format!("0x{:0>64}", delegator_clean);

    let data = format!(
        "0x{:064x}{:064x}{:064x}",
        withdrawal_id, amount, activation_epoch
    );

    mock_transaction_log(
        STAKING_CONTRACT_ADDRESS,
        &[event_sig, &val_id_topic, &delegator_topic],
        &data,
    )
}

/// Create ClaimRewards event log
/// Event: ClaimRewards(uint64 indexed valId, address indexed delegator, uint256 amount, uint64 epoch)
#[allow(dead_code)]
pub fn claim_rewards_event_log(
    val_id: u64,
    delegator: &str,
    amount: u128,
    epoch: u64,
) -> serde_json::Value {
    let event_sig = "0xc0b2fc1f945c3a223f1ca3b7c2a0ce515cdbb89b18111314d42ce1115beba6b6";

    let val_id_topic = format!("0x{:064x}", val_id);
    let delegator_clean = delegator.strip_prefix("0x").unwrap_or(delegator);
    let delegator_topic = format!("0x{:0>64}", delegator_clean);

    let data = format!("0x{:064x}{:064x}", amount, epoch);

    mock_transaction_log(
        STAKING_CONTRACT_ADDRESS,
        &[event_sig, &val_id_topic, &delegator_topic],
        &data,
    )
}

/// Create Compound event log (placeholder signature)
/// Event: Compound(uint64 indexed valId, address indexed delegator, uint256 amount)
#[allow(dead_code)]
pub fn compound_event_log(val_id: u64, delegator: &str, amount: u128) -> serde_json::Value {
    let event_sig = "0x1111111111111111111111111111111111111111111111111111111111111111";

    let val_id_topic = format!("0x{:064x}", val_id);
    let delegator_clean = delegator.strip_prefix("0x").unwrap_or(delegator);
    let delegator_topic = format!("0x{:0>64}", delegator_clean);

    let data = format!("0x{:064x}", amount);

    mock_transaction_log(
        STAKING_CONTRACT_ADDRESS,
        &[event_sig, &val_id_topic, &delegator_topic],
        &data,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_number_response() {
        let response = block_number_response(12345);
        assert!(response.contains("0x3039"));
    }

    #[test]
    fn test_syncing_response_synced() {
        let response = syncing_response_synced();
        assert!(response.contains("\"result\":false"));
    }

    #[test]
    fn test_syncing_response_syncing() {
        let response = syncing_response_syncing(0, 50, 100);
        assert!(response.contains("0x32")); // 50 hex
        assert!(response.contains("0x64")); // 100 hex
    }

    #[test]
    fn test_peer_count_response() {
        let response = peer_count_response(25);
        assert!(response.contains("0x19"));
    }

    #[test]
    fn test_chain_id_response() {
        let response = chain_id_response(10143);
        assert!(response.contains("0x279f"));
    }

    #[test]
    fn test_json_rpc_error() {
        let response = json_rpc_error(1, -32601, "Method not found");
        assert!(response.contains("-32601"));
        assert!(response.contains("Method not found"));
    }

    #[test]
    fn test_transaction_receipt_confirmed() {
        let response = transaction_receipt_confirmed(TEST_TX_HASH, 12345, true);
        assert!(response.contains(TEST_TX_HASH));
        assert!(response.contains("0x3039")); // block 12345
        assert!(response.contains("0x1")); // success status
    }
}
