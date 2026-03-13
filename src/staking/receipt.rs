//! Transaction receipt waiting mechanism for staking operations
//!
//! This module provides functionality to wait for transaction receipts
//! after staking operations.
//!
//! # Features
//! - Configurable polling interval (default: 2 seconds)
//! - Configurable timeout (default: 60 seconds)
//! - User-friendly status messages
//! - Transaction success/failure detection
//!
//! # Example
//!
//! ```ignore
//! use monad_val_manager::staking::receipt::{wait_for_receipt, ReceiptConfig};
//! use monad_val_manager::rpc::RpcClient;
//!
//! let client = RpcClient::new("http://localhost:8080")?;
//! let tx_hash = "0x...";
//!
//! let receipt = wait_for_receipt(&client, tx_hash, ReceiptConfig::default()).await?;
//! match receipt.status {
//!     TransactionStatus::Success => println!("Confirmed! Gas used: {}", receipt.gas_used),
//!     TransactionStatus::Failure => println!("Failed: {:?}", receipt.revert_reason),
//! }
//! ```

use crate::rpc::RpcClient;
use crate::utils::error::Result;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Default polling interval in seconds
pub const DEFAULT_POLL_INTERVAL_SECS: u64 = 2;

/// Default timeout in seconds
pub const DEFAULT_TIMEOUT_SECS: u64 = 60;

/// Transaction execution status
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// Transaction executed successfully
    Success,
    /// Transaction execution failed
    Failure,
    /// Transaction is pending (not yet mined)
    #[default]
    Pending,
}

/// Transaction receipt information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionReceipt {
    /// Transaction hash
    pub tx_hash: String,
    /// Block number where transaction was included
    pub block_number: u64,
    /// Transaction execution status
    pub status: TransactionStatus,
    /// Gas used by the transaction
    pub gas_used: u64,
    /// Effective gas price (wei)
    pub effective_gas_price: u64,
    /// Contract address created (if any)
    pub contract_address: Option<String>,
    /// Revert reason (if transaction failed)
    pub revert_reason: Option<String>,
}

impl Default for TransactionReceipt {
    fn default() -> Self {
        Self {
            tx_hash: String::new(),
            block_number: 0,
            status: TransactionStatus::Pending,
            gas_used: 0,
            effective_gas_price: 0,
            contract_address: None,
            revert_reason: None,
        }
    }
}

/// Configuration for receipt waiting
pub struct ReceiptConfig {
    /// Polling interval
    pub poll_interval: Duration,
    /// Maximum time to wait
    pub timeout: Duration,
    /// Callback for status updates (receives elapsed seconds)
    pub on_status: Option<Box<dyn Fn(u64) + Send + Sync>>,
}

impl std::fmt::Debug for ReceiptConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReceiptConfig")
            .field("poll_interval", &self.poll_interval)
            .field("timeout", &self.timeout)
            .field("on_status", &self.on_status.as_ref().map(|_| "callback"))
            .finish()
    }
}

impl Default for ReceiptConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(DEFAULT_POLL_INTERVAL_SECS),
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            on_status: None,
        }
    }
}

impl ReceiptConfig {
    /// Create config with custom polling interval
    pub fn with_poll_interval(mut self, secs: u64) -> Self {
        self.poll_interval = Duration::from_secs(secs);
        self
    }

    /// Create config with custom timeout
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout = Duration::from_secs(secs);
        self
    }

    /// Create config with status callback
    pub fn with_status_callback<F: Fn(u64) + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_status = Some(Box::new(callback));
        self
    }
}

