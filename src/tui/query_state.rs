//! TUI Query State - Query menu and result management
//!
//! Manages the query selection menu and displays query results.

use crate::handlers::format_mon;
use crate::staking::types::{Delegator, EpochInfo, Validator, ValidatorSet, WithdrawalRequest};

// Type aliases for consistency with QueryResult naming
pub type ValidatorInfo = Validator;
pub type DelegatorInfo = Delegator;
pub type WithdrawalInfo = WithdrawalRequest;
pub type ValidatorSetInfo = ValidatorSet;

/// Query type selection menu
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QueryType {
    /// Current epoch information
    #[default]
    Epoch,
    /// Validator information
    Validator,
    /// Delegator information
    Delegator,
    /// Withdrawal request information
    WithdrawalRequest,
    /// All delegations for an address
    Delegations,
    /// Validator set (consensus/execution/snapshot)
    ValidatorSet,
    /// Current proposer validator
    Proposer,
    /// Estimate gas for transaction
    EstimateGas,
    /// Transaction by hash
    Transaction,
}

impl QueryType {
    /// Get all query types in menu order
    pub const fn all() -> [QueryType; 9] {
        [
            QueryType::Epoch,
            QueryType::Validator,
            QueryType::Delegator,
            QueryType::WithdrawalRequest,
            QueryType::Delegations,
            QueryType::ValidatorSet,
            QueryType::Proposer,
            QueryType::EstimateGas,
            QueryType::Transaction,
        ]
    }

    /// Get the display name for this query type
    pub fn name(self) -> &'static str {
        match self {
            QueryType::Epoch => "Query Epoch",
            QueryType::Validator => "Query Validator",
            QueryType::Delegator => "Query Delegator",
            QueryType::WithdrawalRequest => "Query Withdrawal Request",
            QueryType::Delegations => "Query Delegations",
            QueryType::ValidatorSet => "Query Validator Set",
            QueryType::Proposer => "Query Proposer",
            QueryType::EstimateGas => "Estimate Gas",
            QueryType::Transaction => "Query Transaction",
        }
    }

    /// Get the description for this query type
    pub fn description(self) -> &'static str {
        match self {
            QueryType::Epoch => "Get current epoch information",
            QueryType::Validator => "Get validator details by ID",
            QueryType::Delegator => "Get delegator info for validator",
            QueryType::WithdrawalRequest => "Get withdrawal request status",
            QueryType::Delegations => "Get all delegations for address",
            QueryType::ValidatorSet => "Get validator set (consensus/execution/snapshot)",
            QueryType::Proposer => "Get current proposer validator",
            QueryType::EstimateGas => "Estimate gas for transaction",
            QueryType::Transaction => "Get transaction details by hash",
        }
    }

    /// Get the next query type in the menu
    pub fn next(self) -> Self {
        let types = Self::all();
        let current_index = types.iter().position(|&t| t == self).unwrap_or(0);
        let next_index = (current_index + 1) % types.len();
        types[next_index]
    }

    /// Get the previous query type in the menu
    pub fn prev(self) -> Self {
        let types = Self::all();
        let current_index = types.iter().position(|&t| t == self).unwrap_or(0);
        let prev_index = if current_index == 0 {
            types.len() - 1
        } else {
            current_index - 1
        };
        types[prev_index]
    }

    /// Check if this query type requires parameters
    pub fn requires_params(self) -> bool {
        !matches!(
            self,
            QueryType::Epoch | QueryType::ValidatorSet | QueryType::Proposer
        )
    }
}

/// Validator set type selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ValidatorSetType {
    /// Consensus validator set
    #[default]
    Consensus,
    /// Execution validator set
    Execution,
    /// Snapshot validator set
    Snapshot,
}

impl ValidatorSetType {
    /// Get all validator set types
    pub const fn all() -> [ValidatorSetType; 3] {
        [
            ValidatorSetType::Consensus,
            ValidatorSetType::Execution,
            ValidatorSetType::Snapshot,
        ]
    }

    /// Get the display name
    pub fn name(self) -> &'static str {
        match self {
            ValidatorSetType::Consensus => "consensus",
            ValidatorSetType::Execution => "execution",
            ValidatorSetType::Snapshot => "snapshot",
        }
    }

    /// Get the next validator set type
    pub fn next(self) -> Self {
        let types = Self::all();
        let current_index = types.iter().position(|&t| t == self).unwrap_or(0);
        let next_index = (current_index + 1) % types.len();
        types[next_index]
    }
}

impl std::str::FromStr for ValidatorSetType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "consensus" => Ok(ValidatorSetType::Consensus),
            "execution" => Ok(ValidatorSetType::Execution),
            "snapshot" => Ok(ValidatorSetType::Snapshot),
            _ => Err(format!("Unknown validator set type: {}", s)),
        }
    }
}

