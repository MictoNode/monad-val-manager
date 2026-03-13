//! Input Dialog Widget - Modal dialog for user text input
//!
//! This module provides a reusable input dialog for the TUI that can be used
//! for various staking operations like delegate amount, validator address, etc.
//!
//! Features:
//! - Modal overlay with centered dialog box
//! - Text input with cursor positioning
//! - Title and placeholder text
//! - Validation feedback
//! - Confirm/Cancel actions

mod dialog_type;
mod state;
mod validation;
mod widget;

// Re-export all public types and functions
pub use dialog_type::DialogType;
pub use state::InputDialogState;
pub use widget::InputDialogWidget;

// Validation functions are primarily used internally but exposed for testing
#[cfg(test)]
pub use validation::{validate_address, validate_amount};

#[cfg(test)]
mod tests;
