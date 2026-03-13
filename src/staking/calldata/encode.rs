//! ABI calldata encoding for staking operations
//!
//! This module provides functions to encode calldata for Monad staking contract calls.
//! Each function returns a hex-encoded string ready to be used as transaction data.
//!
//! # ABI Encoding Rules
//!
//! - All values are 32-byte aligned
//! - uint64, uint256, uint8 are left-padded with zeros
//! - address is left-padded to 32 bytes
//! - Dynamic types (bytes, arrays) use pointer-based encoding

use crate::staking::constants;
use crate::utils::error::{Error, Result};

// =============================================================================
// ENCODING PRIMITIVES
// =============================================================================

/// Encode a uint64 as 32-byte hex string (big-endian, left-padded)
pub(crate) fn encode_uint64(value: u64) -> String {
    format!("{:064x}", value)
}

/// Encode a uint128 as 32-byte hex string (big-endian, left-padded)
pub(crate) fn encode_uint128(value: u128) -> String {
    format!("{:064x}", value)
}

/// Encode a uint8 as 32-byte hex string (left-padded)
pub(crate) fn encode_uint8(value: u8) -> String {
    format!("{:064x}", value)
}

/// Encode an Ethereum address as 32-byte hex string (left-padded)
/// Address can be with or without 0x prefix
pub(crate) fn encode_address(address: &str) -> Result<String> {
    let clean = address.strip_prefix("0x").unwrap_or(address);
    if clean.len() != 40 {
        return Err(Error::Other(format!(
            "Invalid address length: {}",
            clean.len()
        )));
    }
    // Left-pad address to 32 bytes (64 hex chars)
    Ok(format!("{:0>64}", clean))
}

/// Encode raw bytes as hex (no padding)
#[allow(dead_code)]
pub(crate) fn encode_bytes(data: &[u8]) -> String {
    hex::encode(data)
}

/// Calculate padded length to next 32-byte boundary
pub(crate) fn pad_to_32(len: usize) -> usize {
    len.div_ceil(32) * 32
}

/// Encode bytes with length prefix and padding
pub(crate) fn encode_bytes_dynamic(data: &[u8]) -> String {
    let padded_len = pad_to_32(data.len());
    let mut result = String::with_capacity(padded_len * 2);
    for byte in data {
        result.push_str(&format!("{:02x}", byte));
    }
    // Pad to 32-byte boundary
    for _ in data.len()..padded_len {
        result.push_str("00");
    }
    result
}

// =============================================================================
// WRITE OPERATIONS - State-modifying calldata encoding
// =============================================================================

/// Encode calldata for `delegate(uint64 validator_id)` operation
///
/// # Arguments
/// * `validator_id` - The validator ID to delegate to
/// * `amount` - Amount of MON to delegate (in wei) - sent as transaction value
///
/// # Returns
/// Hex-encoded calldata string with 0x prefix
///
/// # Example
/// ```ignore
/// let data = encode_delegate(1)?;
/// // Returns: 0x84994fec + ABI-encoded uint64(1)
/// ```
pub fn encode_delegate(validator_id: u64) -> Result<String> {
    let selector = constants::DELEGATE_SELECTOR;
    let encoded_id = encode_uint64(validator_id);
    Ok(format!("0x{}{}", selector, encoded_id))
}

/// Encode calldata for `undelegate(uint64 validator_id, uint256 amount, uint8 withdrawal_index)` operation
///
/// # Arguments
/// * `validator_id` - The validator ID to undelegate from
/// * `amount` - Amount of MON to undelegate (in wei)
/// * `withdrawal_index` - Index for this withdrawal (0-255, uint8)
///
/// # Returns
/// Hex-encoded calldata string with 0x prefix
pub fn encode_undelegate(validator_id: u64, amount: u128, withdrawal_index: u8) -> Result<String> {
    let selector = constants::UNDELEGATE_SELECTOR;
    let encoded_id = encode_uint64(validator_id);
    let encoded_amount = encode_uint128(amount);
    let encoded_index = encode_uint8(withdrawal_index);
    Ok(format!(
        "0x{}{}{}{}",
        selector, encoded_id, encoded_amount, encoded_index
    ))
}

