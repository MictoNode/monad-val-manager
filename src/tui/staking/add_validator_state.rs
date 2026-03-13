//! Add Validator State - State management for Add Validator dialog
//!
//! This module provides state management for the multi-field Add Validator
//! dialog, including field navigation, input handling, and validation.
//!
//! Uses ratatui-textarea for cross-platform input handling.

use ratatui_textarea::TextArea;

/// Input field indices for Add Validator dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddValidatorField {
    /// SECP256k1 private key (64 hex chars, with or without 0x prefix)
    SecpPrivkey = 0,
    /// BLS private key (64 hex chars, with or without 0x prefix)
    BlsPrivkey = 1,
    /// Authorized address (42 chars, with 0x prefix)
    AuthAddress = 2,
    /// Amount in MON (minimum 100,000)
    Amount = 3,
}

impl AddValidatorField {
    /// Get all fields in order
    pub fn all() -> &'static [AddValidatorField] {
        &[
            AddValidatorField::SecpPrivkey,
            AddValidatorField::BlsPrivkey,
            AddValidatorField::AuthAddress,
            AddValidatorField::Amount,
        ]
    }

    /// Get field label for display
    pub fn label(&self) -> &'static str {
        match self {
            AddValidatorField::SecpPrivkey => "SECP Key",
            AddValidatorField::BlsPrivkey => "BLS Key",
            AddValidatorField::AuthAddress => "Auth Addr",
            AddValidatorField::Amount => "Amount",
        }
    }

    /// Get field placeholder text
    pub fn placeholder(&self) -> &'static str {
        match self {
            AddValidatorField::SecpPrivkey => "64 hex (0x prefix optional)",
            AddValidatorField::BlsPrivkey => "64 hex (0x prefix optional)",
            AddValidatorField::AuthAddress => "0x...",
            AddValidatorField::Amount => "min: 100000",
        }
    }

    /// Get max length for this field
    pub fn max_length(&self) -> usize {
        match self {
            AddValidatorField::SecpPrivkey => 66, // Allow 0x + 64 hex chars
            AddValidatorField::BlsPrivkey => 66,  // Allow 0x + 64 hex chars
            AddValidatorField::AuthAddress => 42, // 0x + 40 hex chars
            AddValidatorField::Amount => 20,      // Large number
        }
    }

    /// Get next field
    pub fn next(&self) -> Option<AddValidatorField> {
        match self {
            AddValidatorField::SecpPrivkey => Some(AddValidatorField::BlsPrivkey),
            AddValidatorField::BlsPrivkey => Some(AddValidatorField::AuthAddress),
            AddValidatorField::AuthAddress => Some(AddValidatorField::Amount),
            AddValidatorField::Amount => None,
        }
    }

    /// Get previous field
    pub fn prev(&self) -> Option<AddValidatorField> {
        match self {
            AddValidatorField::SecpPrivkey => None,
            AddValidatorField::BlsPrivkey => Some(AddValidatorField::SecpPrivkey),
            AddValidatorField::AuthAddress => Some(AddValidatorField::BlsPrivkey),
            AddValidatorField::Amount => Some(AddValidatorField::AuthAddress),
        }
    }
}

/// State for the Add Validator dialog
#[derive(Debug, Clone)]
pub struct AddValidatorState {
    /// Whether the dialog is active
    pub is_active: bool,
    /// Currently focused field
    pub focused_field: AddValidatorField,
    /// Input fields using TextArea
    pub secp_privkey: TextArea<'static>,
    pub bls_privkey: TextArea<'static>,
    pub auth_address: TextArea<'static>,
    pub amount: TextArea<'static>,
    /// Error message for current field or general error
    pub error: Option<String>,
    /// Error field (which field has the error)
    pub error_field: Option<AddValidatorField>,
    /// Status message (for confirmation prompts)
    pub status: Option<String>,
    /// Whether user has confirmed (first Enter press)
    pub is_confirmed: bool,
    /// Validated parameters (after first Enter)
    pub validated_params: Option<AddValidatorParams>,
}

impl Default for AddValidatorState {
    fn default() -> Self {
        Self {
            is_active: false,
            focused_field: AddValidatorField::SecpPrivkey,
            secp_privkey: TextArea::default(),
            bls_privkey: TextArea::default(),
            auth_address: TextArea::default(),
            amount: TextArea::default(),
            error: None,
            error_field: None,
            status: None,
            is_confirmed: false,
            validated_params: None,
        }
    }
}

