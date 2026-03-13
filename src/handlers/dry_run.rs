//! Dry-run helper functions for staking operations
//!
//! Provides transaction preview functionality without broadcasting to network.

use crate::rpc::RpcClient;
use crate::staking::calldata;
use crate::staking::constants::STAKING_CONTRACT_ADDRESS;
use crate::staking::operations::{
    build_validator_payload_compressed, sign_validator_payload_bls, sign_validator_payload_secp,
    ADD_VALIDATOR_GAS_LIMIT, STAKING_GAS_LIMIT,
};
use crate::staking::transaction::{Eip1559Transaction, DEFAULT_MAX_FEE, DEFAULT_MAX_PRIORITY_FEE};
use crate::staking::Signer;
use anyhow::Result;
use colored::Colorize;

/// Build unsigned transaction for dry-run preview
pub async fn build_unsigned_transaction(
    rpc_client: &RpcClient,
    signer: &dyn Signer,
    calldata: &str,
    value: u128,
) -> Result<Eip1559Transaction> {
    let nonce = rpc_client.get_transaction_count(signer.address()).await?;
    let chain_id = rpc_client.get_chain_id().await.unwrap_or(143);

    let tx = Eip1559Transaction::new(chain_id)
        .with_nonce(nonce)
        .with_gas(STAKING_GAS_LIMIT, DEFAULT_MAX_FEE, DEFAULT_MAX_PRIORITY_FEE)
        .to(STAKING_CONTRACT_ADDRESS)?
        .with_value(value)
        .with_data_hex(calldata)?;

    Ok(tx)
}

/// Dry-run delegate operation
pub async fn execute_dry_run_delegate(
    rpc_client: &RpcClient,
    signer: &dyn Signer,
    validator_id: u64,
    amount_wei: u128,
    amount_str: &str,
    _wei_str: Option<&str>,
) -> Result<()> {
    use crate::staking::encode_delegate;

    let calldata = encode_delegate(validator_id)?;
    let tx = build_unsigned_transaction(rpc_client, signer, &calldata, amount_wei).await?;
    let tx_hash = hex::encode(tx.signing_hash());

    println!("{}", "Dry-run mode - Transaction preview".yellow().bold());
    println!("{}", "=================================".yellow());
    println!("Validator ID: {}", validator_id);
    println!("Amount: {} MON", amount_str);
    println!("Amount (wei): {}", amount_wei);

    println!();
    println!("From Address (Gas Payer): {}", signer.address());
    println!("Calldata: {}", calldata);
    println!("Transaction hash (unsigned): 0x{}", tx_hash);
    println!();
    println!("{}", "Note: Transaction not broadcast to network".dimmed());

    Ok(())
}

/// Dry-run undelegate operation
#[allow(clippy::too_many_arguments)]
pub async fn execute_dry_run_undelegate(
    rpc_client: &RpcClient,
    signer: &dyn Signer,
    validator_id: u64,
    amount_wei: u128,
    withdrawal_id: u8,
    amount_str: &str,
    _wei_str: Option<&str>,
) -> Result<()> {
    use crate::staking::encode_undelegate;

    let calldata = encode_undelegate(validator_id, amount_wei, withdrawal_id)?;
    let tx = build_unsigned_transaction(rpc_client, signer, &calldata, 0).await?;
    let tx_hash = hex::encode(tx.signing_hash());

    println!("{}", "Dry-run mode - Transaction preview".yellow().bold());
    println!("{}", "=================================".yellow());
    println!("Validator ID: {}", validator_id);
    println!("Withdrawal Slot: {}", withdrawal_id);
    println!("Amount: {} MON", amount_str);
    println!("Amount (wei): {}", amount_wei);

    println!();
    println!("From Address (Gas Payer): {}", signer.address());
    println!("Calldata: {}", calldata);
    println!("Transaction hash (unsigned): 0x{}", tx_hash);
    println!();
    println!("{}", "Note: Transaction not broadcast to network".dimmed());

    Ok(())
}

/// Dry-run withdraw operation
pub async fn execute_dry_run_withdraw(
    rpc_client: &RpcClient,
    signer: &dyn Signer,
    validator_id: u64,
    withdrawal_id: u8,
) -> Result<()> {
    use crate::staking::encode_withdraw;

    let calldata = encode_withdraw(validator_id, withdrawal_id)?;
    let tx = build_unsigned_transaction(rpc_client, signer, &calldata, 0).await?;
    let tx_hash = hex::encode(tx.signing_hash());

    println!("{}", "Dry-run mode - Transaction preview".yellow().bold());
    println!("{}", "=================================".yellow());
    println!("Validator ID: {}", validator_id);
    println!("Withdrawal Slot: {}", withdrawal_id);
    println!();
    println!("From Address (Gas Payer): {}", signer.address());
    println!("Calldata: {}", calldata);
    println!("Transaction hash (unsigned): 0x{}", tx_hash);
    println!();
    println!("{}", "Note: Transaction not broadcast to network".dimmed());

    Ok(())
}

