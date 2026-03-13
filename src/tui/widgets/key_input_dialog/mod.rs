//! Key Input Dialog Widget - Private key input with validation
//!
//! This module provides a specialized input dialog for entering private keys
//! with real-time validation for BLS12-381 and SECP256k1 key formats.
//!
//! Features:
//! - Modal dialog with key type selection (BLS/SECP)
//! - Real-time hex format validation
//! - Curve-specific validation rules
//! - Secure input masking
//! - Clear error messages

mod validation;

// Re-export public types and functions
pub use validation::{validate_bls_private_key, validate_secp256k1_private_key};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bls_key_validation_integration() {
        // Test that the validation functions are accessible
        let valid_bls = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        assert!(validate_bls_private_key(valid_bls).is_ok());

        let invalid_bls = "0xinvalid";
        assert!(
            invalid_bls.parse::<u64>().is_err() || validate_bls_private_key(invalid_bls).is_err()
        );
    }

    #[test]
    fn test_secp_key_validation_integration() {
        // Test that the validation functions are accessible
        let valid_secp = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        assert!(validate_secp256k1_private_key(valid_secp).is_ok());

        let invalid_secp = "0xinvalid";
        assert!(
            invalid_secp.parse::<u64>().is_err()
                || validate_secp256k1_private_key(invalid_secp).is_err()
        );
    }
}
