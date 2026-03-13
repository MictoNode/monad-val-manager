//! ABI calldata decoding for staking operations
//!
//! This module provides functions to decode responses from Monad staking contract calls.
//! All decoding follows standard ABI encoding rules with 32-byte alignment.

use crate::staking::types;
use crate::utils::error::{Error, Result};

// =============================================================================
// DECODING PRIMITIVES
// =============================================================================

/// Decode a hex string to bytes, handling 0x prefix (used in tests)
#[allow(dead_code)]
pub(crate) fn hex_to_bytes(hex_str: &str) -> Result<Vec<u8>> {
    let clean = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    hex::decode(clean).map_err(|e| Error::Other(format!("Hex decode error: {}", e)))
}

/// Decode uint64 from 32-byte big-endian bytes
pub(crate) fn decode_uint64(bytes: &[u8]) -> Result<u64> {
    if bytes.len() < 32 {
        return Err(Error::Other("Insufficient bytes for uint64".to_string()));
    }
    // Take last 8 bytes from the 32-byte slot
    Ok(u64::from_be_bytes([
        bytes[24], bytes[25], bytes[26], bytes[27], bytes[28], bytes[29], bytes[30], bytes[31],
    ]))
}

/// Decode uint128 from 32-byte big-endian bytes
///
/// Note: For uint256 values that fit in u128, this takes the full 32 bytes
/// and returns the value if it fits in u128, otherwise returns an error.
pub(crate) fn decode_uint128(bytes: &[u8]) -> Result<u128> {
    if bytes.len() < 32 {
        return Err(Error::Other("Insufficient bytes for uint128".to_string()));
    }
    // For ABI-encoded uint256 that fits in u128:
    // Check if the first 16 bytes are all zeros (value fits in u128)
    for i in 0..16 {
        if bytes[i] != 0 {
            return Err(Error::Other(format!(
                "Uint256 value {} does not fit in u128 (overflow in first 16 bytes)",
                hex::encode(&bytes[0..32])
            )));
        }
    }
    // Take last 16 bytes from the 32-byte slot (the actual value)
    let arr: [u8; 16] = bytes[16..32]
        .try_into()
        .map_err(|_| Error::Other("Failed to decode uint128".to_string()))?;
    Ok(u128::from_be_bytes(arr))
}

/// Decode uint256 from 32-byte big-endian bytes
///
/// Returns u128 if the value fits, otherwise returns an error.
/// This is used for uint256 contract values that are expected to fit in u128.
pub(crate) fn decode_uint256_as_u128(bytes: &[u8]) -> Result<u128> {
    decode_uint128(bytes)
}

/// Decode bool from 32-byte bytes
pub(crate) fn decode_bool(bytes: &[u8]) -> Result<bool> {
    if bytes.len() < 32 {
        return Err(Error::Other("Insufficient bytes for bool".to_string()));
    }
    Ok(bytes[31] != 0)
}

/// Decode address from 32-byte bytes (takes last 20 bytes)
pub(crate) fn decode_address(bytes: &[u8]) -> Result<String> {
    if bytes.len() < 32 {
        return Err(Error::Other("Insufficient bytes for address".to_string()));
    }
    // Address is last 20 bytes of 32-byte slot
    Ok(format!("0x{}", hex::encode(&bytes[12..32])))
}

// =============================================================================
// RESPONSE DECODING
// =============================================================================

/// Decode the response from `get_epoch()` call
///
/// # Arguments
/// * `data` - Raw bytes response from the contract
///
/// # Returns
/// Decoded EpochInfo struct
pub fn decode_epoch_info(data: &[u8]) -> Result<types::EpochInfo> {
    // Response format: (uint64 epoch, bool is_epoch_transition)
    // Each value is 32 bytes
    if data.len() < 64 {
        return Err(Error::Other("Insufficient data for EpochInfo".to_string()));
    }

    let epoch = decode_uint64(&data[0..32])?;
    let is_epoch_transition = decode_bool(&data[32..64])?;

    Ok(types::EpochInfo {
        epoch,
        is_epoch_transition,
    })
}