/// Encode calldata for `withdraw(uint64 validator_id, uint8 withdrawal_index)` operation
///
/// # Arguments
/// * `validator_id` - The validator ID to withdraw from
/// * `withdrawal_index` - Index of the withdrawal request to process (0-255, uint8)
///
/// # Returns
/// Hex-encoded calldata string with 0x prefix
pub fn encode_withdraw(validator_id: u64, withdrawal_index: u8) -> Result<String> {
    let selector = constants::WITHDRAW_SELECTOR;
    let encoded_id = encode_uint64(validator_id);
    let encoded_index = encode_uint8(withdrawal_index);
    Ok(format!("0x{}{}{}", selector, encoded_id, encoded_index))
}

/// Encode calldata for `claim_rewards(uint64 validator_id)` operation
///
/// # Arguments
/// * `validator_id` - The validator ID to claim rewards from
///
/// # Returns
/// Hex-encoded calldata string with 0x prefix
pub fn encode_claim_rewards(validator_id: u64) -> Result<String> {
    let selector = constants::CLAIM_REWARDS_SELECTOR;
    let encoded_id = encode_uint64(validator_id);
    Ok(format!("0x{}{}", selector, encoded_id))
}

/// Encode calldata for `compound(uint64 validator_id)` operation
///
/// Compounds pending rewards back into delegation.
///
/// # Arguments
/// * `validator_id` - The validator ID to compound rewards for
///
/// # Returns
/// Hex-encoded calldata string with 0x prefix
pub fn encode_compound(validator_id: u64) -> Result<String> {
    let selector = constants::COMPOUND_SELECTOR;
    let encoded_id = encode_uint64(validator_id);
    Ok(format!("0x{}{}", selector, encoded_id))
}

/// Encode calldata for `change_commission(uint64 validator_id, uint256 commission)` operation
///
/// # Arguments
/// * `validator_id` - The validator ID to change commission for
/// * `commission_bps` - New commission rate in basis points (100 = 1%)
///
/// # Returns
/// Hex-encoded calldata string with 0x prefix
pub fn encode_change_commission(validator_id: u64, commission_bps: u64) -> Result<String> {
    let selector = constants::CHANGE_COMMISSION_SELECTOR;
    let encoded_id = encode_uint64(validator_id);
    let encoded_commission = encode_uint64(commission_bps);
    Ok(format!(
        "0x{}{}{}",
        selector, encoded_id, encoded_commission
    ))
}

/// Encode calldata for `add_validator(bytes, bytes, bytes)` operation
///
/// Registers a new validator. Requires signed payload with BLS and Secp keys.
///
/// # Arguments
/// * `payload` - The validator registration payload
/// * `secp_signature` - Secp256k1 signature of the payload
/// * `bls_signature` - BLS12-381 signature of the payload
///
/// # Returns
/// Hex-encoded calldata string with 0x prefix
pub fn encode_add_validator(
    payload: &[u8],
    secp_signature: &[u8],
    bls_signature: &[u8],
) -> Result<String> {
    let selector = constants::ADD_VALIDATOR_SELECTOR;

    // ABI encoding for dynamic bytes arrays:
    // 1. Offset to first bytes (32 bytes)
    // 2. Offset to second bytes (32 bytes)
    // 3. Offset to third bytes (32 bytes)
    // 4. First bytes: length (32 bytes) + data (padded to 32 bytes)
    // 5. Second bytes: length (32 bytes) + data (padded to 32 bytes)
    // 6. Third bytes: length (32 bytes) + data (padded to 32 bytes)

    let offset1 = encode_uint64(96); // 3 * 32 = 96 (offset to first dynamic param)
    let payload_len = payload.len();
    let secp_len = secp_signature.len();
    let bls_len = bls_signature.len();

    // Calculate offsets
    let offset2 = 96 + 32 + pad_to_32(payload_len); // after payload
    let offset3 = offset2 + 32 + pad_to_32(secp_len); // after secp_sig

    let encoded_payload = encode_bytes_dynamic(payload);
    let encoded_secp = encode_bytes_dynamic(secp_signature);
    let encoded_bls = encode_bytes_dynamic(bls_signature);

    Ok(format!(
        "0x{}{}{}{}{}{}{}{}{}{}",
        selector,
        offset1,
        encode_uint64(offset2 as u64),
        encode_uint64(offset3 as u64),
        encode_uint64(payload_len as u64),
        encoded_payload,
        encode_uint64(secp_len as u64),
        encoded_secp,
        encode_uint64(bls_len as u64),
        encoded_bls
    ))
}