/// Query result data
#[derive(Debug, Clone)]
pub enum QueryResult {
    /// Epoch query result
    Epoch(EpochInfo),
    /// Validator query result
    Validator(ValidatorInfo),
    /// Delegator query result
    Delegator(DelegatorInfo),
    /// Withdrawal request query result
    WithdrawalRequest(WithdrawalInfo),
    /// Delegations query result (list of validator IDs)
    Delegations(Vec<u64>),
    /// Validator set query result
    ValidatorSet(ValidatorSetInfo),
    /// Proposer query result (validator ID)
    Proposer(u64),
    /// Estimate gas result (gas units)
    EstimateGas(u64),
    /// Transaction query result (JSON string)
    Transaction(String),
    /// Query in progress
    Loading,
    /// Query error
    Error(String),
}

impl QueryResult {
    /// Check if result is loading
    pub fn is_loading(&self) -> bool {
        matches!(self, QueryResult::Loading)
    }

    /// Check if result is an error
    pub fn is_error(&self) -> bool {
        matches!(self, QueryResult::Error(_))
    }

    /// Get error message if result is an error
    pub fn error_message(&self) -> Option<&str> {
        match self {
            QueryResult::Error(msg) => Some(msg),
            _ => None,
        }
    }

    /// Format the result for display
    pub fn format(&self) -> Vec<String> {
        match self {
            QueryResult::Epoch(epoch) => {
                vec![
                    format!("Current Epoch: {}", epoch.epoch),
                    format!("Is Epoch Transition: {}", epoch.is_epoch_transition),
                ]
            }
            QueryResult::Validator(validator) => {
                vec![
                    format!("Auth Address: {}", validator.auth_delegator),
                    format!("Flags: {}", validator.flags),
                    format!(
                        "Execution Stake: {} MON",
                        format_mon(validator.execution_stake)
                    ),
                    format!("Execution Commission: {:.2}%", validator.commission()),
                    format!(
                        "Unclaimed Rewards: {} MON",
                        format_mon(validator.unclaimed_rewards)
                    ),
                    format!(
                        "Consensus Stake: {} MON",
                        format_mon(validator.consensus_stake)
                    ),
                    format!(
                        "Snapshot Stake: {} MON",
                        format_mon(validator.snapshot_stake)
                    ),
                ]
            }
            QueryResult::Delegator(delegator) => {
                // Accumulated rewards per token is a fixed-point number (divide by 1e36)
                let arpt = delegator.accumulated_rewards_per_token as f64 / 1e36;
                vec![
                    format!("Delegated: {} MON", format_mon(delegator.delegated_amount)),
                    format!("Accumulated Rewards per Token: {:.18}", arpt),
                    format!("Rewards: {} MON", format_mon(delegator.rewards)),
                    format!("Delta Stake: {} MON", format_mon(delegator.delta_stake)),
                    format!(
                        "Next Delta Stake: {} MON",
                        format_mon(delegator.next_delta_stake)
                    ),
                    format!("Delta Epoch: {}", delegator.delta_epoch),
                    format!("Next Delta Epoch: {}", delegator.next_delta_epoch),
                ]
            }
            QueryResult::WithdrawalRequest(withdrawal) => {
                vec![
                    format!("Amount: {} MON", format_mon(withdrawal.amount)),
                    format!("Withdrawal Index: {}", withdrawal.withdrawal_index),
                    format!("Activation Epoch: {}", withdrawal.activation_epoch),
                ]
            }
            QueryResult::Delegations(validator_ids) => {
                if validator_ids.is_empty() {
                    vec!["No delegations found.".to_string()]
                } else {
                    let mut lines = vec![format!("Total Delegations: {}", validator_ids.len())];
                    lines.push("Validator IDs:".to_string());
                    for (i, id) in validator_ids.iter().enumerate() {
                        lines.push(format!("  {}. {}", i + 1, id));
                    }
                    lines
                }
            }
            QueryResult::ValidatorSet(valset) => {
                let mut lines = vec![
                    format!("Total Validators: {}", valset.validator_ids.len()),
                    format!("Has More: {}", valset.has_more),
                    format!("Total Count: {}", valset.total_count),
                ];
                if !valset.validator_ids.is_empty() {
                    lines.push("Validator IDs:".to_string());
                    for (i, id) in valset.validator_ids.iter().enumerate() {
                        lines.push(format!("  {}. {}", i + 1, id));
                    }
                }
                lines
            }
            QueryResult::Proposer(validator_id) => {
                vec![format!("Current Proposer: Validator #{}", validator_id)]
            }
            QueryResult::EstimateGas(gas) => {
                vec![
                    format!("Estimated Gas: {}", gas),
                    format!("Gas (hex): 0x{:x}", gas),
                    format!("Recommended (+20% buffer): {}", (*gas as f64 * 1.2) as u64),
                ]
            }
            QueryResult::Transaction(tx_json) => {
                // Parse JSON and format nicely
                // For now, just show the raw JSON
                vec![format!("Transaction: {}", tx_json)]
            }
            QueryResult::Loading => vec!["Loading...".to_string()],
            QueryResult::Error(msg) => vec![format!("Error: {}", msg)],
        }
    }
}

