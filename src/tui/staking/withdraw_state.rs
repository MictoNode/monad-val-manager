//! Withdraw Dialog State - State management for Withdraw dialog
//!
//! This module provides state management for the Withdraw dialog,
//! which allows users to withdraw pending MON by providing:
//! - Validator ID (numeric)
//! - Withdrawal ID (numeric)
//!
//! Uses tui-textarea for cross-platform input handling.

use ratatui_textarea::TextArea;

/// Input field indices for Withdraw dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WithdrawField {
    /// Validator ID (number)
    ValidatorId = 0,
    /// Withdrawal ID (number)
    WithdrawalId = 1,
}

impl WithdrawField {
    /// Get all fields in order
    pub fn all() -> &'static [WithdrawField] {
        &[WithdrawField::ValidatorId, WithdrawField::WithdrawalId]
    }

    /// Get field label for display
    pub fn label(&self) -> &'static str {
        match self {
            WithdrawField::ValidatorId => "Validator ID",
            WithdrawField::WithdrawalId => "Withdrawal ID",
        }
    }

    /// Get field placeholder text
    pub fn placeholder(&self) -> &'static str {
        match self {
            WithdrawField::ValidatorId => "e.g., 224",
            WithdrawField::WithdrawalId => "e.g., 0",
        }
    }

    /// Get next field
    pub fn next(&self) -> Option<WithdrawField> {
        match self {
            WithdrawField::ValidatorId => Some(WithdrawField::WithdrawalId),
            WithdrawField::WithdrawalId => None,
        }
    }

    /// Get previous field
    pub fn prev(&self) -> Option<WithdrawField> {
        match self {
            WithdrawField::ValidatorId => None,
            WithdrawField::WithdrawalId => Some(WithdrawField::ValidatorId),
        }
    }
}

/// State for the Withdraw dialog
#[derive(Debug, Clone)]
pub struct WithdrawState {
    /// Whether the dialog is active
    pub is_active: bool,
    /// Currently focused field
    pub focused_field: WithdrawField,
    /// Input fields using TextArea
    pub validator_id: TextArea<'static>,
    pub withdrawal_id: TextArea<'static>,
    /// Error message for current field or general error
    pub error: Option<String>,
    /// Error field (which field has the error)
    pub error_field: Option<WithdrawField>,
    /// Context hint (e.g., "Ready IDs: 0, 2" or "No withdrawals ready")
    pub context_hint: Option<String>,
}

impl Default for WithdrawState {
    fn default() -> Self {
        Self {
            is_active: false,
            focused_field: WithdrawField::ValidatorId,
            validator_id: TextArea::default(),
            withdrawal_id: TextArea::default(),
            error: None,
            error_field: None,
            context_hint: None,
        }
    }
}