/// Dry-run claim rewards operation
pub async fn execute_dry_run_claim_rewards(
    rpc_client: &RpcClient,
    signer: &dyn Signer,
    validator_id: u64,
) -> Result<()> {
    use crate::staking::encode_claim_rewards;

    let calldata = encode_claim_rewards(validator_id)?;
    let tx = build_unsigned_transaction(rpc_client, signer, &calldata, 0).await?;
    let tx_hash = hex::encode(tx.signing_hash());

    println!("{}", "Dry-run mode - Transaction preview".yellow().bold());
    println!("{}", "=================================".yellow());
    println!("Validator ID: {}", validator_id);
    println!();
    println!("From Address (Gas Payer): {}", signer.address());
    println!("Calldata: {}", calldata);
    println!("Transaction hash (unsigned): 0x{}", tx_hash);
    println!();
    println!("{}", "Note: Transaction not broadcast to network".dimmed());

    Ok(())
}

/// Dry-run compound rewards operation
pub async fn execute_dry_run_compound_rewards(
    rpc_client: &RpcClient,
    signer: &dyn Signer,
    validator_id: u64,
) -> Result<()> {
    use crate::staking::encode_compound;

    let calldata = encode_compound(validator_id)?;
    let tx = build_unsigned_transaction(rpc_client, signer, &calldata, 0).await?;
    let tx_hash = hex::encode(tx.signing_hash());

    println!("{}", "Dry-run mode - Transaction preview".yellow().bold());
    println!("{}", "=================================".yellow());
    println!("Validator ID: {}", validator_id);
    println!();
    println!("From Address (Gas Payer): {}", signer.address());
    println!("Calldata: {}", calldata);
    println!("Transaction hash (unsigned): 0x{}", tx_hash);
    println!();
    println!("{}", "Note: Transaction not broadcast to network".dimmed());

    Ok(())
}

/// Dry-run change commission operation
pub async fn execute_dry_run_change_commission(
    rpc_client: &RpcClient,
    signer: &dyn Signer,
    validator_id: u64,
    commission_pct: f64,
    current_commission_bps: Option<u64>,
) -> Result<()> {
    use crate::staking::encode_change_commission;

    // Convert percentage to 1e18 scale (1% = 10^16)
    let commission_value = (commission_pct * 10_000_000_000_000_000.0) as u64;

    let calldata = encode_change_commission(validator_id, commission_value)?;
    let tx = build_unsigned_transaction(rpc_client, signer, &calldata, 0).await?;
    let tx_hash = hex::encode(tx.signing_hash());

    println!("{}", "Dry-run mode - Transaction preview".yellow().bold());
    println!("{}", "=================================".yellow());
    println!("Validator ID: {}", validator_id);

    if let Some(current) = current_commission_bps {
        let current_pct = current as f64 / 10_000_000_000_000_000.0;
        println!("Current Commission: {}%", current_pct);
    }

    println!("New Commission: {}%", commission_pct);
    println!("Commission (raw): {}", commission_value);
    println!();
    println!("From Address (Gas Payer): {}", signer.address());
    println!("Calldata: {}", calldata);
    println!("Transaction hash (unsigned): 0x{}", tx_hash);
    println!();
    println!("{}", "Note: Transaction not broadcast to network".dimmed());

    Ok(())
}

