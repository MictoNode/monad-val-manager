//! Transfer integration tests
//!
//! Tests for Transfer TUI screen and dialog functionality.
//!
//! Test categories:
//! - Transfer dialog state management
//! - Transfer screen rendering
//! - Transfer action handlers
//! - Input validation (address, amount)
//! - Transaction flow (Address -> Amount -> Confirm -> Processing -> Complete)

use ratatui_textarea::TextArea;

use monad_val_manager::tui::transfer_state::{TransferDialogState, TransferStep};

// =============================================================================
// TRANSFER STATE TESTS
// =============================================================================

#[test]
fn test_transfer_dialog_state_default() {
    let state = TransferDialogState::new();
    assert!(!state.is_active);
    assert_eq!(state.step, TransferStep::Address);
    assert!(state.address.is_empty());
    assert!(state.amount.is_empty());
    assert!(state.error.is_none());
    assert!(state.tx_hash.is_none());
}

#[test]
fn test_transfer_dialog_state_open() {
    let mut state = TransferDialogState::new();
    state.open(Some("100.0 MON".to_string()));

    assert!(state.is_active);
    assert_eq!(state.step, TransferStep::Address);
    assert!(state.address.is_empty());
    assert!(state.amount.is_empty());
    assert!(state.error.is_none());
    assert_eq!(state.available_balance, Some("100.0 MON".to_string()));
}

#[test]
fn test_transfer_dialog_state_close() {
    let mut state = TransferDialogState::new();
    state.open(Some("100.0 MON".to_string()));
    assert!(state.is_active);

    state.close();
    assert!(!state.is_active);
    assert!(state.address.is_empty());
    assert!(state.amount.is_empty());
    assert!(state.tx_hash.is_none());
    assert!(state.available_balance.is_none());
}

#[test]
fn test_transfer_step_navigation_forward() {
    let mut state = TransferDialogState::new();
    state.open(None);

    assert_eq!(state.step, TransferStep::Address);

    state.next_step();
    assert_eq!(state.step, TransferStep::Amount);

    state.next_step();
    assert_eq!(state.step, TransferStep::Confirm);

    state.next_step();
    assert_eq!(state.step, TransferStep::Processing);

    state.next_step();
    assert_eq!(state.step, TransferStep::Complete);

    // Complete stays Complete
    state.next_step();
    assert_eq!(state.step, TransferStep::Complete);
}

#[test]
fn test_transfer_step_navigation_backward() {
    let mut state = TransferDialogState::new();
    state.open(None);

    // Advance to Amount
    state.next_step();
    assert_eq!(state.step, TransferStep::Amount);

    state.prev_step();
    assert_eq!(state.step, TransferStep::Address);

    // Can't go back before Address
    state.prev_step();
    assert_eq!(state.step, TransferStep::Address);
}

#[test]
fn test_transfer_text_area_empty() {
    let state = TransferDialogState::new();
    // TextArea starts empty
    assert!(state.address.is_empty());
    assert!(state.amount.is_empty());
}

// =============================================================================
// VALIDATION TESTS
// =============================================================================

#[test]
fn test_transfer_validate_address_valid() {
    let mut state = TransferDialogState::new();
    state.open(None);

    // Valid 40-char hex address
    state.address = TextArea::from(["1234567890123456789012345678901234567890".to_string()]);
    assert!(state.validate_current().is_ok());
}

#[test]
fn test_transfer_validate_address_invalid_length() {
    let mut state = TransferDialogState::new();
    state.open(None);

    state.address = TextArea::from(["1234".to_string()]);
    let result = state.validate_current();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("42 hex characters"));
}

#[test]
fn test_transfer_validate_address_invalid_chars() {
    let mut state = TransferDialogState::new();
    state.open(None);

    state.address = TextArea::from(["GGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG".to_string()]);
    let result = state.validate_current();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("invalid characters"));
}

