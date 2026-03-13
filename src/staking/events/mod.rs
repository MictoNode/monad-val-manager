//! Event parsing for Monad staking contract
//!
//! This module provides event parsing capabilities for staking contract events.
//! Events are emitted by the staking contract for operations like delegate,
//! undelegate, withdraw, claim_rewards, compound, etc.
//!
//! # Event Structure
//!
//! Ethereum events consist of:
//! - topics[0]: Event signature hash (keccak256 of event signature)
//! - topics[1..]: Indexed parameters (up to 3 additional topics)
//! - data: Non-indexed parameters (ABI-encoded)
//!
//! # Reference
//!
//! Event signatures match the official Monad staking SDK:
//! - Delegate(uint64 indexed valId, address indexed delegator, uint256 amount, uint64 activationEpoch)
//! - Undelegate(uint64 indexed valId, address indexed delegator, uint8 withdrawal_id, uint256 amount, uint64 activationEpoch)
//! - Withdraw(uint64 indexed valId, address indexed delegator, uint8 withdrawal_id, uint256 amount, uint64 activationEpoch)
//! - ClaimRewards(uint64 indexed valId, address indexed delegator, uint256 amount, uint64 epoch)
//!
//! # Usage
//!
//! ```ignore
//! use monad_val_manager::staking::events::{parse_event, TransactionLog, StakingEvent};
//!
//! // Parse a single log
//! let log = TransactionLog {
//!     address: "0x...".to_string(),
//!     topics: vec!["0x...".to_string()],
//!     data: "0x...".to_string(),
//! };
//!
//! match parse_event(&log)? {
//!     Some(StakingEvent::Delegate(e)) => {
//!         println!("Delegated {} to validator {}", e.amount, e.validator_id);
//!     }
//!     Some(StakingEvent::Undelegate(e)) => {
//!         println!("Undelegated {} from validator {}", e.amount, e.validator_id);
//!     }
//!     // ... handle other events
//!     None => println!("Not a staking event"),
//! }
//! ```

// Submodules
mod helpers;
mod parsing;
mod signatures;
mod types;

// Re-export public API

// Event signatures
pub use signatures::{
    compute_event_signature_hash, ADD_VALIDATOR_EVENT_SIGNATURE, CHANGE_COMMISSION_EVENT_SIGNATURE,
    CLAIM_REWARDS_EVENT_SIGNATURE, COMPOUND_EVENT_SIGNATURE, DELEGATE_EVENT_SIGNATURE,
    UNDELEGATE_EVENT_SIGNATURE, WITHDRAW_EVENT_SIGNATURE,
};

// Event types
pub use types::{
    AddValidatorEvent, ChangeCommissionEvent, ClaimRewardsEvent, CompoundEvent, DelegateEvent,
    StakingEvent, TransactionLog, UndelegateEvent, WithdrawEvent,
};

// Parsing functions
pub use parsing::{extract_staking_events, parse_event};
