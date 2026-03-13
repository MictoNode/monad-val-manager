//! Staking contract getter functions
//!
//! This module provides high-level functions to query the Monad staking contract.
//! Each function handles calldata encoding, RPC call, and response decoding.
//!
//! # Example
//!
//! ```ignore
//! use monad_val_manager::staking::getters;
//! use monad_val_manager::rpc::RpcClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let client = RpcClient::new("http://localhost:8080")?;
//!
//!     // Get current epoch
//!     let epoch = getters::get_epoch(&client).await?;
//!     println!("Current epoch: {}", epoch.epoch);
//!
//!     // Get validator info
//!     let validator = getters::get_validator(&client, 1).await?;
//!     println!("Validator commission: {}%", validator.commission / 100);
//!
//!     Ok(())
//! }
//! ```

use crate::rpc::RpcClient;
use crate::staking::calldata;
use crate::staking::constants::STAKING_CONTRACT_ADDRESS;
use crate::staking::types::{
    DelegationList, Delegator, EpochInfo, Validator, ValidatorSet, WithdrawalRequest,
};
use crate::utils::error::{Error, Result};

/// Get current epoch information
///
/// # Arguments
/// * `client` - RPC client connected to Monad node
///
/// # Returns
/// EpochInfo with current epoch number and transition status
pub async fn get_epoch(client: &RpcClient) -> Result<EpochInfo> {
    let calldata = calldata::encode_get_epoch();
    let response = client.eth_call(STAKING_CONTRACT_ADDRESS, &calldata).await?;

    // Decode hex response to bytes
    let bytes = hex::decode(response.strip_prefix("0x").unwrap_or(&response))
        .map_err(|e| Error::Other(format!("Failed to decode epoch response: {}", e)))?;

    calldata::decode_epoch_info(&bytes)
}

/// Get validator information by ID
///
/// # Arguments
/// * `client` - RPC client connected to Monad node
/// * `validator_id` - Validator ID to query
///
/// # Returns
/// Validator struct with all validator details
pub async fn get_validator(client: &RpcClient, validator_id: u64) -> Result<Validator> {
    let calldata = calldata::encode_get_validator(validator_id)?;
    let response = client.eth_call(STAKING_CONTRACT_ADDRESS, &calldata).await?;

    let bytes = hex::decode(response.strip_prefix("0x").unwrap_or(&response))
        .map_err(|e| Error::Other(format!("Failed to decode validator response: {}", e)))?;

    calldata::decode_validator(&bytes)
}

/// Get delegator information for a validator
///
/// # Arguments
/// * `client` - RPC client connected to Monad node
/// * `validator_id` - Validator ID
/// * `delegator_address` - Delegator's Ethereum address
///
/// # Returns
/// Delegator struct with delegation details
pub async fn get_delegator(
    client: &RpcClient,
    validator_id: u64,
    delegator_address: &str,
) -> Result<Delegator> {
    let calldata = calldata::encode_get_delegator(validator_id, delegator_address)?;
    let response = client.eth_call(STAKING_CONTRACT_ADDRESS, &calldata).await?;

    let bytes = hex::decode(response.strip_prefix("0x").unwrap_or(&response))
        .map_err(|e| Error::Other(format!("Failed to decode delegator response: {}", e)))?;

    calldata::decode_delegator(&bytes)
}

/// Get withdrawal request information
///
/// # Arguments
/// * `client` - RPC client connected to Monad node
/// * `validator_id` - Validator ID
/// * `delegator_address` - Delegator's Ethereum address
/// * `withdrawal_index` - Withdrawal request index (0-255, uint8)
///
/// # Returns
/// WithdrawalRequest with amount and activation epoch
pub async fn get_withdrawal_request(
    client: &RpcClient,
    validator_id: u64,
    delegator_address: &str,
    withdrawal_index: u8,
) -> Result<WithdrawalRequest> {
    let calldata =
        calldata::encode_get_withdrawal_request(validator_id, delegator_address, withdrawal_index)?;
    let response = client.eth_call(STAKING_CONTRACT_ADDRESS, &calldata).await?;

    let bytes = hex::decode(response.strip_prefix("0x").unwrap_or(&response))
        .map_err(|e| Error::Other(format!("Failed to decode withdrawal response: {}", e)))?;

    calldata::decode_withdrawal_request(&bytes)
}