// =============================================================================
// READ OPERATIONS - View calldata encoding
// =============================================================================

/// Encode calldata for `get_epoch()` view call
///
/// # Returns
/// Hex-encoded calldata string with 0x prefix (selector only, no arguments)
pub fn encode_get_epoch() -> String {
    format!("0x{}", constants::GET_EPOCH_SELECTOR)
}

/// Encode calldata for `get_validator(uint64 validator_id)` view call
///
/// # Arguments
/// * `validator_id` - The validator ID to query
///
/// # Returns
/// Hex-encoded calldata string with 0x prefix
pub fn encode_get_validator(validator_id: u64) -> Result<String> {
    let selector = constants::GET_VALIDATOR_SELECTOR;
    let encoded_id = encode_uint64(validator_id);
    Ok(format!("0x{}{}", selector, encoded_id))
}

/// Encode calldata for `get_delegator(uint64 validator_id, address delegator)` view call
///
/// # Arguments
/// * `validator_id` - The validator ID
/// * `delegator_address` - The delegator's Ethereum address (hex string)
///
/// # Returns
/// Hex-encoded calldata string with 0x prefix
pub fn encode_get_delegator(validator_id: u64, delegator_address: &str) -> Result<String> {
    let selector = constants::GET_DELEGATOR_SELECTOR;
    let encoded_id = encode_uint64(validator_id);
    let encoded_address = encode_address(delegator_address)?;
    Ok(format!("0x{}{}{}", selector, encoded_id, encoded_address))
}

/// Encode calldata for `get_withdrawal_request(uint64 validator_id, address delegator, uint8 withdrawal_id)` view call
///
/// # Arguments
/// * `validator_id` - The validator ID
/// * `delegator_address` - The delegator's Ethereum address (hex string)
/// * `withdrawal_index` - The withdrawal request index (0-255, uint8)
///
/// # Returns
/// Hex-encoded calldata string with 0x prefix
pub fn encode_get_withdrawal_request(
    validator_id: u64,
    delegator_address: &str,
    withdrawal_index: u8,
) -> Result<String> {
    let selector = constants::GET_WITHDRAWAL_REQUEST_SELECTOR;
    let encoded_id = encode_uint64(validator_id);
    let encoded_address = encode_address(delegator_address)?;
    let encoded_index = encode_uint8(withdrawal_index);
    Ok(format!(
        "0x{}{}{}{}",
        selector, encoded_id, encoded_address, encoded_index
    ))
}

/// Encode calldata for `get_delegations(address delegator, uint64 index)` view call
///
/// # Arguments
/// * `delegator_address` - The delegator's Ethereum address (hex string)
/// * `index` - Pagination index (start from 0)
///
/// # Returns
/// Hex-encoded calldata string with 0x prefix
pub fn encode_get_delegations(delegator_address: &str, index: u64) -> Result<String> {
    let selector = constants::GET_DELEGATIONS_SELECTOR;
    let encoded_address = encode_address(delegator_address)?;
    let encoded_index = encode_uint64(index);
    Ok(format!(
        "0x{}{}{}",
        selector, encoded_address, encoded_index
    ))
}

/// Encode calldata for `get_delegators(uint64 validator_id, address start_address)` view call
///
/// # Arguments
/// * `validator_id` - The validator ID
/// * `start_address` - Address to start pagination from (use 0x0...0 for first page)
///
/// # Returns
/// Hex-encoded calldata string with 0x prefix
pub fn encode_get_delegators(validator_id: u64, start_address: &str) -> Result<String> {
    let selector = constants::GET_DELEGATORS_SELECTOR;
    let encoded_id = encode_uint64(validator_id);
    let encoded_address = encode_address(start_address)?;
    Ok(format!("0x{}{}{}", selector, encoded_id, encoded_address))
}