/// Wait for a transaction receipt
///
/// Polls the node for the transaction receipt until it's available or timeout.
///
/// # Arguments
/// * `client` - RPC client
/// * `tx_hash` - Transaction hash (with 0x prefix)
/// * `config` - Receipt waiting configuration
///
/// # Returns
/// Transaction receipt if found within timeout
///
/// # Errors
/// Returns error if:
/// - Transaction not found within timeout
/// - RPC call fails
/// - Receipt data is malformed
pub async fn wait_for_receipt(
    client: &RpcClient,
    tx_hash: &str,
    config: ReceiptConfig,
) -> Result<TransactionReceipt> {
    let start = std::time::Instant::now();
    let mut elapsed_secs = 0u64;

    loop {
        // Check timeout
        if start.elapsed() >= config.timeout {
            return Err(crate::utils::error::Error::Timeout(format!(
                "Transaction receipt timeout for {}",
                tx_hash
            )));
        }

        // Try to get receipt
        match get_transaction_receipt(client, tx_hash).await {
            Ok(Some(receipt)) => {
                // Receipt found
                if receipt.status != TransactionStatus::Pending {
                    return Ok(receipt);
                }
                // Receipt exists but still pending - continue polling
            }
            Ok(None) => {
                // Receipt not yet available - continue polling
            }
            Err(e) => {
                // RPC error - log and continue (might be temporary)
                tracing::debug!("RPC error while fetching receipt: {}", e);
            }
        }

        // Call status callback if set
        if let Some(ref callback) = config.on_status {
            callback(elapsed_secs);
        }

        // Wait before next poll
        tokio::time::sleep(config.poll_interval).await;
        elapsed_secs = start.elapsed().as_secs();
    }
}

/// Get transaction receipt from RPC (raw)
///
/// Returns None if transaction is not yet mined.
async fn get_transaction_receipt(
    client: &RpcClient,
    tx_hash: &str,
) -> Result<Option<TransactionReceipt>> {
    // Use the RPC client's wait_for_transaction_receipt with short timeout
    // to get the raw receipt, then parse it
    let raw = client.get_transaction_receipt_raw(tx_hash).await?;

    if raw.is_null() {
        return Ok(None);
    }

    Ok(Some(parse_receipt(tx_hash, &raw)?))
}

/// Parse raw JSON receipt into TransactionReceipt
fn parse_receipt(tx_hash: &str, raw: &serde_json::Value) -> Result<TransactionReceipt> {
    let obj = raw.as_object().context("Receipt is not an object")?;

    // Parse block number
    let block_number = obj
        .get("blockNumber")
        .and_then(|v| v.as_str())
        .map(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16))
        .transpose()
        .context("Failed to parse blockNumber")?
        .unwrap_or(0);

    // Parse status (0x0 = failure, 0x1 = success)
    let status = obj
        .get("status")
        .and_then(|v| v.as_str())
        .map(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16))
        .transpose()
        .context("Failed to parse status")?
        .map(|s| {
            if s == 1 {
                TransactionStatus::Success
            } else {
                TransactionStatus::Failure
            }
        })
        .unwrap_or(TransactionStatus::Pending);

    // Parse gas used
    let gas_used = obj
        .get("gasUsed")
        .and_then(|v| v.as_str())
        .map(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16))
        .transpose()
        .context("Failed to parse gasUsed")?
        .unwrap_or(0);

    // Parse effective gas price
    let effective_gas_price = obj
        .get("effectiveGasPrice")
        .and_then(|v| v.as_str())
        .map(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16))
        .transpose()
        .context("Failed to parse effectiveGasPrice")?
        .unwrap_or(0);

    // Parse contract address (if any)
    let contract_address = obj
        .get("contractAddress")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    // Parse revert reason (if any)
    let revert_reason = obj
        .get("revertReason")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(TransactionReceipt {
        tx_hash: tx_hash.to_string(),
        block_number,
        status,
        gas_used,
        effective_gas_price,
        contract_address,
        revert_reason,
    })
}

/// Format receipt for CLI output
///
/// Returns a user-friendly string describing the transaction result.
pub fn format_receipt(receipt: &TransactionReceipt) -> String {
    match receipt.status {
        TransactionStatus::Success => {
            format!(
                "Transaction confirmed!\n  Block: {}\n  Gas used: {}",
                receipt.block_number, receipt.gas_used
            )
        }
        TransactionStatus::Failure => {
            let reason = receipt.revert_reason.as_deref().unwrap_or("Unknown reason");
            format!(
                "Transaction failed!\n  Block: {}\n  Reason: {}",
                receipt.block_number, reason
            )
        }
        TransactionStatus::Pending => {
            format!("Transaction pending... (Block: {})", receipt.block_number)
        }
    }
}

