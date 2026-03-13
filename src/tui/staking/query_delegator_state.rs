//! Query Delegator State - State management for Query Delegator dialog
//!
//! This module provides state management for the Query Delegator dialog,
//! which allows users to query their delegations by providing:
//! - Validator ID
//! - Delegator address
//!
//! Uses ratatui-textarea for cross-platform input handling.

use ratatui_textarea::TextArea;

/// Input field indices for Query Delegator dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryDelegatorField {
    /// Validator ID (number)
    ValidatorId = 0,
    /// Delegator address (42 chars, with 0x prefix)
    DelegatorAddress = 1,
}

impl QueryDelegatorField {
    /// Get all fields in order
    pub fn all() -> &'static [QueryDelegatorField] {
        &[
            QueryDelegatorField::ValidatorId,
            QueryDelegatorField::DelegatorAddress,
        ]
    }

    /// Get field label for display
    pub fn label(&self) -> &'static str {
        match self {
            QueryDelegatorField::ValidatorId => "Validator ID",
            QueryDelegatorField::DelegatorAddress => "Delegator Address",
        }
    }

    /// Get field placeholder text
    pub fn placeholder(&self) -> &'static str {
        match self {
            QueryDelegatorField::ValidatorId => "e.g., 224",
            QueryDelegatorField::DelegatorAddress => "0x...",
        }
    }

    /// Get next field
    pub fn next(&self) -> Option<QueryDelegatorField> {
        match self {
            QueryDelegatorField::ValidatorId => Some(QueryDelegatorField::DelegatorAddress),
            QueryDelegatorField::DelegatorAddress => None,
        }
    }

    /// Get previous field
    pub fn prev(&self) -> Option<QueryDelegatorField> {
        match self {
            QueryDelegatorField::ValidatorId => None,
            QueryDelegatorField::DelegatorAddress => Some(QueryDelegatorField::ValidatorId),
        }
    }
}

/// State for the Query Delegator dialog
#[derive(Debug, Clone)]
pub struct QueryDelegatorState {
    /// Whether the dialog is active
    pub is_active: bool,
    /// Currently focused field
    pub focused_field: QueryDelegatorField,
    /// Input fields using TextArea
    pub validator_id: TextArea<'static>,
    pub delegator_address: TextArea<'static>,
    /// Error message for current field or general error
    pub error: Option<String>,
    /// Error field (which field has the error)
    pub error_field: Option<QueryDelegatorField>,
    /// Result of the query (for display)
    pub query_result: Option<String>,
    /// Whether a query is in progress
    pub is_querying: bool,
}

impl Default for QueryDelegatorState {
    fn default() -> Self {
        Self {
            is_active: false,
            focused_field: QueryDelegatorField::ValidatorId,
            validator_id: TextArea::default(),
            delegator_address: TextArea::default(),
            error: None,
            error_field: None,
            query_result: None,
            is_querying: false,
        }
    }
}

impl QueryDelegatorState {
    /// Create new Query Delegator state
    pub fn new() -> Self {
        Self::default()
    }

    /// Open the dialog
    pub fn open(&mut self) {
        self.reset();
        self.is_active = true;
    }

    /// Close the dialog
    pub fn close(&mut self) {
        self.reset();
        self.is_active = false;
    }

    /// Reset all state
    pub fn reset(&mut self) {
        self.focused_field = QueryDelegatorField::ValidatorId;
        self.validator_id = TextArea::default();
        self.delegator_address = TextArea::default();
        self.error = None;
        self.error_field = None;
        self.query_result = None;
        self.is_querying = false;
    }