/// Encode calldata for `get_proposer_val_id()` view call
///
/// # Returns
/// Hex-encoded calldata string with 0x prefix (selector only)
pub fn encode_get_proposer_val_id() -> String {
    format!("0x{}", constants::GET_PROPOSER_VAL_ID_SELECTOR)
}

/// Encode calldata for `get_consensus_valset(uint64 index)` view call
///
/// # Arguments
/// * `index` - Pagination index
///
/// # Returns
/// Hex-encoded calldata string with 0x prefix
pub fn encode_get_consensus_valset(index: u64) -> Result<String> {
    let selector = constants::GET_CONSENSUS_VALSET_SELECTOR;
    let encoded_index = encode_uint64(index);
    Ok(format!("0x{}{}", selector, encoded_index))
}

/// Encode calldata for `get_snapshot_valset(uint64 index)` view call
///
/// # Arguments
/// * `index` - Pagination index
///
/// # Returns
/// Hex-encoded calldata string with 0x prefix
pub fn encode_get_snapshot_valset(index: u64) -> Result<String> {
    let selector = constants::GET_SNAPSHOT_VALSET_SELECTOR;
    let encoded_index = encode_uint64(index);
    Ok(format!("0x{}{}", selector, encoded_index))
}

/// Encode calldata for `get_execution_valset(uint64 index)` view call
///
/// # Arguments
/// * `index` - Pagination index
///
/// # Returns
/// Hex-encoded calldata string with 0x prefix
pub fn encode_get_execution_valset(index: u64) -> Result<String> {
    let selector = constants::GET_EXECUTION_VALSET_SELECTOR;
    let encoded_index = encode_uint64(index);
    Ok(format!("0x{}{}", selector, encoded_index))
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_uint64() {
        let result = encode_uint64(1);
        assert_eq!(result.len(), 64);
        assert_eq!(
            result,
            "0000000000000000000000000000000000000000000000000000000000000001"
        );
    }

    #[test]
    fn test_encode_uint64_max() {
        let result = encode_uint64(u64::MAX);
        assert_eq!(result.len(), 64);
        // uint64 max is 0xffffffffffffffff, right-aligned in 32 bytes
        assert_eq!(
            result,
            "000000000000000000000000000000000000000000000000ffffffffffffffff"
        );
    }

    #[test]
    fn test_encode_address() {
        let result = encode_address("0x1234567890123456789012345678901234567890").unwrap();
        assert_eq!(result.len(), 64);
        assert!(result.ends_with("1234567890123456789012345678901234567890"));
    }

    #[test]
    fn test_encode_address_no_prefix() {
        let result = encode_address("1234567890123456789012345678901234567890").unwrap();
        assert_eq!(result.len(), 64);
    }

    #[test]
    fn test_encode_address_invalid() {
        let result = encode_address("0x123");
        assert!(result.is_err());
    }

    #[test]
    fn test_encode_get_epoch() {
        let result = encode_get_epoch();
        assert!(result.starts_with("0x"));
        assert_eq!(&result[2..], constants::GET_EPOCH_SELECTOR);
    }

    #[test]
    fn test_encode_get_proposer_val_id() {
        let result = encode_get_proposer_val_id();
        assert!(result.starts_with("0x"));
        assert_eq!(&result[2..], constants::GET_PROPOSER_VAL_ID_SELECTOR);
    }

    #[test]
    fn test_encode_delegate() {
        let result = encode_delegate(1).unwrap();
        assert!(result.starts_with("0x"));
        assert!(result.starts_with(&format!("0x{}", constants::DELEGATE_SELECTOR)));
        // Should have selector (8 chars) + uint64 (64 chars) = 72 chars after 0x
        assert_eq!(result.len(), 2 + 8 + 64);
    }

    #[test]
    fn test_encode_undelegate() {
        let result = encode_undelegate(1, 1000000000000000000u128, 0).unwrap();
        assert!(result.starts_with("0x"));
        assert!(result.starts_with(&format!("0x{}", constants::UNDELEGATE_SELECTOR)));
        // selector (8) + uint64 (64) + uint256 (64) + uint8 (64) = 200 chars after 0x
        assert_eq!(result.len(), 2 + 8 + 64 + 64 + 64);
    }

    #[test]
    fn test_encode_get_validator() {
        let result = encode_get_validator(42).unwrap();
        assert!(result.starts_with("0x"));
        assert!(result.starts_with(&format!("0x{}", constants::GET_VALIDATOR_SELECTOR)));
        assert_eq!(result.len(), 2 + 8 + 64);
    }

    #[test]
    fn test_encode_get_delegator() {
        let result = encode_get_delegator(1, "0x1234567890123456789012345678901234567890").unwrap();
        assert!(result.starts_with("0x"));
        assert!(result.starts_with(&format!("0x{}", constants::GET_DELEGATOR_SELECTOR)));
        // selector (8) + uint64 (64) + address (64) = 136 chars after 0x
        assert_eq!(result.len(), 2 + 8 + 64 + 64);
    }

    // ===== Enhanced Tests with Content Validation =====

    #[test]
    fn test_encode_delegate_content_validation() {
        let result = encode_delegate(123).unwrap();
        // Verify selector
        assert!(result.starts_with(&format!("0x{}", constants::DELEGATE_SELECTOR)));
        // Verify validator_id is properly encoded (should be ...0000007b for 123)
        assert!(
            result.ends_with("000000000000000000000000000000000000000000000000000000000000007b")
        );
    }

    #[test]
    fn test_encode_delegate_zero_validator_id() {
        // Edge case: validator_id = 0
        let result = encode_delegate(0).unwrap();
        assert!(result.starts_with("0x"));
        // Should end with all zeros for validator_id
        assert!(result.ends_with(&"0".repeat(64)));
    }

    #[test]
    fn test_encode_delegate_max_validator_id() {
        // Edge case: validator_id = u64::MAX
        let result = encode_delegate(u64::MAX).unwrap();
        assert!(result.starts_with("0x"));
        // u64::MAX = 0xffffffffffffffff, should be encoded at the end (right-aligned in 32 bytes)
        assert!(result.ends_with("ffffffffffffffff"));
    }

    #[test]
    fn test_encode_withdraw_content_validation() {
        let result = encode_withdraw(1, 5).unwrap();
        assert!(result.starts_with("0x"));
        // Verify selector
        assert!(result.starts_with(&format!("0x{}", constants::WITHDRAW_SELECTOR)));
        // Should end with withdrawal_index (uint8)
        // 5 in uint64 encoding is ...00000005
        assert!(
            result.ends_with("0000000000000000000000000000000000000000000000000000000000000005")
        );
    }

    #[test]
    fn test_encode_claim_rewards_content_validation() {
        let result = encode_claim_rewards(999).unwrap();
        assert!(result.starts_with("0x"));
        // Verify validator_id is properly encoded
        // 999 = 0x3e7
        assert!(
            result.ends_with("00000000000000000000000000000000000000000000000000000000000003e7")
        );
    }

    #[test]
    fn test_encode_compound_content_validation() {
        let result = encode_compound(42).unwrap();
        assert!(result.starts_with("0x"));
        // 42 = 0x2a
        assert!(
            result.ends_with("000000000000000000000000000000000000000000000000000000000000002a")
        );
    }

    #[test]
    fn test_encode_change_commission_content_validation() {
        let result = encode_change_commission(100, 500).unwrap();
        assert!(result.starts_with("0x"));
        // validator_id = 100 (0x64), commission = 500 (0x1f4)
        let expected_validator = "0000000000000000000000000000000000000000000000000000000000000064";
        let expected_commission =
            "00000000000000000000000000000000000000000000000000000000000001f4";
        let after_selector = &result[10..]; // Skip 0x + selector (8 chars)
        assert_eq!(&after_selector[0..64], expected_validator);
        assert_eq!(&after_selector[64..128], expected_commission);
    }

    #[test]
    fn test_encode_change_commission_max_commission() {
        // Edge case: 100% commission (10000 basis points = 100%)
        let result = encode_change_commission(1, 10000).unwrap();
        assert!(result.starts_with("0x"));
        // 10000 = 0x2710
        assert!(
            result.ends_with("0000000000000000000000000000000000000000000000000000000000002710")
        );
    }

    #[test]
    fn test_encode_undelegate_content_validation() {
        let amount = 1_000_000_000_000_000_000u128; // 1 MON
        let result = encode_undelegate(1, amount, 3).unwrap();
        assert!(result.starts_with("0x"));

        // Verify structure: selector + validator_id + amount + withdrawal_index
        assert!(result.contains("de0b6b3a7640000")); // 1 MON in wei (hex)
        assert!(
            result.ends_with("0000000000000000000000000000000000000000000000000000000000000003")
        ); // withdrawal_index = 3
    }

    #[test]
    fn test_encode_undelegate_max_amount() {
        // Edge case: max u128 amount - just verify it doesn't panic
        let result = encode_undelegate(1, u128::MAX, 0);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.starts_with("0x"));
        assert_eq!(result.len(), 2 + 8 + 64 + 64 + 64); // selector + 3 params
    }

    #[test]
    fn test_encode_undelegate_max_withdrawal_index() {
        // Edge case: max u8 withdrawal index (255)
        let result = encode_undelegate(1, 1000, 255).unwrap();
        assert!(result.starts_with("0x"));
        // 255 = 0xff
        assert!(
            result.ends_with("00000000000000000000000000000000000000000000000000000000000000ff")
        );
    }

    #[test]
    fn test_encode_get_delegations_content_validation() {
        let result =
            encode_get_delegations("0xabcdabcdabcdabcdabcdabcdabcdabcdabcdabcd", 5).unwrap();
        assert!(result.starts_with("0x"));

        // Verify the encoded data contains our address and index
        assert!(result.contains("abcdabcdabcdabcdabcdabcdabcdabcdabcdabcd"));
        // Verify index encoding - 5 = 0x5
        assert!(
            result.ends_with("0000000000000000000000000000000000000000000000000000000000000005")
        );
    }

    #[test]
    fn test_encode_get_delegations_zero_index() {
        // Edge case: index = 0
        let result =
            encode_get_delegations("0x0000000000000000000000000000000000000000", 0).unwrap();
        assert!(result.starts_with("0x"));
        assert!(result.ends_with(&"0".repeat(64)));
    }

    #[test]
    fn test_encode_get_delegators_content_validation() {
        let result =
            encode_get_delegators(123, "0x0000000000000000000000000000000000000000").unwrap();
        assert!(result.starts_with("0x"));

        let after_selector = &result[10..];

        // Verify validator_id - 123 = 0x7b
        assert_eq!(
            &after_selector[0..64],
            "000000000000000000000000000000000000000000000000000000000000007b"
        );

        // Verify start address encoding
        assert_eq!(
            &after_selector[64..128],
            "0000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn test_encode_get_withdrawal_request_content_validation() {
        let result =
            encode_get_withdrawal_request(1, "0xabcdabcdabcdabcdabcdabcdabcdabcdabcdabcd", 7)
                .unwrap();
        assert!(result.starts_with("0x"));

        let after_selector = &result[10..];

        // Verify validator_id
        assert_eq!(
            &after_selector[0..64],
            "0000000000000000000000000000000000000000000000000000000000000001"
        );

        // Verify delegator address
        assert!(after_selector[64..128].contains("abcdabcdabcdabcdabcdabcdabcdabcdabcdabcd"));

        // Verify withdrawal_index - 7 = 0x7
        assert_eq!(
            &after_selector[128..192],
            "0000000000000000000000000000000000000000000000000000000000000007"
        );
    }

    #[test]
    fn test_encode_get_withdrawal_request_max_withdrawal_id() {
        // Edge case: max u8 withdrawal_id (255)
        let result =
            encode_get_withdrawal_request(1, "0x0000000000000000000000000000000000000000", 255)
                .unwrap();
        assert!(result.starts_with("0x"));
        assert!(
            result.ends_with("00000000000000000000000000000000000000000000000000000000000000ff")
        );
    }
}
