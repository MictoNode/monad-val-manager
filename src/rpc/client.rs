//! JSON-RPC client for Monad node communication

use alloy_primitives::U256;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::config::Config;

/// JSON-RPC client for Monad node
#[derive(Clone)]
pub struct RpcClient {
    client: Client,
    endpoint: String,
}

/// JSON-RPC request
#[derive(Debug, Serialize)]
struct RpcRequest<T> {
    jsonrpc: &'static str,
    method: &'static str,
    params: T,
    id: u64,
}

/// JSON-RPC response
#[derive(Debug, Deserialize)]
struct RpcResponse<T> {
    #[allow(dead_code)]
    jsonrpc: Option<String>,
    result: Option<T>,
    error: Option<RpcError>,
    #[allow(dead_code)]
    id: Option<u64>,
}

/// JSON-RPC error
#[derive(Debug, Deserialize)]
struct RpcError {
    #[allow(dead_code)]
    code: i32,
    #[allow(dead_code)]
    message: String,
}

/// Sync status response
#[derive(Debug, Clone)]
pub enum SyncStatus {
    Synced,
    Syncing {
        starting_block: u64,
        current_block: u64,
        highest_block: u64,
    },
}

/// Node information from Prometheus metrics
#[derive(Debug, Clone, Default)]
pub struct NodeInfo {
    /// Node client version (from monad_node_info)
    pub version: Option<String>,
    /// Node uptime in seconds (from monad_total_uptime_us)
    pub uptime_seconds: Option<u64>,
}

/// Consensus information from Prometheus metrics
#[derive(Debug, Clone, Default)]
pub struct ConsensusInfo {
    /// Current epoch (from monad_consensus_epoch)
    pub epoch: Option<u64>,
    /// Current round (from monad_consensus_round)
    pub round: Option<u64>,
    /// Forkpoint epoch (from monad_consensus_forkpoint_epoch)
    pub forkpoint_epoch: Option<u64>,
    /// Forkpoint round (from monad_consensus_forkpoint_round)
    pub forkpoint_round: Option<u64>,
}

/// StateSync information from Prometheus metrics
#[derive(Debug, Clone, Default)]
pub struct StateSyncInfo {
    /// Is statesyncing (from monad_statesync_syncing)
    pub is_syncing: Option<bool>,
    /// Progress estimate 0-1 (from monad_statesync_progress_estimate)
    pub progress_estimate: Option<f64>,
    /// Last target (from monad_statesync_last_target)
    pub last_target: Option<u64>,
    /// Current prefix (from parsed log if available)
    pub current_prefix: Option<u64>,
}

/// Service status for systemd services (Linux)
#[derive(Debug, Clone, Default)]
pub struct ServicesStatus {
    /// monad-bft service status
    pub bft: Option<ServiceState>,
    /// monad-execution service status
    pub execution: Option<ServiceState>,
    /// monad-rpc service status
    pub rpc: Option<ServiceState>,
    /// monad-archiver service status
    pub archiver: Option<ServiceState>,
    /// otelcol service status
    pub otelcol: Option<ServiceState>,
}

/// Service state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceState {
    Running,
    Stopped,
    Unknown,
}

