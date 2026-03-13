//! High-level staking operations for Monad blockchain
//!
//! This module provides convenient functions for performing staking operations
//! such as delegating, undelegating, withdrawing, claiming rewards, and
//! registering validators.
//!
//! # Architecture
//!
//! Each operation follows this pattern:
//! 1. Build the calldata using `calldata` module
//! 2. Build the EIP-1559 transaction
//! 3. Sign the transaction
//! 4. Broadcast to the network
//! 5. Return the transaction hash
//!
//! # Receipt Waiting
//!
//! For operations that need to wait for transaction confirmation, use the
//! `_with_receipt` variants (e.g., `delegate_with_receipt`). These functions
//! poll for the transaction receipt with configurable timeout.

use crate::rpc::RpcClient;
use crate::staking::calldata;
use crate::staking::constants::{STAKING_CONTRACT_ADDRESS, WITHDRAWAL_DELAY};
use crate::staking::getters;
use crate::staking::receipt::{wait_for_receipt, ReceiptConfig, TransactionReceipt};
use crate::staking::signer::Signer;
use crate::staking::transaction::{Eip1559Transaction, DEFAULT_MAX_FEE, DEFAULT_MAX_PRIORITY_FEE};
use crate::utils::error::{Error, Result};
use anyhow::Context;

/// Format wei amount as MON (human-readable)
fn format_mon(wei: u128) -> String {
    const WEI_PER_MON: u128 = 1_000_000_000_000_000_000; // 10^18

    if wei == 0 {
        return "0".to_string();
    }

    let mon = wei as f64 / WEI_PER_MON as f64;

    if mon >= 1_000_000.0 {
        format!("{:.2}M", mon / 1_000_000.0)
    } else if mon >= 1_000.0 {
        format!("{:.2}K", mon / 1_000.0)
    } else if mon >= 1.0 {
        format!("{:.4}", mon)
    } else {
        format!("{:.8}", mon)
    }
}

/// Gas limit for delegate/undelegate/withdraw operations
pub const STAKING_GAS_LIMIT: u64 = 1_000_000;

/// Gas limit for add_validator operation (higher due to BLS verification)
pub const ADD_VALIDATOR_GAS_LIMIT: u64 = 2_000_000;

/// Result of a staking operation
#[derive(Debug, Clone)]
pub struct StakingResult {
    /// Transaction hash
    pub tx_hash: String,
    /// Raw signed transaction (for inspection)
    pub raw_tx: String,
}

/// Result of a staking operation with receipt
#[derive(Debug, Clone)]
pub struct StakingResultWithReceipt {
    /// Transaction hash
    pub tx_hash: String,
    /// Raw signed transaction (for inspection)
    pub raw_tx: String,
    /// Transaction receipt (if waited)
    pub receipt: TransactionReceipt,
}

/// Parameters for registering a new validator
///
/// This struct groups the many parameters needed for `add_validator` operation.
#[derive(Debug, Clone)]
pub struct AddValidatorParams<'a> {
    /// SECP256k1 public key (64 bytes uncompressed)
    pub secp_pubkey: &'a [u8],
    /// BLS12-381 public key (48 bytes)
    pub bls_pubkey: &'a [u8],
    /// Authorized address for validator operations
    pub auth_address: &'a str,
    /// Amount of MON to stake (minimum 100,000 MON for validators)
    pub amount: u128,
    /// Initial commission rate in basis points (100 = 1%)
    pub commission_bps: u64,
    /// SECP256k1 signature of the payload (BLAKE3 hash)
    pub secp_signature: &'a [u8],
    /// BLS12-381 signature of the payload
    pub bls_signature: &'a [u8],
}

/// Delegate MON to a validator
///
/// # Arguments
/// * `client` - RPC client for the Monad node
/// * `signer` - Signer for the transaction
/// * `validator_id` - Validator ID to delegate to
/// * `amount` - Amount of MON to delegate (in wei)
///
/// # Returns
/// Transaction hash
pub async fn delegate(
    client: &RpcClient,
    signer: &dyn Signer,
    validator_id: u64,
    amount: u128,
) -> Result<StakingResult> {
    // Build calldata
    let data = calldata::encode_delegate(validator_id)?;

    // Get nonce
    let nonce = client
        .get_transaction_count(signer.address())
        .await
        .context("Failed to get nonce")?;

    // Build transaction
    let tx = Eip1559Transaction::new(client.get_chain_id().await.unwrap_or(143))
        .with_nonce(nonce)
        .with_gas(STAKING_GAS_LIMIT, DEFAULT_MAX_FEE, DEFAULT_MAX_PRIORITY_FEE)
        .to(STAKING_CONTRACT_ADDRESS)?
        .with_value(amount)
        .with_data_hex(&data)?;

    // Sign and broadcast
    sign_and_broadcast(client, signer, &tx).await
}

/// Undelegate MON from a validator
///
/// # Arguments
/// * `client` - RPC client for the Monad node
/// * `signer` - Signer for the transaction
/// * `validator_id` - Validator ID to undelegate from
/// * `amount` - Amount of MON to undelegate (in wei)
/// * `withdrawal_index` - Index for this withdrawal (0-255, uint8)
///
/// # Returns
/// Transaction hash
pub async fn undelegate(
    client: &RpcClient,
    signer: &dyn Signer,
    validator_id: u64,
    amount: u128,
    withdrawal_index: u8,
) -> Result<StakingResult> {
    let data = calldata::encode_undelegate(validator_id, amount, withdrawal_index)?;

    let nonce = client
        .get_transaction_count(signer.address())
        .await
        .context("Failed to get nonce")?;

    let tx = Eip1559Transaction::new(client.get_chain_id().await.unwrap_or(143))
        .with_nonce(nonce)
        .with_gas(STAKING_GAS_LIMIT, DEFAULT_MAX_FEE, DEFAULT_MAX_PRIORITY_FEE)
        .to(STAKING_CONTRACT_ADDRESS)?
        .with_data_hex(&data)?;

    sign_and_broadcast(client, signer, &tx).await
}

