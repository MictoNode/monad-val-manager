//! Input dialog state management
//!
//! This module provides [`InputDialogState`] which manages the state
//! of the input dialog including input handling and cursor management.
//!
//! Uses tui-textarea for cross-platform input handling.

use super::dialog_type::DialogType;
use super::validation::{validate_address, validate_amount};
use ratatui_textarea::TextArea;

/// State for the input dialog
#[derive(Debug, Clone)]
pub struct InputDialogState {
    /// Type of dialog
    pub dialog_type: DialogType,
    /// Current input value (using TextArea)
    pub input: TextArea<'static>,
    /// Error message (if validation fails)
    pub error: Option<String>,
    /// Is the dialog currently active/visible
    pub is_active: bool,
    /// Optional maximum length for input
    pub max_length: Option<usize>,
    /// Context-specific hint (e.g., available balance)
    pub context_hint: Option<String>,
}

impl Default for InputDialogState {
    fn default() -> Self {
        Self {
            dialog_type: DialogType::Generic,
            input: TextArea::default(),
            error: None,
            is_active: false,
            max_length: None,
            context_hint: None,
        }
    }
}

impl InputDialogState {
    /// Create a new input dialog state
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a dialog state for a specific type
    pub fn for_type(dialog_type: DialogType) -> Self {
        Self {
            dialog_type,
            ..Self::default()
        }
    }

    /// Open the dialog with a specific type
    pub fn open(&mut self, dialog_type: DialogType) {
        self.dialog_type = dialog_type;
        self.input = TextArea::default();
        self.error = None;
        self.is_active = true;
    }

    /// Close the dialog and reset state
    pub fn close(&mut self) {
        self.input = TextArea::default();
        self.error = None;
        self.is_active = false;
        self.context_hint = None;
    }

    /// Set error message
    pub fn set_error(&mut self, error: impl Into<String>) {
        self.error = Some(error.into());
    }

    /// Set context hint (e.g., available balance)
    pub fn set_context_hint(&mut self, hint: impl Into<String>) {
        self.context_hint = Some(hint.into());
    }

    /// Validate input as a number (for amounts)
    pub fn validate_as_amount(&self) -> Result<f64, String> {
        let input = self.input.lines().first().map(|s| s.as_str()).unwrap_or("");
        validate_amount(input)
    }

    /// Validate input as an Ethereum address
    pub fn validate_as_address(&self) -> Result<String, String> {
        let input = self.input.lines().first().map(|s| s.as_str()).unwrap_or("");
        validate_address(input)
    }

    /// Get the current input value
    pub fn get_input(&self) -> String {
        self.input.lines().first().cloned().unwrap_or_default()
    }

    /// Get the current input value as a str (for validation)
    pub fn get_input_str(&self) -> &str {
        self.input.lines().first().map(|s| s.as_str()).unwrap_or("")
    }

    /// Check if dialog is active
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Check if there's an error
    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_dialog_state_default() {
        let state = InputDialogState::new();
        assert!(!state.is_active);
        assert_eq!(state.get_input(), "");
        assert!(state.error.is_none());
    }

    #[test]
    fn test_input_dialog_state_for_type() {
        let state = InputDialogState::for_type(DialogType::Delegate);
        assert_eq!(state.dialog_type, DialogType::Delegate);
        assert!(!state.is_active);
    }

    #[test]
    fn test_input_dialog_open() {
        let mut state = InputDialogState::new();
        state.open(DialogType::Withdraw);

        assert!(state.is_active);
        assert_eq!(state.dialog_type, DialogType::Withdraw);
        assert_eq!(state.get_input(), "");
        assert!(state.error.is_none());
    }

    #[test]
    fn test_input_dialog_close() {
        let mut state = InputDialogState::new();
        state.open(DialogType::Delegate);
        state.input = TextArea::from(["100".to_string()]);
        state.error = Some("test error".to_string());

        state.close();

        assert!(!state.is_active);
        assert_eq!(state.get_input(), "");
        assert!(state.error.is_none());
    }

    #[test]
    fn test_input_dialog_validate_as_amount_valid() {
        let mut state = InputDialogState::new();
        state.open(DialogType::Delegate);
        state.input = TextArea::from(["100.5".to_string()]);

        let result = state.validate_as_amount();
        assert!(result.is_ok());
        assert!((result.unwrap() - 100.5).abs() < 0.001);
    }

    #[test]
    fn test_input_dialog_validate_as_amount_empty() {
        let mut state = InputDialogState::new();
        state.open(DialogType::Delegate);
        // Empty textarea

        let result = state.validate_as_amount();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Amount is required");
    }

    #[test]
    fn test_input_dialog_validate_as_amount_invalid() {
        let mut state = InputDialogState::new();
        state.open(DialogType::Delegate);
        state.input = TextArea::from(["abc".to_string()]);

        let result = state.validate_as_amount();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid number format");
    }

    #[test]
    fn test_input_dialog_validate_as_amount_negative() {
        let mut state = InputDialogState::new();
        state.open(DialogType::Delegate);
        state.input = TextArea::from(["-10".to_string()]);

        let result = state.validate_as_amount();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Amount must be positive");
    }

    #[test]
    fn test_input_dialog_validate_as_address_valid() {
        let mut state = InputDialogState::new();
        state.open(DialogType::Generic);
        state.input = TextArea::from(["0x1234567890123456789012345678901234567890".to_string()]);

        let result = state.validate_as_address();
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "0x1234567890123456789012345678901234567890"
        );
    }

    #[test]
    fn test_input_dialog_validate_as_address_no_prefix() {
        let mut state = InputDialogState::new();
        state.open(DialogType::Generic);
        state.input = TextArea::from(["1234567890123456789012345678901234567890".to_string()]);

        let result = state.validate_as_address();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Address must start with 0x");
    }

    #[test]
    fn test_input_dialog_validate_as_address_wrong_length() {
        let mut state = InputDialogState::new();
        state.open(DialogType::Generic);
        state.input = TextArea::from(["0x12345".to_string()]);

        let result = state.validate_as_address();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Address must be 42 characters");
    }

    #[test]
    fn test_input_dialog_validate_as_address_invalid_chars() {
        let mut state = InputDialogState::new();
        state.open(DialogType::Generic);
        state.input = TextArea::from(["0xZZZZ567890123456789012345678901234567890".to_string()]);

        let result = state.validate_as_address();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid hex characters in address");
    }

    #[test]
    fn test_input_dialog_set_error() {
        let mut state = InputDialogState::new();
        state.set_error("Test error");

        assert_eq!(state.error, Some("Test error".to_string()));
        assert!(state.has_error());
    }

    #[test]
    fn test_input_dialog_set_context_hint() {
        let mut state = InputDialogState::new();
        state.set_context_hint("Available: 100 MON");

        assert_eq!(state.context_hint, Some("Available: 100 MON".to_string()));
    }

    #[test]
    fn test_input_dialog_is_active() {
        let mut state = InputDialogState::new();
        assert!(!state.is_active());

        state.open(DialogType::Delegate);
        assert!(state.is_active());

        state.close();
        assert!(!state.is_active());
    }

    #[test]
    fn test_input_dialog_get_input() {
        let mut state = InputDialogState::new();
        state.open(DialogType::Delegate);
        state.input = TextArea::from(["100.5".to_string()]);

        assert_eq!(state.get_input(), "100.5");
    }
}