/// Query state for TUI
#[derive(Debug, Clone, Default)]
pub struct QueryState {
    /// Currently selected query type
    pub selected_query: QueryType,
    /// Selected validator set type (for validator-set query)
    pub selected_set_type: ValidatorSetType,
    /// Query result
    pub result: Option<QueryResult>,
    /// Whether query is active (showing results)
    pub is_active: bool,
    /// Current epoch (for withdrawal readiness calculation)
    pub current_epoch: u64,
}

impl QueryState {
    /// Create new query state
    pub fn new() -> Self {
        Self::default()
    }

    /// Set current epoch
    pub fn set_epoch(&mut self, epoch: u64) {
        self.current_epoch = epoch;
    }

    /// Set query result
    pub fn set_result(&mut self, result: QueryResult) {
        self.result = Some(result);
        self.is_active = true;
    }

    /// Clear query result and return to menu
    pub fn clear_result(&mut self) {
        self.result = None;
        self.is_active = false;
    }

    /// Navigate to next query type
    pub fn next_query(&mut self) {
        self.selected_query = self.selected_query.next();
    }

    /// Navigate to previous query type
    pub fn prev_query(&mut self) {
        self.selected_query = self.selected_query.prev();
    }

    /// Navigate to next validator set type
    pub fn next_set_type(&mut self) {
        self.selected_set_type = self.selected_set_type.next();
    }

    /// Check if current query requires parameters
    pub fn requires_params(&self) -> bool {
        self.selected_query.requires_params()
    }

    /// Check if showing results
    pub fn is_showing_result(&self) -> bool {
        self.is_active && self.result.is_some()
    }

    /// Get formatted result lines
    pub fn format_result(&self) -> Vec<String> {
        self.result
            .as_ref()
            .map(|r| r.format())
            .unwrap_or_else(|| vec!["No result".to_string()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_query_type_next() {
        assert_eq!(QueryType::Epoch.next(), QueryType::Validator);
        assert_eq!(QueryType::Transaction.next(), QueryType::Epoch);
    }

    #[test]
    fn test_query_type_prev() {
        assert_eq!(QueryType::Epoch.prev(), QueryType::Transaction);
        assert_eq!(QueryType::Validator.prev(), QueryType::Epoch);
    }

    #[test]
    fn test_query_type_requires_params() {
        assert!(!QueryType::Epoch.requires_params());
        assert!(!QueryType::ValidatorSet.requires_params());
        assert!(!QueryType::Proposer.requires_params());
        assert!(QueryType::Validator.requires_params());
        assert!(QueryType::Delegator.requires_params());
    }

    #[test]
    fn test_validator_set_type_next() {
        assert_eq!(
            ValidatorSetType::Consensus.next(),
            ValidatorSetType::Execution
        );
        assert_eq!(
            ValidatorSetType::Snapshot.next(),
            ValidatorSetType::Consensus
        );
    }

    #[test]
    fn test_validator_set_type_from_str() {
        assert_eq!(
            ValidatorSetType::from_str("consensus"),
            Ok(ValidatorSetType::Consensus)
        );
        assert_eq!(
            ValidatorSetType::from_str("CONSENSUS"),
            Ok(ValidatorSetType::Consensus)
        );
        assert!(ValidatorSetType::from_str("invalid").is_err());
    }

    #[test]
    fn test_query_state_default() {
        let state = QueryState::new();
        assert_eq!(state.selected_query, QueryType::Epoch);
        assert_eq!(state.selected_set_type, ValidatorSetType::Consensus);
        assert!(!state.is_active);
        assert!(state.result.is_none());
    }

    #[test]
    fn test_query_state_navigation() {
        let mut state = QueryState::new();
        state.next_query();
        assert_eq!(state.selected_query, QueryType::Validator);
        state.prev_query();
        assert_eq!(state.selected_query, QueryType::Epoch);
    }

    #[test]
    fn test_query_state_set_result() {
        let mut state = QueryState::new();
        let result = QueryResult::Epoch(EpochInfo {
            epoch: 100,
            is_epoch_transition: false,
        });
        state.set_result(result);
        assert!(state.is_active);
        assert!(state.result.is_some());
    }

    #[test]
    fn test_query_state_clear_result() {
        let mut state = QueryState::new();
        state.set_result(QueryResult::Epoch(EpochInfo {
            epoch: 100,
            is_epoch_transition: false,
        }));
        state.clear_result();
        assert!(!state.is_active);
        assert!(state.result.is_none());
    }

    #[test]
    fn test_query_result_format_epoch() {
        let result = QueryResult::Epoch(EpochInfo {
            epoch: 100,
            is_epoch_transition: true,
        });
        let lines = result.format();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("100"));
        assert!(lines[1].contains("true"));
    }

    #[test]
    fn test_query_result_format_delegations_empty() {
        let result = QueryResult::Delegations(vec![]);
        let lines = result.format();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("No delegations"));
    }

    #[test]
    fn test_query_result_format_delegations() {
        let result = QueryResult::Delegations(vec![1, 2, 3]);
        let lines = result.format();
        assert_eq!(lines.len(), 5);
        assert!(lines[0].contains("3"));
    }
}
