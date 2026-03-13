//! Mock RPC server using wiremock
//!
//! Provides a mock HTTP server that responds to JSON-RPC requests
//! for testing without a real Monad node.

use wiremock::matchers::{body_string_contains, method};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::fixtures::*;

/// Mock RPC server wrapper
pub struct MockRpcServer {
    pub server: MockServer,
}

impl MockRpcServer {
    /// Start a new mock RPC server
    pub async fn start() -> Self {
        let server = MockServer::start().await;
        Self { server }
    }

    /// Get the URI of the mock server
    pub fn uri(&self) -> String {
        self.server.uri()
    }

    /// Get the URI without trailing slash
    pub fn endpoint(&self) -> String {
        self.uri().trim_end_matches('/').to_string()
    }

    /// Mock eth_blockNumber response
    pub async fn mock_block_number(&self, block: u64) {
        let response = block_number_response(block);
        Mock::given(method("POST"))
            .and(body_string_contains("\"method\":\"eth_blockNumber\""))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(response)
                    .insert_header("Content-Type", "application/json"),
            )
            .mount(&self.server)
            .await;
    }

    /// Mock eth_syncing response (synced)
    pub async fn mock_syncing_synced(&self) {
        let response = syncing_response_synced();
        Mock::given(method("POST"))
            .and(body_string_contains("\"method\":\"eth_syncing\""))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(response)
                    .insert_header("Content-Type", "application/json"),
            )
            .mount(&self.server)
            .await;
    }

    /// Mock eth_syncing response (syncing in progress)
    pub async fn mock_syncing_syncing(&self, starting: u64, current: u64, highest: u64) {
        let response = syncing_response_syncing(starting, current, highest);
        Mock::given(method("POST"))
            .and(body_string_contains("\"method\":\"eth_syncing\""))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(response)
                    .insert_header("Content-Type", "application/json"),
            )
            .mount(&self.server)
            .await;
    }

    /// Mock net_peerCount response
    pub async fn mock_peer_count(&self, count: u64) {
        let response = peer_count_response(count);
        Mock::given(method("POST"))
            .and(body_string_contains("\"method\":\"net_peerCount\""))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(response)
                    .insert_header("Content-Type", "application/json"),
            )
            .mount(&self.server)
            .await;
    }

    /// Mock eth_chainId response
    pub async fn mock_chain_id(&self, chain_id: u64) {
        let response = chain_id_response(chain_id);
        Mock::given(method("POST"))
            .and(body_string_contains("\"method\":\"eth_chainId\""))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(response)
                    .insert_header("Content-Type", "application/json"),
            )
            .mount(&self.server)
            .await;
    }

    /// Mock eth_gasPrice response
    pub async fn mock_gas_price(&self, price: u64) {
        let response = gas_price_response(price);
        Mock::given(method("POST"))
            .and(body_string_contains("\"method\":\"eth_gasPrice\""))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(response)
                    .insert_header("Content-Type", "application/json"),
            )
            .mount(&self.server)
            .await;
    }

    /// Mock eth_getTransactionReceipt response (pending)
    #[allow(dead_code)]
    pub async fn mock_transaction_receipt_pending(&self, tx_hash: &str) {
        let response = transaction_receipt_pending();
        Mock::given(method("POST"))
            .and(body_string_contains(
                "\"method\":\"eth_getTransactionReceipt\"",
            ))
            .and(body_string_contains(tx_hash))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(response)
                    .insert_header("Content-Type", "application/json"),
            )
            .mount(&self.server)
            .await;
    }

    /// Mock eth_getTransactionReceipt response (confirmed)
    #[allow(dead_code)]
    pub async fn mock_transaction_receipt_confirmed(
        &self,
        tx_hash: &str,
        block_number: u64,
        status: bool,
    ) {
        let response = transaction_receipt_confirmed(tx_hash, block_number, status);
        Mock::given(method("POST"))
            .and(body_string_contains(
                "\"method\":\"eth_getTransactionReceipt\"",
            ))
            .and(body_string_contains(tx_hash))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(response)
                    .insert_header("Content-Type", "application/json"),
            )
            .mount(&self.server)
            .await;
    }

    /// Mock eth_call response
    #[allow(dead_code)]
    pub async fn mock_eth_call(&self, _to: &str, _data: &str, response_data: &str) {
        let response = eth_call_response(response_data);
        Mock::given(method("POST"))
            .and(body_string_contains("\"method\":\"eth_call\""))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(response)
                    .insert_header("Content-Type", "application/json"),
            )
            .mount(&self.server)
            .await;
    }

    /// Mock RPC error response
    #[allow(dead_code)]
    pub async fn mock_error(&self, method_name: &str, code: i32, message: &str) {
        let response = json_rpc_error(1, code, message);
        Mock::given(method("POST"))
            .and(body_string_contains(format!(
                "\"method\":\"{}\"",
                method_name
            )))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(response)
                    .insert_header("Content-Type", "application/json"),
            )
            .mount(&self.server)
            .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_server_start() {
        let server = MockRpcServer::start().await;
        assert!(server.uri().starts_with("http://"));
    }

    #[tokio::test]
    async fn test_mock_block_number() {
        let server = MockRpcServer::start().await;
        server.mock_block_number(12345).await;

        let client = reqwest::Client::new();
        let response = client
            .post(server.endpoint())
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "method": "eth_blockNumber",
                "params": [],
                "id": 1
            }))
            .send()
            .await
            .expect("Request failed");

        let body = response.text().await.expect("Failed to read body");
        assert!(body.contains("0x3039")); // 12345 in hex
    }
}
