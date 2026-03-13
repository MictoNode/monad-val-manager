//! Undelegate Dialog Widget rendering
//!
//! This module provides [`UndelegateDialogWidget`] for rendering the 2-field
//! Undelegate modal dialog in the TUI.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::tui::staking::undelegate_state::{UndelegateField, UndelegateState};
use crate::tui::theme::THEME;

/// Widget for rendering the Undelegate dialog
pub struct UndelegateDialogWidget<'a> {
    /// Reference to dialog state
    state: &'a UndelegateState,
    /// Dialog width as percentage (0-100)
    width_percent: u16,
}

impl<'a> UndelegateDialogWidget<'a> {
    /// Create a new Undelegate dialog widget
    pub fn new(state: &'a UndelegateState) -> Self {
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
        let dialog_height = 14; // Fixed height for 2 fields + header + footer

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
            .title(" Undelegate ")
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
                Constraint::Length(3), // Validator ID field
                Constraint::Length(3), // Amount field
                Constraint::Length(1), // Delegated amount hint
                Constraint::Length(1), // Error line
                Constraint::Length(1), // Actions
            ])
            .split(dialog_area);

        // Render instruction
        let instruction = Paragraph::new(Line::from(vec![Span::styled(
            "Enter Validator ID and Amount to undelegate MON",
            THEME.metric_label(),
        )]))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });

        frame.render_widget(instruction, chunks[0]);

        // Render each field
        self.render_field(frame, chunks[1], UndelegateField::ValidatorId);
        self.render_field(frame, chunks[2], UndelegateField::Amount);

        // Render delegated amount hint if set
        if let Some(amount) = self.state.delegated_amount {
            let hint = Line::from(vec![
                Span::styled("Delegated: ", THEME.label()),
                Span::styled(format!("{:.2} MON", amount), THEME.balance()),
            ]);
            let hint_para = Paragraph::new(hint).alignment(Alignment::Center);
            frame.render_widget(hint_para, chunks[3]);
        }

        // Render error if present
        if let Some(ref error) = self.state.error {
            let error_line = Line::styled(format!(" Error: {}", error), THEME.status_error());
            let error_para = Paragraph::new(error_line);
            frame.render_widget(error_para, chunks[4]);
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
        frame.render_widget(actions_para, chunks[5]);
    }

    /// Render a single input field using TextArea widget
    fn render_field(&self, frame: &mut Frame, area: Rect, field: UndelegateField) {
        let is_focused = self.state.focused_field == field;
        let has_error = self.state.field_has_error(field);

        // Get the textarea for this field
        let textarea = match field {
            UndelegateField::ValidatorId => &self.state.validator_id,
            UndelegateField::Amount => &self.state.amount,
        };

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

        // Create border block with title
        let border_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .title(field.label())
            .title_style(title_style);

        // Get inner area (inside the border)
        let inner_area = border_block.inner(area);

        // Render the border block first
        frame.render_widget(border_block, area);

        // Render the textarea widget inside the border
        // Due to Widget trait incompatibility between tui-textarea and ratatui 0.30,
        // we render the content manually using Paragraph.
        // The input handling (backspace, cursor movement, etc.) still works via TextArea.
        let text_content = textarea.lines().first().map(|s| s.as_str()).unwrap_or("");
        let input_paragraph = if text_content.is_empty() && !is_focused {
            // Show placeholder when empty and not focused
            Paragraph::new(field.placeholder()).style(THEME.placeholder())
        } else {
            Paragraph::new(text_content).style(THEME.input_text())
        };
        frame.render_widget(input_paragraph, inner_area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undelegate_dialog_widget_creation() {
        let state = UndelegateState::new();
        let widget = UndelegateDialogWidget::new(&state);
        assert_eq!(widget.width_percent, 60);
    }

    #[test]
    fn test_undelegate_dialog_widget_with_width() {
        let state = UndelegateState::new();
        let widget = UndelegateDialogWidget::new(&state).with_width(80);
        assert_eq!(widget.width_percent, 80);
    }

    #[test]
    fn test_undelegate_dialog_widget_calculate_dialog_area() {
        let state = UndelegateState::new();
        let widget = UndelegateDialogWidget::new(&state).with_width(60);

        let frame_area = Rect::new(0, 0, 100, 30);
        let dialog_area = widget.calculate_dialog_area(frame_area);

        // Dialog should be centered and 60% width
        assert_eq!(dialog_area.width, 60);
        assert_eq!(dialog_area.height, 14);
    }
}