impl AddValidatorState {
    /// Create new Add Validator state
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
        self.focused_field = AddValidatorField::SecpPrivkey;
        self.secp_privkey = TextArea::default();
        self.bls_privkey = TextArea::default();
        self.auth_address = TextArea::default();
        self.amount = TextArea::default();
        self.error = None;
        self.error_field = None;
        self.status = None;
        self.is_confirmed = false;
        self.validated_params = None;
    }

    /// Set status message
    pub fn set_status(&mut self, msg: impl Into<String>, _field: Option<AddValidatorField>) {
        self.status = Some(msg.into());
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
            AddValidatorField::SecpPrivkey => &mut self.secp_privkey,
            AddValidatorField::BlsPrivkey => &mut self.bls_privkey,
            AddValidatorField::AuthAddress => &mut self.auth_address,
            AddValidatorField::Amount => &mut self.amount,
        }
    }

    /// Get reference to currently focused textarea
    pub fn current_textarea(&self) -> &TextArea<'static> {
        match self.focused_field {
            AddValidatorField::SecpPrivkey => &self.secp_privkey,
            AddValidatorField::BlsPrivkey => &self.bls_privkey,
            AddValidatorField::AuthAddress => &self.auth_address,
            AddValidatorField::Amount => &self.amount,
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
    pub fn set_error(&mut self, error: impl Into<String>, field: Option<AddValidatorField>) {
        self.error = Some(error.into());
        self.error_field = field;
    }

    /// Clear error
    pub fn clear_error(&mut self) {
        self.error = None;
        self.error_field = None;
    }

    /// Check if current field has error
    pub fn field_has_error(&self, field: AddValidatorField) -> bool {
        self.error_field == Some(field)
    }

    /// Get value for a specific field
    pub fn get_value(&self, field: AddValidatorField) -> String {
        let textarea = match field {
            AddValidatorField::SecpPrivkey => &self.secp_privkey,
            AddValidatorField::BlsPrivkey => &self.bls_privkey,
            AddValidatorField::AuthAddress => &self.auth_address,
            AddValidatorField::Amount => &self.amount,
        };
        textarea
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("")
            .to_string()
    }

    /// Validate SECP256k1 private key
    fn validate_secp_privkey(&self) -> Result<String, String> {
        let key_str = self
            .secp_privkey
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("");
        if key_str.is_empty() {
            return Err("SECP private key is required".to_string());
        }
        // Add 0x prefix if not present
        let formatted = if key_str.starts_with("0x") {
            key_str.to_string()
        } else {
            format!("0x{}", key_str)
        };
        // Check length (0x + 64 hex chars)
        if formatted.len() != 66 {
            return Err("SECP key must be 64 hex characters (0x prefix optional)".to_string());
        }
        // Check hex characters
        if !formatted[2..].chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("SECP key contains invalid hex characters".to_string());
        }
        Ok(formatted)
    }

    /// Validate BLS private key
    fn validate_bls_privkey(&self) -> Result<String, String> {
        let key_str = self
            .bls_privkey
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("");
        if key_str.is_empty() {
            return Err("BLS private key is required".to_string());
        }
        // Add 0x prefix if not present
        let formatted = if key_str.starts_with("0x") {
            key_str.to_string()
        } else {
            format!("0x{}", key_str)
        };
        // Check length (0x + 64 hex chars)
        if formatted.len() != 66 {
            return Err("BLS key must be 64 hex characters (0x prefix optional)".to_string());
        }
        // Check hex characters
        if !formatted[2..].chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("BLS key contains invalid hex characters".to_string());
        }
        Ok(formatted)
    }

    /// Validate authorized address
    fn validate_auth_address(&self) -> Result<String, String> {
        let addr_str = self
            .auth_address
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("");
        if addr_str.is_empty() {
            return Err("Authorized address is required".to_string());
        }
        // Add 0x prefix if not present
        let formatted = if addr_str.starts_with("0x") {
            addr_str.to_string()
        } else {
            format!("0x{}", addr_str)
        };
        // Check length (0x + 40 hex chars)
        if formatted.len() != 42 {
            return Err("Address must be 40 hex characters (0x prefix optional)".to_string());
        }
        // Check hex characters
        if !formatted[2..].chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("Address contains invalid hex characters".to_string());
        }
        Ok(formatted)
    }

    /// Validate amount
    fn validate_amount(&self) -> Result<u64, String> {
        let amount_str = self
            .amount
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("");
        if amount_str.is_empty() {
            return Err("Amount is required".to_string());
        }
        let amount: u64 = amount_str
            .parse()
            .map_err(|_| "Invalid amount (must be a number)".to_string())?;
        if amount < 100_000 {
            return Err("Amount must be at least 100,000 MON".to_string());
        }
        Ok(amount)
    }

    /// Validate all fields and return parsed values
    pub fn validate(&self) -> Result<AddValidatorParams, String> {
        let secp_privkey = self.validate_secp_privkey()?;
        let bls_privkey = self.validate_bls_privkey()?;
        let auth_address = self.validate_auth_address()?;
        let amount = self.validate_amount()?;

        Ok(AddValidatorParams {
            secp_privkey,
            bls_privkey,
            auth_address,
            amount,
        })
    }

    /// Validate current field only
    pub fn validate_current_field(&self) -> Result<(), String> {
        match self.focused_field {
            AddValidatorField::SecpPrivkey => {
                self.validate_secp_privkey()?;
                Ok(())
            }
            AddValidatorField::BlsPrivkey => {
                self.validate_bls_privkey()?;
                Ok(())
            }
            AddValidatorField::AuthAddress => {
                self.validate_auth_address()?;
                Ok(())
            }
            AddValidatorField::Amount => {
                self.validate_amount()?;
                Ok(())
            }
        }
    }
}

