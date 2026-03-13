//! Staking Types - Type definitions for staking operations
//!
//! This module provides all type definitions used in staking operations,
//! including action types, pending actions, results, and delegation info.

use std::time::Instant;

use super::helpers::format_mon_amount;
use crate::staking::constants::WITHDRAWAL_DELAY;

/// Types of staking actions that can be requested from the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StakingActionType {
    /// Delegate MON to a validator
    Delegate,
    /// Undelegate MON from a validator
    Undelegate,
    /// Withdraw pending undelegated amount
    Withdraw,
    /// Claim staking rewards
    ClaimRewards,
    /// Compound rewards back into delegation
    Compound,
}

impl StakingActionType {
    /// Get human-readable name for the action
    pub fn name(&self) -> &'static str {
        match self {
            StakingActionType::Delegate => "Delegate",
            StakingActionType::Undelegate => "Undelegate",
            StakingActionType::Withdraw => "Withdraw",
            StakingActionType::ClaimRewards => "Claim Rewards",
            StakingActionType::Compound => "Compound",
        }
    }

    /// Check if this action requires an amount input
    pub fn requires_amount(&self) -> bool {
        matches!(
            self,
            StakingActionType::Delegate | StakingActionType::Undelegate
        )
    }

    /// Check if this action uses the selected validator
    pub fn uses_selected_validator(&self) -> bool {
        matches!(
            self,
            StakingActionType::Delegate
                | StakingActionType::Undelegate
                | StakingActionType::Withdraw
                | StakingActionType::ClaimRewards
                | StakingActionType::Compound
        )
    }
}

/// A pending staking action waiting for execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingStakingAction {
    /// Type of action to perform
    pub action_type: StakingActionType,
    /// Target validator ID (from selected delegation)
    pub validator_id: u64,
    /// Amount in smallest unit (for delegate/undelegate)
    pub amount: Option<u128>,
    /// Withdrawal index (0-255, uint8, for withdraw operations)
    pub withdrawal_index: Option<u8>,
    /// Authorized address (for compound, change_commission)
    pub auth_address: Option<String>,
}

impl PendingStakingAction {
    /// Create a new pending action
    pub fn new(action_type: StakingActionType, validator_id: u64) -> Self {
        Self {
            action_type,
            validator_id,
            amount: None,
            withdrawal_index: None,
            auth_address: None,
        }
    }

    /// Create a delegate action
    pub fn delegate(validator_id: u64, amount: u128) -> Self {
        Self {
            action_type: StakingActionType::Delegate,
            validator_id,
            amount: Some(amount),
            withdrawal_index: None,
            auth_address: None,
        }
    }

    /// Create an undelegate action
    pub fn undelegate(validator_id: u64, amount: u128, withdrawal_index: u8) -> Self {
        Self {
            action_type: StakingActionType::Undelegate,
            validator_id,
            amount: Some(amount),
            withdrawal_index: Some(withdrawal_index),
            auth_address: None,
        }
    }

    /// Create a withdraw action
    pub fn withdraw(validator_id: u64, withdrawal_index: u8) -> Self {
        Self {
            action_type: StakingActionType::Withdraw,
            validator_id,
            amount: None,
            withdrawal_index: Some(withdrawal_index),
            auth_address: None,
        }
    }

    /// Create a claim rewards action
    pub fn claim_rewards(validator_id: u64) -> Self {
        Self {
            action_type: StakingActionType::ClaimRewards,
            validator_id,
            amount: None,
            withdrawal_index: None,
            auth_address: None,
        }
    }

    /// Create a compound action
    pub fn compound(validator_id: u64) -> Self {
        Self {
            action_type: StakingActionType::Compound,
            validator_id,
            amount: None,
            withdrawal_index: None,
            auth_address: None,
        }
    }

    /// Get description of this action for display
    pub fn description(&self) -> String {
        match self.action_type {
            StakingActionType::Delegate => {
                let amount_str = self
                    .amount
                    .map(format_mon_amount)
                    .unwrap_or_else(|| "??".to_string());
                format!(
                    "Delegate {} MON to Validator #{}",
                    amount_str, self.validator_id
                )
            }
            StakingActionType::Undelegate => {
                let amount_str = self
                    .amount
                    .map(format_mon_amount)
                    .unwrap_or_else(|| "??".to_string());
                format!(
                    "Undelegate {} MON from Validator #{}",
                    amount_str, self.validator_id
                )
            }
            StakingActionType::Withdraw => {
                format!(
                    "Withdraw from Validator #{} (index {})",
                    self.validator_id,
                    self.withdrawal_index.unwrap_or(0)
                )
            }
            StakingActionType::ClaimRewards => {
                format!("Claim rewards from Validator #{}", self.validator_id)
            }
            StakingActionType::Compound => {
                format!("Compound rewards for Validator #{}", self.validator_id)
            }
        }
    }
}

