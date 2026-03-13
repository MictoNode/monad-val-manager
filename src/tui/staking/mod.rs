//! Staking module - Types and state for staking screen
//!
//! This module provides the state structures for displaying staking information
//! in the TUI. including delegations, pending withdrawals, and loading states.

pub mod add_validator_state;
pub mod change_commission_state;
pub mod delegate_state;
pub mod helpers;
pub mod query_delegator_state;
pub mod query_validator_state;
pub mod state;
pub mod types;
pub mod undelegate_state;
pub mod withdraw_state;

// Re-export all public types for convenience
pub use add_validator_state::{AddValidatorField, AddValidatorParams, AddValidatorState};
pub use change_commission_state::{
    ChangeCommissionField, ChangeCommissionParams, ChangeCommissionState,
};
pub use delegate_state::{DelegateField, DelegateParams, DelegateState};
pub use helpers::{format_balance, format_mon_amount, truncate_address};
pub use query_delegator_state::{QueryDelegatorField, QueryDelegatorParams, QueryDelegatorState};
pub use query_validator_state::{QueryValidatorResult, QueryValidatorState};
pub use state::StakingState;
pub use types::{
    DelegationInfo, PendingStakingAction, PendingWithdrawal, StakingActionResult, StakingActionType,
};
pub use undelegate_state::{UndelegateField, UndelegateParams, UndelegateState};
pub use withdraw_state::{WithdrawField, WithdrawParams, WithdrawState};
