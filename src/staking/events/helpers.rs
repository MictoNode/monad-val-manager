//! Helper functions for event parsing
//!
//! This module provides utility functions for parsing hex data, addresses,
//! and numeric values from Ethereum event logs.

use crate::utils::error::{Error, Result};

// =============================================================================
// HEX NORMALIZATION
// =============================================================================

/// Normalize hex string (remove 0x prefix, lowercase)
pub fn normalize_hex(hex: &str) -> String {
    let clean = hex.strip_prefix("0x").unwrap_or(hex);
    clean.to_lowercase()
}

// =============================================================================
// TOPIC PARSING
// =============================================================================

/// Parse an Ethereum address from a topic (32 bytes, address in last 20 bytes)
///
/// Topics are 32 bytes but addresses are only 20 bytes, so the address
/// is right-aligned (in the last 20 bytes of the 32-byte topic).
pub fn parse_address_from_topic(topic: &str) -> Result<String> {
    let clean = topic.strip_prefix("0x").unwrap_or(topic);
    if clean.len() < 64 {
        return Err(Error::Other(format!(
            "Invalid topic length: {}",
            clean.len()
        )));
    }
    // Address is in the last 40 hex chars (20 bytes)
    let address = &clean[24..64];
    Ok(format!("0x{}", address))
}

/// Parse a uint64 from a topic (32 bytes, value in last 8 bytes)
///
/// Topics are 32 bytes but uint64 is only 8 bytes, so the value
/// is right-aligned (in the last 8 bytes of the 32-byte topic).
pub fn parse_uint64_from_topic(topic: &str) -> Result<u64> {
    let clean = topic.strip_prefix("0x").unwrap_or(topic);
    if clean.len() < 64 {
        return Err(Error::Other(format!(
            "Invalid topic length: {}",
            clean.len()
        )));
    }
    // uint64 is in the last 16 hex chars (8 bytes)
    u64::from_str_radix(&clean[48..64], 16)
        .map_err(|e| Error::Other(format!("Failed to parse uint64: {}", e)))
}

// =============================================================================
// DATA PARSING
// =============================================================================

/// Parse a uint256 from data at the given byte offset
///
/// In EVM, uint values are stored in 32-byte slots and are right-aligned.
/// For uint256, the value occupies all 32 bytes of the slot.
///
/// Note: This implementation extracts the value as u128, which covers
/// the practical range for token amounts (up to ~3.4e38).
pub fn parse_uint256_from_data(data: &str, byte_offset: usize) -> Result<u128> {
    let clean = data.strip_prefix("0x").unwrap_or(data);
    let hex_offset = byte_offset * 2;

    // Need at least 64 hex chars (32 bytes) for a full slot
    if clean.len() < hex_offset + 64 {
        return Err(Error::Other(format!(
            "Insufficient data for uint256: need {} chars, have {}",
            hex_offset + 64,
            clean.len()
        )));
    }

    // uint256 is right-aligned in a 32-byte slot
    // For amounts that fit in u128, we take the last 32 hex chars (16 bytes)
    u128::from_str_radix(&clean[hex_offset + 32..hex_offset + 64], 16)
        .map_err(|e| Error::Other(format!("Failed to parse uint256: {}", e)))
}

/// Parse a uint64 from data at the given byte offset
///
/// The value is right-aligned in a 32-byte slot.
pub fn parse_uint64_from_data(data: &str, byte_offset: usize) -> Result<u64> {
    let clean = data.strip_prefix("0x").unwrap_or(data);
    let hex_offset = byte_offset * 2;

    if clean.len() < hex_offset + 64 {
        return Err(Error::Other("Insufficient data for uint64".to_string()));
    }

    // uint64 is in the last 16 hex chars (8 bytes) of the 32-byte slot
    u64::from_str_radix(&clean[hex_offset + 48..hex_offset + 64], 16)
        .map_err(|e| Error::Other(format!("Failed to parse uint64: {}", e)))
}

/// Parse a uint8 from data at the given byte offset
///
/// The value is right-aligned in a 32-byte slot.
pub fn parse_uint8_from_data(data: &str, byte_offset: usize) -> Result<u8> {
    let clean = data.strip_prefix("0x").unwrap_or(data);
    let hex_offset = byte_offset * 2;

    if clean.len() < hex_offset + 64 {
        return Err(Error::Other("Insufficient data for uint8".to_string()));
    }

    // uint8 is in the last 2 hex chars (1 byte) of the 32-byte slot
    u8::from_str_radix(&clean[hex_offset + 62..hex_offset + 64], 16)
        .map_err(|e| Error::Other(format!("Failed to parse uint8: {}", e)))
}