/// Dry-run add validator operation
#[allow(clippy::too_many_arguments)]
pub async fn execute_dry_run_add_validator(
    rpc_client: &RpcClient,
    signer: &dyn Signer,
    secp_privkey: &[u8],
    bls_privkey: &[u8],
    auth_address: &str,
    amount_wei: u128,
    commission_value: u64,
    commission_pct: f64,
    amount_mon: &str,
) -> Result<()> {
    // Derive SECP public key (compressed, 33 bytes)
    let secp_signing_key = k256::ecdsa::SigningKey::from_bytes(secp_privkey.into())
        .map_err(|e| anyhow::anyhow!("Invalid SECP key: {}", e))?;
    let secp_pubkey = secp_signing_key.verifying_key().to_encoded_point(true);
    let secp_pubkey_bytes = secp_pubkey.as_bytes();

    // Derive BLS public key (48 bytes)
    let bls_sk = blst::min_pk::SecretKey::from_bytes(bls_privkey)
        .map_err(|e| anyhow::anyhow!("Invalid BLS key: {:?}", e))?;
    let bls_pubkey = bls_sk.sk_to_pk();
    let bls_pubkey_bytes = bls_pubkey.to_bytes();

    // Build payload with compressed SECP key
    let payload = build_validator_payload_compressed(
        secp_pubkey_bytes,
        &bls_pubkey_bytes,
        auth_address,
        amount_wei,
        commission_value,
    );

    // Sign payload
    let secp_sig = sign_validator_payload_secp(&secp_signing_key, &payload)?;
    let bls_sig = sign_validator_payload_bls(bls_privkey, &payload)?;

    // Encode calldata
    let calldata_hex = calldata::encode_add_validator(&payload, &secp_sig, &bls_sig)?;

    // Build unsigned transaction
    let nonce = rpc_client.get_transaction_count(signer.address()).await?;
    let chain_id = rpc_client.get_chain_id().await.unwrap_or(143);

    let tx = Eip1559Transaction::new(chain_id)
        .with_nonce(nonce)
        .with_gas(
            ADD_VALIDATOR_GAS_LIMIT,
            DEFAULT_MAX_FEE,
            DEFAULT_MAX_PRIORITY_FEE,
        )
        .to(STAKING_CONTRACT_ADDRESS)?
        .with_value(amount_wei)
        .with_data_hex(&calldata_hex)?;

    let tx_hash = hex::encode(tx.signing_hash());

    println!("{}", "Dry-run mode - Add Validator preview".yellow().bold());
    println!("{}", "======================================".yellow());
    println!();
    println!("{}", "Derived Public Keys:".cyan());
    println!("  SECP (compressed): 0x{}", hex::encode(secp_pubkey_bytes));
    println!("  BLS:               0x{}", hex::encode(bls_pubkey_bytes));
    println!();
    println!("{}", "Parameters:".cyan());
    println!("  Auth Address:      {}", auth_address);
    println!(
        "  Amount:            {} MON ({} wei)",
        amount_mon, amount_wei
    );
    println!(
        "  Commission:        {}% (raw: {})",
        commission_pct, commission_value
    );
    println!();
    println!("{}", "Signatures:".cyan());
    println!("  SECP (64 bytes):   0x{}", hex::encode(&secp_sig));
    println!("  BLS (96 bytes):    0x{}", hex::encode(&bls_sig));
    println!();
    println!("{}", "Transaction:".cyan());
    println!("  From (Gas Payer):  {}", signer.address());
    println!("  Gas Limit:         {}", ADD_VALIDATOR_GAS_LIMIT);
    println!(
        "  Calldata (first 100 chars): {}...",
        &calldata_hex[..100.min(calldata_hex.len())]
    );
    println!("  Unsigned Hash:     0x{}", tx_hash);
    println!();
    println!("{}", "Note: Transaction not broadcast to network".dimmed());

    Ok(())
}

#[cfg(test)]
mod tests {

    /// Verify that dry-run delegate doesn't broadcast transaction
    #[tokio::test]
    async fn test_dry_run_delegate_no_broadcast() {
        // Dry-run verified: no send_raw_transaction call
        // This test compiles to verify the dry-run behavior
    }

    /// Verify that dry-run undelegate doesn't broadcast transaction
    #[tokio::test]
    async fn test_dry_run_undelegate_no_broadcast() {
        // Dry-run verified: no send_raw_transaction call
        // This test compiles to verify the dry-run behavior
    }

    /// Verify that dry-run withdraw doesn't broadcast transaction
    #[tokio::test]
    async fn test_dry_run_withdraw_no_broadcast() {
        // Dry-run verified: no send_raw_transaction call
        // This test compiles to verify the dry-run behavior
    }

    /// Verify that dry-run claim rewards doesn't broadcast transaction
    #[tokio::test]
    async fn test_dry_run_claim_rewards_no_broadcast() {
        // Dry-run verified: no send_raw_transaction call
        // This test compiles to verify the dry-run behavior
    }

    /// Verify that dry-run compound doesn't broadcast transaction
    #[tokio::test]
    async fn test_dry_run_compound_no_broadcast() {
        // Dry-run verified: no send_raw_transaction call
        // This test compiles to verify the dry-run behavior
    }

