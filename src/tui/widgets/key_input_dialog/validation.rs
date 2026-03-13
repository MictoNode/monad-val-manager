//! Key validation functions for BLS12-381 and SECP256k1 private keys
//!
//! This module provides validation functions for different types of private keys
//! used in the Monad staking system:
//! - BLS12-381 keys for validator consensus operations
//! - SECP256k1 keys for validator execution operations

use anyhow::{bail, Result};

/// Validate a BLS12-381 private key
///
/// # Arguments
/// * `private_key` - Private key as hex string (with or without 0x prefix)
///
/// # Returns
/// Ok(()) if valid, Err otherwise
///
/// # Examples
/// ```
/// # use monad_val_manager::tui::widgets::key_input_dialog::validate_bls_private_key;
/// // Valid BLS key (64 hex chars)
/// assert!(validate_bls_private_key("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").is_ok());
///
/// // Invalid - too short
/// assert!(validate_bls_private_key("0x1234").is_err());
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn validate_bls_private_key(private_key: &str) -> Result<()> {
    // Check for empty string
    if private_key.is_empty() {
        bail!("Private key cannot be empty");
    }

    // Remove 0x or 0X prefix if present
    let hex_str = private_key
        .strip_prefix("0x")
        .or_else(|| private_key.strip_prefix("0X"))
        .unwrap_or(private_key);

    // Check length (should be 64 hex chars for 32 bytes)
    if hex_str.len() != 64 {
        bail!(
            "Invalid BLS private key length: expected 64 hex characters, got {}",
            hex_str.len()
        );
    }

    // Validate hex characters
    if !hex_str.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!("Private key contains invalid hex characters");
    }

    // Check if key is all zeros
    if hex_str.chars().all(|c| c == '0') {
        bail!("Private key cannot be zero");
    }

    // Note: Full validation would require big integers to check against BLS12-381 curve order
    // For now, we validate format and ensure it's non-zero

    Ok(())
}

/// Validate a SECP256k1 private key
///
/// # Arguments
/// * `private_key` - Private key as hex string (with or without 0x prefix)
///
/// # Returns
/// Ok(()) if valid, Err otherwise
///
/// # Examples
/// ```
/// # use monad_val_manager::tui::widgets::key_input_dialog::validate_secp256k1_private_key;
/// // Valid SECP key (64 hex chars)
/// assert!(validate_secp256k1_private_key("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").is_ok());
///
/// // Invalid - too short
/// assert!(validate_secp256k1_private_key("0x1234").is_err());
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn validate_secp256k1_private_key(private_key: &str) -> Result<()> {
    // Check for empty string
    if private_key.is_empty() {
        bail!("Private key cannot be empty");
    }

    // Remove 0x or 0X prefix if present
    let hex_str = private_key
        .strip_prefix("0x")
        .or_else(|| private_key.strip_prefix("0X"))
        .unwrap_or(private_key);

    // Check length (should be 64 hex chars for 32 bytes)
    if hex_str.len() != 64 {
        bail!(
            "Invalid SECP256k1 private key length: expected 64 hex characters, got {}",
            hex_str.len()
        );
    }

    // Validate hex characters
    if !hex_str.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!("Private key contains invalid hex characters");
    }

    // Check if key is all zeros
    if hex_str.chars().all(|c| c == '0') {
        bail!("Private key cannot be zero");
    }

    // Note: Full validation would require checking against the full secp256k1 curve order
    // For now, we validate format and ensure it's non-zero

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_bls_private_key_valid() {
        // Valid 64-char hex string with 0x prefix
        let valid_key = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        assert!(validate_bls_private_key(valid_key).is_ok());

        // Valid 64-char hex string without 0x prefix
        let valid_key_no_prefix =
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        assert!(validate_bls_private_key(valid_key_no_prefix).is_ok());
    }

    #[test]
    fn test_validate_bls_private_key_empty() {
        assert!(validate_bls_private_key("").is_err());
    }

    #[test]
    fn test_validate_bls_private_key_too_short() {
        assert!(validate_bls_private_key("0x1234").is_err());
    }

    #[test]
    fn test_validate_bls_private_key_too_long() {
        let too_long = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234";
        assert!(validate_bls_private_key(too_long).is_err());
    }

    #[test]
    fn test_validate_bls_private_key_invalid_hex() {
        assert!(
            validate_bls_private_key("0xghijklmnopqrstuvwxyz1234567890abcdef1234567890abcdef")
                .is_err()
        );
    }

    #[test]
    fn test_validate_bls_private_key_zero() {
        assert!(validate_bls_private_key(
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        )
        .is_err());
    }

    #[test]
    fn test_validate_secp256k1_private_key_valid() {
        // Valid 64-char hex string with 0x prefix
        let valid_key = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        assert!(validate_secp256k1_private_key(valid_key).is_ok());

        // Valid 64-char hex string without 0x prefix
        let valid_key_no_prefix =
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        assert!(validate_secp256k1_private_key(valid_key_no_prefix).is_ok());
    }

    #[test]
    fn test_validate_secp256k1_private_key_empty() {
        assert!(validate_secp256k1_private_key("").is_err());
    }

    #[test]
    fn test_validate_secp256k1_private_key_too_short() {
        assert!(validate_secp256k1_private_key("0x1234").is_err());
    }

    #[test]
    fn test_validate_secp256k1_private_key_too_long() {
        let too_long = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234";
        assert!(validate_secp256k1_private_key(too_long).is_err());
    }

    #[test]
    fn test_validate_secp256k1_private_key_invalid_hex() {
        assert!(validate_secp256k1_private_key(
            "0xghijklmnopqrstuvwxyz1234567890abcdef1234567890abcdef"
        )
        .is_err());
    }

    #[test]
    fn test_validate_secp256k1_private_key_zero() {
        assert!(validate_secp256k1_private_key(
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        )
        .is_err());
    }

    #[test]
    fn test_validate_bls_private_key_lowercase() {
        // All lowercase should be valid
        let lowercase = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        assert!(validate_bls_private_key(lowercase).is_ok());
    }

    #[test]
    fn test_validate_bls_private_key_uppercase() {
        // All uppercase should also be valid
        let uppercase = "0X1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF";
        assert!(validate_bls_private_key(uppercase).is_ok());
    }

    #[test]
    fn test_validate_bls_private_key_mixed_case() {
        // Mixed case should be valid
        let mixed = "0x1234567890AbCdEf1234567890AbCdEf1234567890AbCdEf1234567890AbCdEf";
        assert!(validate_bls_private_key(mixed).is_ok());
    }
}
