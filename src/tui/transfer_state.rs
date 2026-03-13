//! Transfer dialog state management
//!
//! This module provides [`TransferDialogState`] which manages the state
//! for the native MON transfer dialog in the TUI.
//!
//! Uses tui-textarea for cross-platform input handling.

use ratatui_textarea::TextArea;

/// Steps in the transfer flow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransferStep {
    /// Input recipient address
    #[default]
    Address,
    /// Input amount
    Amount,
    /// Confirm transaction
    Confirm,
    /// Transaction in progress
    Processing,
    /// Transaction completed
    Complete,
}

/// State for the transfer dialog
#[derive(Debug, Clone)]
pub struct TransferDialogState {
    /// Current step in the transfer flow
    pub step: TransferStep,
    /// Recipient address (using TextArea)
    pub address: TextArea<'static>,
    /// Amount in MON (human-readable) (using TextArea)
    pub amount: TextArea<'static>,
    /// Error message (if validation fails)
    pub error: Option<String>,
    /// Is the dialog currently active/visible
    pub is_active: bool,
    /// Transaction hash (if completed)
    pub tx_hash: Option<String>,
    /// Available balance for context
    pub available_balance: Option<String>,
}

impl Default for TransferDialogState {
    fn default() -> Self {
        Self {
            step: TransferStep::Address,
            address: TextArea::default(),
            amount: TextArea::default(),
            error: None,
            is_active: false,
            tx_hash: None,
            available_balance: None,
        }
    }
}

impl TransferDialogState {
    /// Create a new transfer dialog state
    pub fn new() -> Self {
        Self::default()
    }

    /// Open the dialog
    pub fn open(&mut self, available_balance: Option<String>) {
        self.step = TransferStep::Address;
        self.address = TextArea::default();
        self.amount = TextArea::default();
        self.error = None;
        self.is_active = true;
        self.tx_hash = None;
        self.available_balance = available_balance;
    }

    /// Close the dialog and reset state
    pub fn close(&mut self) {
        self.address = TextArea::default();
        self.amount = TextArea::default();
        self.error = None;
        self.is_active = false;
        self.tx_hash = None;
        self.available_balance = None;
    }

    /// Move to next step
    pub fn next_step(&mut self) {
        self.step = match self.step {
            TransferStep::Address => TransferStep::Amount,
            TransferStep::Amount => TransferStep::Confirm,
            TransferStep::Confirm => TransferStep::Processing,
            TransferStep::Processing => TransferStep::Complete,
            TransferStep::Complete => TransferStep::Complete,
        };
        self.error = None;
    }

    /// Move to previous step
    pub fn prev_step(&mut self) {
        self.step = match self.step {
            TransferStep::Address => TransferStep::Address,
            TransferStep::Amount => TransferStep::Address,
            TransferStep::Confirm => TransferStep::Amount,
            TransferStep::Processing => TransferStep::Processing,
            TransferStep::Complete => TransferStep::Complete,
        };
        self.error = None;
    }