/// Check if a withdrawal is ready to be claimed
///
/// This function verifies that the current epoch is past the withdrawal's
/// activation epoch plus the WITHDRAWAL_DELAY period.
///
/// # Arguments
/// * `client` - RPC client for the Monad node
/// * `validator_id` - Validator ID
/// * `delegator_address` - Delegator's address
/// * `withdrawal_index` - Withdrawal request index (0-255)
///
/// # Returns
/// Ok(()) if withdrawal is ready, Err with details if not ready
///
/// # Errors
/// Returns an error with a user-friendly message if:
/// - Current epoch < withdrawal activation epoch + WITHDRAWAL_DELAY
/// - Withdrawal request not found
/// - RPC call fails
pub async fn check_withdrawal_ready(
    client: &RpcClient,
    validator_id: u64,
    delegator_address: &str,
    withdrawal_index: u8,
) -> Result<()> {
    // Get current epoch
    let epoch_info = getters::get_epoch(client).await?;
    let current_epoch = epoch_info.epoch;

    // Get withdrawal request info
    let withdrawal =
        getters::get_withdrawal_request(client, validator_id, delegator_address, withdrawal_index)
            .await?;

    // Check if this is a valid (non-empty) withdrawal request
    if withdrawal.amount == 0 {
        return Err(Error::Other(format!(
            "No withdrawal request found at index {} for validator {}",
            withdrawal_index, validator_id
        )));
    }

    // Calculate required epoch
    let required_epoch = withdrawal.activation_epoch.saturating_add(WITHDRAWAL_DELAY);

    // Check if withdrawal is ready
    if current_epoch < required_epoch {
        return Err(Error::WithdrawalNotReady {
            current_epoch,
            withdrawal_epoch: withdrawal.activation_epoch,
            required_epoch,
            withdrawal_index,
        });
    }

    Ok(())
}

/// Withdraw undelegated MON
///
/// # Arguments
/// * `client` - RPC client for the Monad node
/// * `signer` - Signer for the transaction
/// * `validator_id` - Validator ID to withdraw from
/// * `withdrawal_index` - Index of the withdrawal request (0-255, uint8)
///
/// # Returns
/// Transaction hash
pub async fn withdraw(
    client: &RpcClient,
    signer: &dyn Signer,
    validator_id: u64,
    withdrawal_index: u8,
) -> Result<StakingResult> {
    // Check if withdrawal is ready (epoch validation)
    check_withdrawal_ready(client, validator_id, signer.address(), withdrawal_index).await?;

    let data = calldata::encode_withdraw(validator_id, withdrawal_index)?;

    let nonce = client
        .get_transaction_count(signer.address())
        .await
        .context("Failed to get nonce")?;

    let tx = Eip1559Transaction::new(client.get_chain_id().await.unwrap_or(143))
        .with_nonce(nonce)
        .with_gas(STAKING_GAS_LIMIT, DEFAULT_MAX_FEE, DEFAULT_MAX_PRIORITY_FEE)
        .to(STAKING_CONTRACT_ADDRESS)?
        .with_data_hex(&data)?;

    sign_and_broadcast(client, signer, &tx).await
}

/// Preflight check results for claim/compound operations
///
/// Contains all the information needed for validation before executing
/// a claim or compound transaction.
#[derive(Debug, Clone)]
pub struct ClaimCompoundPreflight {
    /// Delegator information before the operation
    pub delegator: crate::staking::types::Delegator,
    /// Whether validator exists
    pub validator_exists: bool,
    /// User's balance before the operation (for claim)
    pub balance_before: Option<u128>,
    /// Current epoch
    pub current_epoch: u64,
}

impl ClaimCompoundPreflight {
    /// Check if rewards are available
    pub fn has_rewards(&self) -> bool {
        self.delegator.rewards > 0
    }

    /// Get active stake in MON
    pub fn active_stake_mon(&self) -> f64 {
        self.delegator.delegated_amount as f64 / 1e18
    }

    /// Get pending stake in MON
    pub fn pending_stake_mon(&self) -> f64 {
        self.delegator.next_delta_stake as f64 / 1e18
    }

    /// Get rewards in MON
    pub fn rewards_mon(&self) -> f64 {
        self.delegator.rewards as f64 / 1e18
    }

    /// Check if delegation exists (has active stake OR rewards)
    pub fn has_delegation(&self) -> bool {
        self.delegator.delegated_amount > 0 || self.delegator.rewards > 0
    }

    /// Check if validator is valid
    pub fn is_validator_valid(&self) -> bool {
        self.validator_exists
    }
}

/// Post-transaction validation results for claim operation
///
/// Contains information about the changes after a claim transaction.
#[derive(Debug, Clone)]
pub struct ClaimPostValidation {
    /// Delegator information after claiming
    pub delegator_after: crate::staking::types::Delegator,
    /// Balance after claiming
    pub balance_after: u128,
    /// Balance change (wei)
    pub balance_change: i128,
    /// Rewards before claiming (wei)
    pub rewards_before: u128,
    /// Rewards after claiming (wei)
    pub rewards_after: u128,
}

impl ClaimPostValidation {
    /// Format balance change for display
    pub fn format_balance_change(&self) -> String {
        format_mon(self.balance_change.unsigned_abs())
    }

    /// Check if claim was successful (rewards decreased)
    pub fn is_successful(&self) -> bool {
        self.rewards_after < self.rewards_before
    }
}

