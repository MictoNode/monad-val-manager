//! Helper functions for integration tests

use std::time::Duration;

/// Default timeout for async operations in tests
#[allow(dead_code)]
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// Create a test RPC client with the given endpoint
pub fn create_test_rpc_client(endpoint: &str) -> monad_val_manager::rpc::RpcClient {
    monad_val_manager::rpc::RpcClient::new(endpoint).expect("Failed to create test RPC client")
}

/// Run an async test with a timeout
#[allow(dead_code)]
pub async fn run_with_timeout<F, T>(future: F) -> T
where
    F: std::future::Future<Output = T>,
{
    tokio::time::timeout(DEFAULT_TIMEOUT, future)
        .await
        .expect("Test timed out")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_rpc_client() {
        let client = create_test_rpc_client("http://localhost:8080");
        // Client should be created successfully
        let _ = client;
    }
}
