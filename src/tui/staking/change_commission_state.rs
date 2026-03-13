//! Change Commission State - State management for Change Commission dialog
//!
//! This module provides state management for the Change Commission dialog,
//! The dialog allows changing validator commission rate (0.0 to 100.0%)
//! with two input fields: Validator ID and Commission.
//!
//! Uses ratatui-textarea for cross-platform input handling.

use ratatui_textarea::TextArea;

/// Input field indices for Change Commission dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeCommissionField {
    /// Validator ID field
    ValidatorId = 0,
    /// Commission rate field (0.0 to 100.0)
    Commission = 1,
}

impl ChangeCommissionField {
    /// Get all fields in order
    pub fn all() -> &'static [ChangeCommissionField] {
        &[
            ChangeCommissionField::ValidatorId,
            ChangeCommissionField::Commission,
        ]
    }

    /// Get field label for display
    pub fn label(&self) -> &'static str {
        match self {
            ChangeCommissionField::ValidatorId => "Validator ID",
            ChangeCommissionField::Commission => "Commission",
        }
    }

    /// Get field placeholder text
    pub fn placeholder(&self) -> &'static str {
        match self {
            ChangeCommissionField::ValidatorId => "e.g., 1",
            ChangeCommissionField::Commission => "0.0 - 100.0",
        }
    }

    /// Get next field
    pub fn next(&self) -> Option<ChangeCommissionField> {
        match self {
            ChangeCommissionField::ValidatorId => Some(ChangeCommissionField::Commission),
            ChangeCommissionField::Commission => None,
        }
    }

    /// Get previous field
    pub fn prev(&self) -> Option<ChangeCommissionField> {
        match self {
            ChangeCommissionField::ValidatorId => None,
            ChangeCommissionField::Commission => Some(ChangeCommissionField::ValidatorId),
        }
    }
}

/// State for the Change Commission dialog
#[derive(Debug, Clone)]
pub struct ChangeCommissionState {
    /// Whether the dialog is active
    pub is_active: bool,
    /// Currently focused field
    pub focused_field: ChangeCommissionField,
    /// Input fields using TextArea
    pub validator_id: TextArea<'static>,
    pub commission: TextArea<'static>,
    /// Current commission (fetched from validator, for display)
    pub current_commission: Option<f64>,
    /// Error message
    pub error: Option<String>,
    /// Error field (which field has the error)
    pub error_field: Option<ChangeCommissionField>,
    /// Status message (for confirmation prompts)
    pub status: Option<String>,
    /// Whether user has confirmed (first Enter press)
    pub is_confirmed: bool,
    /// Validated parameters (after first Enter)
    pub validated_params: Option<ChangeCommissionParams>,
}

impl Default for ChangeCommissionState {
    fn default() -> Self {
        Self {
            is_active: false,
            focused_field: ChangeCommissionField::ValidatorId,
            validator_id: TextArea::default(),
            commission: TextArea::default(),
            current_commission: None,
            error: None,
            error_field: None,
            status: None,
            is_confirmed: false,
            validated_params: None,
        }
    }
}