/// Post-transaction validation results for compound operation
///
/// Contains information about the changes after a compound transaction.
#[derive(Debug, Clone)]
pub struct CompoundPostValidation {
    /// Delegator information after compounding
    pub delegator_after: crate::staking::types::Delegator,
    /// Stake before compounding (wei)
    pub stake_before: u128,
    /// Stake after compounding (wei)
    pub stake_after: u128,
    /// Stake increase (wei)
    pub stake_increase: u128,
    /// Rewards before compounding (wei)
    pub rewards_before: u128,
    /// Rewards after compounding (wei)
    pub rewards_after: u128,
}

impl CompoundPostValidation {
    /// Format stake increase for display
    pub fn format_stake_increase(&self) -> String {
        format_mon(self.stake_increase)
    }

    /// Check if compound was successful (stake increased)
    pub fn is_successful(&self) -> bool {
        self.stake_after > self.stake_before
    }

    /// Get rewards change (should be negative or zero)
    pub fn rewards_change(&self) -> i128 {
        self.rewards_after as i128 - self.rewards_before as i128
    }
}

/// Preflight check for claim_rewards operation
///
/// Performs all necessary validation before submitting a claim transaction.
/// Returns detailed preflight information for display.
///
/// # Arguments
/// * `client` - RPC client for the Monad node
/// * `validator_id` - Validator ID to claim rewards from
/// * `delegator_address` - Delegator's address
///
/// # Returns
/// Preflight check results with delegation info, validator existence, and balance
///
/// # Errors
/// Returns error if:
/// - No delegation found with this validator
/// - No rewards available to claim
/// - Validator not found
pub async fn claim_rewards_preflight(
    client: &RpcClient,
    validator_id: u64,
    delegator_address: &str,
) -> Result<ClaimCompoundPreflight> {
    // 1. Check delegator information
    let delegator = getters::get_delegator(client, validator_id, delegator_address).await?;

    // 2. Check validator existence
    let validator = getters::get_validator(client, validator_id).await?;
    let validator_exists = validator.auth_delegator != "0x0000000000000000000000000000000000000000";

    // 3. Get balance before claiming (convert f64 MON to u128 wei)
    let balance_before = client
        .get_balance(delegator_address)
        .await
        .ok()
        .map(|bal_mon| (bal_mon * 1e18) as u128);

    // 4. Get current epoch
    let epoch_info = getters::get_epoch(client).await?;

    Ok(ClaimCompoundPreflight {
        delegator,
        validator_exists,
        balance_before,
        current_epoch: epoch_info.epoch,
    })
}

/// Validate that claim_rewards can proceed
///
/// Checks all preflight conditions and returns detailed error if any check fails.
///
/// # Arguments
/// * `preflight` - Preflight check results from `claim_rewards_preflight`
///
/// # Returns
/// Ok(()) if all checks pass, Err with specific error message
pub fn validate_claim_preflight(preflight: &ClaimCompoundPreflight) -> Result<()> {
    // 1. Check if delegation exists
    if !preflight.has_delegation() {
        return Err(Error::NoDelegation {
            validator_id: 0, // Will be filled by caller
        });
    }

    // 2. Check if rewards are available
    if !preflight.has_rewards() {
        return Err(Error::NoRewardsAvailable {
            validator_id: 0, // Will be filled by caller
            rewards: preflight.delegator.rewards,
        });
    }

    // 3. Check if validator exists
    if !preflight.is_validator_valid() {
        return Err(Error::ValidatorNotFound {
            validator_id: 0, // Will be filled by caller
        });
    }

    Ok(())
}

/// Claim staking rewards from a validator
///
/// # Arguments
/// * `client` - RPC client for the Monad node
/// * `signer` - Signer for the transaction
/// * `validator_id` - Validator ID to claim rewards from
///
/// # Returns
/// Transaction hash
pub async fn claim_rewards(
    client: &RpcClient,
    signer: &dyn Signer,
    validator_id: u64,
) -> Result<StakingResult> {
    // Preflight checks
    let preflight = claim_rewards_preflight(client, validator_id, signer.address()).await?;

    // Validate with context
    if !preflight.has_delegation() {
        return Err(Error::NoDelegation { validator_id });
    }

    if !preflight.has_rewards() {
        return Err(Error::NoRewardsAvailable {
            validator_id,
            rewards: preflight.delegator.rewards,
        });
    }

    if !preflight.is_validator_valid() {
        return Err(Error::ValidatorNotFound { validator_id });
    }

    // Build and send transaction
    let data = calldata::encode_claim_rewards(validator_id)?;

    let nonce = client
        .get_transaction_count(signer.address())
        .await
        .context("Failed to get nonce")?;

    let tx = Eip1559Transaction::new(client.get_chain_id().await.unwrap_or(143))
        .with_nonce(nonce)
        .with_gas(STAKING_GAS_LIMIT, DEFAULT_MAX_FEE, DEFAULT_MAX_PRIORITY_FEE)
        .to(STAKING_CONTRACT_ADDRESS)?
        .with_data_hex(&data)?;

    sign_and_broadcast(client, signer, &tx).await
}

