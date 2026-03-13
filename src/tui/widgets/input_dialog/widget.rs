//! Input dialog widget rendering
//!
//! This module provides [`InputDialogWidget`] for rendering the modal
//! input dialog in the TUI.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use super::state::InputDialogState;
use crate::tui::theme::THEME;

/// Widget for rendering the input dialog
pub struct InputDialogWidget<'a> {
    /// Reference to dialog state
    pub(crate) state: &'a InputDialogState,
    /// Dialog width as percentage (0-100)
    pub(crate) width_percent: u16,
    /// Dialog height in lines
    pub(crate) height: u16,
}

impl<'a> InputDialogWidget<'a> {
    /// Create a new input dialog widget
    pub fn new(state: &'a InputDialogState) -> Self {
        Self {
            state,
            width_percent: 60,
            height: 11,
        }
    }

    /// Set dialog width as percentage
    pub fn with_width(mut self, percent: u16) -> Self {
        self.width_percent = percent.min(100);
        self
    }

    /// Set dialog height
    pub fn with_height(mut self, height: u16) -> Self {
        self.height = height.max(5);
        self
    }

    /// Calculate the centered dialog area
    fn calculate_dialog_area(&self, frame_area: Rect) -> Rect {
        let dialog_width = (frame_area.width * self.width_percent / 100).max(40);
        let dialog_height = self.height;

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
            .title(self.state.dialog_type.title())
            .title_style(THEME.dialog_title())
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(THEME.dialog_border());

        frame.render_widget(title_block, dialog_area);

        // Create layout for dialog content
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(2), // Instruction
                Constraint::Length(3), // Input field
                Constraint::Length(1), // Hint
                Constraint::Length(1), // Error (if any)
                Constraint::Length(1), // Actions
            ])
            .split(dialog_area);

        // Render instruction
        let instruction_text = match self.state.dialog_type {
            super::DialogType::Claim => "Enter Validator ID to claim rewards",
            super::DialogType::Compound => "Enter Validator ID to compound rewards",
            _ => self.state.dialog_type.hint(),
        };
        let instruction = Paragraph::new(Line::from(vec![Span::styled(
            instruction_text,
            THEME.metric_label(),
        )]))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });
        frame.render_widget(instruction, chunks[0]);

        // Render input field with Block and Rounded border
        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(THEME.input_border_focused())
            .title("Input")
            .title_style(THEME.dialog_title());

        let input_inner = input_block.inner(chunks[1]);
        frame.render_widget(input_block, chunks[1]);

        // Render the textarea content manually due to Widget trait incompatibility
        let text_content = self
            .state
            .input
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("");
        let input_paragraph = if text_content.is_empty() {
            Paragraph::new(self.state.dialog_type.placeholder()).style(THEME.placeholder())
        } else {
            Paragraph::new(text_content).style(THEME.input_text())
        };
        frame.render_widget(input_paragraph, input_inner);

        // Render hint
        let hint_text = self
            .state
            .context_hint
            .as_deref()
            .unwrap_or_else(|| self.state.dialog_type.hint());

        let hint = Paragraph::new(Line::from(vec![Span::styled(hint_text, THEME.label())]))
            .alignment(Alignment::Center);
        frame.render_widget(hint, chunks[2]);

        // Render error if present
        if let Some(ref error) = self.state.error {
            let error_line = Line::styled(format!(" Error: {}", error), THEME.input_error());
            let error_para = Paragraph::new(error_line);
            frame.render_widget(error_para, chunks[3]);
        }

        // Render action hints with proper theme styles
        let actions = Line::from(vec![
            Span::styled("[Enter]", THEME.keybind()),
            Span::styled(" Confirm  ", THEME.keybind_description()),
            Span::styled("[Esc]", THEME.keybind()),
            Span::styled(" Cancel", THEME.keybind_description()),
        ]);
        let actions_para = Paragraph::new(actions).alignment(Alignment::Center);
        frame.render_widget(actions_para, chunks[4]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_input_dialog_widget_calculate_dialog_area() {
        let state = InputDialogState::new();
        let widget = InputDialogWidget::new(&state).with_width(50).with_height(7);

        let frame_area = Rect::new(0, 0, 100, 30);
        let dialog_area = widget.calculate_dialog_area(frame_area);

        // Dialog should be centered
        assert_eq!(dialog_area.width, 50);
        assert_eq!(dialog_area.height, 7);
    }

    #[test]
    fn test_input_dialog_widget_calculate_dialog_area_small_frame() {
        let state = InputDialogState::new();
        let widget = InputDialogWidget::new(&state).with_width(80).with_height(7);

        let frame_area = Rect::new(0, 0, 60, 20);
        let dialog_area = widget.calculate_dialog_area(frame_area);

        // Should use minimum width of 40
        assert!(dialog_area.width >= 40);
    }

    #[test]
    fn test_input_dialog_widget_builder_chain() {
        let state = InputDialogState::new();
        let widget = InputDialogWidget::new(&state).with_width(70).with_height(9);

        assert_eq!(widget.width_percent, 70);
        assert_eq!(widget.height, 9);
    }
}
