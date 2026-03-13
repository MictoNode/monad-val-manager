//! Staking State - State management for staking screen
//!
//! This module provides the main StakingState struct for displaying staking information
//! in the TUI, including delegations, pending withdrawals, and loading states.

use std::time::Instant;

use super::helpers::{format_balance, format_mon_amount, truncate_address};
use super::types::{DelegationInfo, PendingStakingAction, PendingWithdrawal, StakingActionResult};

/// State for the staking screen
///
/// Holds all staking-related data for display, including user's delegations,
/// pending withdrawals, and UI state (loading, errors).
/// Also tracks pending actions and execution results.
#[derive(Debug, Clone, Default)]
pub struct StakingState {
    /// User's delegator address (if connected)
    pub delegator_address: Option<String>,
    /// User's MON balance (human-readable, e.g., 100.5 for 100.5 MON)
    pub balance: f64,
    /// Current epoch number
    pub current_epoch: u64,
    /// List of user's delegations
    pub delegations: Vec<DelegationInfo>,
    /// List of pending withdrawal requests
    pub pending_withdrawals: Vec<PendingWithdrawal>,
    /// Total rewards available across all delegations
    pub total_rewards: u128,
    /// Is data currently being loaded
    pub is_loading: bool,
    /// Last error message (if any)
    pub error: Option<String>,
    /// Status message (for displaying action results)
    pub status_message: Option<String>,
    /// Timestamp of last successful refresh
    pub last_refresh: Option<Instant>,
    /// Selected delegation index for navigation
    pub selected_index: usize,
    /// Is an action currently being executed
    pub is_executing: bool,
    /// Pending action waiting for confirmation/execution
    pub pending_action: Option<PendingStakingAction>,
    /// Result of the last executed action
    pub last_result: Option<StakingActionResult>,
}

impl StakingState {
    /// Create new staking state
    pub fn new() -> Self {
        Self::default()
    }

    /// Create staking state with delegator address
    pub fn with_address(address: impl Into<String>) -> Self {
        Self {
            delegator_address: Some(address.into()),
            ..Self::default()
        }
    }