/// Compound staking rewards back into delegation
///
/// # Arguments
/// * `client` - RPC client for the Monad node
/// * `signer` - Signer for the transaction
/// * `validator_id` - Validator ID to compound rewards for
///
/// # Returns
/// Transaction hash
pub async fn compound(
    client: &RpcClient,
    signer: &dyn Signer,
    validator_id: u64,
) -> Result<StakingResult> {
    // Preflight checks (same as claim_rewards)
    let preflight = claim_rewards_preflight(client, validator_id, signer.address()).await?;

    // Validate with context
    if !preflight.has_delegation() {
        return Err(Error::NoDelegation { validator_id });
    }

    if !preflight.has_rewards() {
        return Err(Error::NoRewardsAvailable {
            validator_id,
            rewards: preflight.delegator.rewards,
        });
    }

    if !preflight.is_validator_valid() {
        return Err(Error::ValidatorNotFound { validator_id });
    }

    // Build and send transaction
    let data = calldata::encode_compound(validator_id)?;

    let nonce = client
        .get_transaction_count(signer.address())
        .await
        .context("Failed to get nonce")?;

    let tx = Eip1559Transaction::new(client.get_chain_id().await.unwrap_or(143))
        .with_nonce(nonce)
        .with_gas(STAKING_GAS_LIMIT, DEFAULT_MAX_FEE, DEFAULT_MAX_PRIORITY_FEE)
        .to(STAKING_CONTRACT_ADDRESS)?
        .with_data_hex(&data)?;

    sign_and_broadcast(client, signer, &tx).await
}

/// Change validator commission rate
///
/// # Arguments
/// * `client` - RPC client for the Monad node
/// * `signer` - Signer for the transaction (must be validator owner)
/// * `validator_id` - Validator ID to change commission for
/// * `commission_bps` - New commission rate in basis points (100 = 1%)
///
/// # Returns
/// Transaction hash
pub async fn change_commission(
    client: &RpcClient,
    signer: &dyn Signer,
    validator_id: u64,
    commission_value: u64,
) -> Result<StakingResult> {
    // Validate commission (max 100% = 10^18 in 1e18 scale)
    if commission_value > 100 * 10_000_000_000_000_000u64 {
        return Err(crate::utils::error::Error::InvalidInput(
            "Commission cannot exceed 100%".to_string(),
        ));
    }

    let data = calldata::encode_change_commission(validator_id, commission_value)?;

    let nonce = client
        .get_transaction_count(signer.address())
        .await
        .context("Failed to get nonce")?;

    let tx = Eip1559Transaction::new(client.get_chain_id().await.unwrap_or(143))
        .with_nonce(nonce)
        .with_gas(STAKING_GAS_LIMIT, DEFAULT_MAX_FEE, DEFAULT_MAX_PRIORITY_FEE)
        .to(STAKING_CONTRACT_ADDRESS)?
        .with_data_hex(&data)?;

    sign_and_broadcast(client, signer, &tx).await
}

/// Register a new validator
///
/// This is a complex operation requiring both SECP256k1 and BLS12-381 keys.
/// The payload is signed with both keys to prove ownership.
///
/// # Arguments
/// * `client` - RPC client for the Monad node
/// * `signer` - Signer for the transaction (funded account)
/// * `params` - Validator registration parameters (see `AddValidatorParams`)
///
/// # Returns
/// Transaction hash
///
/// # Example
/// ```ignore
/// use monad_val_manager::staking::operations::{add_validator, AddValidatorParams};
///
/// let params = AddValidatorParams {
///     secp_pubkey: &secp_pubkey_bytes,
///     bls_pubkey: &bls_pubkey_bytes,
///     auth_address: "0x...",
///     amount: 100_000_000_000_000_000_000_000u128, // 100,000 MON
///     commission_bps: 500, // 5%
///     secp_signature: &secp_sig,
///     bls_signature: &bls_sig,
/// };
///
/// let result = add_validator(&client, &signer, params).await?;
/// ```
pub async fn add_validator(
    client: &RpcClient,
    signer: &dyn Signer,
    params: AddValidatorParams<'_>,
) -> Result<StakingResult> {
    // Build payload for calldata encoding
    // The calldata module expects payload, secp_sig, bls_sig
    let data = calldata::encode_add_validator(
        &build_validator_payload(
            params.secp_pubkey,
            params.bls_pubkey,
            params.auth_address,
            params.amount,
            params.commission_bps,
        ),
        params.secp_signature,
        params.bls_signature,
    )?;

    let nonce = client
        .get_transaction_count(signer.address())
        .await
        .context("Failed to get nonce")?;

    let tx = Eip1559Transaction::new(client.get_chain_id().await.unwrap_or(143))
        .with_nonce(nonce)
        .with_gas(
            ADD_VALIDATOR_GAS_LIMIT,
            DEFAULT_MAX_FEE,
            DEFAULT_MAX_PRIORITY_FEE,
        )
        .to(STAKING_CONTRACT_ADDRESS)?
        .with_value(params.amount)
        .with_data_hex(&data)?;

    sign_and_broadcast(client, signer, &tx).await
}

