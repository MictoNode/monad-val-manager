//! ABI calldata encoding and decoding for staking operations
//!
//! This module provides functions to encode calldata for Monad staking contract calls
//! and decode responses from the contract.
//!
//! # Modules
//!
//! - [`encode`] - Encoding functions for write and read operations
//! - [`decode`] - Decoding functions for contract responses
//!
//! # ABI Encoding Rules
//!
//! - All values are 32-byte aligned
//! - uint64, uint256, uint8 are left-padded with zeros
//! - address is left-padded to 32 bytes
//! - Dynamic types (bytes, arrays) use pointer-based encoding

mod decode;
mod encode;

// Re-export all public encoding functions
pub use encode::{
    encode_add_validator, encode_change_commission, encode_claim_rewards, encode_compound,
    encode_delegate, encode_get_consensus_valset, encode_get_delegations, encode_get_delegator,
    encode_get_delegators, encode_get_epoch, encode_get_execution_valset,
    encode_get_proposer_val_id, encode_get_snapshot_valset, encode_get_validator,
    encode_get_withdrawal_request, encode_undelegate, encode_withdraw,
};

// Re-export all public decoding functions
pub use decode::{
    decode_delegation_list, decode_delegator, decode_delegator_list, decode_epoch_info,
    decode_validator, decode_validator_set, decode_withdrawal_request,
};