    /// Set loading state
    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
        if loading {
            self.error = None;
        }
    }

    /// Set error state
    pub fn set_error(&mut self, error: impl Into<String>) {
        self.is_loading = false;
        self.error = Some(error.into());
    }

    /// Clear error state
    pub fn clear_error(&mut self) {
        self.error = None;
    }

    /// Set status message
    pub fn set_status_message(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
    }

    /// Clear status message
    pub fn clear_status_message(&mut self) {
        self.status_message = None;
    }

    /// Mark refresh as complete
    pub fn mark_refreshed(&mut self) {
        self.is_loading = false;
        self.error = None;
        self.last_refresh = Some(Instant::now());
    }

    /// Update delegator address
    pub fn set_address(&mut self, address: impl Into<String>) {
        self.delegator_address = Some(address.into());
    }

    /// Update balance (MON as f64)
    pub fn set_balance(&mut self, balance: f64) {
        self.balance = balance;
    }

    /// Update current epoch
    pub fn set_epoch(&mut self, epoch: u64) {
        self.current_epoch = epoch;
    }

    /// Set delegations (also calculates total rewards)
    pub fn set_delegations(&mut self, delegations: Vec<DelegationInfo>) {
        self.total_rewards = delegations.iter().map(|d| d.rewards).sum();
        self.delegations = delegations;
        // Reset selection if out of bounds
        if self.selected_index >= self.delegations.len() && !self.delegations.is_empty() {
            self.selected_index = self.delegations.len() - 1;
        }
    }

    /// Set pending withdrawals
    pub fn set_withdrawals(&mut self, withdrawals: Vec<PendingWithdrawal>) {
        self.pending_withdrawals = withdrawals;
    }

    /// Format balance for display (balance is stored as f64 MON)
    pub fn format_balance(&self) -> String {
        format_balance(self.balance)
    }

    /// Format total rewards for display
    pub fn format_total_rewards(&self) -> String {
        format_mon_amount(self.total_rewards)
    }

    /// Format delegator address for display (truncated)
    pub fn format_address(&self) -> String {
        match &self.delegator_address {
            Some(addr) => truncate_address(addr),
            None => "Not connected".to_string(),
        }
    }

    /// Check if user has any delegations
    pub fn has_delegations(&self) -> bool {
        !self.delegations.is_empty()
    }

    /// Check if user has pending withdrawals
    pub fn has_pending_withdrawals(&self) -> bool {
        !self.pending_withdrawals.is_empty()
    }

    /// Get count of ready withdrawals
    pub fn ready_withdrawal_count(&self) -> usize {
        self.pending_withdrawals
            .iter()
            .filter(|w| w.is_ready(self.current_epoch))
            .count()
    }

    /// Move selection up
    pub fn select_prev(&mut self) {
        if !self.delegations.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.delegations.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if !self.delegations.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.delegations.len();
        }
    }

    /// Get selected delegation
    pub fn selected_delegation(&self) -> Option<&DelegationInfo> {
        self.delegations.get(self.selected_index)
    }

    /// Reset state to defaults
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    // === Action Execution State Management ===

    /// Start executing an action
    pub fn start_execution(&mut self, action: PendingStakingAction) {
        self.is_executing = true;
        self.pending_action = Some(action);
        self.last_result = None;
        self.error = None;
    }

    /// Complete execution with a result
    pub fn complete_execution(&mut self, result: StakingActionResult) {
        self.is_executing = false;
        self.last_result = Some(result);
        self.pending_action = None;
    }

    /// Cancel a pending action
    pub fn cancel_execution(&mut self) {
        self.is_executing = false;
        self.pending_action = None;
    }

    /// Clear the last result
    pub fn clear_result(&mut self) {
        self.last_result = None;
    }

    /// Check if there's a recent result to display
    pub fn has_recent_result(&self, max_age_secs: u64) -> bool {
        self.last_result
            .as_ref()
            .map(|r| r.is_recent(max_age_secs))
            .unwrap_or(false)
    }

    /// Get the selected validator ID, if any
    pub fn selected_validator_id(&self) -> Option<u64> {
        self.selected_delegation().map(|d| d.validator_id)
    }

    /// Prepare a delegate action from current selection and amount
    pub fn prepare_delegate(&self, amount: u128) -> Option<PendingStakingAction> {
        self.selected_validator_id()
            .map(|vid| PendingStakingAction::delegate(vid, amount))
    }

    /// Prepare an undelegate action from current selection and amount
    pub fn prepare_undelegate(
        &self,
        amount: u128,
        withdrawal_index: u8,
    ) -> Option<PendingStakingAction> {
        self.selected_validator_id()
            .map(|vid| PendingStakingAction::undelegate(vid, amount, withdrawal_index))
    }

    /// Prepare a withdraw action from current selection
    pub fn prepare_withdraw(&self, withdrawal_index: u8) -> Option<PendingStakingAction> {
        self.selected_validator_id()
            .map(|vid| PendingStakingAction::withdraw(vid, withdrawal_index))
    }

    /// Prepare a claim rewards action from current selection
    pub fn prepare_claim_rewards(&self) -> Option<PendingStakingAction> {
        self.selected_validator_id()
            .map(PendingStakingAction::claim_rewards)
    }

    /// Prepare a compound action from current selection
    ///
    /// Note: This creates a compound action for the selected validator.
    pub fn prepare_compound(&self) -> Option<PendingStakingAction> {
        self.selected_validator_id()
            .map(PendingStakingAction::compound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_staking_state_default() {
        let state = StakingState::new();
        assert!(state.delegator_address.is_none());
        assert_eq!(state.balance, 0.0);
        assert!(!state.is_loading);
        assert!(state.error.is_none());
    }

    #[test]
    fn test_staking_state_with_address() {
        let state = StakingState::with_address("0x1234567890abcdef");
        assert_eq!(
            state.delegator_address,
            Some("0x1234567890abcdef".to_string())
        );
    }

    #[test]
    fn test_staking_state_set_loading() {
        let mut state = StakingState::new();
        state.error = Some("previous error".to_string());
        state.set_loading(true);
        assert!(state.is_loading);
        assert!(state.error.is_none());
    }

    #[test]
    fn test_staking_state_set_error() {
        let mut state = StakingState::new();
        state.is_loading = true;
        state.set_error("test error");
        assert!(!state.is_loading);
        assert_eq!(state.error, Some("test error".to_string()));
    }

    #[test]
    fn test_staking_state_mark_refreshed() {
        let mut state = StakingState::new();
        state.is_loading = true;
        state.error = Some("error".to_string());
        state.mark_refreshed();
        assert!(!state.is_loading);
        assert!(state.error.is_none());
        assert!(state.last_refresh.is_some());
    }

    #[test]
    fn test_staking_state_set_delegations() {
        let mut state = StakingState::new();
        let mut d1 = DelegationInfo::new(1);
        d1.rewards = 100;
        let mut d2 = DelegationInfo::new(2);
        d2.rewards = 200;

        state.set_delegations(vec![d1, d2]);
        assert_eq!(state.delegations.len(), 2);
        assert_eq!(state.total_rewards, 300);
    }

    #[test]
    fn test_staking_state_navigation() {
        let mut state = StakingState::new();
        let d1 = DelegationInfo::new(1);
        let d2 = DelegationInfo::new(2);
        let d3 = DelegationInfo::new(3);
        state.set_delegations(vec![d1, d2, d3]);

        assert_eq!(state.selected_index, 0);
        state.select_next();
        assert_eq!(state.selected_index, 1);
        state.select_next();
        assert_eq!(state.selected_index, 2);
        state.select_next();
        assert_eq!(state.selected_index, 0); // Wraps around

        state.select_prev();
        assert_eq!(state.selected_index, 2); // Wraps around
    }

    #[test]
    fn test_staking_state_selected_delegation() {
        let mut state = StakingState::new();
        let d1 = DelegationInfo::new(1);
        let d2 = DelegationInfo::new(2);
        state.set_delegations(vec![d1, d2]);

        state.selected_index = 1;
        let selected = state.selected_delegation();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().validator_id, 2);
    }

    #[test]
    fn test_staking_state_ready_withdrawals() {
        let mut state = StakingState::new();
        state.current_epoch = 100;
        state.set_withdrawals(vec![
            PendingWithdrawal::new(1, 100, 98, 0),  // Ready (100 >= 98 + 1)
            PendingWithdrawal::new(2, 200, 99, 0),  // Ready (100 >= 99 + 1)
            PendingWithdrawal::new(3, 300, 100, 0), // Not ready (100 < 100 + 1)
        ]);

        assert_eq!(state.ready_withdrawal_count(), 2);
    }

    #[test]
    fn test_staking_state_format_address() {
        let mut state = StakingState::new();
        assert_eq!(state.format_address(), "Not connected");

        state.set_address("0x1234567890abcdef1234567890abcdef12345678");
        assert_eq!(state.format_address(), "0x1234...5678");
    }

    #[test]
    fn test_staking_state_reset() {
        let mut state = StakingState::with_address("0x1234");
        state.balance = 1000.0; // Balance is now f64 (MON)
        state.set_delegations(vec![DelegationInfo::new(1)]);
        state.reset();

        assert!(state.delegator_address.is_none());
        assert_eq!(state.balance, 0.0);
        assert!(state.delegations.is_empty());
    }

    #[test]
    fn test_staking_state_start_execution() {
        let mut state = StakingState::new();
        let action = PendingStakingAction::delegate(1, 1000);
        state.start_execution(action.clone());

        assert!(state.is_executing);
        assert_eq!(state.pending_action, Some(action));
        assert!(state.last_result.is_none());
    }

    #[test]
    fn test_staking_state_complete_execution() {
        let mut state = StakingState::new();
        let action = PendingStakingAction::delegate(1, 1000);
        state.start_execution(action);

        let result = StakingActionResult::success("0xabc".to_string());
        state.complete_execution(result.clone());

        assert!(!state.is_executing);
        assert!(state.pending_action.is_none());
        assert_eq!(state.last_result, Some(result));
    }

    #[test]
    fn test_staking_state_cancel_execution() {
        let mut state = StakingState::new();
        let action = PendingStakingAction::delegate(1, 1000);
        state.start_execution(action);
        state.cancel_execution();

        assert!(!state.is_executing);
        assert!(state.pending_action.is_none());
    }

    #[test]
    fn test_staking_state_prepare_delegate() {
        let mut state = StakingState::new();
        state.set_delegations(vec![DelegationInfo::new(42)]);

        let action = state.prepare_delegate(1_000_000_000_000_000_000);
        assert!(action.is_some());
        let action = action.unwrap();
        assert_eq!(action.action_type, crate::tui::StakingActionType::Delegate);
        assert_eq!(action.validator_id, 42);
    }

    #[test]
    fn test_staking_state_prepare_delegate_no_selection() {
        let state = StakingState::new();
        let action = state.prepare_delegate(1000);
        assert!(action.is_none());
    }

    #[test]
    fn test_staking_state_selected_validator_id() {
        let mut state = StakingState::new();
        assert!(state.selected_validator_id().is_none());

        state.set_delegations(vec![DelegationInfo::new(5), DelegationInfo::new(10)]);
        assert_eq!(state.selected_validator_id(), Some(5));

        state.selected_index = 1;
        assert_eq!(state.selected_validator_id(), Some(10));
    }

    #[test]
    fn test_staking_state_has_recent_result() {
        let mut state = StakingState::new();
        assert!(!state.has_recent_result(5));

        state.last_result = Some(StakingActionResult::success("0xabc".to_string()));
        assert!(state.has_recent_result(5));
    }
}
