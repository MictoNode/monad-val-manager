//! Transfer Dialog Widget - Modal dialog for native MON transfers
//!
//! This widget provides the UI for the transfer dialog with:
//! - Address input step
//! - Amount input step
//! - Confirmation step
//! - Processing step
//! - Complete step

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::*,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use ratatui_textarea::TextArea;

use crate::tui::theme::THEME;
use crate::tui::transfer_state::{TransferDialogState, TransferStep};

/// Wrapper to make TextArea compatible with ratatui 0.30 Widget trait
struct TextAreaWrapper<'a> {
    textarea: &'a TextArea<'a>,
}

impl<'a> TextAreaWrapper<'a> {
    fn new(textarea: &'a TextArea<'a>) -> Self {
        Self { textarea }
    }
}

impl<'a> Widget for TextAreaWrapper<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        <&TextArea as ratatui::widgets::Widget>::render(self.textarea, area, buf);
    }
}

/// Widget for rendering the transfer dialog
pub struct TransferDialogWidget<'a> {
    /// Reference to transfer dialog state
    pub(crate) state: &'a TransferDialogState,
}

impl<'a> TransferDialogWidget<'a> {
    /// Create a new transfer dialog widget
    pub fn new(state: &'a TransferDialogState) -> Self {
        Self { state }
    }

    /// Calculate the centered dialog area
    fn calculate_dialog_area(&self, frame_area: Rect) -> Rect {
        let dialog_width = (frame_area.width * 60 / 100).clamp(50, 70);
        let dialog_height = match self.state.step {
            TransferStep::Address => 14,
            TransferStep::Amount => 14,
            TransferStep::Confirm => 16,
            TransferStep::Processing => 10,
            TransferStep::Complete => 12,
        };

        let x = (frame_area.width.saturating_sub(dialog_width)) / 2;
        let y = (frame_area.height.saturating_sub(dialog_height)) / 2;

        Rect::new(x, y, dialog_width, dialog_height)
    }

    /// Get title for current step
    fn step_title(&self) -> &'static str {
        match self.state.step {
            TransferStep::Address => " Transfer - Recipient Address ",
            TransferStep::Amount => " Transfer - Amount ",
            TransferStep::Confirm => " Transfer - Confirm ",
            TransferStep::Processing => " Transfer - Processing ",
            TransferStep::Complete => " Transfer - Complete ",
        }
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