/// Register a new validator using private keys
///
/// This is a convenience function that derives public keys from private keys,
/// builds the payload, signs it, and submits the transaction.
///
/// # Arguments
/// * `client` - RPC client for the Monad node
/// * `tx_signer` - Signer for the transaction (funded account paying gas)
/// * `secp_privkey` - SECP256k1 private key (32 bytes)
/// * `bls_privkey` - BLS12-381 private key (32 bytes)
/// * `auth_address` - Authorized address for validator operations
/// * `amount` - Amount of MON to stake (in wei)
/// * `commission_value` - Commission rate in 1e18 scale (1% = 10^16)
///
/// # Returns
/// Transaction hash
pub async fn add_validator_from_privkeys(
    client: &RpcClient,
    tx_signer: &dyn Signer,
    secp_privkey: &[u8],
    bls_privkey: &[u8],
    auth_address: &str,
    amount: u128,
    commission_value: u64,
) -> Result<StakingResult> {
    // Derive SECP public key (compressed, 33 bytes)
    let secp_signing_key = k256::ecdsa::SigningKey::from_bytes(secp_privkey.into())
        .map_err(|e| crate::utils::error::Error::Signing(format!("Invalid SECP key: {}", e)))?;
    let secp_pubkey = secp_signing_key.verifying_key().to_encoded_point(true);
    let secp_pubkey_bytes = secp_pubkey.as_bytes();

    // Derive BLS public key (48 bytes)
    let bls_sk = blst::min_pk::SecretKey::from_bytes(bls_privkey)
        .map_err(|e| crate::utils::error::Error::Signing(format!("Invalid BLS key: {:?}", e)))?;
    let bls_pubkey = bls_sk.sk_to_pk();
    let bls_pubkey_bytes = bls_pubkey.to_bytes();

    // Build payload with compressed SECP key (33 bytes)
    let payload = build_validator_payload_compressed(
        secp_pubkey_bytes,
        &bls_pubkey_bytes,
        auth_address,
        amount,
        commission_value,
    );

    // Sign payload with SECP (BLAKE3 hash)
    let secp_sig = sign_validator_payload_secp(&secp_signing_key, &payload)?;

    // Sign payload with BLS
    let bls_sig = sign_validator_payload_bls(bls_privkey, &payload)?;

    // Build calldata
    let data = calldata::encode_add_validator(&payload, &secp_sig, &bls_sig)?;

    // Get nonce
    let nonce = client
        .get_transaction_count(tx_signer.address())
        .await
        .context("Failed to get nonce")?;

    // Build transaction
    let tx = Eip1559Transaction::new(client.get_chain_id().await.unwrap_or(143))
        .with_nonce(nonce)
        .with_gas(
            ADD_VALIDATOR_GAS_LIMIT,
            DEFAULT_MAX_FEE,
            DEFAULT_MAX_PRIORITY_FEE,
        )
        .to(STAKING_CONTRACT_ADDRESS)?
        .with_value(amount)
        .with_data_hex(&data)?;

    sign_and_broadcast(client, tx_signer, &tx).await
}

/// Build validator registration payload with compressed SECP key
///
/// Payload format:
/// secp_pubkey (33 bytes compressed) || bls_pubkey (48 bytes) || auth_address (20 bytes) || amount (32 bytes) || commission (32 bytes)
pub fn build_validator_payload_compressed(
    secp_pubkey: &[u8],
    bls_pubkey: &[u8],
    auth_address: &str,
    amount: u128,
    commission: u64,
) -> Vec<u8> {
    let mut payload = Vec::with_capacity(33 + 48 + 20 + 32 + 32);

    // SECP256k1 public key (33 bytes compressed)
    payload.extend_from_slice(secp_pubkey);

    // BLS12-381 public key (48 bytes)
    payload.extend_from_slice(bls_pubkey);

    // Auth address (20 bytes)
    let addr_clean = auth_address.strip_prefix("0x").unwrap_or(auth_address);
    let addr_bytes = hex::decode(addr_clean).unwrap_or_default();
    payload.extend_from_slice(&addr_bytes);

    // Amount (32 bytes big-endian)
    let mut amount_bytes = [0u8; 32];
    amount_bytes[16..32].copy_from_slice(&amount.to_be_bytes());
    payload.extend_from_slice(&amount_bytes);

    // Commission (32 bytes big-endian)
    let mut commission_bytes = [0u8; 32];
    commission_bytes[24..32].copy_from_slice(&commission.to_be_bytes());
    payload.extend_from_slice(&commission_bytes);

    payload
}

/// Build the validator registration payload
///
/// Payload format:
/// secp_pubkey (64 bytes) || bls_pubkey (48 bytes) || auth_address (20 bytes) || amount (32 bytes) || commission (32 bytes)
pub fn build_validator_payload(
    secp_pubkey: &[u8],
    bls_pubkey: &[u8],
    auth_address: &str,
    amount: u128,
    commission: u64,
) -> Vec<u8> {
    let mut payload = Vec::with_capacity(64 + 48 + 20 + 32 + 32);

    // SECP256k1 public key (64 bytes uncompressed)
    payload.extend_from_slice(secp_pubkey);

    // BLS12-381 public key (48 bytes)
    payload.extend_from_slice(bls_pubkey);

    // Auth address (20 bytes)
    let addr_clean = auth_address.strip_prefix("0x").unwrap_or(auth_address);
    let addr_bytes = hex::decode(addr_clean).unwrap_or_default();
    payload.extend_from_slice(&addr_bytes);

    // Amount (32 bytes big-endian)
    let mut amount_bytes = [0u8; 32];
    amount_bytes[16..32].copy_from_slice(&amount.to_be_bytes());
    payload.extend_from_slice(&amount_bytes);

    // Commission (32 bytes big-endian)
    let mut commission_bytes = [0u8; 32];
    commission_bytes[24..32].copy_from_slice(&commission.to_be_bytes());
    payload.extend_from_slice(&commission_bytes);

    payload
}

/// Compute BLAKE3 hash of data (for add_validator signatures)
pub fn blake3_hash(data: &[u8]) -> [u8; 32] {
    blake3::hash(data).into()
}

/// Sign the validator payload with SECP256k1 (using BLAKE3 hash)
///
/// # Arguments
/// * `signing_key` - SECP256k1 signing key
/// * `payload` - Validator registration payload
///
/// # Returns
/// 64-byte signature (r || s)
pub fn sign_validator_payload_secp(
    signing_key: &k256::ecdsa::SigningKey,
    payload: &[u8],
) -> crate::utils::error::Result<Vec<u8>> {
    // Hash with BLAKE3
    let hash = blake3_hash(payload);

    // Sign the hash using prehash (we already hashed with BLAKE3)
    let (signature, _recovery_id) = signing_key
        .sign_prehash_recoverable(&hash)
        .map_err(|e| crate::utils::error::Error::Signing(format!("SECP signing failed: {}", e)))?;

    Ok(signature.to_bytes().to_vec())
}