/// Decode the response from `get_validator()` call
///
/// # Arguments
/// * `data` - Raw bytes response from the contract
///
/// # Returns
/// Decoded Validator struct with correct field mapping
///
/// # Field Mapping
/// 1. auth_delegator (address)
/// 2. flags (uint256) - NOT commission!
/// 3. execution_stake (uint256)
/// 4. accumulated_rewards_per_token (uint256)
/// 5. execution_commission (uint256)
/// 6. unclaimed_rewards (uint256)
/// 7. consensus_stake (uint256)
/// 8. consensus_commission (uint256)
/// 9. snapshot_stake (uint256)
/// 10. snapshot_commission (uint256)
/// 11. secp_pub_key (bytes)
/// 12. bls_pub_key (bytes)
pub fn decode_validator(data: &[u8]) -> Result<types::Validator> {
    // Response format: (address, uint256, uint256, uint256, uint256, uint256, uint256, uint256, uint256, uint256, bytes, bytes)
    // First 10 static params = 10 * 32 = 320 bytes
    // Then dynamic bytes for secp_pub_key and bls_pub_key

    if data.len() < 320 {
        return Err(Error::Other("Insufficient data for Validator".to_string()));
    }

    let auth_delegator = decode_address(&data[0..32])?;

    // Field 2: flags (uint256 in contract)
    let flags = decode_uint256_as_u128(&data[32..64])?;

    // Field 3: execution_stake (uint256)
    let execution_stake = decode_uint256_as_u128(&data[64..96])?;

    // Field 4: accumulated_rewards_per_token (uint256, scaled by 1e36)
    let accumulated_rewards_per_token = decode_uint256_as_u128(&data[96..128])?;

    // Field 5: execution_commission (uint256, scaled by 1e18)
    let execution_commission = decode_uint256_as_u128(&data[128..160])?;

    // Field 6: unclaimed_rewards (uint256)
    let unclaimed_rewards = decode_uint256_as_u128(&data[160..192])?;

    // Field 7: consensus_stake (uint256)
    let consensus_stake = decode_uint256_as_u128(&data[192..224])?;

    // Field 8: consensus_commission (uint256, scaled by 1e18)
    let consensus_commission = decode_uint256_as_u128(&data[224..256])?;

    // Field 9: snapshot_stake (uint256)
    let snapshot_stake = decode_uint256_as_u128(&data[256..288])?;

    // Field 10: snapshot_commission (uint256, scaled by 1e18)
    let snapshot_commission = decode_uint256_as_u128(&data[288..320])?;

    // Dynamic bytes: offset and length encoding
    // ABI encoding for tuple with dynamic types:
    // - First 10 static params: 10 * 32 = 320 bytes
    // - Offset to secp_pub_key at byte 320-352
    // - Offset to bls_pub_key at byte 352-384

    // Read offset to secp_pub_key (first dynamic param)
    let secp_offset = if data.len() >= 352 {
        decode_uint64(&data[320..352])? as usize
    } else {
        return Err(Error::Other("Missing secp_pub_key offset".to_string()));
    };

    // Read offset to bls_pub_key (second dynamic param)
    let bls_offset = if data.len() >= 384 {
        decode_uint64(&data[352..384])? as usize
    } else {
        return Err(Error::Other("Missing bls_pub_key offset".to_string()));
    };

    // Parse secp_pub_key bytes
    let secp_pub_key = if data.len() > secp_offset + 32 {
        let len = decode_uint64(&data[secp_offset..secp_offset + 32])? as usize;
        if data.len() >= secp_offset + 32 + len {
            format!(
                "0x{}",
                hex::encode(&data[secp_offset + 32..secp_offset + 32 + len])
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // Parse bls_pub_key bytes
    let bls_pub_key = if data.len() > bls_offset + 32 {
        let len = decode_uint64(&data[bls_offset..bls_offset + 32])? as usize;
        if data.len() >= bls_offset + 32 + len {
            format!(
                "0x{}",
                hex::encode(&data[bls_offset + 32..bls_offset + 32 + len])
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    Ok(types::Validator {
        auth_delegator,
        flags,
        execution_stake,
        accumulated_rewards_per_token,
        execution_commission,
        unclaimed_rewards,
        consensus_stake,
        consensus_commission,
        snapshot_stake,
        snapshot_commission,
        secp_pub_key,
        bls_pub_key,
    })
}

/// Decode the response from `get_delegator()` call
///
/// # Arguments
/// * `data` - Raw bytes response from the contract
///
/// # Returns
/// Decoded Delegator struct
///
/// # ABI Format
/// Response format: (uint256, uint256, uint256, uint256, uint256, uint64, uint64)
/// 7 * 32 = 224 bytes
///
/// Fields:
/// 1. delegated_amount (uint256) - Current delegated MON amount (Stake)
/// 2. accumulated_rewards_per_token (uint256) - Rewards per token (divide by 1e36)
/// 3. rewards (uint256) - Unclaimed rewards (Total Rewards)
/// 4. delta_stake (uint256) - Amount pending undelegation (Delta Stake)
/// 5. next_delta_stake (uint256) - Next amount pending undelegation (Next Delta Stake)
/// 6. delta_epoch (uint64) - Epoch when current undelegation completes (Delta Epoch)
/// 7. next_delta_epoch (uint64) - Epoch when next undelegation completes (Next Delta Epoch)
pub fn decode_delegator(data: &[u8]) -> Result<types::Delegator> {
    // Response format: (uint256, uint256, uint256, uint256, uint256, uint64, uint64)
    // 7 * 32 = 224 bytes

    if data.len() < 224 {
        return Err(Error::Other("Insufficient data for Delegator".to_string()));
    }

    let delegated_amount = decode_uint256_as_u128(&data[0..32])?;
    let accumulated_rewards_per_token = decode_uint256_as_u128(&data[32..64])?;
    let rewards = decode_uint256_as_u128(&data[64..96])?;
    let delta_stake = decode_uint256_as_u128(&data[96..128])?;
    let next_delta_stake = decode_uint256_as_u128(&data[128..160])?;
    let delta_epoch = decode_uint64(&data[160..192])?;
    let next_delta_epoch = decode_uint64(&data[192..224])?;

    Ok(types::Delegator {
        delegated_amount,
        accumulated_rewards_per_token,
        rewards,
        delta_stake,
        next_delta_stake,
        delta_epoch,
        next_delta_epoch,
    })
}

/// Decode the response from `get_withdrawal_request()` call
///
/// # Arguments
/// * `data` - Raw bytes response from the contract
///
/// # Returns
/// Decoded WithdrawalRequest struct
pub fn decode_withdrawal_request(data: &[u8]) -> Result<types::WithdrawalRequest> {
    // Response format: (uint256, uint256, uint64)
    // 3 * 32 = 96 bytes

    if data.len() < 96 {
        return Err(Error::Other(
            "Insufficient data for WithdrawalRequest".to_string(),
        ));
    }

    let amount = decode_uint128(&data[0..32])?;
    let withdrawal_index = decode_uint64(&data[32..64])? as u8;
    let activation_epoch = decode_uint64(&data[64..96])?;

    Ok(types::WithdrawalRequest {
        amount,
        withdrawal_index,
        activation_epoch,
    })
}

/// Decode the response from valset calls (consensus/snapshot/execution)
///
/// # Arguments
/// * `data` - Raw bytes response from the contract
///
/// # Returns
/// Decoded ValidatorSet struct
pub fn decode_validator_set(data: &[u8]) -> Result<types::ValidatorSet> {
    // Response format: (bool, uint64, uint64[])
    // is_done (32), total_count (32), then dynamic array
    // NOTE: Contract returns "is_done" (bool), not "has_more"
    // We need to invert: has_more = !is_done

    if data.len() < 96 {
        return Err(Error::Other(
            "Insufficient data for ValidatorSet".to_string(),
        ));
    }

    // CRITICAL FIX: Contract returns "is_done", not "has_more"
    // Must invert: has_more = !is_done
    let is_done = decode_bool(&data[0..32])?;
    let has_more = !is_done;

    let total_count = decode_uint64(&data[32..64])?;

    // Dynamic array starts at offset in bytes 64-96
    let array_offset = decode_uint64(&data[64..96])? as usize;

    if data.len() < array_offset + 32 {
        return Ok(types::ValidatorSet {
            has_more,
            total_count,
            validator_ids: Vec::new(),
        });
    }

    let array_len = decode_uint64(&data[array_offset..array_offset + 32])? as usize;
    let mut validator_ids = Vec::with_capacity(array_len);

    for i in 0..array_len {
        let start = array_offset + 32 + (i * 32);
        if start + 32 <= data.len() {
            validator_ids.push(decode_uint64(&data[start..start + 32])?);
        }
    }

    Ok(types::ValidatorSet {
        has_more,
        total_count,
        validator_ids,
    })
}

/// Decode the response from `get_delegations()` call
///
/// # Arguments
/// * `data` - Raw bytes response from the contract
///
/// # Returns
/// Decoded DelegationList struct
pub fn decode_delegation_list(data: &[u8]) -> Result<types::DelegationList> {
    // Same format as ValidatorSet: (bool, uint64, uint64[])
    let valset = decode_validator_set(data)?;
    Ok(types::DelegationList {
        has_more: valset.has_more,
        total_count: valset.total_count,
        validator_ids: valset.validator_ids,
    })
}

/// Decode the response from `get_delegators()` call
///
/// # Arguments
/// * `data` - Raw bytes response from the contract
///
/// # Returns
/// Decoded DelegatorList struct
///
/// # ABI Format
/// (bool has_more, address last_address, address[] addresses)
/// - has_more: 32 bytes (bool)
/// - last_address: 32 bytes (address in last 20 bytes)
/// - addresses offset: 32 bytes (pointer to dynamic array)
/// - addresses length: 32 bytes (at offset)
/// - addresses data: N * 32 bytes
pub fn decode_delegator_list(data: &[u8]) -> Result<types::DelegatorList> {
    // Minimum: has_more (32) + last_address (32) + array_offset (32) = 96 bytes
    if data.len() < 96 {
        return Err(Error::Other(
            "Insufficient data for DelegatorList".to_string(),
        ));
    }

    // CRITICAL FIX: Contract returns "is_done" (bool), not "has_more"
    // We need to invert: has_more = !is_done
    // This matches Python SDK behavior where response[0] is "is_done"
    let is_done = decode_bool(&data[0..32])?;
    let has_more = !is_done;

    let last_address = decode_address(&data[32..64])?;

    // Dynamic array offset
    let array_offset = decode_uint64(&data[64..96])? as usize;

    if data.len() < array_offset + 32 {
        // Empty array case
        return Ok(types::DelegatorList {
            has_more,
            last_address,
            addresses: Vec::new(),
        });
    }

    // Read array length
    let array_len = decode_uint64(&data[array_offset..array_offset + 32])? as usize;
    let mut addresses = Vec::with_capacity(array_len);

    // Read each address (32 bytes each)
    for i in 0..array_len {
        let start = array_offset + 32 + (i * 32);
        if start + 32 <= data.len() {
            addresses.push(decode_address(&data[start..start + 32])?);
        }
    }

    Ok(types::DelegatorList {
        has_more,
        last_address,
        addresses,
    })
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_uint64() {
        // 32-byte encoded uint64(1)
        let bytes =
            hex_to_bytes("0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap();
        let result = decode_uint64(&bytes).unwrap();
        assert_eq!(result, 1u64);
    }

    #[test]
    fn test_decode_bool() {
        let bytes_true =
            hex_to_bytes("0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap();
        let bytes_false =
            hex_to_bytes("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap();
        assert!(decode_bool(&bytes_true).unwrap());
        assert!(!decode_bool(&bytes_false).unwrap());
    }

    #[test]
    fn test_decode_epoch_info() {
        // Encoded (epoch=100, is_epoch_transition=true)
        let hex_data = "0000000000000000000000000000000000000000000000000000000000000064\
                        0000000000000000000000000000000000000000000000000000000000000001";
        let bytes = hex_to_bytes(hex_data).unwrap();
        let result = decode_epoch_info(&bytes).unwrap();
        assert_eq!(result.epoch, 100);
        assert!(result.is_epoch_transition);
    }

    #[test]
    fn test_decode_delegator() {
        // Minimal valid delegator response (7 * 32 bytes)
        let mut hex_data = String::new();
        for i in 0..7 {
            hex_data.push_str(&format!("{:064x}", i * 100));
        }
        let bytes = hex_to_bytes(&hex_data).unwrap();
        let result = decode_delegator(&bytes).unwrap();
        assert_eq!(result.delegated_amount, 0u128);
        assert_eq!(result.accumulated_rewards_per_token, 100u128);
        assert_eq!(result.delta_epoch, 500u64);
    }

    #[test]
    fn test_decode_validator_with_pubkeys() {
        // Construct a valid validator response with dynamic bytes
        // Format: 10 static params (320 bytes) + 2 offsets (64 bytes) + secp_key + bls_key
        let mut hex_data = String::new();

        // Static params (10 * 32 bytes)
        // auth_delegator (address)
        hex_data.push_str("0000000000000000000000001234567890123456789012345678901234567890");
        // flags (uint256) - ACTIVE flag set
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000001");
        // execution_stake (uint256) - 1 MON
        hex_data.push_str("0000000000000000000000000000000000000000000000000de0b6b3a7640000");
        // accumulated_rewards_per_token (uint256) - some value
        hex_data.push_str("0000000000000000000000000000000000000000000000000de0b6b3a7640000");
        // execution_commission (uint256) - 10% (10 * 10^16 = 100000000000000000)
        hex_data.push_str("000000000000000000000000000000000000000000000000016345785d8a0000");
        // unclaimed_rewards (uint256)
        hex_data.push_str("0000000000000000000000000000000000000000000000000de0b6b3a7640000");
        // consensus_stake (uint256)
        hex_data.push_str("0000000000000000000000000000000000000000000000000de0b6b3a7640000");
        // consensus_commission (uint256) - 10%
        hex_data.push_str("000000000000000000000000000000000000000000000000016345785d8a0000");
        // snapshot_stake (uint256)
        hex_data.push_str("0000000000000000000000000000000000000000000000000de0b6b3a7640000");
        // snapshot_commission (uint256) - 10%
        hex_data.push_str("000000000000000000000000000000000000000000000000016345785d8a0000");

        // Offset to secp_pub_key (384 = 0x180 = 12 * 32)
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000180");
        // Offset to bls_pub_key (480 = 0x1e0 = 15 * 32)
        hex_data.push_str("00000000000000000000000000000000000000000000000000000000000001e0");

        // secp_pub_key: length (64 bytes = 0x40) + 64 bytes of data
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000040");
        // 64 bytes of secp key (padded to 32-byte boundary = 64 bytes, which is already aligned)
        for _ in 0..64 {
            hex_data.push_str("ab");
        }

        // bls_pub_key: length (48 bytes = 0x30) + 48 bytes of data
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000030");
        // 48 bytes of bls key (padded to 32-byte boundary = 64 bytes)
        for _ in 0..48 {
            hex_data.push_str("cd");
        }
        // Padding to 32-byte boundary (48 needs 16 bytes padding)
        for _ in 0..16 {
            hex_data.push_str("00");
        }

        let bytes = hex_to_bytes(&hex_data).unwrap();
        let result = decode_validator(&bytes).unwrap();

        assert_eq!(
            result.auth_delegator,
            "0x1234567890123456789012345678901234567890"
        );
        assert_eq!(result.flags, 1);
        assert_eq!(result.execution_stake, 1000000000000000000u128);
        assert_eq!(
            result.accumulated_rewards_per_token,
            1000000000000000000u128
        );
        assert_eq!(result.execution_commission, 100000000000000000u128);
        assert_eq!(result.unclaimed_rewards, 1000000000000000000u128);
        assert_eq!(result.consensus_stake, 1000000000000000000u128);
        assert_eq!(result.consensus_commission, 100000000000000000u128);
        assert_eq!(result.snapshot_stake, 1000000000000000000u128);
        assert_eq!(result.snapshot_commission, 100000000000000000u128);

        // Check helper methods
        assert_eq!(result.commission(), 10.0); // 10%
        assert_eq!(result.delegated_amount(), 1000000000000000000u128);
        assert_eq!(result.status_flags(), 1);
        assert!(result.is_active());
        assert!(!result.is_slashed());

        // Check that secp_pub_key is parsed (should be 128 hex chars = 64 bytes)
        assert_eq!(result.secp_pub_key.len(), 130); // "0x" + 128 chars
        assert!(result.secp_pub_key.starts_with("0x"));
        assert!(result.secp_pub_key.contains("ab"));

        // Check that bls_pub_key is parsed (should be 96 hex chars = 48 bytes)
        assert_eq!(result.bls_pub_key.len(), 98); // "0x" + 96 chars
        assert!(result.bls_pub_key.starts_with("0x"));
        assert!(result.bls_pub_key.contains("cd"));
    }

    #[test]
    fn test_decode_delegator_list() {
        // Construct a valid delegator list response
        // Format: has_more (32) + last_address (32) + array_offset (32) + array_len (32) + addresses
        let mut hex_data = String::new();

        // is_done = false (means has_more = true after inversion)
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000000");

        // last_address
        hex_data.push_str("000000000000000000000000abcdefabcdefabcdefabcdefabcdefabcdefabcd");

        // array_offset = 96 (0x60)
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000060");

        // array_len = 2
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000002");

        // address 1
        hex_data.push_str("0000000000000000000000001111111111111111111111111111111111111111");

        // address 2
        hex_data.push_str("0000000000000000000000002222222222222222222222222222222222222222");

        let bytes = hex_to_bytes(&hex_data).unwrap();
        let result = decode_delegator_list(&bytes).unwrap();

        assert!(result.has_more);
        assert_eq!(
            result.last_address,
            "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd"
        );
        assert_eq!(result.addresses.len(), 2);
        assert_eq!(
            result.addresses[0],
            "0x1111111111111111111111111111111111111111"
        );
        assert_eq!(
            result.addresses[1],
            "0x2222222222222222222222222222222222222222"
        );
    }

    #[test]
    fn test_decode_delegator_list_empty() {
        // Empty delegator list
        let mut hex_data = String::new();

        // is_done = true (means has_more = false after inversion)
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000001");

        // last_address = 0x0
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000000");

        // array_offset = 96
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000060");

        // array_len = 0
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000000");

        let bytes = hex_to_bytes(&hex_data).unwrap();
        let result = decode_delegator_list(&bytes).unwrap();

        assert!(!result.has_more);
        assert_eq!(
            result.last_address,
            "0x0000000000000000000000000000000000000000"
        );
        assert_eq!(result.addresses.len(), 0);
    }

    // ===== Enhanced Decode Primitive Tests =====

    #[test]
    fn test_decode_uint64_zero() {
        let bytes = [0u8; 32];
        let result = decode_uint64(&bytes).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_decode_uint64_max() {
        let mut bytes = [0u8; 32];
        // Set last 8 bytes to 0xFF (u64::MAX)
        bytes[24..32].copy_from_slice(&[0xFFu8; 8]);
        let result = decode_uint64(&bytes).unwrap();
        assert_eq!(result, u64::MAX);
    }

    #[test]
    fn test_decode_uint64_value() {
        let mut bytes = [0u8; 32];
        // Set last 8 bytes to represent 12345 = 0x3039
        bytes[24..32].copy_from_slice(&[0, 0, 0, 0, 0, 0, 0x30, 0x39]);
        let result = decode_uint64(&bytes).unwrap();
        assert_eq!(result, 12345);
    }

    #[test]
    fn test_decode_uint64_insufficient_bytes() {
        let bytes = [0u8; 16]; // Only 16 bytes
        let result = decode_uint64(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_uint128_zero() {
        let bytes = [0u8; 32];
        let result = decode_uint128(&bytes).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_decode_uint128_max() {
        let mut bytes = [0u8; 32];
        // First 16 bytes must be zero, last 16 bytes = u128::MAX
        bytes[16..32].copy_from_slice(&[0xFFu8; 16]);
        let result = decode_uint128(&bytes).unwrap();
        assert_eq!(result, u128::MAX);
    }

    #[test]
    fn test_decode_uint128_overflow() {
        let mut bytes = [0u8; 32];
        // Set first byte to non-zero (causes overflow)
        bytes[0] = 0x01;
        let result = decode_uint128(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_uint128_value() {
        let mut bytes = [0u8; 32];
        // Set value to 1 MON = 1000000000000000000 = 0xde0b6b3a7640000
        let value = 1_000_000_000_000_000_000u128;
        bytes[16..32].copy_from_slice(&value.to_be_bytes());
        let result = decode_uint128(&bytes).unwrap();
        assert_eq!(result, value);
    }

    #[test]
    fn test_decode_bool_true() {
        let mut bytes = [0u8; 32];
        bytes[31] = 0x01; // Last byte = 1
        let result = decode_bool(&bytes).unwrap();
        assert!(result);
    }

    #[test]
    fn test_decode_bool_false() {
        let bytes = [0u8; 32];
        let result = decode_bool(&bytes).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_decode_bool_non_zero_is_true() {
        let mut bytes = [0u8; 32];
        bytes[31] = 0xFF; // Any non-zero value = true
        let result = decode_bool(&bytes).unwrap();
        assert!(result);
    }

    #[test]
    fn test_decode_bool_insufficient_bytes() {
        let bytes = [0u8; 16];
        let result = decode_bool(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_address() {
        let mut bytes = [0u8; 32];
        // Set last 20 bytes to address
        let addr_hex = hex::decode("1234567890123456789012345678901234567890").unwrap();
        bytes[12..32].copy_from_slice(&addr_hex);
        let result = decode_address(&bytes).unwrap();
        assert_eq!(result, "0x1234567890123456789012345678901234567890");
    }

    #[test]
    fn test_decode_address_zero_address() {
        let bytes = [0u8; 32];
        let result = decode_address(&bytes).unwrap();
        assert_eq!(result, "0x0000000000000000000000000000000000000000");
    }

    #[test]
    fn test_decode_address_insufficient_bytes() {
        let bytes = [0u8; 16];
        let result = decode_address(&bytes);
        assert!(result.is_err());
    }

    // ===== Epoch Info Decode Tests =====

    #[test]
    fn test_decode_epoch_info_with_transition() {
        // epoch = 100, is_epoch_transition = true
        let mut hex_data = String::new();
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000064"); // epoch = 100
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000001"); // is_transition = true

        let bytes = hex_to_bytes(&hex_data).unwrap();
        let result = decode_epoch_info(&bytes).unwrap();

        assert_eq!(result.epoch, 100);
        assert!(result.is_epoch_transition);
    }

    #[test]
    fn test_decode_epoch_info_no_transition() {
        // epoch = 500, is_epoch_transition = false
        let mut hex_data = String::new();
        hex_data.push_str("00000000000000000000000000000000000000000000000000000000000001f4"); // epoch = 500
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000000"); // is_transition = false

        let bytes = hex_to_bytes(&hex_data).unwrap();
        let result = decode_epoch_info(&bytes).unwrap();

        assert_eq!(result.epoch, 500);
        assert!(!result.is_epoch_transition);
    }

    #[test]
    fn test_decode_epoch_info_insufficient_data() {
        let bytes = [0u8; 32]; // Only 32 bytes, need 64
        let result = decode_epoch_info(&bytes);
        assert!(result.is_err());
    }

    // ===== Delegator Decode Tests =====

    #[test]
    fn test_decode_delegator_minimal() {
        // Minimal delegator with zero values (7 fields * 32 bytes = 224 bytes)
        let mut hex_data = String::new();

        // delegated_amount = 0
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000000");
        // accumulated_rewards_per_token = 0
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000000");
        // rewards = 0
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000000");
        // delta_stake = 0
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000000");
        // next_delta_stake = 0
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000000");
        // delta_epoch = 0
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000000");
        // next_delta_epoch = 0
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000000");

        let bytes = hex_to_bytes(&hex_data).unwrap();
        let result = decode_delegator(&bytes).unwrap();

        assert_eq!(result.delegated_amount, 0);
        assert_eq!(result.rewards, 0);
    }

    #[test]
    fn test_decode_delegator_with_values() {
        let mut hex_data = String::new();

        // delegated_amount = 5 MON = 5000000000000000000 = 0x4563918244f40000 (16 hex chars)
        // Padded to 64 hex chars (32 bytes): 48 leading zeros + 16 char value
        hex_data.push_str("0000000000000000000000000000000000000000000000004563918244f40000");
        // accumulated_rewards_per_token = 1 (padded to 64 hex chars)
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000001");
        // rewards = 0.5 MON = 500000000000000000 = 0x6f05b59d3b20000 (15 hex chars)
        // Padded to 64 hex chars (32 bytes): 49 leading zeros + 15 char value
        hex_data.push_str("00000000000000000000000000000000000000000000000006f05b59d3b20000");
        // delta_stake = 0
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000000");
        // next_delta_stake = 0
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000000");
        // delta_epoch = 0
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000000");
        // next_delta_epoch = 0
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000000");

        let bytes = hex_to_bytes(&hex_data).unwrap();
        let result = decode_delegator(&bytes).unwrap();

        assert_eq!(result.delegated_amount, 5_000_000_000_000_000_000);
        assert_eq!(result.rewards, 500_000_000_000_000_000);
    }

    #[test]
    fn test_decode_delegator_insufficient_data() {
        let bytes = [0u8; 32]; // Only 32 bytes, need 64
        let result = decode_delegator(&bytes);
        assert!(result.is_err());
    }

    // ===== Withdrawal Request Decode Tests =====

    #[test]
    fn test_decode_withdrawal_request_pending() {
        // amount = 10 MON, withdrawal_index = 100, activation_epoch = 200
        let mut hex_data = String::new();

        // amount = 10 MON = 10000000000000000000 = 0x8ac7230489e80000
        // Padded to 64 hex chars (32 bytes)
        hex_data.push_str("0000000000000000000000000000000000000000000000008ac7230489e80000");
        // withdrawal_index = 100 = 0x64
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000064");
        // activation_epoch = 200 = 0xc8
        hex_data.push_str("00000000000000000000000000000000000000000000000000000000000000c8");

        let bytes = hex_to_bytes(&hex_data).unwrap();
        let result = decode_withdrawal_request(&bytes).unwrap();

        assert_eq!(result.amount, 10_000_000_000_000_000_000);
        assert_eq!(result.withdrawal_index, 100);
        assert_eq!(result.activation_epoch, 200);
    }

    #[test]
    fn test_decode_withdrawal_request_zero_amount() {
        let mut hex_data = String::new();

        // amount = 0
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000000");
        // withdrawal_index = 0
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000000");
        // activation_epoch = 0
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000000");

        let bytes = hex_to_bytes(&hex_data).unwrap();
        let result = decode_withdrawal_request(&bytes).unwrap();

        assert_eq!(result.amount, 0);
        assert_eq!(result.withdrawal_index, 0);
        assert_eq!(result.activation_epoch, 0);
    }

    #[test]
    fn test_decode_withdrawal_request_insufficient_data() {
        let bytes = [0u8; 32]; // Only 32 bytes, need 64
        let result = decode_withdrawal_request(&bytes);
        assert!(result.is_err());
    }

    // ===== Validator Set Decode Tests =====

    #[test]
    fn test_decode_validator_set_empty() {
        let mut hex_data = String::new();

        // is_done = true (means has_more = false after inversion)
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000001");
        // total_count = 0
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000000");
        // array_offset = 96 (0x60)
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000060");
        // array_len = 0
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000000");

        let bytes = hex_to_bytes(&hex_data).unwrap();
        let result = decode_validator_set(&bytes).unwrap();

        assert_eq!(result.total_count, 0);
        assert!(!result.has_more);
        assert_eq!(result.validator_ids.len(), 0);
    }

    #[test]
    fn test_decode_validator_set_with_validators() {
        let mut hex_data = String::new();

        // is_done = true (means has_more = false after inversion)
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000001");
        // total_count = 2
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000002");
        // array_offset = 96 (0x60) - offset is relative to data start, not after the header
        // After has_more (32) + total_count (32) = 64 bytes
        // array_offset = 96 means array starts at byte 96
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000060");
        // array_len = 2
        hex_data.push_str("0000000000000000000000000000000000000000000000000000000000000002");
        // validator_id 1 = 42
        hex_data.push_str("000000000000000000000000000000000000000000000000000000000000002a");
        // validator_id 2 = 123
        hex_data.push_str("000000000000000000000000000000000000000000000000000000000000007b");

        let bytes = hex_to_bytes(&hex_data).unwrap();
        let result = decode_validator_set(&bytes).unwrap();

        assert_eq!(result.total_count, 2);
        assert!(!result.has_more);
        assert_eq!(result.validator_ids.len(), 2);
        assert_eq!(result.validator_ids[0], 42);
        assert_eq!(result.validator_ids[1], 123);
    }
}