/// Wait for receipt with progress output
///
/// Prints "Waiting for receipt..." and progress dots while waiting.
pub async fn wait_for_receipt_with_progress(
    client: &RpcClient,
    tx_hash: &str,
    config: ReceiptConfig,
) -> Result<TransactionReceipt> {
    print!("Waiting for receipt...");
    let _ = std::io::Write::flush(&mut std::io::stdout());

    let config = config.with_status_callback(|_elapsed| {
        print!(".");
        let _ = std::io::Write::flush(&mut std::io::stdout());
    });

    let receipt = wait_for_receipt(client, tx_hash, config).await?;

    // Print newline after dots
    println!();

    // Print result
    println!("{}", format_receipt(&receipt));

    Ok(receipt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_status_default() {
        let status = TransactionStatus::default();
        assert_eq!(status, TransactionStatus::Pending);
    }

    #[test]
    fn test_transaction_receipt_default() {
        let receipt = TransactionReceipt::default();
        assert!(receipt.tx_hash.is_empty());
        assert_eq!(receipt.block_number, 0);
        assert_eq!(receipt.status, TransactionStatus::Pending);
        assert_eq!(receipt.gas_used, 0);
        assert!(receipt.contract_address.is_none());
        assert!(receipt.revert_reason.is_none());
    }

    #[test]
    fn test_receipt_config_default() {
        let config = ReceiptConfig::default();
        assert_eq!(config.poll_interval, Duration::from_secs(2));
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert!(config.on_status.is_none());
    }

    #[test]
    fn test_receipt_config_builder() {
        let config = ReceiptConfig::default()
            .with_poll_interval(5)
            .with_timeout(120);

        assert_eq!(config.poll_interval, Duration::from_secs(5));
        assert_eq!(config.timeout, Duration::from_secs(120));
    }

    #[test]
    fn test_parse_receipt_success() {
        let raw = serde_json::json!({
            "blockNumber": "0x1234",
            "status": "0x1",
            "gasUsed": "0x5208",
            "effectiveGasPrice": "0x3b9aca00"
        });

        let receipt = parse_receipt("0xabc", &raw).unwrap();
        assert_eq!(receipt.tx_hash, "0xabc");
        assert_eq!(receipt.block_number, 0x1234);
        assert_eq!(receipt.status, TransactionStatus::Success);
        assert_eq!(receipt.gas_used, 0x5208);
        assert_eq!(receipt.effective_gas_price, 0x3b9aca00);
    }

    #[test]
    fn test_parse_receipt_failure() {
        let raw = serde_json::json!({
            "blockNumber": "0x1234",
            "status": "0x0",
            "gasUsed": "0x5208",
            "effectiveGasPrice": "0x3b9aca00",
            "revertReason": "Insufficient balance"
        });

        let receipt = parse_receipt("0xabc", &raw).unwrap();
        assert_eq!(receipt.status, TransactionStatus::Failure);
        assert_eq!(
            receipt.revert_reason,
            Some("Insufficient balance".to_string())
        );
    }

    #[test]
    fn test_parse_receipt_with_contract() {
        let raw = serde_json::json!({
            "blockNumber": "0x1234",
            "status": "0x1",
            "gasUsed": "0x5208",
            "effectiveGasPrice": "0x3b9aca00",
            "contractAddress": "0x1234567890123456789012345678901234567890"
        });

        let receipt = parse_receipt("0xabc", &raw).unwrap();
        assert_eq!(
            receipt.contract_address,
            Some("0x1234567890123456789012345678901234567890".to_string())
        );
    }

    #[test]
    fn test_format_receipt_success() {
        let receipt = TransactionReceipt {
            tx_hash: "0xabc".to_string(),
            block_number: 1000,
            status: TransactionStatus::Success,
            gas_used: 21000,
            effective_gas_price: 1_000_000_000,
            contract_address: None,
            revert_reason: None,
        };

        let output = format_receipt(&receipt);
        assert!(output.contains("Transaction confirmed!"));
        assert!(output.contains("1000"));
        assert!(output.contains("21000"));
    }

    #[test]
    fn test_format_receipt_failure() {
        let receipt = TransactionReceipt {
            tx_hash: "0xabc".to_string(),
            block_number: 1000,
            status: TransactionStatus::Failure,
            gas_used: 21000,
            effective_gas_price: 1_000_000_000,
            contract_address: None,
            revert_reason: Some("Out of gas".to_string()),
        };

        let output = format_receipt(&receipt);
        assert!(output.contains("Transaction failed!"));
        assert!(output.contains("Out of gas"));
    }

    #[test]
    fn test_format_receipt_pending() {
        let receipt = TransactionReceipt {
            tx_hash: "0xabc".to_string(),
            block_number: 0,
            status: TransactionStatus::Pending,
            gas_used: 0,
            effective_gas_price: 0,
            contract_address: None,
            revert_reason: None,
        };

        let output = format_receipt(&receipt);
        assert!(output.contains("Transaction pending"));
    }

    #[test]
    fn test_default_constants() {
        assert_eq!(DEFAULT_POLL_INTERVAL_SECS, 2);
        assert_eq!(DEFAULT_TIMEOUT_SECS, 60);
    }

    // ===== Edge Case Tests =====

    #[test]
    fn test_parse_receipt_missing_fields() {
        // Receipt with only some fields present
        let raw = serde_json::json!({
            "blockNumber": "0x1234"
            // status, gasUsed, effectiveGasPrice missing
        });

        let receipt = parse_receipt("0xabc", &raw).unwrap();
        assert_eq!(receipt.tx_hash, "0xabc");
        assert_eq!(receipt.block_number, 0x1234);
        assert_eq!(receipt.status, TransactionStatus::Pending); // Default when missing
        assert_eq!(receipt.gas_used, 0); // Default when missing
        assert_eq!(receipt.effective_gas_price, 0); // Default when missing
    }

    #[test]
    fn test_parse_receipt_invalid_hex_block_number() {
        let raw = serde_json::json!({
            "blockNumber": "not_a_hex",
            "status": "0x1"
        });

        let result = parse_receipt("0xabc", &raw);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_receipt_invalid_hex_status() {
        let raw = serde_json::json!({
            "blockNumber": "0x1234",
            "status": "not_a_hex"
        });

        let result = parse_receipt("0xabc", &raw);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_receipt_empty_contract_address() {
        let raw = serde_json::json!({
            "blockNumber": "0x1234",
            "status": "0x1",
            "gasUsed": "0x5208",
            "effectiveGasPrice": "0x3b9aca00",
            "contractAddress": ""  // Empty string
        });

        let receipt = parse_receipt("0xabc", &raw).unwrap();
        assert!(receipt.contract_address.is_none()); // Empty string -> None
    }

    #[test]
    fn test_parse_receipt_without_contract_address() {
        let raw = serde_json::json!({
            "blockNumber": "0x1234",
            "status": "0x1",
            "gasUsed": "0x5208",
            "effectiveGasPrice": "0x3b9aca00"
            // No contractAddress field
        });

        let receipt = parse_receipt("0xabc", &raw).unwrap();
        assert!(receipt.contract_address.is_none());
    }

    #[test]
    fn test_parse_receipt_null_revert_reason() {
        let raw = serde_json::json!({
            "blockNumber": "0x1234",
            "status": "0x0",
            "gasUsed": "0x5208",
            "effectiveGasPrice": "0x3b9aca00",
            "revertReason": null
        });

        let receipt = parse_receipt("0xabc", &raw).unwrap();
        assert!(receipt.revert_reason.is_none());
    }

    #[test]
    fn test_parse_receipt_without_revert_reason() {
        let raw = serde_json::json!({
            "blockNumber": "0x1234",
            "status": "0x0",
            "gasUsed": "0x5208",
            "effectiveGasPrice": "0x3b9aca00"
            // No revertReason field
        });

        let receipt = parse_receipt("0xabc", &raw).unwrap();
        assert!(receipt.revert_reason.is_none());
    }

    #[test]
    fn test_parse_receipt_zero_values() {
        // Test that 0x0 is parsed correctly for gas/price fields
        let raw = serde_json::json!({
            "blockNumber": "0x0",
            "status": "0x1",
            "gasUsed": "0x0",
            "effectiveGasPrice": "0x0"
        });

        let receipt = parse_receipt("0xabc", &raw).unwrap();
        assert_eq!(receipt.block_number, 0);
        assert_eq!(receipt.gas_used, 0);
        assert_eq!(receipt.effective_gas_price, 0);
    }

    #[test]
    fn test_parse_receipt_large_values() {
        // Test large u64 values
        let raw = serde_json::json!({
            "blockNumber": "0xFFFFFFFF",
            "status": "0x1",
            "gasUsed": "0xFFFFFFFFFFFFFFFF",
            "effectiveGasPrice": "0xFFFFFFFFFFFFFFFF"
        });

        let receipt = parse_receipt("0xabc", &raw).unwrap();
        assert_eq!(receipt.block_number, 0xFFFFFFFF);
        assert_eq!(receipt.gas_used, 0xFFFFFFFFFFFFFFFF);
        assert_eq!(receipt.effective_gas_price, 0xFFFFFFFFFFFFFFFF);
    }

    #[test]
    fn test_receipt_config_with_status_callback() {
        let callback_called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let callback_called_clone = callback_called.clone();

        let config = ReceiptConfig::default().with_status_callback(move |_| {
            callback_called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
        });

        assert!(config.on_status.is_some());
        // Call the callback
        if let Some(ref callback) = config.on_status {
            callback(5);
        }
        assert!(callback_called.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn test_receipt_config_chain_building() {
        // Test fluent API chaining
        let callback_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let callback_count_clone = callback_count.clone();

        let config = ReceiptConfig::default()
            .with_poll_interval(10)
            .with_timeout(300)
            .with_status_callback(move |_| {
                callback_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            });

        assert_eq!(config.poll_interval, Duration::from_secs(10));
        assert_eq!(config.timeout, Duration::from_secs(300));
        assert!(config.on_status.is_some());
    }

    #[test]
    fn test_receipt_config_debug_format() {
        let config = ReceiptConfig::default().with_status_callback(|_| ());

        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("ReceiptConfig"));
        assert!(debug_str.contains("poll_interval"));
        assert!(debug_str.contains("timeout"));
        assert!(debug_str.contains("callback")); // on_status should show as "callback"
    }

    #[test]
    fn test_transaction_status_equality() {
        assert_eq!(TransactionStatus::Success, TransactionStatus::Success);
        assert_eq!(TransactionStatus::Failure, TransactionStatus::Failure);
        assert_eq!(TransactionStatus::Pending, TransactionStatus::Pending);
        assert_ne!(TransactionStatus::Success, TransactionStatus::Failure);
        assert_ne!(TransactionStatus::Success, TransactionStatus::Pending);
        assert_ne!(TransactionStatus::Failure, TransactionStatus::Pending);
    }

    #[test]
    fn test_transaction_receipt_clone() {
        let receipt = TransactionReceipt {
            tx_hash: "0xabc".to_string(),
            block_number: 1000,
            status: TransactionStatus::Success,
            gas_used: 21000,
            effective_gas_price: 1_000_000_000,
            contract_address: Some("0x1234".to_string()),
            revert_reason: Some("Error".to_string()),
        };

        let cloned = receipt.clone();
        assert_eq!(receipt.tx_hash, cloned.tx_hash);
        assert_eq!(receipt.block_number, cloned.block_number);
        assert_eq!(receipt.status, cloned.status);
        assert_eq!(receipt.gas_used, cloned.gas_used);
        assert_eq!(receipt.effective_gas_price, cloned.effective_gas_price);
        assert_eq!(receipt.contract_address, cloned.contract_address);
        assert_eq!(receipt.revert_reason, cloned.revert_reason);
    }
}