/// Sign the validator payload with BLS12-381
///
/// # Arguments
/// * `bls_private_key` - BLS12-381 private key (32 bytes)
/// * `payload` - Validator registration payload
///
/// # Returns
/// 96-byte BLS signature
pub fn sign_validator_payload_bls(
    bls_private_key: &[u8],
    payload: &[u8],
) -> crate::utils::error::Result<Vec<u8>> {
    use blst::min_pk::SecretKey;

    // Create BLS secret key
    let sk = SecretKey::from_bytes(bls_private_key)
        .map_err(|e| crate::utils::error::Error::Signing(format!("Invalid BLS key: {:?}", e)))?;

    // Sign the payload
    let signature = sk.sign(payload, &[], &[]);

    Ok(signature.to_bytes().to_vec())
}

/// Helper function to sign and broadcast a transaction
async fn sign_and_broadcast(
    client: &RpcClient,
    signer: &dyn Signer,
    tx: &Eip1559Transaction,
) -> Result<StakingResult> {
    // Sign the transaction
    let raw_tx = signer.sign_transaction_hex(tx)?;

    // Broadcast
    let tx_hash = client.send_raw_transaction(&raw_tx).await?;

    Ok(StakingResult { tx_hash, raw_tx })
}

/// Helper function to sign, broadcast, and wait for receipt
async fn sign_broadcast_and_wait(
    client: &RpcClient,
    signer: &dyn Signer,
    tx: &Eip1559Transaction,
    config: Option<ReceiptConfig>,
) -> Result<StakingResultWithReceipt> {
    // Sign and broadcast
    let result = sign_and_broadcast(client, signer, tx).await?;

    // Wait for receipt
    let config = config.unwrap_or_default();
    let receipt = wait_for_receipt(client, &result.tx_hash, config).await?;

    Ok(StakingResultWithReceipt {
        tx_hash: result.tx_hash,
        raw_tx: result.raw_tx,
        receipt,
    })
}

// =============================================================================
// Receipt-Waiting Variants
// =============================================================================

/// Delegate MON to a validator and wait for receipt
///
/// # Arguments
/// * `client` - RPC client for the Monad node
/// * `signer` - Signer for the transaction
/// * `validator_id` - Validator ID to delegate to
/// * `amount` - Amount of MON to delegate (in wei)
/// * `config` - Optional receipt waiting configuration
///
/// # Returns
/// Transaction result with receipt
pub async fn delegate_with_receipt(
    client: &RpcClient,
    signer: &dyn Signer,
    validator_id: u64,
    amount: u128,
    config: Option<ReceiptConfig>,
) -> Result<StakingResultWithReceipt> {
    let data = calldata::encode_delegate(validator_id)?;

    let nonce = client
        .get_transaction_count(signer.address())
        .await
        .context("Failed to get nonce")?;

    let tx = Eip1559Transaction::new(client.get_chain_id().await.unwrap_or(143))
        .with_nonce(nonce)
        .with_gas(STAKING_GAS_LIMIT, DEFAULT_MAX_FEE, DEFAULT_MAX_PRIORITY_FEE)
        .to(STAKING_CONTRACT_ADDRESS)?
        .with_value(amount)
        .with_data_hex(&data)?;

    sign_broadcast_and_wait(client, signer, &tx, config).await
}

/// Undelegate MON from a validator and wait for receipt
///
/// # Arguments
/// * `client` - RPC client for the Monad node
/// * `signer` - Signer for the transaction
/// * `validator_id` - Validator ID to undelegate from
/// * `amount` - Amount of MON to undelegate (in wei)
/// * `withdrawal_index` - Index for this withdrawal (0-255, uint8)
/// * `config` - Optional receipt waiting configuration
///
/// # Returns
/// Transaction result with receipt
pub async fn undelegate_with_receipt(
    client: &RpcClient,
    signer: &dyn Signer,
    validator_id: u64,
    amount: u128,
    withdrawal_index: u8,
    config: Option<ReceiptConfig>,
) -> Result<StakingResultWithReceipt> {
    let data = calldata::encode_undelegate(validator_id, amount, withdrawal_index)?;

    let nonce = client
        .get_transaction_count(signer.address())
        .await
        .context("Failed to get nonce")?;

    let tx = Eip1559Transaction::new(client.get_chain_id().await.unwrap_or(143))
        .with_nonce(nonce)
        .with_gas(STAKING_GAS_LIMIT, DEFAULT_MAX_FEE, DEFAULT_MAX_PRIORITY_FEE)
        .to(STAKING_CONTRACT_ADDRESS)?
        .with_data_hex(&data)?;

    sign_broadcast_and_wait(client, signer, &tx, config).await
}

/// Withdraw undelegated MON and wait for receipt
///
/// # Arguments
/// * `client` - RPC client for the Monad node
/// * `signer` - Signer for the transaction
/// * `validator_id` - Validator ID to withdraw from
/// * `withdrawal_index` - Index of the withdrawal request (0-255, uint8)
/// * `config` - Optional receipt waiting configuration
///
/// # Returns
/// Transaction result with receipt
pub async fn withdraw_with_receipt(
    client: &RpcClient,
    signer: &dyn Signer,
    validator_id: u64,
    withdrawal_index: u8,
    config: Option<ReceiptConfig>,
) -> Result<StakingResultWithReceipt> {
    let data = calldata::encode_withdraw(validator_id, withdrawal_index)?;

    let nonce = client
        .get_transaction_count(signer.address())
        .await
        .context("Failed to get nonce")?;

    let tx = Eip1559Transaction::new(client.get_chain_id().await.unwrap_or(143))
        .with_nonce(nonce)
        .with_gas(STAKING_GAS_LIMIT, DEFAULT_MAX_FEE, DEFAULT_MAX_PRIORITY_FEE)
        .to(STAKING_CONTRACT_ADDRESS)?
        .with_data_hex(&data)?;

    sign_broadcast_and_wait(client, signer, &tx, config).await
}

