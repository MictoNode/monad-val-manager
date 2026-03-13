//! Query Validator Dialog State
//!
//! Manages the state for the Query Validator dialog.

use crate::staking::types::Validator;
use ratatui_textarea::TextArea;

/// Query Validator dialog state
#[derive(Debug, Clone, Default)]
pub struct QueryValidatorState {
    /// Validator ID input
    pub validator_id: TextArea<'static>,
    /// Query result
    pub result: Option<QueryValidatorResult>,
    /// Whether currently querying
    pub is_querying: bool,
    /// Error message if validation/query fails
    pub error: Option<String>,
    /// Is dialog active
    pub is_active: bool,
    /// Last successfully queried validator ID (for sharing with Change Commission)
    pub last_validator_id: Option<u64>,
    /// Last successfully queried validator commission (for sharing with Change Commission)
    pub last_commission: Option<f64>,
}

/// Query result for validator
#[derive(Debug, Clone)]
pub enum QueryValidatorResult {
    /// Successful query with validator data
    Success(Validator),
    /// Query failed
    Error(String),
}

impl Default for QueryValidatorResult {
    fn default() -> Self {
        Self::Error("No result".to_string())
    }
}

impl QueryValidatorState {
    /// Create new query validator state
    pub fn new() -> Self {
        Self::default()
    }

    /// Open the dialog
    pub fn open(&mut self) {
        self.validator_id = TextArea::default();
        self.result = None;
        self.error = None;
        self.is_active = true;
        self.is_querying = false;
    }

    /// Close the dialog
    pub fn close(&mut self) {
        self.validator_id = TextArea::default();
        self.result = None;
        self.error = None;
        self.is_active = false;
        self.is_querying = false;
        // Keep last_validator_id and last_commission for sharing with Change Commission
    }

    /// Check if dialog is active
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Get the validator ID input value
    pub fn get_validator_id(&self) -> String {
        self.validator_id
            .lines()
            .first()
            .cloned()
            .unwrap_or_default()
    }

    /// Validate inputs and return validator ID if valid
    pub fn validate(&self) -> Result<u64, String> {
        let input = self.get_validator_id();
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Err("Validator ID is required".to_string());
        }

        let validator_id = trimmed
            .parse::<u64>()
            .map_err(|_| "Invalid validator ID format".to_string())?;

        Ok(validator_id)
    }

    /// Set query result
    pub fn set_result(&mut self, result: QueryValidatorResult) {
        match &result {
            QueryValidatorResult::Success(validator) => {
                // Store validator ID and commission for Change Commission dialog
                let validator_id_str = self.get_validator_id();
                if let Ok(vid) = validator_id_str.parse::<u64>() {
                    self.last_validator_id = Some(vid);
                    self.last_commission = Some(validator.commission());
                }
            }
            QueryValidatorResult::Error(_) => {}
        }
        self.result = Some(result);
        self.is_querying = false;
    }

    /// Set error message
    pub fn set_error(&mut self, error: impl Into<String>) {
        self.error = Some(error.into());
    }

    /// Check if currently querying
    pub fn is_querying(&self) -> bool {
        self.is_querying
    }

    /// Start querying state
    pub fn start_querying(&mut self) {
        self.is_querying = true;
        self.error = None;
        self.result = None;
    }

    /// Format validator result for display
    pub fn format_result(&self) -> Option<Vec<String>> {
        self.result.as_ref().map(|r| match r {
            QueryValidatorResult::Success(validator) => {
                vec![
                    format!("Auth Address: {}", validator.auth_delegator),
                    format!("Flags: {}", validator.flags),
                    format!(
                        "Execution Stake: {} MON",
                        validator.execution_stake as f64 / 1e18
                    ),
                    format!("Execution Commission: {:.2}%", validator.commission()),
                    format!(
                        "Unclaimed Rewards: {} MON",
                        validator.unclaimed_rewards as f64 / 1e18
                    ),
                    format!(
                        "Consensus Stake: {} MON",
                        validator.consensus_stake as f64 / 1e18
                    ),
                    format!(
                        "Snapshot Stake: {} MON",
                        validator.snapshot_stake as f64 / 1e18
                    ),
                ]
            }
            QueryValidatorResult::Error(msg) => vec![format!("Error: {}", msg)],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_validator_state_default() {
        let state = QueryValidatorState::new();
        assert!(!state.is_active());
        assert!(!state.is_querying());
        assert!(state.error.is_none());
        assert!(state.result.is_none());
        assert_eq!(state.get_validator_id(), "");
    }

    #[test]
    fn test_query_validator_state_open() {
        let mut state = QueryValidatorState::new();
        state.open();
        assert!(state.is_active());
        assert!(!state.is_querying());
        assert_eq!(state.get_validator_id(), "");
    }

    #[test]
    fn test_query_validator_state_close() {
        let mut state = QueryValidatorState::new();
        state.open();
        state.validator_id = TextArea::from(["42".to_string()]);
        state.close();
        assert!(!state.is_active());
        assert_eq!(state.get_validator_id(), "");
    }

    #[test]
    fn test_query_validator_validate_valid() {
        let mut state = QueryValidatorState::new();
        state.validator_id = TextArea::from(["123".to_string()]);
        let result = state.validate();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 123);
    }

    #[test]
    fn test_query_validator_validate_empty() {
        let state = QueryValidatorState::new();
        let result = state.validate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Validator ID is required");
    }

    #[test]
    fn test_query_validator_validate_invalid() {
        let mut state = QueryValidatorState::new();
        state.validator_id = TextArea::from(["abc".to_string()]);
        let result = state.validate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid validator ID format");
    }

    #[test]
    fn test_query_validator_set_error() {
        let mut state = QueryValidatorState::new();
        state.set_error("Test error");
        assert_eq!(state.error, Some("Test error".to_string()));
    }

    #[test]
    fn test_query_validator_start_querying() {
        let mut state = QueryValidatorState::new();
        state.start_querying();
        assert!(state.is_querying());
        assert!(state.error.is_none());
        assert!(state.result.is_none());
    }

    #[test]
    fn test_query_validator_set_result() {
        let mut state = QueryValidatorState::new();
        state.start_querying();
        let validator = Validator {
            auth_delegator: "0x1234".to_string(),
            flags: 0,
            execution_stake: 1000,
            consensus_stake: 2000,
            snapshot_stake: 3000,
            ..Default::default()
        };
        state.set_result(QueryValidatorResult::Success(validator));
        assert!(!state.is_querying());
        assert!(state.result.is_some());
    }
}
