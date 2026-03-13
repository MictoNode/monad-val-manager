//! Delegate Dialog State - State management for Delegate dialog
//!
//! This module provides state management for the Delegate dialog,
//! which allows users to delegate MON to a validator by providing:
//! - Validator ID (numeric)
//! - Amount (decimal, in MON)
//!
//! Uses tui-textarea for cross-platform input handling.

use ratatui_textarea::TextArea;

/// Input field indices for Delegate dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelegateField {
    /// Validator ID (number)
    ValidatorId = 0,
    /// Amount (decimal, in MON)
    Amount = 1,
}

impl DelegateField {
    /// Get all fields in order
    pub fn all() -> &'static [DelegateField] {
        &[DelegateField::ValidatorId, DelegateField::Amount]
    }

    /// Get field label for display
    pub fn label(&self) -> &'static str {
        match self {
            DelegateField::ValidatorId => "Validator ID",
            DelegateField::Amount => "Amount (MON)",
        }
    }

    /// Get field placeholder text
    pub fn placeholder(&self) -> &'static str {
        match self {
            DelegateField::ValidatorId => "e.g., 224",
            DelegateField::Amount => "e.g., 100.5",
        }
    }

    /// Get next field
    pub fn next(&self) -> Option<DelegateField> {
        match self {
            DelegateField::ValidatorId => Some(DelegateField::Amount),
            DelegateField::Amount => None,
        }
    }

    /// Get previous field
    pub fn prev(&self) -> Option<DelegateField> {
        match self {
            DelegateField::ValidatorId => None,
            DelegateField::Amount => Some(DelegateField::ValidatorId),
        }
    }
}

/// State for the Delegate dialog
#[derive(Debug, Clone)]
pub struct DelegateState {
    /// Whether the dialog is active
    pub is_active: bool,
    /// Currently focused field
    pub focused_field: DelegateField,
    /// Input fields using TextArea
    pub validator_id: TextArea<'static>,
    pub amount: TextArea<'static>,
    /// Error message for current field or general error
    pub error: Option<String>,
    /// Error field (which field has the error)
    pub error_field: Option<DelegateField>,
    /// Available balance hint (optional)
    pub available_balance: Option<f64>,
}

impl Default for DelegateState {
    fn default() -> Self {
        Self {
            is_active: false,
            focused_field: DelegateField::ValidatorId,
            validator_id: TextArea::default(),
            amount: TextArea::default(),
            error: None,
            error_field: None,
            available_balance: None,
        }
    }
}

impl DelegateState {
    /// Create new Delegate state
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
        self.focused_field = DelegateField::ValidatorId;
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
            DelegateField::ValidatorId => &mut self.validator_id,
            DelegateField::Amount => &mut self.amount,
        }
    }

    /// Get reference to currently focused textarea
    pub fn current_textarea(&self) -> &TextArea<'static> {
        match self.focused_field {
            DelegateField::ValidatorId => &self.validator_id,
            DelegateField::Amount => &self.amount,
        }
    }

    /// Set available balance hint
    pub fn set_available_balance(&mut self, balance: f64) {
        self.available_balance = Some(balance);
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
    pub fn set_error(&mut self, error: impl Into<String>, field: Option<DelegateField>) {
        self.error = Some(error.into());
        self.error_field = field;
    }

    /// Clear error
    pub fn clear_error(&mut self) {
        self.error = None;
        self.error_field = None;
    }

    /// Check if current field has error
    pub fn field_has_error(&self, field: DelegateField) -> bool {
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
        // Check against available balance if set
        if let Some(balance) = self.available_balance {
            if amount > balance {
                return Err(format!(
                    "Amount exceeds available balance ({:.2} MON)",
                    balance
                ));
            }
        }
        Ok(amount)
    }

    /// Validate all fields and return parsed values
    pub fn validate(&self) -> Result<DelegateParams, String> {
        let validator_id = self.validate_validator_id()?;
        let amount = self.validate_amount()?;

        Ok(DelegateParams {
            validator_id,
            amount,
        })
    }

    /// Validate current field only
    pub fn validate_current_field(&self) -> Result<(), String> {
        match self.focused_field {
            DelegateField::ValidatorId => {
                self.validate_validator_id()?;
                Ok(())
            }
            DelegateField::Amount => {
                self.validate_amount()?;
                Ok(())
            }
        }
    }

    /// Get value for a specific field
    pub fn get_value(&self, field: DelegateField) -> String {
        let textarea = match field {
            DelegateField::ValidatorId => &self.validator_id,
            DelegateField::Amount => &self.amount,
        };
        textarea
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("")
            .to_string()
    }
}

/// Validated parameters for Delegate operation
#[derive(Debug, Clone, PartialEq)]
pub struct DelegateParams {
    /// Validator ID
    pub validator_id: u64,
    /// Amount in MON
    pub amount: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delegate_field_order() {
        let fields = DelegateField::all();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0], DelegateField::ValidatorId);
        assert_eq!(fields[1], DelegateField::Amount);
    }

    #[test]
    fn test_delegate_state_default() {
        let state = DelegateState::new();
        assert!(!state.is_active());
        assert_eq!(state.focused_field, DelegateField::ValidatorId);
    }

    #[test]
    fn test_delegate_state_open_close() {
        let mut state = DelegateState::new();
        state.open();
        assert!(state.is_active());

        state.close();
        assert!(!state.is_active());
    }

    #[test]
    fn test_delegate_validate_validator_id() {
        let mut state = DelegateState::new();
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
    fn test_delegate_validate_amount() {
        let mut state = DelegateState::new();
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
        state.amount = TextArea::from(["100".to_string()]);
        assert_eq!(state.validate_amount().unwrap(), 100.0);

        // Valid decimal
        state.amount = TextArea::from(["100.5".to_string()]);
        assert_eq!(state.validate_amount().unwrap(), 100.5);
    }

    #[test]
    fn test_delegate_validate_with_balance() {
        let mut state = DelegateState::new();
        state.open();
        state.set_available_balance(50.0);

        state.amount = TextArea::from(["100".to_string()]);
        assert!(state.validate_amount().is_err());

        state.amount = TextArea::from(["50".to_string()]);
        assert_eq!(state.validate_amount().unwrap(), 50.0);

        state.amount = TextArea::from(["25.5".to_string()]);
        assert_eq!(state.validate_amount().unwrap(), 25.5);
    }

    #[test]
    fn test_delegate_field_navigation() {
        let mut state = DelegateState::new();
        state.open();

        assert_eq!(state.focused_field, DelegateField::ValidatorId);

        state.next_field();
        assert_eq!(state.focused_field, DelegateField::Amount);

        state.next_field(); // No next field
        assert_eq!(state.focused_field, DelegateField::Amount);

        state.prev_field();
        assert_eq!(state.focused_field, DelegateField::ValidatorId);

        state.prev_field(); // No prev field
        assert_eq!(state.focused_field, DelegateField::ValidatorId);
    }

    #[test]
    fn test_delegate_params() {
        let params = DelegateParams {
            validator_id: 224,
            amount: 100.5,
        };

        assert_eq!(params.validator_id, 224);
        assert_eq!(params.amount, 100.5);
    }
}