/// Claim staking rewards and wait for receipt
///
/// # Arguments
/// * `client` - RPC client for the Monad node
/// * `signer` - Signer for the transaction
/// * `validator_id` - Validator ID to claim rewards from
/// * `config` - Optional receipt waiting configuration
///
/// # Returns
/// Transaction result with receipt
pub async fn claim_rewards_with_receipt(
    client: &RpcClient,
    signer: &dyn Signer,
    validator_id: u64,
    config: Option<ReceiptConfig>,
) -> Result<StakingResultWithReceipt> {
    let data = calldata::encode_claim_rewards(validator_id)?;

    let nonce = client
        .get_transaction_count(signer.address())
        .await
        .context("Failed to get nonce")?;

    let tx = Eip1559Transaction::new(client.get_chain_id().await.unwrap_or(143))
        .with_nonce(nonce)
        .with_gas(STAKING_GAS_LIMIT, DEFAULT_MAX_FEE, DEFAULT_MAX_PRIORITY_FEE)
        .to(STAKING_CONTRACT_ADDRESS)?
        .with_data_hex(&data)?;

    sign_broadcast_and_wait(client, signer, &tx, config).await
}

/// Compound staking rewards and wait for receipt
///
/// # Arguments
/// * `client` - RPC client for the Monad node
/// * `signer` - Signer for the transaction
/// * `validator_id` - Validator ID to compound rewards for
/// * `config` - Optional receipt waiting configuration
///
/// # Returns
/// Transaction result with receipt
pub async fn compound_with_receipt(
    client: &RpcClient,
    signer: &dyn Signer,
    validator_id: u64,
    config: Option<ReceiptConfig>,
) -> Result<StakingResultWithReceipt> {
    let data = calldata::encode_compound(validator_id)?;

    let nonce = client
        .get_transaction_count(signer.address())
        .await
        .context("Failed to get nonce")?;

    let tx = Eip1559Transaction::new(client.get_chain_id().await.unwrap_or(143))
        .with_nonce(nonce)
        .with_gas(STAKING_GAS_LIMIT, DEFAULT_MAX_FEE, DEFAULT_MAX_PRIORITY_FEE)
        .to(STAKING_CONTRACT_ADDRESS)?
        .with_data_hex(&data)?;

    sign_broadcast_and_wait(client, signer, &tx, config).await
}

/// Change validator commission rate and wait for receipt
///
/// # Arguments
/// * `client` - RPC client for the Monad node
/// * `signer` - Signer for the transaction (must be validator owner)
/// * `validator_id` - Validator ID to change commission for
/// * `commission_bps` - New commission rate in basis points (100 = 1%)
/// * `config` - Optional receipt waiting configuration
///
/// # Returns
/// Transaction result with receipt
pub async fn change_commission_with_receipt(
    client: &RpcClient,
    signer: &dyn Signer,
    validator_id: u64,
    commission_bps: u64,
    config: Option<ReceiptConfig>,
) -> Result<StakingResultWithReceipt> {
    if commission_bps > 10000 {
        return Err(crate::utils::error::Error::InvalidInput(
            "Commission cannot exceed 10000 basis points (100%)".to_string(),
        ));
    }

    let data = calldata::encode_change_commission(validator_id, commission_bps)?;

    let nonce = client
        .get_transaction_count(signer.address())
        .await
        .context("Failed to get nonce")?;

    let tx = Eip1559Transaction::new(client.get_chain_id().await.unwrap_or(143))
        .with_nonce(nonce)
        .with_gas(STAKING_GAS_LIMIT, DEFAULT_MAX_FEE, DEFAULT_MAX_PRIORITY_FEE)
        .to(STAKING_CONTRACT_ADDRESS)?
        .with_data_hex(&data)?;

    sign_broadcast_and_wait(client, signer, &tx, config).await
}