impl RpcClient {
    /// Create new RPC client
    pub fn new(endpoint: &str) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(5)) // Reduced from 30s for better TUI responsiveness
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            endpoint: endpoint.to_string(),
        })
    }

    /// Create RPC client from config
    pub fn from_config(config: &Config) -> Result<Self> {
        Self::new(config.rpc_endpoint())
    }

    /// Check if node is responding
    pub async fn check_connection(&self) -> Result<bool> {
        match self.get_block_number().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Get current block number
    pub async fn get_block_number(&self) -> Result<u64> {
        let result: String = self.call("eth_blockNumber", ()).await?;
        u64::from_str_radix(result.trim_start_matches("0x"), 16)
            .context("Failed to parse block number")
    }

    /// Get sync status (returns true if syncing)
    pub async fn get_sync_status(&self) -> Result<bool> {
        let response: serde_json::Value = self.call("eth_syncing", ()).await?;

        // If result is false, node is synced
        if let Some(false) = response.as_bool() {
            return Ok(false);
        }

        // If result is an object, node is syncing
        if response.is_object() {
            return Ok(true);
        }

        // Default to not syncing
        Ok(false)
    }

    /// Get detailed sync status
    pub async fn get_sync_status_detailed(&self) -> Result<SyncStatus> {
        let response: serde_json::Value = self.call("eth_syncing", ()).await?;

        if let Some(false) = response.as_bool() {
            return Ok(SyncStatus::Synced);
        }

        if let Some(obj) = response.as_object() {
            let starting = obj
                .get("startingBlock")
                .and_then(|v| v.as_str())
                .map(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap_or(0))
                .unwrap_or(0);

            let current = obj
                .get("currentBlock")
                .and_then(|v| v.as_str())
                .map(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap_or(0))
                .unwrap_or(0);

            let highest = obj
                .get("highestBlock")
                .and_then(|v| v.as_str())
                .map(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap_or(0))
                .unwrap_or(0);

            return Ok(SyncStatus::Syncing {
                starting_block: starting,
                current_block: current,
                highest_block: highest,
            });
        }

        Ok(SyncStatus::Synced)
    }

    /// Get peer count via net_peerCount RPC
    ///
    /// Note: Monad does NOT support `net_peerCount`. This method will fail on Monad nodes.
    /// Use `get_peer_count_monad()` for Monad nodes instead.
    pub async fn get_peer_count(&self) -> Result<u64> {
        let result: String = self.call("net_peerCount", ()).await?;
        u64::from_str_radix(result.trim_start_matches("0x"), 16)
            .context("Failed to parse peer count")
    }

    /// Get peer count from Prometheus metrics
    pub async fn get_peer_count_prometheus(&self) -> Result<u64> {
        let response = self
            .client
            .get("http://localhost:8889/metrics")
            .send()
            .await
            .context("Failed to connect to Prometheus metrics endpoint")?;

        let body = response.text().await?;

        // Parse monad_peer_disc_num_peers metric
        for line in body.lines() {
            if line.starts_with("monad_peer_disc_num_peers") {
                // Format: monad_peer_disc_num_peers{labels} value timestamp
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(count) = parts[1].parse::<u64>() {
                        return Ok(count);
                    }
                }
            }
        }

        anyhow::bail!("monad_peer_disc_num_peers metric not found")
    }

    /// Get node information from Prometheus metrics
    ///
    /// Returns node version and uptime from Prometheus metrics endpoint.
    pub async fn get_node_info_prometheus(&self) -> Result<NodeInfo> {
        let response = self
            .client
            .get("http://localhost:8889/metrics")
            .send()
            .await
            .context("Failed to connect to Prometheus metrics endpoint")?;

        let body = response.text().await?;

        let mut info = NodeInfo::default();

        for line in body.lines() {
            // Parse monad_node_info metric for version
            // Format: monad_node_info{...,service_version="x.y.z",...} 1
            if line.starts_with("monad_node_info{") {
                if let Some(version) = Self::extract_label_value(line, "service_version") {
                    info.version = Some(version);
                }
            }

            // Parse monad_total_uptime_us for uptime (microseconds)
            // Format: monad_total_uptime_us value
            if line.starts_with("monad_total_uptime_us") && !line.contains("{") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(uptime_us) = parts[1].parse::<u64>() {
                        info.uptime_seconds = Some(uptime_us / 1_000_000);
                    }
                }
            }
        }

        Ok(info)
    }

    /// Extract a label value from a Prometheus metric line
    fn extract_label_value(line: &str, label_name: &str) -> Option<String> {
        let pattern = format!("{}=\"", label_name);
        if let Some(start) = line.find(&pattern) {
            let start = start + pattern.len();
            if let Some(end) = line[start..].find('"') {
                return Some(line[start..start + end].to_string());
            }
        }
        None
    }

    /// Get validator status from Prometheus metrics
    ///
    /// Returns true if the node is registered as a validator in the staking contract.
    /// This is determined by checking if the node has validator-specific metrics.
    pub async fn get_validator_status_prometheus(&self) -> Result<bool> {
        let response = self
            .client
            .get("http://localhost:8889/metrics")
            .send()
            .await
            .context("Failed to connect to Prometheus metrics endpoint")?;

        let body = response.text().await?;

        // Check if any validator-specific metrics exist
        // Validator nodes have metrics like: monad_validator_status, monad_proposals, etc.
        let is_validator = body.lines().any(|line| {
            line.starts_with("monad_validator_")
                || line.starts_with("consensus_committed_block_num")
                || line.contains("validator")
        });

        Ok(is_validator)
    }

    /// Get chain ID
    pub async fn get_chain_id(&self) -> Result<u64> {
        let result: String = self.call("eth_chainId", ()).await?;
        u64::from_str_radix(result.trim_start_matches("0x"), 16).context("Failed to parse chain ID")
    }

    /// Get gas price
    pub async fn get_gas_price(&self) -> Result<u64> {
        let result: String = self.call("eth_gasPrice", ()).await?;
        u64::from_str_radix(result.trim_start_matches("0x"), 16)
            .context("Failed to parse gas price")
    }

    /// Get client version
    pub async fn get_client_version(&self) -> Result<String> {
        self.call("web3_clientVersion", ()).await
    }

    /// Get network version
    pub async fn get_network_version(&self) -> Result<String> {
        self.call("net_version", ()).await
    }

    /// Make an eth_call to a contract (read-only, no transaction)
    ///
    /// This is used for staking contract getter functions.
    ///
    /// # Arguments
    /// * `to` - Contract address (hex string with 0x prefix)
    /// * `data` - Call data (hex string with 0x prefix)
    ///
    /// # Returns
    /// Raw bytes response as hex string (with 0x prefix)
    pub async fn eth_call(&self, to: &str, data: &str) -> Result<String> {
        let call_obj = serde_json::json!({
            "to": to,
            "data": data,
        });

        // Use "latest" block
        self.call("eth_call", (&call_obj, "latest")).await
    }

    /// Get account balance in MON (human-readable)
    ///
    /// # Arguments
    /// * `address` - Ethereum address (hex string with 0x prefix)
    ///
    /// # Returns
    /// Balance in MON as f64 (wei / 10^18)
    pub async fn get_balance(&self, address: &str) -> Result<f64> {
        let result: String = self.call("eth_getBalance", (&address, "latest")).await?;

        // Parse hex string to U256 (handles large wei values)
        let wei = U256::from_str_radix(result.trim_start_matches("0x"), 16)
            .context("Failed to parse balance as U256")?;

        // Convert wei to MON (1 MON = 10^18 wei)
        let mon_value = wei.to::<u128>() as f64 / 1e18;

        Ok(mon_value)
    }

    /// Get transaction count (nonce) for an address
    ///
    /// # Arguments
    /// * `address` - Ethereum address (hex string with 0x prefix)
    ///
    /// # Returns
    /// Transaction count as u64 (nonce for next transaction)
    pub async fn get_transaction_count(&self, address: &str) -> Result<u64> {
        let result: String = self
            .call("eth_getTransactionCount", (&address, "latest"))
            .await?;
        u64::from_str_radix(result.trim_start_matches("0x"), 16)
            .context("Failed to parse transaction count")
    }

    /// Estimate gas for a transaction
    ///
    /// # Arguments
    /// * `from` - Sender address (hex string with 0x prefix)
    /// * `to` - Recipient address (hex string with 0x prefix)
    /// * `data` - Transaction data (hex string with 0x prefix)
    /// * `value` - Value to send in wei (hex string with 0x prefix)
    ///
    /// # Returns
    /// Estimated gas as u64
    pub async fn estimate_gas(&self, from: &str, to: &str, data: &str, value: &str) -> Result<u64> {
        let tx_obj = serde_json::json!({
            "from": from,
            "to": to,
            "data": data,
            "value": value,
        });

        let result: String = self.call("eth_estimateGas", (&tx_obj, "latest")).await?;
        u64::from_str_radix(result.trim_start_matches("0x"), 16)
            .context("Failed to parse gas estimate")
    }

    /// Get suggested priority fee (EIP-1559)
    ///
    /// # Returns
    /// Suggested max priority fee per gas in wei
    pub async fn get_max_priority_fee_per_gas(&self) -> Result<u64> {
        let result: String = self.call("eth_maxPriorityFeePerGas", ()).await?;
        u64::from_str_radix(result.trim_start_matches("0x"), 16)
            .context("Failed to parse priority fee")
    }

    /// Get fee history for EIP-1559 fee calculation
    ///
    /// # Arguments
    /// * `block_count` - Number of blocks to query
    /// * `newest_block` - Newest block to query ("latest" or block number)
    /// * `reward_percentiles` - Percentiles to sample for priority fees
    ///
    /// # Returns
    /// Raw JSON fee history response
    pub async fn get_fee_history(
        &self,
        block_count: u64,
        newest_block: &str,
        reward_percentiles: &[f64],
    ) -> Result<serde_json::Value> {
        let block_count_hex = format!("0x{:x}", block_count);
        self.call(
            "eth_feeHistory",
            (&block_count_hex, newest_block, reward_percentiles),
        )
        .await
    }

    /// Broadcast a signed raw transaction
    ///
    /// # Arguments
    /// * `raw_tx` - Raw signed transaction bytes (hex string with 0x prefix)
    ///
    /// # Returns
    /// Transaction hash (hex string with 0x prefix)
    pub async fn send_raw_transaction(&self, raw_tx: &str) -> Result<String> {
        self.call("eth_sendRawTransaction", (&raw_tx,)).await
    }

    /// Get transaction receipt (single call, no polling)
    ///
    /// # Arguments
    /// * `tx_hash` - Transaction hash (hex string with 0x prefix)
    ///
    /// # Returns
    /// Transaction receipt as JSON value, or null if not yet available
    pub async fn get_transaction_receipt_raw(&self, tx_hash: &str) -> Result<serde_json::Value> {
        self.call("eth_getTransactionReceipt", (&tx_hash,)).await
    }

    /// Wait for transaction receipt
    ///
    /// # Arguments
    /// * `tx_hash` - Transaction hash (hex string with 0x prefix)
    /// * `timeout_secs` - Maximum time to wait in seconds
    ///
    /// # Returns
    /// Transaction receipt as JSON value
    pub async fn wait_for_transaction_receipt(
        &self,
        tx_hash: &str,
        timeout_secs: u64,
    ) -> Result<serde_json::Value> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);

        while start.elapsed() < timeout {
            let receipt: serde_json::Value =
                self.call("eth_getTransactionReceipt", (&tx_hash,)).await?;

            if !receipt.is_null() {
                return Ok(receipt);
            }

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        anyhow::bail!("Transaction receipt timeout for {}", tx_hash)
    }

    /// Get transaction by hash
    ///
    /// # Arguments
    /// * `tx_hash` - Transaction hash (hex string with 0x prefix)
    ///
    /// # Returns
    /// Transaction details as JSON value
    pub async fn get_transaction_by_hash(&self, tx_hash: &str) -> Result<serde_json::Value> {
        // Validate hash format
        if tx_hash.is_empty() {
            anyhow::bail!("Transaction hash cannot be empty");
        }

        // Check for 0x prefix
        let hash = if tx_hash.starts_with("0x") {
            tx_hash
        } else {
            anyhow::bail!("Transaction hash must start with 0x prefix");
        };

        // Validate hex string length (should be 66 chars for 32 bytes + 0x)
        if hash.len() != 66 {
            anyhow::bail!(
                "Invalid transaction hash length: expected 66 characters (0x + 64 hex), got {}",
                hash.len()
            );
        }

        // Validate hex characters
        if !hash[2..].chars().all(|c| c.is_ascii_hexdigit()) {
            anyhow::bail!("Transaction hash contains invalid hex characters");
        }

        self.call("eth_getTransactionByHash", (&hash,)).await
    }

    /// Make a JSON-RPC call
    async fn call<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        method: &'static str,
        params: T,
    ) -> Result<R> {
        let response = self.call_raw(method, params).await?;

        if let Some(error) = response.error {
            anyhow::bail!("RPC error {}: {}", error.code, error.message);
        }

        response
            .result
            .ok_or_else(|| anyhow::anyhow!("No result in RPC response"))
    }

    /// Make a raw JSON-RPC call and return full response
    async fn call_raw<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        method: &'static str,
        params: T,
    ) -> Result<RpcResponse<R>> {
        let request = RpcRequest {
            jsonrpc: "2.0",
            method,
            params,
            id: 1,
        };

        let response = self
            .client
            .post(&self.endpoint)
            .json(&request)
            .send()
            .await
            .context("Failed to send RPC request")?;

        let response = response
            .json::<RpcResponse<R>>()
            .await
            .context("Failed to parse RPC response")?;

        Ok(response)
    }

    /// Get consensus information from Prometheus metrics
    ///
    /// Returns epoch, round, and forkpoint information from Prometheus metrics endpoint.
    pub async fn get_consensus_info_prometheus(&self) -> Result<ConsensusInfo> {
        let response = self
            .client
            .get("http://localhost:8889/metrics")
            .send()
            .await
            .context("Failed to connect to Prometheus metrics endpoint")?;

        let body = response.text().await?;
        let mut info = ConsensusInfo::default();

        for line in body.lines() {
            // Parse monad_consensus_epoch
            if line.starts_with("monad_consensus_epoch") && !line.contains("{") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(epoch) = parts[1].parse::<u64>() {
                        info.epoch = Some(epoch);
                    }
                }
            }

            // Parse monad_consensus_round
            if line.starts_with("monad_consensus_round") && !line.contains("{") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(round) = parts[1].parse::<u64>() {
                        info.round = Some(round);
                    }
                }
            }

            // Parse monad_consensus_forkpoint_epoch
            if line.starts_with("monad_consensus_forkpoint_epoch") && !line.contains("{") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(epoch) = parts[1].parse::<u64>() {
                        info.forkpoint_epoch = Some(epoch);
                    }
                }
            }

            // Parse monad_consensus_forkpoint_round
            if line.starts_with("monad_consensus_forkpoint_round") && !line.contains("{") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(round) = parts[1].parse::<u64>() {
                        info.forkpoint_round = Some(round);
                    }
                }
            }
        }

        Ok(info)
    }

    /// Get statesync information from Prometheus metrics
    ///
    /// Returns statesync progress and target information from Prometheus metrics endpoint.
    pub async fn get_statesync_info_prometheus(&self) -> Result<StateSyncInfo> {
        let response = self
            .client
            .get("http://localhost:8889/metrics")
            .send()
            .await
            .context("Failed to connect to Prometheus metrics endpoint")?;

        let body = response.text().await?;
        let mut info = StateSyncInfo::default();

        for line in body.lines() {
            // Parse monad_statesync_syncing (1 = syncing, 0 = live)
            if line.starts_with("monad_statesync_syncing") && !line.contains("{") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(syncing) = parts[1].parse::<u64>() {
                        info.is_syncing = Some(syncing == 1);
                    }
                }
            }

            // Parse monad_statesync_progress_estimate
            if line.starts_with("monad_statesync_progress_estimate") && !line.contains("{") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(progress) = parts[1].parse::<f64>() {
                        info.progress_estimate = Some(progress);
                    }
                }
            }

            // Parse monad_statesync_last_target
            if line.starts_with("monad_statesync_last_target") && !line.contains("{") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(target) = parts[1].parse::<u64>() {
                        info.last_target = Some(target);
                    }
                }
            }
        }

        Ok(info)
    }

    /// Get services status (Linux only - uses systemctl)
    ///
    /// Returns status of monad services via systemctl is-active.
    pub async fn get_services_status(&self) -> Result<ServicesStatus> {
        #[cfg(target_os = "linux")]
        {
            use tokio::process::Command;

            let mut status = ServicesStatus::default();

            // Check each service
            let services = [
                ("monad-bft", ServiceState::Unknown),
                ("monad-execution", ServiceState::Unknown),
                ("monad-rpc", ServiceState::Unknown),
                ("monad-archiver", ServiceState::Unknown),
                ("otelcol", ServiceState::Unknown),
            ];

            for (service, _) in services {
                let output = Command::new("systemctl")
                    .args(["is-active", service])
                    .output()
                    .await;

                if let Ok(output) = output {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let state = if stdout.trim() == "active" {
                        ServiceState::Running
                    } else if stdout.trim() == "inactive" {
                        ServiceState::Stopped
                    } else {
                        ServiceState::Unknown
                    };

                    match service {
                        "monad-bft" => status.bft = Some(state),
                        "monad-execution" => status.execution = Some(state),
                        "monad-rpc" => status.rpc = Some(state),
                        "monad-archiver" => status.archiver = Some(state),
                        "otelcol" => status.otelcol = Some(state),
                        _ => {}
                    }
                }
            }

            Ok(status)
        }

        #[cfg(not(target_os = "linux"))]
        {
            // On non-Linux platforms, return empty status
            Ok(ServicesStatus::default())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_client_creation() {
        let client = RpcClient::new("http://localhost:8080");
        assert!(client.is_ok());
    }
}