/// Result of a staking action execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StakingActionResult {
    /// Whether the action succeeded
    pub success: bool,
    /// Transaction hash (if successful)
    pub tx_hash: Option<String>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Timestamp when result was received
    pub timestamp: Instant,
}

impl StakingActionResult {
    /// Create a successful result
    pub fn success(tx_hash: String) -> Self {
        Self {
            success: true,
            tx_hash: Some(tx_hash),
            error: None,
            timestamp: Instant::now(),
        }
    }

    /// Create a failed result
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            tx_hash: None,
            error: Some(error.into()),
            timestamp: Instant::now(),
        }
    }

    /// Check if this result is still recent (within last N seconds)
    pub fn is_recent(&self, max_age_secs: u64) -> bool {
        self.timestamp.elapsed().as_secs() < max_age_secs
    }

    /// Format for display
    pub fn format_summary(&self) -> String {
        if self.success {
            let hash_display = self
                .tx_hash
                .as_ref()
                .map(|h| {
                    if h.len() > 16 {
                        format!("{}...", &h[..16])
                    } else {
                        h.clone()
                    }
                })
                .unwrap_or_else(|| "N/A".to_string());
            format!("Success! TX: {}", hash_display)
        } else {
            format!(
                "Failed: {}",
                self.error.as_deref().unwrap_or("Unknown error")
            )
        }
    }
}

/// Delegation information for display in the TUI
///
/// Represents a user's delegation to a single validator with associated metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelegationInfo {
    /// Validator ID
    pub validator_id: u64,
    /// Validator's authorized address (for display)
    pub validator_address: String,
    /// Amount delegated in MON (as smallest unit, 18 decimals)
    pub delegated_amount: u128,
    /// Pending amount awaiting activation
    pub pending_amount: u128,
    /// Unclaimed rewards
    pub rewards: u128,
    /// Validator's commission rate in basis points (100 = 1%)
    pub commission: u64,
    /// Whether the validator is active
    pub is_active: bool,
}

impl Default for DelegationInfo {
    fn default() -> Self {
        Self {
            validator_id: 0,
            validator_address: String::new(),
            delegated_amount: 0,
            pending_amount: 0,
            rewards: 0,
            commission: 0,
            is_active: true,
        }
    }
}

impl DelegationInfo {
    /// Create a new delegation info with validator ID
    pub fn new(validator_id: u64) -> Self {
        Self {
            validator_id,
            ..Self::default()
        }
    }

    /// Format delegated amount for display (converts from smallest unit to MON)
    pub fn format_delegated_amount(&self) -> String {
        format_mon_amount(self.delegated_amount)
    }

    /// Format pending amount for display
    pub fn format_pending_amount(&self) -> String {
        format_mon_amount(self.pending_amount)
    }

    /// Format rewards for display
    pub fn format_rewards(&self) -> String {
        format_mon_amount(self.rewards)
    }

    /// Format commission rate as percentage string
    pub fn format_commission(&self) -> String {
        let percent = self.commission as f64 / 100.0;
        format!("{:.1}%", percent)
    }

    /// Create DelegationInfo from query result (RPC response)
    ///
    /// This converts the delegator query response into DelegationInfo format
    /// for display in the TUI staking screen.
    pub fn from_query_result(
        validator_id: u64,
        delegator: &crate::staking::types::Delegator,
    ) -> Self {
        // Delta stake represents pending undelegation, not pending activation
        // For now, set pending_amount to 0 since we don't track pending activations
        let pending_amount = delegator.next_delta_stake;

        Self {
            validator_id,
            validator_address: format!("Validator #{}", validator_id),
            delegated_amount: delegator.delegated_amount,
            pending_amount,
            rewards: delegator.rewards,
            commission: 0, // Commission comes from validator query, not delegator query
            is_active: true, // Assume active if queried
        }
    }
}

/// Pending withdrawal request for display
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PendingWithdrawal {
    /// Validator ID
    pub validator_id: u64,
    /// Amount to withdraw in smallest unit
    pub amount: u128,
    /// Epoch when withdrawal becomes available
    pub activation_epoch: u64,
    /// Withdrawal index (0-255, uint8)
    pub withdrawal_index: u8,
}

impl PendingWithdrawal {
    /// Create a new pending withdrawal
    pub fn new(validator_id: u64, amount: u128, activation_epoch: u64, index: u8) -> Self {
        Self {
            validator_id,
            amount,
            activation_epoch,
            withdrawal_index: index,
        }
    }

    /// Format amount for display
    pub fn format_amount(&self) -> String {
        format_mon_amount(self.amount)
    }

    /// Check if withdrawal is ready based on current epoch
    /// Matches contract logic: withdrawal is ready after WITHDRAWAL_DELAY epochs
    pub fn is_ready(&self, current_epoch: u64) -> bool {
        let required_epoch = self.activation_epoch.saturating_add(WITHDRAWAL_DELAY);
        current_epoch >= required_epoch
    }
}
