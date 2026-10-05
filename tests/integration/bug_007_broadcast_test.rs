//! BUG-007 Integration Test - Broadcast Transaction Test
//!
//! BUG-007: broadcasting a signed EIP-1559 transaction to the node failed with
//! "Transaction decoding error", while the dry-run encoding looked correct.
//!
//! This test pins the RPC contract of the broadcast path against a mock node:
//!
//! 1. the payload handed to `eth_sendRawTransaction` must be a signed EIP-1559
//!    transaction (0x02 type prefix), and
//! 2. when the node rejects that payload, the node's error must propagate to
//!    the caller instead of being swallowed.

use monad_val_manager::rpc::RpcClient;
use monad_val_manager::staking::operations;
use monad_val_manager::staking::signer::{LocalSigner, Signer};

use wiremock::matchers::{body_string_contains, method};
use wiremock::{Mock, ResponseTemplate};

use crate::mocks::{json_rpc_error, MockRpcServer};

const TEST_KEY: &str = "0000000000000000000000000000000000000000000000000000000000000001";
const TESTNET_CHAIN_ID: u64 = 10143;
const VALIDATOR_ID: u64 = 224;
const ONE_MON: u128 = 1_000_000_000_000_000_000;

#[tokio::test]
async fn test_bug_007_broadcast_surfaces_node_error() {
    let mock = MockRpcServer::start().await;
    mock.mock_chain_id(TESTNET_CHAIN_ID).await;
    mock.mock_transaction_count(0).await;

    // Only answer eth_sendRawTransaction when the payload really is a signed
    // EIP-1559 transaction. If the encoder regresses, this mock stops matching,
    // the request falls through to a 404 and the assertions below fail.
    Mock::given(method("POST"))
        .and(body_string_contains("\"method\":\"eth_sendRawTransaction\""))
        .and(body_string_contains("\"0x02"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(json_rpc_error(1, -32603, "Transaction decoding error"))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&mock.server)
        .await;

    let signer = LocalSigner::from_private_key(TEST_KEY).expect("Valid key");
    assert!(
        signer.address().starts_with("0x"),
        "Signer must expose an address"
    );

    let client = RpcClient::new(&mock.endpoint()).expect("Failed to create RPC client");
    let result = operations::delegate(&client, &signer, VALIDATOR_ID, ONE_MON).await;

    // The node rejected the broadcast, so the operation must fail with the
    // node's own error rather than a silent success.
    let error = result.expect_err("delegate must fail when the node rejects the broadcast");
    let message = error.to_string();
    assert!(
        message.contains("decoding error") || message.contains("-32603"),
        "node error must propagate to the caller, got: {message} \
         (or the broadcast payload was not a signed EIP-1559 transaction)"
    );
}