        match self.state.step {
            TransferStep::Address => self.render_address_step(frame, dialog_area),
            TransferStep::Amount => self.render_amount_step(frame, dialog_area),
            TransferStep::Confirm => self.render_confirm_step(frame, dialog_area),
            TransferStep::Processing => self.render_processing_step(frame, dialog_area),
            TransferStep::Complete => self.render_complete_step(frame, dialog_area),
        }
    }

    /// Render address input step
    fn render_address_step(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(2), // Instructions
                Constraint::Length(3), // Input field with Block border
                Constraint::Length(2), // Error (if any)
                Constraint::Length(2), // Hint
                Constraint::Length(1), // Actions
            ])
            .split(area);

        // Render outer border with title and Double border type
        let title_block = Block::default()
            .title(self.step_title())
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(THEME.dialog_border())
            .title_style(THEME.dialog_title());

        frame.render_widget(title_block, area);

        // Instructions
        let instructions = Paragraph::new(Line::from(vec![Span::styled(
            "Enter recipient address (42 hex characters including 0x prefix)",
            THEME.metric_label(),
        )]))
        .alignment(Alignment::Center);

        frame.render_widget(instructions, chunks[0]);

        // Input field with Block border (like staking dialogs)
        let border_style = if self.state.error.is_some() {
            THEME.status_error()
        } else {
            THEME.input_border_focused() // Cyan for focused
        };

        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .title(" Address ")
            .title_style(THEME.dialog_title());

        let inner_area = input_block.inner(chunks[1]);
        frame.render_widget(input_block, chunks[1]);

        // Render the textarea using wrapper for compatibility
        frame.render_widget(TextAreaWrapper::new(&self.state.address), inner_area);

        // Error message (if any)
        if let Some(ref error) = self.state.error {
            let error_line = Line::styled(format!(" Error: {}", error), THEME.status_error());
            let error_para = Paragraph::new(error_line);
            frame.render_widget(error_para, chunks[2]);
        }

        // Hint
        let hint_text = format!("Characters: {} / 42", self.state.address_len());
        let hint = Line::styled(hint_text, THEME.muted());
        let hint_para = Paragraph::new(hint).alignment(Alignment::Center);
        frame.render_widget(hint_para, chunks[3]);

        // Action hints (matching staking dialog style)
        let actions = Line::from(vec![
            Span::styled("[Enter]", THEME.keybind()),
            Span::styled(" Next  ", THEME.keybind_description()),
            Span::styled("[Esc]", THEME.keybind()),
            Span::styled(" Cancel", THEME.keybind_description()),
        ]);
        let actions_para = Paragraph::new(actions).alignment(Alignment::Center);
        frame.render_widget(actions_para, chunks[4]);
    }

    /// Render amount input step
    fn render_amount_step(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(2), // Instructions
                Constraint::Length(3), // Input field with Block border
                Constraint::Length(2), // Error (if any)
                Constraint::Length(2), // Hint
                Constraint::Length(1), // Actions
            ])
            .split(area);

        // Render outer border with title and Double border type
        let title_block = Block::default()
            .title(self.step_title())
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(THEME.dialog_border())
            .title_style(THEME.dialog_title());

        frame.render_widget(title_block, area);

        // Instructions
        let instructions = Paragraph::new(Line::from(vec![Span::styled(
            "Enter amount to transfer (in MON)",
            THEME.metric_label(),
        )]))
        .alignment(Alignment::Center);

        frame.render_widget(instructions, chunks[0]);

        // Input field with Block border (like staking dialogs)
        let border_style = if self.state.error.is_some() {
            THEME.status_error()
        } else {
            THEME.input_border_focused() // Cyan for focused
        };

        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .title(" Amount (MON) ")
            .title_style(THEME.dialog_title());

        let inner_area = input_block.inner(chunks[1]);
        frame.render_widget(input_block, chunks[1]);

        // Render the textarea using wrapper for compatibility
        frame.render_widget(TextAreaWrapper::new(&self.state.amount), inner_area);

        // Error message (if any)
        if let Some(ref error) = self.state.error {
            let error_line = Line::styled(format!(" Error: {}", error), THEME.status_error());
            let error_para = Paragraph::new(error_line);
            frame.render_widget(error_para, chunks[2]);
        }

        // Hint with balance
        let balance_text = if let Some(ref balance) = self.state.available_balance {
            format!("Available: {}", balance)
        } else {
            "Enter amount to transfer".to_string()
        };

        let hint = Line::styled(balance_text, THEME.muted());
        let hint_para = Paragraph::new(hint).alignment(Alignment::Center);
        frame.render_widget(hint_para, chunks[3]);

        // Action hints (matching staking dialog style)
        let actions = Line::from(vec![
            Span::styled("[Enter]", THEME.keybind()),
            Span::styled(" Next  ", THEME.keybind_description()),
            Span::styled("[Esc]", THEME.keybind()),
            Span::styled(" Back", THEME.keybind_description()),
        ]);
        let actions_para = Paragraph::new(actions).alignment(Alignment::Center);
        frame.render_widget(actions_para, chunks[4]);
    }

    /// Render confirmation step
    fn render_confirm_step(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([
                Constraint::Length(3), // Title (handled by block)
                Constraint::Length(1), // Spacer
                Constraint::Length(2), // Address label
                Constraint::Length(2), // Address value
                Constraint::Length(2), // Amount label
                Constraint::Length(2), // Amount value
                Constraint::Length(1), // Spacer
                Constraint::Length(2), // Actions
            ])
            .split(area);

        // Render border with title and Double border type
        let title_block = Block::default()
            .title(self.step_title())
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(THEME.dialog_border())
            .title_style(THEME.dialog_title());

        frame.render_widget(title_block, area);

        // Address label
        let addr_label = Line::styled("  Recipient Address:", THEME.metric_label());
        let addr_label_para = Paragraph::new(addr_label);
        frame.render_widget(addr_label_para, chunks[2]);

        // Address value
        let addr_value = Line::styled(
            format!("  {}", self.state.formatted_address()),
            THEME.metric_value(),
        );
        let addr_value_para = Paragraph::new(addr_value);
        frame.render_widget(addr_value_para, chunks[3]);

        // Amount label
        let amount_label = Line::styled("  Amount:", THEME.metric_label());
        let amount_label_para = Paragraph::new(amount_label);
        frame.render_widget(amount_label_para, chunks[4]);

        // Amount value
        let amount_value = Line::styled(
            format!("  {} MON", self.state.get_amount_str()),
            THEME.amount_positive(),
        );
        let amount_value_para = Paragraph::new(amount_value);
        frame.render_widget(amount_value_para, chunks[5]);

        // Action hints (matching staking dialog style)
        let actions = Line::from(vec![
            Span::styled("[Enter]", THEME.keybind()),
            Span::styled(" Confirm Transfer  ", THEME.keybind_description()),
            Span::styled("[Esc]", THEME.keybind()),
            Span::styled(" Cancel", THEME.keybind_description()),
        ]);
        let actions_para = Paragraph::new(actions).alignment(Alignment::Center);
        frame.render_widget(actions_para, chunks[7]);
    }

    /// Render processing step
    fn render_processing_step(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([
                Constraint::Length(3), // Title (handled by block)
                Constraint::Length(1), // Spacer
                Constraint::Length(2), // Status text
                Constraint::Length(1), // Spacer
            ])
            .split(area);

        // Render border with title and Double border type
        let title_block = Block::default()
            .title(self.step_title())
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(THEME.dialog_border())
            .title_style(THEME.dialog_title());

        frame.render_widget(title_block, area);

        // Status text
        let status_line = Line::styled("  Broadcasting transaction...", THEME.status_info());
        let status_para = Paragraph::new(status_line).alignment(Alignment::Center);
        frame.render_widget(status_para, chunks[2]);
    }

    /// Render complete step
    fn render_complete_step(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([
                Constraint::Length(3), // Title (handled by block)
                Constraint::Length(1), // Spacer
                Constraint::Length(2), // Success text
                Constraint::Length(2), // TX hash (if available)
                Constraint::Length(1), // Spacer
                Constraint::Length(2), // Actions
            ])
            .split(area);

        // Render border with title and Double border type
        let title_block = Block::default()
            .title(self.step_title())
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(THEME.dialog_border())
            .title_style(THEME.dialog_title());

        frame.render_widget(title_block, area);

        // Success text
        let success_line = Line::styled("  Transfer successful!", THEME.status_success());
        let success_para = Paragraph::new(success_line).alignment(Alignment::Center);
        frame.render_widget(success_para, chunks[2]);

        // TX hash (if available)
        if let Some(ref tx_hash) = self.state.tx_hash {
            let tx_line = Line::styled(format!("  TX: {}", tx_hash), THEME.metric_value());
            let tx_para = Paragraph::new(tx_line)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });
            frame.render_widget(tx_para, chunks[3]);
        } else {
            let no_tx_line = Line::styled("  Transaction hash not available", THEME.muted());
            let no_tx_para = Paragraph::new(no_tx_line).alignment(Alignment::Center);
            frame.render_widget(no_tx_para, chunks[3]);
        }

        // Action hints (matching staking dialog style)
        let actions = Line::from(vec![
            Span::styled("[Enter]", THEME.keybind()),
            Span::styled(" Close ", THEME.keybind_description()),
            Span::styled("[Esc]", THEME.keybind()),
            Span::styled(" Close", THEME.keybind_description()),
        ]);
        let actions_para = Paragraph::new(actions).alignment(Alignment::Center);
        frame.render_widget(actions_para, chunks[5]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::transfer_state::TransferDialogState;

    #[test]
    fn test_transfer_dialog_widget_creation() {
        let state = TransferDialogState::new();
        let widget = TransferDialogWidget::new(&state);
        assert!(widget.state.is_address_empty());
    }

    #[test]
    fn test_step_title() {
        let state = TransferDialogState::new();
        let widget = TransferDialogWidget::new(&state);

        assert_eq!(widget.step_title(), " Transfer - Recipient Address ");

        let mut state = TransferDialogState::new();
        state.step = TransferStep::Amount;
        let widget = TransferDialogWidget::new(&state);
        assert_eq!(widget.step_title(), " Transfer - Amount ");

        state.step = TransferStep::Confirm;
        let widget = TransferDialogWidget::new(&state);
        assert_eq!(widget.step_title(), " Transfer - Confirm ");
    }

    #[test]
    fn test_calculate_dialog_area() {
        let state = TransferDialogState::new();
        let widget = TransferDialogWidget::new(&state);

        let frame_area = Rect::new(0, 0, 100, 30);
        let dialog_area = widget.calculate_dialog_area(frame_area);

        // Dialog should be centered and have reasonable dimensions
        assert!(dialog_area.width >= 50);
        assert!(dialog_area.width <= 70);
        assert!(dialog_area.height >= 8);
        assert!(dialog_area.height <= 14);
    }
}
