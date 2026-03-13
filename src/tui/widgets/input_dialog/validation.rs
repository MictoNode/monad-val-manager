//! Validation functions for input dialog
//!
//! This module provides validation utilities for user input in dialogs,
//! including amount and address validation.

/// Validate input as a numeric amount
///
/// # Errors
///
/// Returns an error string if:
/// - Input is empty
/// - Input cannot be parsed as a number
/// - Number is not positive
pub fn validate_amount(input: &str) -> Result<f64, String> {
    if input.is_empty() {
        return Err("Amount is required".to_string());
    }

    let amount: f64 = input
        .parse()
        .map_err(|_| "Invalid number format".to_string())?;

    if amount <= 0.0 {
        return Err("Amount must be positive".to_string());
    }

    Ok(amount)
}

/// Validate input as an Ethereum address
///
/// # Errors
///
/// Returns an error string if:
/// - Input is empty
/// - Input does not start with "0x"
/// - Input is not 42 characters
/// - Input contains invalid hex characters
pub fn validate_address(input: &str) -> Result<String, String> {
    if input.is_empty() {
        return Err("Address is required".to_string());
    }

    if !input.starts_with("0x") {
        return Err("Address must start with 0x".to_string());
    }

    if input.len() != 42 {
        return Err("Address must be 42 characters".to_string());
    }

    if !input[2..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Invalid hex characters in address".to_string());
    }

    Ok(input.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_amount_valid() {
        let result = validate_amount("100.5");
        assert!(result.is_ok());
        assert!((result.unwrap() - 100.5).abs() < 0.001);
    }

    #[test]
    fn test_validate_amount_integer() {
        let result = validate_amount("100");
        assert!(result.is_ok());
        assert!((result.unwrap() - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_validate_amount_empty() {
        let result = validate_amount("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Amount is required");
    }

    #[test]
    fn test_validate_amount_invalid() {
        let result = validate_amount("abc");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid number format");
    }

    #[test]
    fn test_validate_amount_negative() {
        let result = validate_amount("-10");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Amount must be positive");
    }

    #[test]
    fn test_validate_amount_zero() {
        let result = validate_amount("0");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Amount must be positive");
    }

    #[test]
    fn test_validate_amount_scientific_notation() {
        let result = validate_amount("1e10");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_address_valid() {
        let result = validate_address("0x1234567890123456789012345678901234567890");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "0x1234567890123456789012345678901234567890"
        );
    }

    #[test]
    fn test_validate_address_uppercase() {
        let result = validate_address("0xABCDEF1234567890ABCDEF1234567890ABCDEF12");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "0xabcdef1234567890abcdef1234567890abcdef12"
        );
    }

    #[test]
    fn test_validate_address_empty() {
        let result = validate_address("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Address is required");
    }

    #[test]
    fn test_validate_address_no_prefix() {
        let result = validate_address("1234567890123456789012345678901234567890");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Address must start with 0x");
    }

    #[test]
    fn test_validate_address_wrong_length() {
        let result = validate_address("0x12345");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Address must be 42 characters");
    }

    #[test]
    fn test_validate_address_invalid_chars() {
        let result = validate_address("0xZZZZ567890123456789012345678901234567890");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid hex characters in address");
    }

    #[test]
    fn test_validate_address_too_long() {
        let result = validate_address("0x123456789012345678901234567890123456789012345");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Address must be 42 characters");
    }
}
