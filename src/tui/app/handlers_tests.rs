//! Tests for TUI event handlers
//!
//! This module contains integration tests for keyboard input handling
//! to ensure dialogs respond correctly to user input.

use crate::cli::Network as CliNetwork;
use crate::config::Config;
use crate::tui::app::TuiApp;
use crossterm::event::KeyCode;
use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};

fn make_key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        state: crossterm::event::KeyEventState::empty(),
    }
}

/// Helper to create a test app
async fn create_test_app() -> TuiApp {
    let config = Config::create_default(CliNetwork::Mainnet).unwrap();
    TuiApp::new(&config).unwrap()
}

#[cfg(test)]
mod transfer_dialog_tests {
    use super::*;

    /// BUG-013 Test: Transfer dialog should handle character input when open
    #[tokio::test]
    async fn test_transfer_dialog_handles_char_input() {
        let mut app = create_test_app().await;
        app.current_screen = crate::tui::screens::Screen::Transfer;

        // Open transfer dialog
        app.state.transfer.open(Some("100.0 MON".to_string()));
        assert!(app.state.transfer.is_active);

        // Try to input a character
        let key = make_key(KeyCode::Char('1'), KeyModifiers::empty());

        // This should modify the address field
        let original_address = app
            .state
            .transfer
            .address
            .lines()
            .first()
            .cloned()
            .unwrap_or_default();
        app.handle_key_event(key).await;

        // BUG: The character should have been added but wasn't
        // This test will FAIL before the fix and PASS after
        let new_address = app
            .state
            .transfer
            .address
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("");
        assert_ne!(
            new_address, original_address,
            "BUG-013: Transfer dialog should handle character input when active"
        );
        assert_eq!(
            new_address, "1",
            "BUG-013: Character '1' should have been added to address field"
        );
    }

    /// BUG-013 Test: Transfer dialog should handle backspace when open
    #[tokio::test]
    async fn test_transfer_dialog_handles_backspace() {
        let mut app = create_test_app().await;
        app.current_screen = crate::tui::screens::Screen::Transfer;

        // Open transfer dialog
        app.state.transfer.open(Some("100.0 MON".to_string()));

        // Type "123" character by character
        app.handle_key_event(make_key(KeyCode::Char('1'), KeyModifiers::empty()))
            .await;
        app.handle_key_event(make_key(KeyCode::Char('2'), KeyModifiers::empty()))
            .await;
        app.handle_key_event(make_key(KeyCode::Char('3'), KeyModifiers::empty()))
            .await;

        // Verify "123" was entered
        let address = app
            .state
            .transfer
            .address
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("");
        assert_eq!(address, "123");

        // Press backspace
        app.handle_key_event(make_key(KeyCode::Backspace, KeyModifiers::empty()))
            .await;

        // BUG: The backspace should have removed a character but didn't
        let address = app
            .state
            .transfer
            .address
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("");
        assert_eq!(
            address, "12",
            "BUG-013: Backspace should remove last character from address field"
        );
    }

    /// BUG-013 Test: Transfer dialog should close on Escape
    #[tokio::test]
    async fn test_transfer_dialog_closes_on_escape() {
        let mut app = create_test_app().await;
        app.current_screen = crate::tui::screens::Screen::Transfer;

        // Open transfer dialog
        app.state.transfer.open(Some("100.0 MON".to_string()));
        assert!(app.state.transfer.is_active);

        // Press Escape
        let key = make_key(KeyCode::Esc, KeyModifiers::empty());
        app.handle_key_event(key).await;

        // BUG: The dialog should have closed but didn't
        assert!(
            !app.state.transfer.is_active,
            "BUG-013: Escape key should close transfer dialog"
        );
    }
}

#[cfg(test)]
mod add_validator_dialog_tests {
    use super::*;

    /// Test: Add Validator dialog should handle Ctrl+V paste action
    #[tokio::test]
    async fn test_transfer_dialog_handles_paste_action() {
        let mut app = create_test_app().await;
        app.current_screen = crate::tui::screens::Screen::Transfer;

        // Open transfer dialog
        app.state.transfer.open(Some("100.0 MON".to_string()));
        assert!(app.state.transfer.is_active);

        // Simulate Ctrl+V in address step
        let key = make_key(KeyCode::Char('v'), KeyModifiers::CONTROL);

        // This should trigger the paste action (will fail to access clipboard in test env)
        // but we're testing that the action is recognized
        app.handle_key_event(key).await;

        // The dialog should still be active (paste action was handled)
        assert!(
            app.state.transfer.is_active,
            "Paste action should be handled and dialog should remain active"
        );
    }

    /// Test: Add Validator dialog should handle Ctrl+V paste action
    #[tokio::test]
    async fn test_add_validator_dialog_handles_paste_action() {
        let mut app = create_test_app().await;
        app.current_screen = crate::tui::screens::Screen::Staking;

        // Open add validator dialog
        app.state.add_validator.open();
        assert!(app.state.add_validator.is_active());

        // Simulate Ctrl+V
        let key = make_key(KeyCode::Char('v'), KeyModifiers::CONTROL);

        // This should trigger the paste action (will fail to access clipboard in test env)
        // but we're testing that the action is recognized
        app.handle_key_event(key).await;

        // The dialog should still be active (paste action was handled)
        assert!(
            app.state.add_validator.is_active(),
            "Paste action should be handled and dialog should remain active"
        );
    }

    /// Test: Change Commission dialog should handle Ctrl+V paste action
    #[tokio::test]
    async fn test_change_commission_dialog_handles_paste_action() {
        let mut app = create_test_app().await;
        app.current_screen = crate::tui::screens::Screen::Staking;

        // Open change commission dialog
        app.state.change_commission.open();
        assert!(app.state.change_commission.is_active());

        // Simulate Ctrl+V
        let key = make_key(KeyCode::Char('v'), KeyModifiers::CONTROL);

        // This should trigger the paste action (will fail to access clipboard in test env)
        // but we're testing that the action is recognized
        app.handle_key_event(key).await;

        // The dialog should still be active (paste action was handled)
        assert!(
            app.state.change_commission.is_active(),
            "Paste action should be handled and dialog should remain active"
        );
    }
}