    /// Get current textarea based on step
    pub fn current_textarea_mut(&mut self) -> &mut TextArea<'static> {
        match self.step {
            TransferStep::Address => &mut self.address,
            TransferStep::Amount => &mut self.amount,
            _ => &mut self.address,
        }
    }

    /// Get current textarea based on step
    pub fn current_textarea(&self) -> &TextArea<'static> {
        match self.step {
            TransferStep::Address => &self.address,
            TransferStep::Amount => &self.amount,
            _ => &self.address,
        }
    }

    /// Get current input string based on step
    pub fn current_input(&self) -> String {
        self.current_textarea()
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("")
            .to_string()
    }

    /// Validate current step input
    pub fn validate_current(&self) -> Result<(), String> {
        match self.step {
            TransferStep::Address => {
                let input = self.current_input();
                let addr = input.trim().trim_start_matches("0x");
                if addr.len() != 40 {
                    return Err(
                        "Address must be 42 hex characters (including 0x prefix)".to_string()
                    );
                }
                if !addr.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Err("Address contains invalid characters".to_string());
                }
                Ok(())
            }
            TransferStep::Amount => {
                let amount_str = self.current_input();
                if amount_str.is_empty() {
                    return Err("Amount cannot be empty".to_string());
                }
                if amount_str.parse::<f64>().is_err() {
                    return Err("Invalid amount format".to_string());
                }
                let amount = amount_str.parse::<f64>().unwrap();
                if amount <= 0.0 {
                    return Err("Amount must be greater than 0".to_string());
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Set transaction hash (for completion step)
    pub fn set_tx_hash(&mut self, hash: String) {
        self.tx_hash = Some(hash);
    }

    /// Get formatted address for display
    pub fn formatted_address(&self) -> String {
        let addr = self
            .address
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("");
        if addr.starts_with("0x") {
            addr.to_string()
        } else {
            format!("0x{}", addr)
        }
    }

    /// Get amount string for display
    pub fn get_amount_str(&self) -> String {
        self.amount
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("")
            .to_string()
    }

    /// Get address string
    pub fn get_address_str(&self) -> String {
        self.address
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("")
            .to_string()
    }

    /// Check if address is empty
    pub fn is_address_empty(&self) -> bool {
        self.address
            .lines()
            .first()
            .map(|s| s.is_empty())
            .unwrap_or(true)
    }

    /// Check if amount is empty
    pub fn is_amount_empty(&self) -> bool {
        self.amount
            .lines()
            .first()
            .map(|s| s.is_empty())
            .unwrap_or(true)
    }

    /// Get address length (for character count hint)
    pub fn address_len(&self) -> usize {
        self.address.lines().first().map(|s| s.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_state_open_close() {
        let mut state = TransferDialogState::new();
        assert!(!state.is_active);

        state.open(Some("100.0 MON".to_string()));
        assert!(state.is_active);
        assert_eq!(state.step, TransferStep::Address);
        assert_eq!(state.available_balance, Some("100.0 MON".to_string()));

        state.close();
        assert!(!state.is_active);
        assert_eq!(state.current_input(), "");
    }

    #[test]
    fn test_transfer_step_navigation() {
        let mut state = TransferDialogState::new();
        state.open(None);

        assert_eq!(state.step, TransferStep::Address);
        state.next_step();
        assert_eq!(state.step, TransferStep::Amount);
        state.next_step();
        assert_eq!(state.step, TransferStep::Confirm);
        state.next_step();
        assert_eq!(state.step, TransferStep::Processing);

        state.prev_step(); // Still Processing
        assert_eq!(state.step, TransferStep::Processing);

        state.step = TransferStep::Amount;
        state.prev_step();
        assert_eq!(state.step, TransferStep::Address);
    }

    #[test]
    fn test_textarea_input() {
        let mut state = TransferDialogState::new();
        state.open(None);

        state.address = TextArea::from(["123".to_string()]);
        assert_eq!(state.current_input(), "123");
    }

    #[test]
    fn test_validate_address_valid() {
        let mut state = TransferDialogState::new();
        state.open(None);

        state.address = TextArea::from(["1234567890123456789012345678901234567890".to_string()]);
        assert!(state.validate_current().is_ok());
    }

    #[test]
    fn test_validate_address_invalid_length() {
        let mut state = TransferDialogState::new();
        state.open(None);

        state.address = TextArea::from(["1234".to_string()]);
        assert!(state.validate_current().is_err());
    }

    #[test]
    fn test_validate_amount_valid() {
        let mut state = TransferDialogState::new();
        state.open(None);
        state.step = TransferStep::Amount;

        state.amount = TextArea::from(["1.5".to_string()]);
        assert!(state.validate_current().is_ok());
    }

    #[test]
    fn test_validate_amount_zero() {
        let mut state = TransferDialogState::new();
        state.open(None);
        state.step = TransferStep::Amount;

        state.amount = TextArea::from(["0".to_string()]);
        assert!(state.validate_current().is_err());
    }

    #[test]
    fn test_formatted_address() {
        let mut state = TransferDialogState::new();
        state.open(None);

        state.address = TextArea::from(["1234567890123456789012345678901234567890".to_string()]);
        assert_eq!(
            state.formatted_address(),
            "0x1234567890123456789012345678901234567890"
        );

        state.address = TextArea::from(["0x1234567890123456789012345678901234567890".to_string()]);
        assert_eq!(
            state.formatted_address(),
            "0x1234567890123456789012345678901234567890"
        );
    }
}
