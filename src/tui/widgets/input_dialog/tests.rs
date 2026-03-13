//! Tests for input_dialog module
//!
//! Tests for DialogType, InputDialogState, InputDialogWidget,
//! and validation functions.

use ratatui_textarea::TextArea;

use super::*;

// ============================================================================
// DialogType Tests
// ============================================================================

#[test]
fn test_dialog_type_titles() {
    assert_eq!(DialogType::Delegate.title(), " Delegate ");
    assert_eq!(DialogType::Undelegate.title(), " Undelegate ");
    assert_eq!(DialogType::Withdraw.title(), " Withdraw ");
    assert_eq!(DialogType::Claim.title(), " Claim Rewards ");
    assert_eq!(DialogType::Compound.title(), " Compound ");
    assert_eq!(DialogType::Generic.title(), " Input ");
}

#[test]
fn test_dialog_type_placeholders() {
    assert_eq!(
        DialogType::Delegate.placeholder(),
        "Enter: VALIDATOR_ID AMOUNT (e.g., \"1 100.5\")"
    );
    assert_eq!(
        DialogType::Undelegate.placeholder(),
        "Enter: VALIDATOR_ID AMOUNT WITHDRAWAL_ID (e.g., \"1 50.0 0\")"
    );
}

// ============================================================================
// InputDialogState Tests
// ============================================================================

#[test]
fn test_input_dialog_state_default() {
    let state = InputDialogState::new();
    assert!(!state.is_active);
    assert!(state.get_input().is_empty());
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
    assert!(state.get_input().is_empty());
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
    assert!(state.get_input().is_empty());
    assert!(state.error.is_none());
}

// ============================================================================
// Validation Tests - Amount
// ============================================================================

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

// ============================================================================
// Validation Tests - Address
// ============================================================================

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

// ============================================================================
// Error Handling Tests
// ============================================================================

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

// ============================================================================
// State Query Tests
// ============================================================================

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

// ============================================================================
// Widget Tests
// ============================================================================

#[test]
fn test_input_dialog_widget_creation() {
    let state = InputDialogState::new();
    let widget = InputDialogWidget::new(&state);
    assert_eq!(widget.width_percent, 60);
    assert_eq!(widget.height, 11);
}

#[test]
fn test_input_dialog_widget_with_width() {
    let state = InputDialogState::new();
    let widget = InputDialogWidget::new(&state).with_width(60);
    assert_eq!(widget.width_percent, 60);
}

#[test]
fn test_input_dialog_widget_with_height() {
    let state = InputDialogState::new();
    let widget = InputDialogWidget::new(&state).with_height(10);
    assert_eq!(widget.height, 10);
}

#[test]
fn test_input_dialog_widget_width_capped_at_100() {
    let state = InputDialogState::new();
    let widget = InputDialogWidget::new(&state).with_width(150);
    assert_eq!(widget.width_percent, 100);
}

#[test]
fn test_input_dialog_widget_height_minimum() {
    let state = InputDialogState::new();
    let widget = InputDialogWidget::new(&state).with_height(2);
    assert_eq!(widget.height, 5); // Minimum is 5
}

// ============================================================================
// Validation Function Tests (standalone functions)
// ============================================================================

#[test]
fn test_validate_amount_standalone() {
    assert!(validate_amount("100.5").is_ok());
    assert!(validate_amount("").is_err());
    assert!(validate_amount("abc").is_err());
    assert!(validate_amount("-10").is_err());
}

#[test]
fn test_validate_address_standalone() {
    assert!(validate_address("0x1234567890123456789012345678901234567890").is_ok());
    assert!(validate_address("1234567890123456789012345678901234567890").is_err());
    assert!(validate_address("0x12345").is_err());
    assert!(validate_address("0xZZZZ567890123456789012345678901234567890").is_err());
}