    /// Check if dialog is active
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Get mutable reference to currently focused textarea
    pub fn current_textarea_mut(&mut self) -> &mut TextArea<'static> {
        match self.focused_field {
            QueryDelegatorField::ValidatorId => &mut self.validator_id,
            QueryDelegatorField::DelegatorAddress => &mut self.delegator_address,
        }
    }

    /// Get reference to currently focused textarea
    pub fn current_textarea(&self) -> &TextArea<'static> {
        match self.focused_field {
            QueryDelegatorField::ValidatorId => &self.validator_id,
            QueryDelegatorField::DelegatorAddress => &self.delegator_address,
        }
    }

    /// Move to next field (Tab)
    pub fn next_field(&mut self) {
        if let Some(next) = self.focused_field.next() {
            self.focused_field = next;
            self.clear_error();
        }
    }

    /// Move to previous field (Shift+Tab)
    pub fn prev_field(&mut self) {
        if let Some(prev) = self.focused_field.prev() {
            self.focused_field = prev;
            self.clear_error();
        }
    }

    /// Set error message
    pub fn set_error(&mut self, error: impl Into<String>, field: Option<QueryDelegatorField>) {
        self.error = Some(error.into());
        self.error_field = field;
    }

    /// Clear error
    pub fn clear_error(&mut self) {
        self.error = None;
        self.error_field = None;
    }

    /// Check if current field has error
    pub fn field_has_error(&self, field: QueryDelegatorField) -> bool {
        self.error_field == Some(field)
    }

    /// Get value for a specific field
    pub fn get_value(&self, field: QueryDelegatorField) -> String {
        let textarea = match field {
            QueryDelegatorField::ValidatorId => &self.validator_id,
            QueryDelegatorField::DelegatorAddress => &self.delegator_address,
        };
        textarea
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("")
            .to_string()
    }

    /// Validate validator ID
    fn validate_validator_id(&self) -> Result<u64, String> {
        let id_str = self
            .validator_id
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("");
        if id_str.is_empty() {
            return Err("Validator ID is required".to_string());
        }
        let id: u64 = id_str
            .parse()
            .map_err(|_| "Invalid validator ID (must be a number)".to_string())?;
        if id == 0 {
            return Err("Validator ID must be greater than 0".to_string());
        }
        Ok(id)
    }

    /// Validate delegator address
    fn validate_delegator_address(&self) -> Result<String, String> {
        let addr_str = self
            .delegator_address
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("");
        if addr_str.is_empty() {
            return Err("Delegator address is required".to_string());
        }
        // Add 0x prefix if not present
        let formatted = if addr_str.starts_with("0x") {
            addr_str.to_string()
        } else {
            format!("0x{}", addr_str)
        };
        // Check length (0x + 40 hex chars)
        if formatted.len() != 42 {
            return Err("Address must be 42 characters (0x + 40 hex)".to_string());
        }
        // Check hex characters
        if !formatted[2..].chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("Address contains invalid hex characters".to_string());
        }
        Ok(formatted)
    }

    /// Validate all fields and return parsed values
    pub fn validate(&self) -> Result<QueryDelegatorParams, String> {
        let validator_id = self.validate_validator_id()?;
        let delegator_address = self.validate_delegator_address()?;

        Ok(QueryDelegatorParams {
            validator_id,
            delegator_address,
        })
    }

    /// Validate current field only
    pub fn validate_current_field(&self) -> Result<(), String> {
        match self.focused_field {
            QueryDelegatorField::ValidatorId => {
                self.validate_validator_id()?;
                Ok(())
            }
            QueryDelegatorField::DelegatorAddress => {
                self.validate_delegator_address()?;
                Ok(())
            }
        }
    }

    /// Set query result
    pub fn set_query_result(&mut self, result: impl Into<String>) {
        self.query_result = Some(result.into());
    }

    /// Set querying state
    pub fn set_querying(&mut self, querying: bool) {
        self.is_querying = querying;
    }
}

/// Validated parameters for Query Delegator operation
#[derive(Debug, Clone, PartialEq)]
pub struct QueryDelegatorParams {
    /// Validator ID
    pub validator_id: u64,
    /// Delegator address (with 0x prefix)
    pub delegator_address: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_delegator_field_order() {
        let fields = QueryDelegatorField::all();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0], QueryDelegatorField::ValidatorId);
        assert_eq!(fields[1], QueryDelegatorField::DelegatorAddress);
    }

    #[test]
    fn test_query_delegator_state_default() {
        let state = QueryDelegatorState::new();
        assert!(!state.is_active());
        assert_eq!(state.focused_field, QueryDelegatorField::ValidatorId);
    }

    #[test]
    fn test_query_delegator_state_open_close() {
        let mut state = QueryDelegatorState::new();
        state.open();
        assert!(state.is_active());

        state.close();
        assert!(!state.is_active());
    }

    #[test]
    fn test_query_delegator_validate_validator_id() {
        let mut state = QueryDelegatorState::new();
        state.open();

        // Empty
        assert!(state.validate_validator_id().is_err());

        // Invalid (non-numeric)
        state.validator_id = TextArea::from(["abc".to_string()]);
        assert!(state.validate_validator_id().is_err());

        // Zero (invalid)
        state.validator_id = TextArea::from(["0".to_string()]);
        assert!(state.validate_validator_id().is_err());

        // Valid
        state.validator_id = TextArea::from(["224".to_string()]);
        assert_eq!(state.validate_validator_id().unwrap(), 224);
    }

    #[test]
    fn test_query_delegator_validate_delegator_address() {
        let mut state = QueryDelegatorState::new();
        state.open();

        // Empty
        assert!(state.validate_delegator_address().is_err());

        // Invalid (no 0x prefix, will be added)
        state.delegator_address = TextArea::from(["123".to_string()]);
        assert!(state.validate_delegator_address().is_err());

        // Valid (without 0x)
        state.delegator_address =
            TextArea::from(["1234567890123456789012345678901234567890".to_string()]);
        let result = state.validate_delegator_address().unwrap();
        assert_eq!(result, "0x1234567890123456789012345678901234567890");

        // Valid (with 0x)
        state.delegator_address =
            TextArea::from(["0x1234567890123456789012345678901234567890".to_string()]);
        let result = state.validate_delegator_address().unwrap();
        assert_eq!(result, "0x1234567890123456789012345678901234567890");
    }

    #[test]
    fn test_query_delegator_params() {
        let params = QueryDelegatorParams {
            validator_id: 224,
            delegator_address: "0x1234567890123456789012345678901234567890".to_string(),
        };

        assert_eq!(params.validator_id, 224);
        assert_eq!(
            params.delegator_address,
            "0x1234567890123456789012345678901234567890"
        );
    }
}
