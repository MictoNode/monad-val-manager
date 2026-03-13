//! Transfer command handler for Monad blockchain
//!
//! This module handles native MON transfers between addresses.
//! Follows the standard pattern: validate → confirm → broadcast → report.

use crate::config::{load_env, Config};
use crate::handlers::format_mon;
use crate::rpc::RpcClient;
use crate::staking::create_signer;
use anyhow::{Context, Result};
use colored::Colorize;

/// Gas limit for native ETH/MON transfer (standard 21,000)
pub const TRANSFER_GAS_LIMIT: u64 = 21_000;

/// Parse transfer amount from string to wei
///
/// Supports decimal inputs like "1.5" for 1.5 MON
/// Returns amount in wei (u128)
pub fn parse_transfer_amount(amount_str: &str) -> Result<u128> {
    const WEI_PER_MON: u128 = 1_000_000_000_000_000_000; // 10^18

    // Check if amount contains decimal point
    if amount_str.contains('.') {
        // Parse as decimal
        let parts: Vec<&str> = amount_str.split('.').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid amount format: {}", amount_str));
        }

        let whole: u128 = if parts[0].is_empty() {
            0
        } else {
            parts[0].parse().context("Invalid whole number part")?
        };

        let fractional = parts[1];
        if fractional.len() > 18 {
            return Err(anyhow::anyhow!(
                "Amount has too many decimal places (max 18)"
            ));
        }

        // Pad fractional part to 18 digits with trailing zeros
        let padded = format!("{:0<18}", fractional);
        let fractional_wei: u128 = padded.parse().context("Invalid fractional part")?;

        let wei = whole
            .checked_mul(WEI_PER_MON)
            .and_then(|w| w.checked_add(fractional_wei))
            .context("Amount overflow")?;

        Ok(wei)
    } else {
        // Parse as MON (not wei) for user convenience
        let mon: u128 = amount_str.parse().context("Invalid amount format")?;
        let wei = mon.checked_mul(WEI_PER_MON).context("Amount overflow")?;
        Ok(wei)
    }
}

/// Validate Ethereum address format
///
/// Checks if address is valid H160 format (20 bytes, 42 chars with 0x prefix)
pub fn validate_address(address: &str) -> Result<()> {
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

/// Execute transfer command
///
/// # Arguments
/// * `config` - Application configuration
/// * `to_address` - Recipient address
/// * `amount` - Amount to transfer in wei
/// * `dry_run` - Preview transaction without broadcasting
/// * `skip_confirm` - Skip confirmation prompt (-y flag)
///
/// # Returns
/// Result indicating success or error
pub async fn execute(
    config: &Config,
    to_address: String,
    amount: u128,
    dry_run: bool,
    skip_confirm: bool,
) -> Result<()> {
    // Load environment variables
    load_env().ok();

    // Validate recipient address
    validate_address(&to_address)?;

    // Initialize RPC client and signer
    let client = RpcClient::new(config.rpc_endpoint())?;
    let signer = create_signer(config)?;

    // Get sender address
    let from_address = signer.address();

    // Display transfer summary
    println!("{}", "Transfer Summary".cyan().bold());
    println!("From:   {}", from_address);
    println!("To:     {}", to_address);
    println!("Amount: {} MON", format_mon(amount));
    println!();

    // Dry-run mode: preview transaction without broadcasting
    if dry_run {
        // Get nonce
        let nonce = client
            .get_transaction_count(from_address)
            .await
            .context("Failed to get nonce")?;

        // Build native transfer transaction
        let tx = crate::staking::transaction::Eip1559Transaction::new(
            client.get_chain_id().await.unwrap_or(143),
        )
        .with_nonce(nonce)
        .with_gas(
            TRANSFER_GAS_LIMIT,
            crate::staking::transaction::DEFAULT_MAX_FEE,
            crate::staking::transaction::DEFAULT_MAX_PRIORITY_FEE,
        )
        .to(&to_address)?
        .with_value(amount)
        .with_data_hex("0x")?;

        let tx_hash = hex::encode(tx.signing_hash());

        println!("{}", "Dry-run mode - Transaction preview".yellow().bold());
        println!("{}", "=================================".yellow());
        println!("From Address: {}", from_address);
        println!("To Address: {}", to_address);
        println!("Amount: {} MON", format_mon(amount));
        println!("Amount (wei): {}", amount);
        println!();
        println!("Transaction hash (unsigned): 0x{}", tx_hash);
        println!();
        println!("{}", "Note: Transaction not broadcast to network".dimmed());

        return Ok(());
    }

    // Confirmation prompt (unless -y flag)
    if !skip_confirm {
        print!("Confirm transfer? [y/N]: ");
        use std::io::Write;
        std::io::stdout()
            .flush()
            .context("Failed to flush stdout")?;

        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .context("Failed to read confirmation")?;

        let input = input.trim().to_lowercase();
        if input != "y" && input != "yes" {
            println!("Transfer cancelled.");
            return Ok(());
        }
    }

    // Get nonce
    let nonce = client
        .get_transaction_count(from_address)
        .await
        .context("Failed to get nonce")?;

    // Build native transfer transaction
    let tx = crate::staking::transaction::Eip1559Transaction::new(
        client.get_chain_id().await.unwrap_or(143),
    )
    .with_nonce(nonce)
    .with_gas(
        TRANSFER_GAS_LIMIT,
        crate::staking::transaction::DEFAULT_MAX_FEE,
        crate::staking::transaction::DEFAULT_MAX_PRIORITY_FEE,
    )
    .to(&to_address)?
    .with_value(amount)
    .with_data_hex("0x")?;

    // Sign and broadcast
    println!("Broadcasting transaction...");
    let signed_hex = signer.sign_transaction_hex(&tx)?;
    let tx_hash = client.send_raw_transaction(&signed_hex).await?;

    println!("{} {}", "✓".green(), "Transfer successful".green().bold());
    println!("Transaction hash: {}", tx_hash.white().bold());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_transfer_amount_whole() {
        let result = parse_transfer_amount("1").unwrap();
        assert_eq!(result, 1_000_000_000_000_000_000);
    }

    #[test]
    fn test_parse_transfer_amount_decimal() {
        let result = parse_transfer_amount("1.5").unwrap();
        assert_eq!(result, 1_500_000_000_000_000_000);
    }

    #[test]
    fn test_parse_transfer_amount_zero() {
        let result = parse_transfer_amount("0").unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_parse_transfer_amount_small() {
        let result = parse_transfer_amount("0.00000001").unwrap();
        assert_eq!(result, 10_000_000_000);
    }

    #[test]
    fn test_parse_transfer_amount_invalid() {
        let result = parse_transfer_amount("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_transfer_amount_too_many_decimals() {
        let result = parse_transfer_amount("1.1234567890123456789");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_address_valid() {
        let result = validate_address("0x1234567890123456789012345678901234567890");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_address_without_prefix() {
        let result = validate_address("1234567890123456789012345678901234567890");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_address_too_short() {
        let result = validate_address("0x1234");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_address_invalid_chars() {
        let result = validate_address("0xGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG");
        assert!(result.is_err());
    }
}
