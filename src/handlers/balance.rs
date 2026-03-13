//! Balance command handler for Monad blockchain
//!
//! This module handles balance queries for addresses.
//! Supports both explicit addresses and .env file configuration.

use crate::config::{load_env, Config};
use crate::rpc::RpcClient;
use anyhow::{Context, Result};
use colored::Colorize;

/// Execute balance command
///
/// # Arguments
/// * `config` - Application configuration
/// * `address` - Optional address to query (uses .env if not provided)
///
/// # Returns
/// Result indicating success or error
pub async fn execute(config: &Config, address: Option<String>) -> Result<()> {
    // Load environment variables
    load_env().context("Failed to load .env file")?;

    // Determine address to query
    let target_address = if let Some(addr) = address {
        addr
    } else {
        // Try to get address from environment
        get_address_from_env()?
    };

    // Validate address format
    validate_address(&target_address)?;

    // Create RPC client
    let rpc = RpcClient::from_config(config).context("Failed to create RPC client")?;

    // Query balance (returns MON as f64)
    let balance_mon = rpc
        .get_balance(&target_address)
        .await
        .context("Failed to query balance from RPC node")?;

    // Format and display balance
    let balance_formatted = if balance_mon >= 1_000_000.0 {
        format!("{:.2}M MON", balance_mon / 1_000_000.0)
    } else if balance_mon >= 1_000.0 {
        format!("{:.2}K MON", balance_mon / 1_000.0)
    } else {
        format!("{:.4} MON", balance_mon)
    };

    println!(
        "{} {}",
        "Balance:".green().bold(),
        balance_formatted.bright_yellow().bold()
    );
    println!(
        "{} {}",
        "Address:".dimmed(),
        target_address.bright_black().bold()
    );
    println!(
        "{} {}",
        "Network:".dimmed(),
        format!("{:?}", config.network).bright_black()
    );

    Ok(())
}

/// Get address from environment variable
///
/// Returns the address from the appropriate environment variable
/// based on the current network.
fn get_address_from_env() -> Result<String> {
    // For now, we'll use the private key to derive the address
    // In the future, we might add separate MONAD_TESTNET_ADDRESS variables
    let env_var = match std::env::var("MONAD_TESTNET_PRIVATE_KEY") {
        Ok(key) => {
            // Derive address from private key
            derive_address_from_private_key(&key)?
        }
        Err(_) => {
            return Err(anyhow::anyhow!(
                "No address provided and no MONAD_TESTNET_PRIVATE_KEY found in .env file. \
                 Please run with --address <ADDRESS> or set up your .env file."
            ));
        }
    };

    Ok(env_var)
}

/// Derive address from private key
///
/// Takes a private key hex string and returns the derived Ethereum address.
fn derive_address_from_private_key(private_key: &str) -> Result<String> {
    use k256::ecdsa::SigningKey;

    // Remove 0x prefix if present
    let key_hex = private_key.strip_prefix("0x").unwrap_or(private_key);

    // Parse private key
    let key_bytes = hex::decode(key_hex).context("Invalid private key hex")?;

    // Create signing key
    let signing_key = SigningKey::from_slice(&key_bytes)
        .context("Invalid private key (not a valid SECP256k1 key)")?;

    // Get public key
    let verifying_key = signing_key.verifying_key();

    // Encode public key as uncompressed point (65 bytes)
    let encoded_point = verifying_key.to_encoded_point(false);

    // Take last 20 bytes of hash of public key (skip first byte which is 0x04)
    let public_key_bytes = encoded_point.as_bytes();
    let public_key = &public_key_bytes[1..]; // Skip 0x04 prefix

    use sha3::{Digest, Keccak256};
    let hash = Keccak256::digest(public_key);

    // Take last 20 bytes as address
    let address_bytes = &hash[hash.len() - 20..];
    let address = format!("0x{}", hex::encode(address_bytes));

    Ok(address)
}

/// Validate Ethereum address format
///
/// Checks if address is valid H160 format (20 bytes, 42 chars with 0x prefix)
fn validate_address(address: &str) -> Result<()> {
    let address = address.strip_prefix("0x").unwrap_or(address);

    if address.len() != 40 {
        return Err(anyhow::anyhow!(
            "Invalid address length: expected 40 hex chars (20 bytes), got {}",
            address.len()
        ));
    }

    if !address.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(anyhow::anyhow!("Address contains non-hex characters"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_address_valid() {
        let valid_addresses = [
            "0x1234567890123456789012345678901234567890",
            "0xabcdefABCDEF1234567890123456789012345678",
            "1234567890123456789012345678901234567890", // without 0x
        ];

        for addr in &valid_addresses {
            assert!(validate_address(addr).is_ok(), "Should validate: {}", addr);
        }
    }

    #[test]
    fn test_validate_address_invalid_length() {
        let invalid_addresses = [
            "0x1234",                                           // too short
            "0x1234567890123456789012345678901234567890123456", // too long
        ];

        for addr in &invalid_addresses {
            assert!(validate_address(addr).is_err(), "Should reject: {}", addr);
        }
    }

    #[test]
    fn test_validate_address_invalid_chars() {
        let invalid_addresses = [
            "0x123456789012345678901234567890123456789g", // 'g' is not hex
            "0x123456789012345678901234567890123456789 ", // space
        ];

        for addr in &invalid_addresses {
            assert!(validate_address(addr).is_err(), "Should reject: {}", addr);
        }
    }

    #[test]
    fn test_validate_address_empty() {
        assert!(validate_address("").is_err());
        assert!(validate_address("0x").is_err());
    }

    #[test]
    fn test_derive_address_from_private_key() {
        // Known test vector: private key -> address
        // Private key: 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
        // Expected address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
        // Note: Addresses are case-insensitive in Ethereum, so we compare lowercase
        let private_key = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        let expected_address = "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266";

        let result = derive_address_from_private_key(private_key);
        assert!(result.is_ok());
        let address = result.unwrap();
        assert_eq!(address.to_lowercase(), expected_address);
    }

    #[test]
    fn test_derive_address_with_0x_prefix() {
        let private_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        let expected_address = "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266";

        let result = derive_address_from_private_key(private_key);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_lowercase(), expected_address);
    }

    #[test]
    fn test_derive_address_invalid_hex() {
        let invalid_keys = [
            "not-hex-at-all",
            "0xgggg",
            "abc123", // too short
        ];

        for key in &invalid_keys {
            assert!(
                derive_address_from_private_key(key).is_err(),
                "Should reject invalid key: {}",
                key
            );
        }
    }

    #[test]
    fn test_validate_address_without_0x_prefix() {
        // Address without 0x prefix should still validate
        let addr = "1234567890123456789012345678901234567890";
        assert!(validate_address(addr).is_ok());
    }

    #[test]
    fn test_validate_address_with_mixed_case() {
        // Mixed case addresses should validate
        let addr = "0xabcdefABCDEF1234567890123456789012345678";
        assert!(validate_address(addr).is_ok());
    }

    #[test]
    fn test_validate_address_exact_length() {
        // Exactly 40 chars (without 0x) should pass
        let addr = "0x1234567890123456789012345678901234567890";
        assert!(validate_address(addr).is_ok());
    }
}