/// Parse validator public keys from AddValidator event data
///
/// Data format (ABI-encoded dynamic bytes):
/// - offset to secp_pubkey (32 bytes)
/// - offset to bls_pubkey (32 bytes)
/// - secp_pubkey length (32 bytes) + data
/// - bls_pubkey length (32 bytes) + data
///
/// Returns (secp_pubkey, bls_pubkey) as hex strings with 0x prefix
pub fn parse_validator_pubkeys_from_data(data: &str) -> Result<(String, String)> {
    let clean = data.strip_prefix("0x").unwrap_or(data);

    // Read offsets (each is 32 bytes = 64 hex chars)
    if clean.len() < 128 {
        return Err(Error::Other("Insufficient data for offsets".to_string()));
    }

    let secp_offset = usize::from_str_radix(&clean[56..64], 16)
        .map_err(|e| Error::Other(format!("Failed to parse secp offset: {}", e)))?;
    let bls_offset = usize::from_str_radix(&clean[120..128], 16)
        .map_err(|e| Error::Other(format!("Failed to parse bls offset: {}", e)))?;

    // Read secp_pubkey
    let secp_hex_offset = secp_offset * 2;
    if clean.len() < secp_hex_offset + 64 {
        return Err(Error::Other(
            "Insufficient data for secp_pubkey length".to_string(),
        ));
    }
    let secp_len = usize::from_str_radix(&clean[secp_hex_offset + 56..secp_hex_offset + 64], 16)
        .map_err(|e| Error::Other(format!("Failed to parse secp_pubkey length: {}", e)))?;
    let secp_data_start = secp_hex_offset + 64;
    let secp_data_end = secp_data_start + (secp_len * 2);
    if clean.len() < secp_data_end {
        return Err(Error::Other(
            "Insufficient data for secp_pubkey".to_string(),
        ));
    }
    let secp_pubkey = format!("0x{}", &clean[secp_data_start..secp_data_end]);

    // Read bls_pubkey
    let bls_hex_offset = bls_offset * 2;
    if clean.len() < bls_hex_offset + 64 {
        return Err(Error::Other(
            "Insufficient data for bls_pubkey length".to_string(),
        ));
    }
    let bls_len = usize::from_str_radix(&clean[bls_hex_offset + 56..bls_hex_offset + 64], 16)
        .map_err(|e| Error::Other(format!("Failed to parse bls_pubkey length: {}", e)))?;
    let bls_data_start = bls_hex_offset + 64;
    let bls_data_end = bls_data_start + (bls_len * 2);
    if clean.len() < bls_data_end {
        return Err(Error::Other("Insufficient data for bls_pubkey".to_string()));
    }
    let bls_pubkey = format!("0x{}", &clean[bls_data_start..bls_data_end]);

    Ok((secp_pubkey, bls_pubkey))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_hex() {
        assert_eq!(normalize_hex("0xABC123"), "abc123");
        assert_eq!(normalize_hex("abc123"), "abc123");
        assert_eq!(normalize_hex("0XABC123"), "0xabc123"); // Note: only lowercase 0x
    }

    #[test]
    fn test_parse_address_from_topic() {
        // Address is right-padded in 32 bytes
        let topic = "0x0000000000000000000000001234567890123456789012345678901234567890";
        let result = parse_address_from_topic(topic).unwrap();
        assert_eq!(result, "0x1234567890123456789012345678901234567890");
    }

    #[test]
    fn test_parse_uint64_from_topic() {
        // uint64(42) in 32-byte slot
        let topic = "0x000000000000000000000000000000000000000000000000000000000000002a";
        let result = parse_uint64_from_topic(topic).unwrap();
        assert_eq!(result, 42u64);
    }

    #[test]
    fn test_parse_uint256_from_data() {
        // uint256(1000000000000000000) = 0x0de0b6b3a7640000 (16 hex chars)
        // In EVM, values are right-aligned in 32-byte slots:
        // 0x000000000000000000000000000000000de0b6b3a7640000
        // That's still only 50 chars - need full 64 chars (32 bytes):
        // 0x0000000000000000000000000000000000000000000000000de0b6b3a7640000
        let data = "0x0000000000000000000000000000000000000000000000000de0b6b3a7640000";
        let result = parse_uint256_from_data(data, 0).unwrap();
        assert_eq!(result, 1_000_000_000_000_000_000u128);
    }

    #[test]
    fn test_parse_uint8_from_data() {
        // uint8(5) at offset 0
        let data = "0x0000000000000000000000000000000000000000000000000000000000000005\
                    0000000000000000000000000000000000000000000000000000000000000000";
        let result = parse_uint8_from_data(data, 0).unwrap();
        assert_eq!(result, 5u8);
    }
}