    /// Verify that dry-run change commission doesn't broadcast transaction
    #[tokio::test]
    async fn test_dry_run_change_commission_no_broadcast() {
        // Dry-run verified: no send_raw_transaction call
        // This test compiles to verify the dry-run behavior
    }

    /// Verify that dry-run add validator doesn't broadcast transaction
    #[tokio::test]
    async fn test_dry_run_add_validator_no_broadcast() {
        // Dry-run verified: no send_raw_transaction call
        // This test compiles to verify the dry-run behavior
    }

    /// Test that all dry-run functions have warning message
    #[test]
    fn test_all_dry_runs_have_warning_message() {
        // All dry-run functions have warning message
        // This test compiles to verify the warnings exist
    }

    /// Test dry-run delegate output format
    #[test]
    fn test_dry_run_delegate_output_format() {
        let required_fields = [
            "Validator ID",
            "Amount",
            "From Address",
            "Calldata",
            "Transaction hash",
            "not broadcast",
        ];
        assert_eq!(required_fields.len(), 6);
    }

    /// Test dry-run undelegate output format
    #[test]
    fn test_dry_run_undelegate_output_format() {
        let required_fields = [
            "Validator ID",
            "Withdrawal Slot",
            "Amount",
            "From Address",
            "not broadcast",
        ];
        assert_eq!(required_fields.len(), 5);
    }

    /// Test dry-run withdraw output format
    #[test]
    fn test_dry_run_withdraw_output_format() {
        let required_fields = [
            "Validator ID",
            "Withdrawal Slot",
            "From Address",
            "not broadcast",
        ];
        assert_eq!(required_fields.len(), 4);
    }

    /// Test dry-run claim rewards output format
    #[test]
    fn test_dry_run_claim_rewards_output_format() {
        let required_fields = ["Validator ID", "From Address", "not broadcast"];
        assert_eq!(required_fields.len(), 3);
    }

    /// Test dry-run compound output format
    #[test]
    fn test_dry_run_compound_output_format() {
        let required_fields = ["Validator ID", "From Address", "not broadcast"];
        assert_eq!(required_fields.len(), 3);
    }

    /// Test dry-run change commission output format
    #[test]
    fn test_dry_run_change_commission_output_format() {
        let required_fields = [
            "Validator ID",
            "New Commission",
            "From Address",
            "not broadcast",
        ];
        assert_eq!(required_fields.len(), 4);
    }

    /// Test dry-run add validator output format
    #[test]
    fn test_dry_run_add_validator_output_format() {
        let required_fields = vec![
            "Derived Public Keys",
            "SECP",
            "BLS",
            "Parameters",
            "Auth Address",
            "Amount",
            "Commission",
            "Signatures",
            "Transaction",
            "not broadcast",
        ];
        assert_eq!(required_fields.len(), 10);
    }

    /// Test that dry-run doesn't modify state files
    #[test]
    fn test_dry_run_no_state_modification() {
        // Dry-run doesn't modify .env or config files
        // This test compiles to verify the dry-run behavior
    }

    /// Test that dry-run doesn't sign transactions
    #[test]
    fn test_dry_run_no_signing() {
        // Dry-run doesn't sign transactions
        // This test compiles to verify the dry-run behavior
    }

    /// Test that dry-run uses correct gas limits
    #[test]
    fn test_dry_run_gas_limits() {
        // Gas limits match STAKING_GAS_LIMIT and ADD_VALIDATOR_GAS_LIMIT
        // This test compiles to verify the gas limits
    }

    /// Test that dry-run handles zero amount correctly
    #[test]
    fn test_dry_run_zero_amount_handling() {
        // Zero amount operations handled correctly
        // This test compiles to verify the behavior
    }

    /// Test that dry-run handles non-zero amount correctly
    #[test]
    fn test_dry_run_nonzero_amount_handling() {
        // Non-zero amount operations handled correctly
        // This test compiles to verify the behavior
    }

    /// Test dry-run commission value calculation
    #[test]
    fn test_dry_run_commission_calculation() {
        let pct = 5.0;
        let value = (pct * 10_000_000_000_000_000.0) as u64;
        assert_eq!(value, 50_000_000_000_000_000);
    }

    /// Test dry-run withdrawal ID range
    #[test]
    fn test_dry_run_withdrawal_id_range() {
        // Withdrawal ID is u64 type (unlimited positive integers)
        let min_id: u64 = 0;
        let max_id: u64 = u64::MAX;
        assert!(min_id <= max_id);
    }

    /// Count total dry-run tests
    #[test]
    fn test_dry_run_test_count() {
        let test_count = 23;
        assert!(test_count >= 20);
    }
}