impl ChangeCommissionState {
    /// Create new Change Commission state
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
        self.focused_field = ChangeCommissionField::ValidatorId;
        self.validator_id = TextArea::default();
        self.commission = TextArea::default();
        self.error = None;
        self.error_field = None;
        self.status = None;
        self.is_confirmed = false;
        self.validated_params = None;
    }

    /// Open the dialog with validator ID pre-filled
    pub fn open_with_validator(&mut self, validator_id: u64) {
        self.reset();
        self.validator_id = TextArea::from([validator_id.to_string()]);
        self.is_active = true;
        // Note: current_commission will be fetched asynchronously by handler
    }

    /// Set current commission rate (for display)
    pub fn set_current_commission(&mut self, commission: f64) {
        self.current_commission = Some(commission);
    }

    /// Set status message
    pub fn set_status(&mut self, msg: impl Into<String>, _field: Option<ChangeCommissionField>) {
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
            ChangeCommissionField::ValidatorId => &mut self.validator_id,
            ChangeCommissionField::Commission => &mut self.commission,
        }
    }

    /// Get reference to currently focused textarea
    pub fn current_textarea(&self) -> &TextArea<'static> {
        match self.focused_field {
            ChangeCommissionField::ValidatorId => &self.validator_id,
            ChangeCommissionField::Commission => &self.commission,
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
    pub fn set_error(&mut self, error: impl Into<String>, field: Option<ChangeCommissionField>) {
        self.error = Some(error.into());
        self.error_field = field;
    }

    /// Clear error
    pub fn clear_error(&mut self) {
        self.error = None;
        self.error_field = None;
    }

    /// Check if current field has error
    pub fn field_has_error(&self, field: ChangeCommissionField) -> bool {
        self.error_field == Some(field)
    }

    /// Get value for a specific field
    pub fn get_value(&self, field: ChangeCommissionField) -> String {
        let textarea = match field {
            ChangeCommissionField::ValidatorId => &self.validator_id,
            ChangeCommissionField::Commission => &self.commission,
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

    /// Validate commission rate
    fn validate_commission(&self) -> Result<f64, String> {
        let comm_str = self
            .commission
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("");
        if comm_str.is_empty() {
            return Err("Commission is required".to_string());
        }
        let comm: f64 = comm_str
            .parse()
            .map_err(|_| "Invalid commission (must be a number)".to_string())?;
        if !(0.0..=100.0).contains(&comm) {
            return Err("Commission must be between 0 and 100".to_string());
        }
        Ok(comm)
    }

    /// Validate all fields and return parsed values
    pub fn validate(&self) -> Result<ChangeCommissionParams, String> {
        let validator_id = self.validate_validator_id()?;
        let commission = self.validate_commission()?;

        Ok(ChangeCommissionParams {
            validator_id,
            commission,
        })
    }

    /// Validate current field only
    pub fn validate_current_field(&self) -> Result<(), String> {
        match self.focused_field {
            ChangeCommissionField::ValidatorId => {
                self.validate_validator_id()?;
                Ok(())
            }
            ChangeCommissionField::Commission => {
                self.validate_commission()?;
                Ok(())
            }
        }
    }
}

/// Validated parameters for Change Commission operation
#[derive(Debug, Clone, PartialEq)]
pub struct ChangeCommissionParams {
    /// Validator ID
    pub validator_id: u64,
    /// Commission rate (0.0 - 100.0)
    pub commission: f64,
}

impl ChangeCommissionParams {
    /// Get a description of the operation for confirmation
    pub fn description(&self) -> String {
        format!(
            "Validator ID: {}, Commission: {:.2}%",
            self.validator_id, self.commission
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_commission_field_order() {
        let fields = ChangeCommissionField::all();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0], ChangeCommissionField::ValidatorId);
        assert_eq!(fields[1], ChangeCommissionField::Commission);
    }

    #[test]
    fn test_change_commission_state_default() {
        let state = ChangeCommissionState::new();
        assert!(!state.is_active());
        assert_eq!(state.focused_field, ChangeCommissionField::ValidatorId);
    }

    #[test]
    fn test_change_commission_state_open_close() {
        let mut state = ChangeCommissionState::new();
        state.open();
        assert!(state.is_active());

        state.close();
        assert!(!state.is_active());
    }

    #[test]
    fn test_change_commission_validate_validator_id() {
        let mut state = ChangeCommissionState::new();
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
        state.validator_id = TextArea::from(["1".to_string()]);
        assert_eq!(state.validate_validator_id().unwrap(), 1);
    }

    #[test]
    fn test_change_commission_validate_commission() {
        let mut state = ChangeCommissionState::new();
        state.open();

        // Empty
        assert!(state.validate_commission().is_err());

        // Invalid (non-numeric)
        state.commission = TextArea::from(["abc".to_string()]);
        assert!(state.validate_commission().is_err());

        // Too low
        state.commission = TextArea::from(["-1".to_string()]);
        assert!(state.validate_commission().is_err());

        // Too high
        state.commission = TextArea::from(["101".to_string()]);
        assert!(state.validate_commission().is_err());

        // Valid
        state.commission = TextArea::from(["50.5".to_string()]);
        assert_eq!(state.validate_commission().unwrap(), 50.5);
    }

    #[test]
    fn test_change_commission_params() {
        let params = ChangeCommissionParams {
            validator_id: 1,
            commission: 50.0,
        };

        assert_eq!(params.validator_id, 1);
        assert_eq!(params.commission, 50.0);
    }
}