impl WithdrawState {
    /// Create new Withdraw state
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
        self.focused_field = WithdrawField::ValidatorId;
        self.validator_id = TextArea::default();
        self.withdrawal_id = TextArea::default();
        self.error = None;
        self.error_field = None;
        self.context_hint = None;
    }

    /// Check if dialog is active
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Get mutable reference to currently focused textarea
    pub fn current_textarea_mut(&mut self) -> &mut TextArea<'static> {
        match self.focused_field {
            WithdrawField::ValidatorId => &mut self.validator_id,
            WithdrawField::WithdrawalId => &mut self.withdrawal_id,
        }
    }

    /// Get reference to currently focused textarea
    pub fn current_textarea(&self) -> &TextArea<'static> {
        match self.focused_field {
            WithdrawField::ValidatorId => &self.validator_id,
            WithdrawField::WithdrawalId => &self.withdrawal_id,
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
    pub fn set_error(&mut self, error: impl Into<String>, field: Option<WithdrawField>) {
        self.error = Some(error.into());
        self.error_field = field;
    }

    /// Clear error
    pub fn clear_error(&mut self) {
        self.error = None;
        self.error_field = None;
    }

    /// Check if current field has error
    pub fn field_has_error(&self, field: WithdrawField) -> bool {
        self.error_field == Some(field)
    }

    /// Set context hint
    pub fn set_context_hint(&mut self, hint: impl Into<String>) {
        self.context_hint = Some(hint.into());
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

    /// Validate withdrawal ID
    fn validate_withdrawal_id(&self) -> Result<u8, String> {
        let id_str = self
            .withdrawal_id
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("");
        if id_str.is_empty() {
            return Err("Withdrawal ID is required".to_string());
        }
        let id: u64 = id_str
            .parse()
            .map_err(|_| "Invalid withdrawal ID (must be a number)".to_string())?;
        if id > u8::MAX as u64 {
            return Err(format!("Withdrawal ID must be 0-{}", u8::MAX));
        }
        Ok(id as u8)
    }

    /// Validate all fields and return parsed values
    pub fn validate(&self) -> Result<WithdrawParams, String> {
        let validator_id = self.validate_validator_id()?;
        let withdrawal_id = self.validate_withdrawal_id()?;

        Ok(WithdrawParams {
            validator_id,
            withdrawal_id,
        })
    }

    /// Validate current field only
    pub fn validate_current_field(&self) -> Result<(), String> {
        match self.focused_field {
            WithdrawField::ValidatorId => {
                self.validate_validator_id()?;
                Ok(())
            }
            WithdrawField::WithdrawalId => {
                self.validate_withdrawal_id()?;
                Ok(())
            }
        }
    }

    /// Get value for a specific field
    pub fn get_value(&self, field: WithdrawField) -> String {
        let textarea = match field {
            WithdrawField::ValidatorId => &self.validator_id,
            WithdrawField::WithdrawalId => &self.withdrawal_id,
        };
        textarea
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("")
            .to_string()
    }
}

/// Validated parameters for Withdraw operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithdrawParams {
    /// Validator ID
    pub validator_id: u64,
    /// Withdrawal ID (0-255)
    pub withdrawal_id: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_withdraw_field_order() {
        let fields = WithdrawField::all();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0], WithdrawField::ValidatorId);
        assert_eq!(fields[1], WithdrawField::WithdrawalId);
    }

    #[test]
    fn test_withdraw_state_default() {
        let state = WithdrawState::new();
        assert!(!state.is_active());
        assert_eq!(state.focused_field, WithdrawField::ValidatorId);
    }

    #[test]
    fn test_withdraw_state_open_close() {
        let mut state = WithdrawState::new();
        state.open();
        assert!(state.is_active());

        state.close();
        assert!(!state.is_active());
    }

    #[test]
    fn test_withdraw_validate_validator_id() {
        let mut state = WithdrawState::new();
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
    fn test_withdraw_validate_withdrawal_id() {
        let mut state = WithdrawState::new();
        state.open();

        // Empty
        assert!(state.validate_withdrawal_id().is_err());

        // Invalid (non-numeric)
        state.withdrawal_id = TextArea::from(["abc".to_string()]);
        assert!(state.validate_withdrawal_id().is_err());

        // Valid (0)
        state.withdrawal_id = TextArea::from(["0".to_string()]);
        assert_eq!(state.validate_withdrawal_id().unwrap(), 0);

        // Valid (255 - max u8)
        state.withdrawal_id = TextArea::from(["255".to_string()]);
        assert_eq!(state.validate_withdrawal_id().unwrap(), 255);

        // Invalid (256 - exceeds u8)
        state.withdrawal_id = TextArea::from(["256".to_string()]);
        assert!(state.validate_withdrawal_id().is_err());
    }

    #[test]
    fn test_withdraw_params() {
        let params = WithdrawParams {
            validator_id: 224,
            withdrawal_id: 5,
        };

        assert_eq!(params.validator_id, 224);
        assert_eq!(params.withdrawal_id, 5);
    }
}
