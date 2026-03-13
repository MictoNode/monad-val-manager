//! Integration tests for query tx command
//!
//! Tests the transaction query functionality by hash.

use monad_val_manager::rpc::RpcClient;

#[tokio::test]
async fn test_get_transaction_by_hash() {
    // This test will fail until we implement get_transaction_by_hash
    let client = RpcClient::new("http://localhost:8080").unwrap();

    // Use a known transaction hash (this will fail with "not found" but should compile)
    let tx_hash = "0x0000000000000000000000000000000000000000000000000000000000000001";

    // This should fail until we implement the method
    let result = client.get_transaction_by_hash(tx_hash).await;

    // We expect this to either succeed or fail with "transaction not found"
    // but NOT with "method not implemented"
    match result {
        Ok(_) => {}
        Err(e) => {
            // The error should not be about missing method
            assert!(!e.to_string().contains("not implemented"));
        }
    }
}

#[tokio::test]
async fn test_get_transaction_by_hash_invalid_format() {
    let client = RpcClient::new("http://localhost:8080").unwrap();

    // Invalid hash format (missing 0x prefix)
    let tx_hash = "invalid";

    let result = client.get_transaction_by_hash(tx_hash).await;

    // Should fail with validation error
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_transaction_by_hash_empty() {
    let client = RpcClient::new("http://localhost:8080").unwrap();

    // Empty hash
    let tx_hash = "";

    let result = client.get_transaction_by_hash(tx_hash).await;

    // Should fail with validation error
    assert!(result.is_err());
}
