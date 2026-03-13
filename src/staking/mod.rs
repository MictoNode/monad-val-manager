//! Monad Staking Module
//!
//! This module provides staking operations for the Monad blockchain, including:
//! - Contract selectors and constants
//! - Data types for validators, delegators, and withdrawal requests
//! - ABI encoding/decoding for staking contract calls
//! - High-level getter functions for querying the staking contract
//! - EIP-1559 transaction building and signing
//! - Write operations for staking (delegate, undelegate, withdraw, etc.)
//!
//! # Architecture
//!
//! The module is organized into submodules:
//! - `constants`: Contract addresses and function selectors
//! - `types`: Data structures for staking entities
//! - `calldata`: ABI encoding functions for contract calls
//! - `getters`: High-level functions for querying staking state
//! - `transaction`: EIP-1559 transaction builder
//! - `signer`: Cryptographic signers for transactions
//! - `operations`: High-level staking operations
//!
//! # Example (Read Operations)
//!
//! ```ignore
//! use monad_val_manager::staking::{constants, types, calldata, getters};
//! use monad_val_manager::rpc::RpcClient;
//!
//! // Get the staking contract address
//! let address = constants::STAKING_CONTRACT_ADDRESS;
//!
//! // Encode a get_epoch call (no arguments)
//! let data = calldata::encode_get_epoch();
//!
//! // Query the staking contract
//! let client = RpcClient::new("http://localhost:8080")?;
//! let epoch = getters::get_epoch(&client).await?;
//! println!("Current epoch: {}", epoch.epoch);
//! ```
//!
//! # Example (Write Operations)
//!
//! ```ignore
//! use monad_val_manager::staking::{operations, signer::LocalSigner};
//! use monad_val_manager::rpc::RpcClient;
//!
//! // Create signer from private key
//! let signer = LocalSigner::from_private_key("0x...")?;
//!
//! // Delegate 1 MON to validator #1
//! let client = RpcClient::new("http://localhost:8080")?;
//! let result = operations::delegate(&client, &signer, 1, 1_000_000_000_000_000_000).await?;
//! println!("Transaction hash: {}", result.tx_hash);
//! ```

pub mod calldata;
pub mod constants;
pub mod events;
pub mod getters;
pub mod ledger_signer;
pub mod operations;
pub mod receipt;
pub mod signer;
pub mod signer_factory;
pub mod transaction;
// pub mod transaction_alloy; // Disabled during development
pub mod types;

// Re-export commonly used items for convenience
pub use calldata::{
    decode_delegator, decode_epoch_info, decode_validator, decode_withdrawal_request,
    encode_change_commission, encode_claim_rewards, encode_compound, encode_delegate,
    encode_get_delegations, encode_get_delegator, encode_get_epoch, encode_get_proposer_val_id,
    encode_get_validator, encode_get_withdrawal_request, encode_undelegate, encode_withdraw,
};
pub use constants::{STAKING_CONTRACT_ADDRESS, WITHDRAWAL_DELAY};
pub use getters::{
    get_all_delegations, get_consensus_valset, get_delegations, get_delegator, get_epoch,
    get_execution_valset, get_proposer_val_id, get_snapshot_valset, get_validator,
    get_withdrawal_request,
};
#[cfg(feature = "ledger")]
pub use ledger_signer::LedgerSigner;
pub use operations::{
    add_validator, add_validator_from_privkeys, build_validator_payload_compressed,
    change_commission, change_commission_with_receipt, check_withdrawal_ready, claim_rewards,
    claim_rewards_with_receipt, compound, compound_with_receipt, delegate, delegate_with_receipt,
    sign_validator_payload_bls, sign_validator_payload_secp, undelegate, undelegate_with_receipt,
    wait_for_staking_receipt, withdraw, withdraw_with_receipt, AddValidatorParams, StakingResult,
    StakingResultWithReceipt, ADD_VALIDATOR_GAS_LIMIT, STAKING_GAS_LIMIT,
};
pub use receipt::{
    format_receipt, wait_for_receipt, wait_for_receipt_with_progress, ReceiptConfig,
    TransactionReceipt, TransactionStatus, DEFAULT_POLL_INTERVAL_SECS, DEFAULT_TIMEOUT_SECS,
};
pub use signer::{EcdsaSignature, LocalSigner, Signer};
pub use signer_factory::{
    create_signer, create_signer_with_type, get_signer_type, is_ledger_supported, SignerType,
};
pub use transaction::{
    Eip1559Transaction, DEFAULT_GAS_LIMIT, DEFAULT_MAX_FEE, DEFAULT_MAX_PRIORITY_FEE,
};
pub use types::{Delegator, EpochInfo, Validator, WithdrawalRequest};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify re-exports are accessible
        let _ = STAKING_CONTRACT_ADDRESS;
        let _ = EpochInfo::default();
        let _ = Validator::default();
        let _ = Delegator::default();
        let _ = WithdrawalRequest::default();
    }

    #[test]
    fn test_encode_get_epoch_accessible() {
        let data = encode_get_epoch();
        assert!(data.starts_with("0x"));
    }
}