#[test]
fn test_transfer_validate_amount_valid() {
    let mut state = TransferDialogState::new();
    state.open(None);
    state.step = TransferStep::Amount;

    state.amount = TextArea::from(["1.5".to_string()]);
    assert!(state.validate_current().is_ok());

    state.amount = TextArea::from(["100".to_string()]);
    assert!(state.validate_current().is_ok());

    state.amount = TextArea::from(["0.00000001".to_string()]);
    assert!(state.validate_current().is_ok());
}

#[test]
fn test_transfer_validate_amount_empty() {
    let mut state = TransferDialogState::new();
    state.open(None);
    state.step = TransferStep::Amount;

    state.amount = TextArea::from(["".to_string()]);
    let result = state.validate_current();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot be empty"));
}

#[test]
fn test_transfer_validate_amount_zero() {
    let mut state = TransferDialogState::new();
    state.open(None);
    state.step = TransferStep::Amount;

    state.amount = TextArea::from(["0".to_string()]);
    let result = state.validate_current();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("greater than 0"));
}

#[test]
fn test_transfer_validate_amount_negative() {
    let mut state = TransferDialogState::new();
    state.open(None);
    state.step = TransferStep::Amount;

    state.amount = TextArea::from(["-1.5".to_string()]);
    let result = state.validate_current();
    assert!(result.is_err());
}

#[test]
fn test_transfer_validate_amount_invalid_format() {
    let mut state = TransferDialogState::new();
    state.open(None);
    state.step = TransferStep::Amount;

    state.amount = TextArea::from(["abc".to_string()]);
    let result = state.validate_current();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid amount"));
}

#[test]
fn test_transfer_formatted_address_without_prefix() {
    let mut state = TransferDialogState::new();
    state.open(None);

    state.address = TextArea::from(["1234567890123456789012345678901234567890".to_string()]);
    assert_eq!(
        state.formatted_address(),
        "0x1234567890123456789012345678901234567890"
    );
}

#[test]
fn test_transfer_formatted_address_with_prefix() {
    let mut state = TransferDialogState::new();
    state.open(None);

    state.address = TextArea::from(["0x1234567890123456789012345678901234567890".to_string()]);
    assert_eq!(
        state.formatted_address(),
        "0x1234567890123456789012345678901234567890"
    );
}

#[test]
fn test_transfer_set_tx_hash() {
    let mut state = TransferDialogState::new();
    state.set_tx_hash("0xabc123".to_string());
    assert_eq!(state.tx_hash, Some("0xabc123".to_string()));
}

#[test]
fn test_transfer_available_balance_display() {
    let mut state = TransferDialogState::new();
    state.open(Some("50.0 MON".to_string()));
    assert_eq!(state.available_balance, Some("50.0 MON".to_string()));
}

#[test]
fn test_transfer_current_input() {
    let mut state = TransferDialogState::new();
    state.open(None);

    state.address = TextArea::from(["test_address".to_string()]);
    assert_eq!(state.current_input(), "test_address");

    state.step = TransferStep::Amount;
    state.amount = TextArea::from(["10.5".to_string()]);
    assert_eq!(state.current_input(), "10.5");

    // Note: Confirm step returns address content via default fallback
    state.step = TransferStep::Confirm;
    assert_eq!(state.current_input(), "test_address");

    // Processing step also returns address via default fallback
    state.step = TransferStep::Processing;
    assert_eq!(state.current_input(), "test_address");
}

#[test]
fn test_transfer_is_address_empty() {
    let state = TransferDialogState::new();
    assert!(state.is_address_empty());

    let mut state = TransferDialogState::new();
    state.address = TextArea::from(["0x123".to_string()]);
    assert!(!state.is_address_empty());
}

#[test]
fn test_transfer_is_amount_empty() {
    let state = TransferDialogState::new();
    assert!(state.is_amount_empty());

    let mut state = TransferDialogState::new();
    state.amount = TextArea::from(["1.5".to_string()]);
    assert!(!state.is_amount_empty());
}

#[test]
fn test_transfer_address_len() {
    let state = TransferDialogState::new();
    assert_eq!(state.address_len(), 0);

    let mut state = TransferDialogState::new();
    state.address = TextArea::from(["0x1234567890123456789012345678901234567890".to_string()]);
    assert_eq!(state.address_len(), 42);
}
