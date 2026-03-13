//! Add Validator Dialog Widget rendering
//!
//! This module provides [`AddValidatorDialogWidget`] for rendering the multi-field
//! Add Validator modal dialog in the TUI.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::tui::staking::add_validator_state::{AddValidatorField, AddValidatorState};
use crate::tui::theme::THEME;

/// Widget for rendering the Add Validator dialog
pub struct AddValidatorDialogWidget<'a> {
    /// Reference to dialog state
    state: &'a AddValidatorState,
    /// Dialog width as percentage (0-100)
    width_percent: u16,
}

impl<'a> AddValidatorDialogWidget<'a> {
    /// Create a new Add Validator dialog widget
    pub fn new(state: &'a AddValidatorState) -> Self {
        Self {
            state,
            width_percent: 60,
        }
    }

    /// Set dialog width as percentage
    pub fn with_width(mut self, percent: u16) -> Self {
        self.width_percent = percent.min(100);
        self
    }

    /// Calculate the centered dialog area
    fn calculate_dialog_area(&self, frame_area: Rect) -> Rect {
        let dialog_width = (frame_area.width * self.width_percent / 100).clamp(50, 70);
        let dialog_height = 18; // Fixed height for 4 fields + header + footer

        let x = (frame_area.width.saturating_sub(dialog_width)) / 2;
        let y = (frame_area.height.saturating_sub(dialog_height)) / 2;

        Rect::new(x, y, dialog_width, dialog_height)
    }

    /// Render the dialog
    pub fn render(&self, frame: &mut Frame) {
        if !self.state.is_active {
            return;
        }

        let frame_area = frame.area();
        let dialog_area = self.calculate_dialog_area(frame_area);

        // Clear the area where the dialog will be rendered
        frame.render_widget(Clear, dialog_area);

        // Render border and title with Double border type
        let title_block = Block::default()
            .title(" Add Validator ")
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(THEME.dialog_border())
            .title_style(THEME.dialog_title());

        frame.render_widget(title_block, dialog_area);

        // Create layout for dialog content
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(2), // Instruction
                Constraint::Length(3), // SECP key field
                Constraint::Length(3), // BLS key field
                Constraint::Length(3), // Auth address field
                Constraint::Length(3), // Amount field
                Constraint::Length(1), // Error line
                Constraint::Length(1), // Actions
            ])
            .split(dialog_area);

        // Render instruction
        let instruction = Paragraph::new(Line::from(vec![Span::styled(
            "Enter SECP key, BLS key, auth address, and amount to add validator",
            THEME.metric_label(),
        )]))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });

        frame.render_widget(instruction, chunks[0]);

        // Render each field
        self.render_field(frame, chunks[1], AddValidatorField::SecpPrivkey);
        self.render_field(frame, chunks[2], AddValidatorField::BlsPrivkey);
        self.render_field(frame, chunks[3], AddValidatorField::AuthAddress);
        self.render_field(frame, chunks[4], AddValidatorField::Amount);

        // Render error if present
        if let Some(ref error) = self.state.error {
            let error_line = Line::styled(format!(" Error: {}", error), THEME.status_error());
            let error_para = Paragraph::new(error_line);
            frame.render_widget(error_para, chunks[5]);
        }

        // Render action hints
        let actions = Line::from(vec![
            Span::styled("[Tab]", THEME.keybind()),
            Span::styled(" Next  ", THEME.keybind_description()),
            Span::styled("[Shift+Tab]", THEME.keybind()),
            Span::styled(" Prev  ", THEME.keybind_description()),
            Span::styled("[Enter]", THEME.keybind()),
            Span::styled(" Confirm  ", THEME.keybind_description()),
            Span::styled("[Esc]", THEME.keybind()),
            Span::styled(" Cancel", THEME.keybind_description()),
        ]);
        let actions_para = Paragraph::new(actions).alignment(Alignment::Center);
        frame.render_widget(actions_para, chunks[6]);
    }

    /// Render a single input field
    fn render_field(&self, frame: &mut Frame, area: Rect, field: AddValidatorField) {
        let is_focused = self.state.focused_field == field;
        let has_error = self.state.field_has_error(field);

        // Determine border style based on focus state
        let border_style = if has_error {
            THEME.status_error()
        } else if is_focused {
            THEME.input_border_focused() // Cyan for focused
        } else {
            THEME.input_border() // Purple for unfocused
        };

        // Determine title style (Bold when focused)
        let title_style = if is_focused {
            THEME.dialog_title() // Bold purple
        } else {
            THEME.label()
        };

        // Build the field content - get value from TextArea
        let value = self.state.get_value(field);

        // Create display value - show placeholder if empty, otherwise show value
        // Mask sensitive fields when not focused
        let display_value = if value.is_empty() {
            field.placeholder().to_string()
        } else if is_focused {
            value.clone()
        } else {
            // Mask sensitive fields
            match field {
                AddValidatorField::SecpPrivkey | AddValidatorField::BlsPrivkey => {
                    "*".repeat(value.len().min(20))
                }
                _ => value.clone(),
            }
        };

        // Create the input block with rounded border
        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .title(field.label())
            .title_style(title_style);

        // Create inner area for the input text
        let inner_area = input_block.inner(area);

        // Render the border block first
        frame.render_widget(input_block, area);

        // Render the input text
        let input_style = if is_focused {
            THEME.input_text()
        } else if value.is_empty() {
            THEME.placeholder()
        } else {
            THEME.input_text()
        };

        let input_line = Line::from(vec![Span::styled(display_value, input_style)]);
        let input_para = Paragraph::new(input_line);
        frame.render_widget(input_para, inner_area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_validator_dialog_widget_creation() {
        let state = AddValidatorState::new();
        let widget = AddValidatorDialogWidget::new(&state);
        assert_eq!(widget.width_percent, 60);
    }

    #[test]
    fn test_add_validator_dialog_widget_with_width() {
        let state = AddValidatorState::new();
        let widget = AddValidatorDialogWidget::new(&state).with_width(80);
        assert_eq!(widget.width_percent, 80);
    }

    #[test]
    fn test_add_validator_dialog_widget_width_capped_at_100() {
        let state = AddValidatorState::new();
        let widget = AddValidatorDialogWidget::new(&state).with_width(150);
        assert_eq!(widget.width_percent, 100);
    }

    #[test]
    fn test_add_validator_dialog_widget_calculate_dialog_area() {
        let state = AddValidatorState::new();
        let widget = AddValidatorDialogWidget::new(&state).with_width(60);

        let frame_area = Rect::new(0, 0, 100, 30);
        let dialog_area = widget.calculate_dialog_area(frame_area);

        // Dialog should be centered
        assert_eq!(dialog_area.width, 60);
        assert_eq!(dialog_area.height, 18);
    }

    #[test]
    fn test_add_validator_dialog_widget_minimum_width() {
        let state = AddValidatorState::new();
        let widget = AddValidatorDialogWidget::new(&state).with_width(30);

        let frame_area = Rect::new(0, 0, 100, 30);
        let dialog_area = widget.calculate_dialog_area(frame_area);

        // Should use minimum width of 50
        assert!(dialog_area.width >= 50);
    }

    #[test]
    fn test_add_validator_dialog_widget_builder_chain() {
        let state = AddValidatorState::new();
        let widget = AddValidatorDialogWidget::new(&state).with_width(70);

        assert_eq!(widget.width_percent, 70);
    }

    #[test]
    fn test_add_validator_dialog_widget_not_active() {
        let state = AddValidatorState::new();
        // State is not active by default
        assert!(!state.is_active());
    }
}
