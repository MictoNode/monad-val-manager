//! BUG-007 Integration Test - Broadcast Transaction Test
//!
//! This test attempts to reproduce the actual BUG-007 error by broadcasting
//! a transaction to a mock RPC server.

use monad_val_manager::rpc::RpcClient;
use monad_val_manager::staking::signer::{LocalSigner, Signer};
use monad_val_manager::staking::transaction::Eip1559Transaction;
use monad_val_manager::staking::operations;

#[tokio::test]
async fn test_bug_007_broadcast_with_mock_server() {
    use std::sync::Arc;

    // Start a mock server that simulates Monad testnet
    let mock_server = MockRpcServer::start().await;

    // Setup mock responses
    mock_server.mock_chain_id(10143).await; // testnet
    mock_server.mock_transaction_count(0).await; // nonce

    // Mock eth_sendRawTransaction to return the "decoding error"
    // This simulates what the real Monad testnet does
    mock_server
        .mock_send_raw_transaction_error(
            -32603,
            "Transaction decoding error",
        )
        .await;

    // Create a test signer
    let test_key = "0000000000000000000000000000000000000000000000000000000000000001";
    let signer = Arc::new(LocalSigner::from_private_key(test_key).expect("Valid key"));

    // Create RPC client
    let rpc_client = RpcClient::new(&mock_server.endpoint()).expect("Failed to create RPC client");

    // Try to perform a delegate operation
    let result = operations::delegate(
        &rpc_client,
        signer.as_ref(),
        224, // validator_id
        1_000_000_000_000_000_000, // 1 MON in wei
    )
    .await;

    // This should fail with the decoding error
    assert!(result.is_err(), "Expected error from mock server");
    let error = result.unwrap_err().to_string();
    assert!(
        error.contains("decoding error") || error.contains("-32603"),
        "Expected decoding error, got: {}",
        error
    );

    println!("BUG-007 reproduced: {}", error);
}

// =============================================================================
// Mock Server Extension for BUG-007 Testing
// =============================================================================

struct MockRpcServer {
    // In a real test, this would be a proper mock server
    // For now, we'll use a placeholder
}

impl MockRpcServer {
    async fn start() -> Self {
        Self {}
    }

    async fn mock_chain_id(&self, _chain_id: u64) {}
    async fn mock_transaction_count(&self, _count: u64) {}
    async fn mock_send_raw_transaction_error(&self, _code: i32, _message: &str) {}

    fn endpoint(&self) -> String {
        "http://localhost:9999".to_string()
    }
}
