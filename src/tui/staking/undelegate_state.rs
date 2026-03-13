//! Undelegate Dialog State - State management for Undelegate dialog
//!
//! This module provides state management for the Undelegate dialog,
//! which allows users to undelegate MON from a validator by providing:
//! - Validator ID (numeric)
//! - Amount (decimal, in MON)
//!
//! Uses tui-textarea for cross-platform input handling.

use ratatui_textarea::TextArea;

/// Input field indices for Undelegate dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UndelegateField {
    /// Validator ID (number)
    ValidatorId = 0,
    /// Amount (decimal, in MON)
    Amount = 1,
}

impl UndelegateField {
    /// Get all fields in order
    pub fn all() -> &'static [UndelegateField] {
        &[UndelegateField::ValidatorId, UndelegateField::Amount]
    }

    /// Get field label for display
    pub fn label(&self) -> &'static str {
        match self {
            UndelegateField::ValidatorId => "Validator ID",
            UndelegateField::Amount => "Amount (MON)",
        }
    }

    /// Get field placeholder text
    pub fn placeholder(&self) -> &'static str {
        match self {
            UndelegateField::ValidatorId => "e.g., 224",
            UndelegateField::Amount => "e.g., 50.0",
        }
    }

    /// Get next field
    pub fn next(&self) -> Option<UndelegateField> {
        match self {
            UndelegateField::ValidatorId => Some(UndelegateField::Amount),
            UndelegateField::Amount => None,
        }
    }

    /// Get previous field
    pub fn prev(&self) -> Option<UndelegateField> {
        match self {
            UndelegateField::ValidatorId => None,
            UndelegateField::Amount => Some(UndelegateField::ValidatorId),
        }
    }
}

/// State for the Undelegate dialog
#[derive(Debug, Clone)]
pub struct UndelegateState {
    /// Whether the dialog is active
    pub is_active: bool,
    /// Currently focused field
    pub focused_field: UndelegateField,
    /// Input fields using TextArea
    pub validator_id: TextArea<'static>,
    pub amount: TextArea<'static>,
    /// Error message for current field or general error
    pub error: Option<String>,
    /// Error field (which field has the error)
    pub error_field: Option<UndelegateField>,
    /// Available delegated amount hint (optional)
    pub delegated_amount: Option<f64>,
}

impl Default for UndelegateState {
    fn default() -> Self {
        Self {
            is_active: false,
            focused_field: UndelegateField::ValidatorId,
            validator_id: TextArea::default(),
            amount: TextArea::default(),
            error: None,
            error_field: None,
            delegated_amount: None,
        }
    }
}