/// Wait for receipt of an already-sent transaction
///
/// This is useful when you already have a transaction hash and want to wait
/// for its confirmation.
///
/// # Arguments
/// * `client` - RPC client for the Monad node
/// * `tx_hash` - Transaction hash (with 0x prefix)
/// * `config` - Receipt waiting configuration
///
/// # Returns
/// Transaction receipt
pub async fn wait_for_staking_receipt(
    client: &RpcClient,
    tx_hash: &str,
    config: ReceiptConfig,
) -> Result<TransactionReceipt> {
    wait_for_receipt(client, tx_hash, config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_validator_payload() {
        let secp_pubkey = [0u8; 64];
        let bls_pubkey = [0u8; 48];
        let auth_address = "0x0000000000000000000000000000000000000001";
        let amount = 100_000_000_000_000_000_000_000u128; // 100,000 MON
        let commission = 5_000_000_000_000_000u64; // 5% in 1e18 scale (5 * 10^16)

        let payload =
            build_validator_payload(&secp_pubkey, &bls_pubkey, auth_address, amount, commission);

        // Should be 64 + 48 + 20 + 32 + 32 = 196 bytes
        assert_eq!(payload.len(), 196);

        // Verify field offsets
        assert_eq!(
            &payload[0..64],
            &secp_pubkey[..],
            "SECP pubkey should be at offset 0"
        );
        assert_eq!(
            &payload[64..112],
            &bls_pubkey[..],
            "BLS pubkey should be at offset 64"
        );
        assert_eq!(
            payload[112..132].len(),
            20,
            "Auth address should be 20 bytes"
        );
        assert_eq!(payload[132..164].len(), 32, "Amount should be 32 bytes");
        assert_eq!(payload[164..196].len(), 32, "Commission should be 32 bytes");
    }

    #[test]
    fn test_build_validator_payload_with_real_values() {
        // Test with realistic non-zero values
        let secp_pubkey = [0xABu8; 64];
        let bls_pubkey = [0xCDu8; 48];
        let auth_address = "0x1234567890123456789012345678901234567890";
        let amount = 500_000_000_000_000_000_000_000u128; // 500,000 MON
        let commission = 10_000_000_000_000_000u64; // 10%

        let payload =
            build_validator_payload(&secp_pubkey, &bls_pubkey, auth_address, amount, commission);

        assert_eq!(payload.len(), 196);

        // Verify SECP pubkey
        assert_eq!(&payload[0..64], &secp_pubkey[..]);

        // Verify BLS pubkey
        assert_eq!(&payload[64..112], &bls_pubkey[..]);

        // Verify auth address (decoded from hex)
        let expected_addr = hex::decode("1234567890123456789012345678901234567890").unwrap();
        assert_eq!(&payload[112..132], expected_addr.as_slice());

        // Verify amount encoding (should be in bytes 16-31 of the 32-byte amount field)
        let amount_bytes = &payload[132..164];
        let expected_amount = [0u8; 16]; // Padding
        let amount_value = amount.to_be_bytes();
        assert_eq!(&amount_bytes[0..16], &expected_amount[..]);
        assert_eq!(&amount_bytes[16..32], &amount_value[..]);

        // Verify commission encoding (should be in bytes 24-31 of the 32-byte commission field)
        let commission_bytes = &payload[164..196];
        let expected_comm_padding = [0u8; 24]; // Padding
        let commission_value = commission.to_be_bytes();
        assert_eq!(&commission_bytes[0..24], &expected_comm_padding[..]);
        assert_eq!(&commission_bytes[24..32], &commission_value[..]);
    }

    #[test]
    fn test_build_validator_payload_address_without_0x_prefix() {
        let secp_pubkey = [0u8; 64];
        let bls_pubkey = [0u8; 48];
        let auth_address = "0000000000000000000000000000000000000001"; // No 0x prefix
        let amount = 100_000_000_000_000_000_000_000u128;
        let commission = 5_000_000_000_000_000u64;

        let payload =
            build_validator_payload(&secp_pubkey, &bls_pubkey, auth_address, amount, commission);

        assert_eq!(payload.len(), 196);
    }

    #[test]
    fn test_build_validator_payload_min_values() {
        // Test with minimum values (edge case)
        let secp_pubkey = [0u8; 64];
        let bls_pubkey = [0u8; 48];
        let auth_address = "0x0000000000000000000000000000000000000000";
        let amount = 0u128;
        let commission = 0u64;

        let payload =
            build_validator_payload(&secp_pubkey, &bls_pubkey, auth_address, amount, commission);

        assert_eq!(payload.len(), 196);

        // Verify amount is all zeros
        assert_eq!(&payload[132..164], &[0u8; 32][..]);

        // Verify commission is all zeros
        assert_eq!(&payload[164..196], &[0u8; 32][..]);
    }

    #[test]
    fn test_build_validator_payload_max_values() {
        // Test with maximum values (edge case)
        let secp_pubkey = [0xFFu8; 64];
        let bls_pubkey = [0xFFu8; 48];
        let auth_address = "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF";
        let amount = u128::MAX;
        let commission = u64::MAX;

        let payload =
            build_validator_payload(&secp_pubkey, &bls_pubkey, auth_address, amount, commission);

        assert_eq!(payload.len(), 196);
    }

    #[test]
    fn test_blake3_hash() {
        let data = b"test data";
        let hash = blake3_hash(data);
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_blake3_hash_deterministic() {
        // Same input should always produce same output
        let data = b"test data";
        let hash1 = blake3_hash(data);
        let hash2 = blake3_hash(data);
        assert_eq!(hash1, hash2, "BLAKE3 should be deterministic");
    }

    #[test]
    fn test_blake3_hash_different_inputs() {
        // Different inputs should produce different outputs
        let hash1 = blake3_hash(b"test data");
        let hash2 = blake3_hash(b"different data");
        assert_ne!(
            hash1, hash2,
            "Different inputs should produce different hashes"
        );
    }

    #[test]
    fn test_blake3_hash_empty_input() {
        // Empty input should still produce valid hash
        let hash = blake3_hash(b"");
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_blake3_hash_known_value() {
        // Test against known BLAKE3 hash output
        // BLAKE3 of "test" is: 4878ca0425c739fa427f7eda20fe845f6b2e46ba5fe2a14df5b1e32f50603215
        let data = b"test";
        let hash = blake3_hash(data);
        let expected_hex = "4878ca0425c739fa427f7eda20fe845f6b2e46ba5fe2a14df5b1e32f50603215";
        let expected = hex::decode(expected_hex).unwrap();
        assert_eq!(hash, expected.as_slice(), "BLAKE3 should match known value");
    }

    #[test]
    fn test_blake3_hash_large_input() {
        // Test with larger input
        let data = vec![0x42u8; 10000]; // 10KB of 0x42
        let hash = blake3_hash(&data);
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_gas_limits() {
        assert_eq!(STAKING_GAS_LIMIT, 1_000_000);
        assert_eq!(ADD_VALIDATOR_GAS_LIMIT, 2_000_000);
    }

    #[test]
    fn test_withdrawal_delay_constant() {
        // WITHDRAWAL_DELAY should match Python SDK value
        assert_eq!(WITHDRAWAL_DELAY, 1);
    }
}