/// Validated parameters for Add Validator operation
#[derive(Debug, Clone, PartialEq)]
pub struct AddValidatorParams {
    /// SECP256k1 private key (with 0x prefix)
    pub secp_privkey: String,
    /// BLS private key (with 0x prefix)
    pub bls_privkey: String,
    /// Authorized address (with 0x prefix)
    pub auth_address: String,
    /// Amount in MON (minimum 100,000)
    pub amount: u64,
}

impl AddValidatorParams {
    /// Get a description of the operation for confirmation
    pub fn description(&self) -> String {
        format!(
            "Auth: {}...{}\nAmount: {} MON",
            &self.auth_address[..10],
            &self.auth_address[self.auth_address.len().saturating_sub(6)..],
            self.amount
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_validator_field_order() {
        let fields = AddValidatorField::all();
        assert_eq!(fields.len(), 4);
        assert_eq!(fields[0], AddValidatorField::SecpPrivkey);
        assert_eq!(fields[1], AddValidatorField::BlsPrivkey);
        assert_eq!(fields[2], AddValidatorField::AuthAddress);
        assert_eq!(fields[3], AddValidatorField::Amount);
    }

    #[test]
    fn test_add_validator_state_default() {
        let state = AddValidatorState::new();
        assert!(!state.is_active());
        assert_eq!(state.focused_field, AddValidatorField::SecpPrivkey);
    }

    #[test]
    fn test_add_validator_state_open_close() {
        let mut state = AddValidatorState::new();
        state.open();
        assert!(state.is_active());

        state.close();
        assert!(!state.is_active());
    }

    #[test]
    fn test_add_validator_validate_secp_privkey() {
        let mut state = AddValidatorState::new();
        state.open();

        // Empty
        assert!(state.validate_secp_privkey().is_err());

        // Invalid length
        state.secp_privkey = TextArea::from(["abc".to_string()]);
        assert!(state.validate_secp_privkey().is_err());

        // Valid without 0x
        state.secp_privkey = TextArea::from([
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
        ]);
        let result = state.validate_secp_privkey().unwrap();
        assert_eq!(
            result,
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        );

        // Valid with 0x
        state.secp_privkey = TextArea::from([
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
        ]);
        let result = state.validate_secp_privkey().unwrap();
        assert_eq!(
            result,
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        );
    }

    #[test]
    fn test_add_validator_validate_bls_privkey() {
        let mut state = AddValidatorState::new();
        state.open();

        // Empty
        assert!(state.validate_bls_privkey().is_err());

        // Invalid length
        state.bls_privkey = TextArea::from(["abc".to_string()]);
        assert!(state.validate_bls_privkey().is_err());

        // Valid
        state.bls_privkey = TextArea::from([
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
        ]);
        let result = state.validate_bls_privkey().unwrap();
        assert_eq!(
            result,
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        );
    }

    #[test]
    fn test_add_validator_validate_auth_address() {
        let mut state = AddValidatorState::new();
        state.open();

        // Empty
        assert!(state.validate_auth_address().is_err());

        // Invalid length
        state.auth_address = TextArea::from(["1234".to_string()]);
        assert!(state.validate_auth_address().is_err());

        // Valid
        state.auth_address =
            TextArea::from(["1234567890123456789012345678901234567890".to_string()]);
        let result = state.validate_auth_address().unwrap();
        assert_eq!(result, "0x1234567890123456789012345678901234567890");
    }

    #[test]
    fn test_add_validator_validate_amount() {
        let mut state = AddValidatorState::new();
        state.open();

        // Empty
        assert!(state.validate_amount().is_err());

        // Too small
        state.amount = TextArea::from(["99999".to_string()]);
        assert!(state.validate_amount().is_err());

        // Valid
        state.amount = TextArea::from(["100000".to_string()]);
        assert_eq!(state.validate_amount().unwrap(), 100000);
    }

    #[test]
    fn test_add_validator_params() {
        let params = AddValidatorParams {
            secp_privkey: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
                .to_string(),
            bls_privkey: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
                .to_string(),
            auth_address: "0x1234567890123456789012345678901234567890".to_string(),
            amount: 100000,
        };

        assert_eq!(params.secp_privkey.len(), 66);
        assert_eq!(params.bls_privkey.len(), 66);
        assert_eq!(params.auth_address.len(), 42);
        assert_eq!(params.amount, 100000);
    }
}