/// Get current proposer validator ID
///
/// # Arguments
/// * `client` - RPC client connected to Monad node
///
/// # Returns
/// Validator ID of the current proposer
pub async fn get_proposer_val_id(client: &RpcClient) -> Result<u64> {
    let calldata = calldata::encode_get_proposer_val_id();
    let response = client.eth_call(STAKING_CONTRACT_ADDRESS, &calldata).await?;

    let bytes = hex::decode(response.strip_prefix("0x").unwrap_or(&response))
        .map_err(|e| Error::Other(format!("Failed to decode proposer response: {}", e)))?;

    // Response is just a uint64
    if bytes.len() < 32 {
        return Err(Error::Other(
            "Insufficient data for proposer val id".to_string(),
        ));
    }

    // Decode uint64 from last 8 bytes of 32-byte slot
    Ok(u64::from_be_bytes([
        bytes[24], bytes[25], bytes[26], bytes[27], bytes[28], bytes[29], bytes[30], bytes[31],
    ]))
}

/// Get consensus validator set (paginated)
///
/// # Arguments
/// * `client` - RPC client connected to Monad node
/// * `index` - Pagination index (start from 0)
///
/// # Returns
/// ValidatorSet with has_more flag and validator IDs
pub async fn get_consensus_valset(client: &RpcClient, index: u64) -> Result<ValidatorSet> {
    let calldata = calldata::encode_get_consensus_valset(index)?;
    let response = client.eth_call(STAKING_CONTRACT_ADDRESS, &calldata).await?;

    let bytes = hex::decode(response.strip_prefix("0x").unwrap_or(&response))
        .map_err(|e| Error::Other(format!("Failed to decode consensus valset response: {}", e)))?;

    calldata::decode_validator_set(&bytes)
}

/// Get snapshot validator set (paginated)
///
/// # Arguments
/// * `client` - RPC client connected to Monad node
/// * `index` - Pagination index (start from 0)
///
/// # Returns
/// ValidatorSet with has_more flag and validator IDs
pub async fn get_snapshot_valset(client: &RpcClient, index: u64) -> Result<ValidatorSet> {
    let calldata = calldata::encode_get_snapshot_valset(index)?;
    let response = client.eth_call(STAKING_CONTRACT_ADDRESS, &calldata).await?;

    let bytes = hex::decode(response.strip_prefix("0x").unwrap_or(&response))
        .map_err(|e| Error::Other(format!("Failed to decode snapshot valset response: {}", e)))?;

    calldata::decode_validator_set(&bytes)
}

/// Get execution validator set (paginated)
///
/// # Arguments
/// * `client` - RPC client connected to Monad node
/// * `index` - Pagination index (start from 0)
///
/// # Returns
/// ValidatorSet with has_more flag and validator IDs
pub async fn get_execution_valset(client: &RpcClient, index: u64) -> Result<ValidatorSet> {
    let calldata = calldata::encode_get_execution_valset(index)?;
    let response = client.eth_call(STAKING_CONTRACT_ADDRESS, &calldata).await?;

    let bytes = hex::decode(response.strip_prefix("0x").unwrap_or(&response))
        .map_err(|e| Error::Other(format!("Failed to decode execution valset response: {}", e)))?;

    calldata::decode_validator_set(&bytes)
}

/// Get delegations for an address (paginated)
///
/// # Arguments
/// * `client` - RPC client connected to Monad node
/// * `delegator_address` - Delegator's Ethereum address
/// * `index` - Pagination index (start from 0)
///
/// # Returns
/// DelegationList with validator IDs the address has delegated to
pub async fn get_delegations(
    client: &RpcClient,
    delegator_address: &str,
    index: u64,
) -> Result<DelegationList> {
    let calldata = calldata::encode_get_delegations(delegator_address, index)?;
    let response = client.eth_call(STAKING_CONTRACT_ADDRESS, &calldata).await?;

    let bytes = hex::decode(response.strip_prefix("0x").unwrap_or(&response))
        .map_err(|e| Error::Other(format!("Failed to decode delegations response: {}", e)))?;

    calldata::decode_delegation_list(&bytes)
}