impl UndelegateState {
    /// Create new Undelegate state
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
        self.focused_field = UndelegateField::ValidatorId;
        self.validator_id = TextArea::default();
        self.amount = TextArea::default();
        self.error = None;
        self.error_field = None;
    }

    /// Check if dialog is active
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Get mutable reference to currently focused textarea
    pub fn current_textarea_mut(&mut self) -> &mut TextArea<'static> {
        match self.focused_field {
            UndelegateField::ValidatorId => &mut self.validator_id,
            UndelegateField::Amount => &mut self.amount,
        }
    }

    /// Get reference to currently focused textarea
    pub fn current_textarea(&self) -> &TextArea<'static> {
        match self.focused_field {
            UndelegateField::ValidatorId => &self.validator_id,
            UndelegateField::Amount => &self.amount,
        }
    }

    /// Set delegated amount hint
    pub fn set_delegated_amount(&mut self, amount: f64) {
        self.delegated_amount = Some(amount);
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
    pub fn set_error(&mut self, error: impl Into<String>, field: Option<UndelegateField>) {
        self.error = Some(error.into());
        self.error_field = field;
    }

    /// Clear error
    pub fn clear_error(&mut self) {
        self.error = None;
        self.error_field = None;
    }

    /// Check if current field has error
    pub fn field_has_error(&self, field: UndelegateField) -> bool {
        self.error_field == Some(field)
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

    /// Validate amount
    fn validate_amount(&self) -> Result<f64, String> {
        let amount_str = self
            .amount
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("");
        if amount_str.is_empty() {
            return Err("Amount is required".to_string());
        }
        let amount: f64 = amount_str
            .parse()
            .map_err(|_| "Invalid amount (must be a number)".to_string())?;
        if amount <= 0.0 {
            return Err("Amount must be greater than 0".to_string());
        }
        // Check against delegated amount if set
        if let Some(delegated) = self.delegated_amount {
            if amount > delegated {
                return Err(format!(
                    "Amount exceeds delegated amount ({:.2} MON)",
                    delegated
                ));
            }
        }
        Ok(amount)
    }

    /// Validate all fields and return parsed values
    pub fn validate(&self) -> Result<UndelegateParams, String> {
        let validator_id = self.validate_validator_id()?;
        let amount = self.validate_amount()?;

        Ok(UndelegateParams {
            validator_id,
            amount,
        })
    }

    /// Validate current field only
    pub fn validate_current_field(&self) -> Result<(), String> {
        match self.focused_field {
            UndelegateField::ValidatorId => {
                self.validate_validator_id()?;
                Ok(())
            }
            UndelegateField::Amount => {
                self.validate_amount()?;
                Ok(())
            }
        }
    }

    /// Get value for a specific field
    pub fn get_value(&self, field: UndelegateField) -> String {
        let textarea = match field {
            UndelegateField::ValidatorId => &self.validator_id,
            UndelegateField::Amount => &self.amount,
        };
        textarea
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("")
            .to_string()
    }
}

/// Validated parameters for Undelegate operation
#[derive(Debug, Clone, PartialEq)]
pub struct UndelegateParams {
    /// Validator ID
    pub validator_id: u64,
    /// Amount in MON
    pub amount: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undelegate_field_order() {
        let fields = UndelegateField::all();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0], UndelegateField::ValidatorId);
        assert_eq!(fields[1], UndelegateField::Amount);
    }

    #[test]
    fn test_undelegate_state_default() {
        let state = UndelegateState::new();
        assert!(!state.is_active());
        assert_eq!(state.focused_field, UndelegateField::ValidatorId);
    }

    #[test]
    fn test_undelegate_state_open_close() {
        let mut state = UndelegateState::new();
        state.open();
        assert!(state.is_active());

        state.close();
        assert!(!state.is_active());
    }

    #[test]
    fn test_undelegate_validate_validator_id() {
        let mut state = UndelegateState::new();
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
    fn test_undelegate_validate_amount() {
        let mut state = UndelegateState::new();
        state.open();

        // Empty
        assert!(state.validate_amount().is_err());

        // Invalid (non-numeric)
        state.amount = TextArea::from(["abc".to_string()]);
        assert!(state.validate_amount().is_err());

        // Zero (invalid)
        state.amount = TextArea::from(["0".to_string()]);
        assert!(state.validate_amount().is_err());

        // Negative (invalid)
        state.amount = TextArea::from(["-10".to_string()]);
        assert!(state.validate_amount().is_err());

        // Valid integer
        state.amount = TextArea::from(["50".to_string()]);
        assert_eq!(state.validate_amount().unwrap(), 50.0);

        // Valid decimal
        state.amount = TextArea::from(["50.5".to_string()]);
        assert_eq!(state.validate_amount().unwrap(), 50.5);
    }

    #[test]
    fn test_undelegate_validate_with_delegated() {
        let mut state = UndelegateState::new();
        state.open();
        state.set_delegated_amount(100.0);

        state.amount = TextArea::from(["150".to_string()]);
        assert!(state.validate_amount().is_err());

        state.amount = TextArea::from(["100".to_string()]);
        assert_eq!(state.validate_amount().unwrap(), 100.0);

        state.amount = TextArea::from(["50.5".to_string()]);
        assert_eq!(state.validate_amount().unwrap(), 50.5);
    }

    #[test]
    fn test_undelegate_params() {
        let params = UndelegateParams {
            validator_id: 224,
            amount: 50.5,
        };

        assert_eq!(params.validator_id, 224);
        assert_eq!(params.amount, 50.5);
    }
}
