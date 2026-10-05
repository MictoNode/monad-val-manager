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
    use super::*;
    use crate::staking::encode_delegate;
    use crate::staking::signer::LocalSigner;
    use wiremock::matchers::{body_string_contains, method};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const TEST_KEY: &str = "0000000000000000000000000000000000000000000000000000000000000001";
    const TESTNET_CHAIN_ID: u64 = 10143;
    const ONE_MON: u128 = 1_000_000_000_000_000_000;

    fn json_rpc_success<T: serde::Serialize>(result: T) -> String {
        serde_json::json!({ "jsonrpc": "2.0", "id": 1, "result": result }).to_string()
    }

    /// Mount only the reads a dry-run needs. Nothing else is mounted, so any
    /// attempt to broadcast would fail the request and surface as an Err.
    async fn mock_reads(server: &MockServer, nonce: u64) {
        for (rpc_method, result) in [
            ("eth_chainId", format!("0x{:x}", TESTNET_CHAIN_ID)),
            ("eth_getTransactionCount", format!("0x{:x}", nonce)),
        ] {
            Mock::given(method("POST"))
                .and(body_string_contains(format!(
                    "\"method\":\"{}\"",
                    rpc_method
                )))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_string(json_rpc_success(result))
                        .insert_header("Content-Type", "application/json"),
                )
                .mount(server)
                .await;
        }
    }

    /// The preview must describe exactly the operation the caller asked for.
    #[tokio::test]
    async fn build_unsigned_transaction_matches_requested_operation() {
        let server = MockServer::start().await;
        mock_reads(&server, 7).await;

        let signer = LocalSigner::from_private_key(TEST_KEY).expect("Valid key");
        let client = RpcClient::new(&server.uri()).expect("Valid endpoint");
        let calldata = encode_delegate(224).expect("Valid calldata");

        let tx = build_unsigned_transaction(&client, &signer, &calldata, ONE_MON)
            .await
            .expect("Dry-run needs nothing beyond reads");

        assert_eq!(tx.chain_id, TESTNET_CHAIN_ID);
        assert_eq!(tx.nonce, 7);
        assert_eq!(tx.gas_limit, STAKING_GAS_LIMIT);
        assert_eq!(tx.value, ONE_MON);
        assert_eq!(tx.to, STAKING_CONTRACT_ADDRESS);
        assert_eq!(hex::encode(&tx.data), calldata.trim_start_matches("0x"));
    }

    /// The whole point of dry-run: it must never reach the network.
    #[tokio::test]
    async fn dry_run_delegate_only_reads_and_never_broadcasts() {
        let server = MockServer::start().await;
        mock_reads(&server, 0).await;

        let signer = LocalSigner::from_private_key(TEST_KEY).expect("Valid key");
        let client = RpcClient::new(&server.uri()).expect("Valid endpoint");

        execute_dry_run_delegate(&client, &signer, 224, ONE_MON, "1", None)
            .await
            .expect("Dry-run must succeed without broadcasting");

        let requests = server
            .received_requests()
            .await
            .expect("mock server records received requests");

        assert!(
            requests.iter().all(|request| {
                let body = String::from_utf8_lossy(&request.body);
                body.contains("eth_chainId") || body.contains("eth_getTransactionCount")
            }),
            "dry-run must only read chain id and nonce, got {} request(s)",
            requests.len()
        );
        assert!(
            !requests.iter().any(|request| {
                String::from_utf8_lossy(&request.body).contains("eth_sendRawTransaction")
            }),
            "dry-run must never call eth_sendRawTransaction"
        );
    }
}