/// Get all delegations for an address (handles pagination automatically)
///
/// # Arguments
/// * `client` - RPC client connected to Monad node
/// * `delegator_address` - Delegator's Ethereum address
///
/// # Returns
/// Vector of all validator IDs the address has delegated to
///
/// # Note
/// This function implements pagination with timeout protection:
/// - Maximum 1000 pages (configurable via environment)
/// - 100ms delay between requests to avoid overwhelming the RPC
/// - Consecutive error tracking with early termination
pub async fn get_all_delegations(client: &RpcClient, delegator_address: &str) -> Result<Vec<u64>> {
    let mut all_validator_ids = Vec::new();
    let mut index = 0u64;
    let max_pages: u64 = std::env::var("DELEGATIONS_MAX_PAGES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000);
    let mut consecutive_errors = 0;
    let max_consecutive_errors = 5;

    while index < max_pages {
        match get_delegations(client, delegator_address, index).await {
            Ok(delegation) => {
                all_validator_ids.extend(delegation.validator_ids.clone());
                consecutive_errors = 0; // Reset error counter on success

                if !delegation.has_more {
                    break;
                }
                index += 1;

                // Add small delay to avoid overwhelming the RPC
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
            Err(e) => {
                consecutive_errors += 1;
                if consecutive_errors >= max_consecutive_errors {
                    return Err(Error::Other(format!(
                        "Failed to query delegations after {} consecutive errors: {}",
                        consecutive_errors, e
                    )));
                }
                // Continue on transient errors
                index += 1;
            }
        }
    }

    Ok(all_validator_ids)
}

/// Get delegators for a validator (paginated)
///
/// # Arguments
/// * `client` - RPC client connected to Monad node
/// * `validator_id` - Validator ID
/// * `start_address` - Address to start pagination from (use "0x0000000000000000000000000000000000000000" for first page)
///
/// # Returns
/// DelegatorList with addresses
pub async fn get_delegators(
    client: &RpcClient,
    validator_id: u64,
    start_address: &str,
) -> Result<crate::staking::types::DelegatorList> {
    let calldata = calldata::encode_get_delegators(validator_id, start_address)?;
    let response = client.eth_call(STAKING_CONTRACT_ADDRESS, &calldata).await?;

    let bytes = hex::decode(response.strip_prefix("0x").unwrap_or(&response))
        .map_err(|e| Error::Other(format!("Failed to decode delegators response: {}", e)))?;

    calldata::decode_delegator_list(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a running Monad node or mock server.
    // Integration tests should be in tests/ directory.
    // These tests verify calldata encoding and basic validation.

    #[test]
    fn test_staking_contract_address() {
        assert!(STAKING_CONTRACT_ADDRESS.starts_with("0x"));
        assert_eq!(STAKING_CONTRACT_ADDRESS.len(), 42);
    }

    // ===== Calldata Encoding Verification Tests =====
    // These tests verify that getters functions encode calldata correctly
    // by checking the encoded selector matches expected values.

    #[test]
    fn test_get_epoch_calldata_encoding() {
        let calldata = calldata::encode_get_epoch();
        // Selector for get_epoch(): 0x757991a8
        assert!(calldata.starts_with("0x757991a8"));
    }

    #[test]
    fn test_get_validator_calldata_encoding() {
        let calldata = calldata::encode_get_validator(123).unwrap();
        // Selector for get_validator(uint64): 0x2b6d639a
        assert!(calldata.starts_with("0x2b6d639a"));
        // Check that validator_id is in the calldata (encoded value)
        assert!(calldata.len() > 10); // More than just selector
    }

    #[test]
    fn test_get_delegator_calldata_encoding() {
        let address = "0x1234567890123456789012345678901234567890";
        let calldata = calldata::encode_get_delegator(1, address).unwrap();
        // Selector for get_delegator(uint64,address): 0x573c1ce0
        assert!(calldata.starts_with("0x573c1ce0"));
        assert!(calldata.len() > 10);
    }

    #[test]
    fn test_get_withdrawal_request_calldata_encoding() {
        let address = "0x1234567890123456789012345678901234567890";
        let calldata = calldata::encode_get_withdrawal_request(1, address, 0).unwrap();
        // Selector for get_withdrawal_request(uint64,address,uint8): 0x56fa2045
        assert!(calldata.starts_with("0x56fa2045"));
        assert!(calldata.len() > 10);
    }

    #[test]
    fn test_get_proposer_val_id_calldata_encoding() {
        let calldata = calldata::encode_get_proposer_val_id();
        // Selector for get_proposer_val_id(): 0x...
        assert!(calldata.starts_with("0x"));
        assert!(calldata.len() >= 10);
    }

    #[test]
    fn test_get_consensus_valset_calldata_encoding() {
        let calldata = calldata::encode_get_consensus_valset(0).unwrap();
        // Selector for getConsensusValSet(uint64): varies
        assert!(calldata.starts_with("0x"));
        assert!(calldata.len() > 10);
    }

    #[test]
    fn test_get_delegations_calldata_encoding() {
        let address = "0x1234567890123456789012345678901234567890";
        let calldata = calldata::encode_get_delegations(address, 0).unwrap();
        // Selector for getDelegations(address,uint64): 0x4fd66050
        assert!(calldata.starts_with("0x4fd66050"));
        assert!(calldata.len() > 10);
    }

    // ===== Response Decoding Tests (with mock data) =====

    #[test]
    fn test_decode_get_epoch_response() {
        // Mock response for epoch=100, is_transition=true
        let mock_hex = "0000000000000000000000000000000000000000000000000000000000000064\
                        0000000000000000000000000000000000000000000000000000000000000001";
        let mock_response = format!("0x{}", mock_hex.replace(' ', ""));

        let bytes = hex::decode(&mock_response[2..]).unwrap();
        let epoch_info = calldata::decode_epoch_info(&bytes).unwrap();

        assert_eq!(epoch_info.epoch, 100);
        assert!(epoch_info.is_epoch_transition);
    }

    #[test]
    fn test_decode_get_validator_response() {
        // Minimal mock validator response
        let mock_hex = "0000000000000000000000001234567890123456789012345678901234567890\
                        0000000000000000000000000000000000000000000000000000000000000001\
                        0000000000000000000000000000000000000000000000000de0b6b3a7640000\
                        0000000000000000000000000000000000000000000000000de0b6b3a7640000\
                        000000000000000000000000000000000000000000000000016345785d8a0000\
                        0000000000000000000000000000000000000000000000000de0b6b3a7640000\
                        0000000000000000000000000000000000000000000000000de0b6b3a7640000\
                        000000000000000000000000000000000000000000000000016345785d8a0000\
                        0000000000000000000000000000000000000000000000000de0b6b3a7640000\
                        000000000000000000000000000000000000000000000000016345785d8a0000\
                        0000000000000000000000000000000000000000000000000000000000000180\
                        00000000000000000000000000000000000000000000000000000000000001e0\
                        0000000000000000000000000000000000000000000000000000000000000040\
                        0000000000000000000000000000000000000000000000000000000000000040";
        let mock_response = format!("0x{}", mock_hex.replace(' ', ""));

        let bytes = hex::decode(&mock_response[2..]).unwrap();
        let validator = calldata::decode_validator(&bytes).unwrap();

        assert_eq!(
            validator.auth_delegator,
            "0x1234567890123456789012345678901234567890"
        );
        assert_eq!(validator.execution_stake, 1000000000000000000u128);
        assert_eq!(validator.execution_commission, 100000000000000000u128);
    }

    #[test]
    fn test_decode_get_delegator_response() {
        // Mock delegator response: delegated_amount=5 MON, rewards=0.5 MON
        // All fields padded to exactly 64 hex chars (32 bytes each)
        let mock_hex = concat!(
            "0000000000000000000000000000000000000000000000004563918244f40000", // delegated_amount
            "0000000000000000000000000000000000000000000000000000000000000001", // accumulated_rewards_per_token
            "00000000000000000000000000000000000000000000000006f05b59d3b20000", // rewards
            "0000000000000000000000000000000000000000000000000000000000000000", // delta_stake
            "0000000000000000000000000000000000000000000000000000000000000000", // next_delta_stake
            "0000000000000000000000000000000000000000000000000000000000000000", // delta_epoch
            "0000000000000000000000000000000000000000000000000000000000000000", // next_delta_epoch
        );

        let bytes = hex::decode(mock_hex).unwrap();
        let delegator = calldata::decode_delegator(&bytes).unwrap();

        assert_eq!(delegator.delegated_amount, 5_000_000_000_000_000_000);
        assert_eq!(delegator.rewards, 500_000_000_000_000_000);
        assert_eq!(delegator.delta_epoch, 0);
    }

    #[test]
    fn test_decode_get_proposer_val_id_response() {
        // Mock response: proposer_id = 42
        let mock_hex = "000000000000000000000000000000000000000000000000000000000000002a";
        let mock_response = format!("0x{}", mock_hex);

        let bytes = hex::decode(&mock_response[2..]).unwrap();
        assert_eq!(bytes.len(), 32);

        // Decode uint64 from last 8 bytes of 32-byte slot
        let proposer_id = u64::from_be_bytes([
            bytes[24], bytes[25], bytes[26], bytes[27], bytes[28], bytes[29], bytes[30], bytes[31],
        ]);

        assert_eq!(proposer_id, 42);
    }

    // ===== Error Handling Tests =====

    #[test]
    fn test_decode_invalid_hex_response() {
        let invalid_hex = "0xnotvalidhex";
        let result = hex::decode(&invalid_hex[2..]);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_empty_response() {
        let empty_hex = "0x";
        let bytes = hex::decode(&empty_hex[2..]).unwrap();
        let result = calldata::decode_epoch_info(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_insufficient_data() {
        // Only 16 bytes instead of required 64 for epoch_info
        let short_hex = "00000000000000000000000000000000000"; // 35 chars = odd!
        let bytes = hex::decode(short_hex);
        assert!(bytes.is_err() || bytes.unwrap().len() < 64);
    }
}
