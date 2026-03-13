//! Query Validator Dialog Widget rendering
//!
//! This module provides [`QueryValidatorDialogWidget`] for rendering the
//! Query Validator modal dialog in the TUI.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::tui::staking::query_validator_state::QueryValidatorState;
use crate::tui::theme::THEME;

/// Widget for rendering the Query Validator dialog
pub struct QueryValidatorDialogWidget<'a> {
    /// Reference to dialog state
    state: &'a QueryValidatorState,
    /// Dialog width as percentage (0-100)
    width_percent: u16,
}

impl<'a> QueryValidatorDialogWidget<'a> {
    /// Create a new Query Validator dialog widget
    pub fn new(state: &'a QueryValidatorState) -> Self {
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
        let dialog_width = (frame_area.width * self.width_percent / 100).clamp(50, 80);
        // Dynamic height based on whether we have results
        let dialog_height = if self.state.result.is_some() {
            18 // More space for results
        } else {
            11 // Standard input dialog height
        };

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
            .title(" Query Validator ")
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(THEME.dialog_border())
            .title_style(THEME.dialog_title());

        frame.render_widget(title_block, dialog_area);

        // Calculate constraints based on whether we have results
        let has_result = self.state.result.is_some();
        let constraints = if has_result {
            vec![
                Constraint::Length(2), // Instruction
                Constraint::Length(3), // Validator ID field
                Constraint::Length(1), // Error (if any)
                Constraint::Min(6),    // Results (expandable)
                Constraint::Length(1), // Actions
            ]
        } else {
            vec![
                Constraint::Length(2), // Instruction
                Constraint::Length(3), // Validator ID field
                Constraint::Length(1), // Error (if any)
                Constraint::Length(1), // Actions
            ]
        };

        // Create layout for dialog content
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(constraints.as_slice())
            .split(dialog_area);

        // Render instruction
        let instruction = Paragraph::new(Line::from(vec![Span::styled(
            "Enter Validator ID to view validator information",
            THEME.metric_label(),
        )]))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });

        frame.render_widget(instruction, chunks[0]);

        // Render validator ID input field
        self.render_input_field(frame, chunks[1]);

        // Render error if present
        if let Some(ref error) = self.state.error {
            let error_line = Line::styled(format!(" Error: {}", error), THEME.status_error());
            let error_para = Paragraph::new(error_line);
            frame.render_widget(error_para, chunks[2]);
        }

        // Render results if present
        if has_result {
            if let Some(ref result) = self.state.result {
                self.render_result(frame, chunks[3], result);
            }
        }

        // Render action hints
        let actions = if self.state.is_querying {
            Line::from(vec![Span::styled("Querying...", THEME.status_warning())])
        } else {
            Line::from(vec![
                Span::styled("[Enter]", THEME.keybind()),
                Span::styled(" Query  ", THEME.keybind_description()),
                Span::styled("[Esc]", THEME.keybind()),
                Span::styled(" Cancel", THEME.keybind_description()),
            ])
        };

        let actions_para = Paragraph::new(actions).alignment(Alignment::Center);
        let actions_chunk = if has_result { chunks[4] } else { chunks[3] };
        frame.render_widget(actions_para, actions_chunk);
    }

    /// Render the validator ID input field
    fn render_input_field(&self, frame: &mut Frame, area: Rect) {
        let value = self.state.get_validator_id();
        let display_value = if value.is_empty() {
            "e.g., 224".to_string()
        } else {
            value.clone()
        };

        // Create the input block with rounded border
        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(THEME.input_border_focused())
            .title("Validator ID")
            .title_style(THEME.dialog_title());

        let inner_area = input_block.inner(area);
        frame.render_widget(input_block, area);

        // Render the input text
        let input_style = if value.is_empty() {
            THEME.placeholder()
        } else {
            THEME.input_text()
        };

        let input_line = Line::from(vec![Span::styled(display_value, input_style)]);
        let input_para = Paragraph::new(input_line);
        frame.render_widget(input_para, inner_area);
    }

    /// Render the query result
    fn render_result(
        &self,
        frame: &mut Frame,
        area: Rect,
        result: &crate::tui::staking::query_validator_state::QueryValidatorResult,
    ) {
        use crate::tui::staking::query_validator_state::QueryValidatorResult;

        let lines = match result {
            QueryValidatorResult::Success(validator) => {
                vec![
                    Line::from(vec![Span::styled(
                        "Validator Information",
                        THEME.dialog_title().bold(),
                    )]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("Auth Address: ", THEME.label()),
                        Span::styled(&validator.auth_delegator, THEME.metric_value()),
                    ]),
                    Line::from(vec![
                        Span::styled("Flags: ", THEME.label()),
                        Span::styled(validator.flags.to_string(), THEME.metric_value()),
                    ]),
                    Line::from(""),
                    Line::from(vec![Span::styled("Execution View:", THEME.dialog_title())]),
                    Line::from(vec![
                        Span::styled("  Stake: ", THEME.label()),
                        Span::styled(
                            format!("{} MON", validator.execution_stake as f64 / 1e18),
                            THEME.metric_value(),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("  Commission: ", THEME.label()),
                        Span::styled(
                            format!("{:.2}%", validator.commission()),
                            THEME.metric_value(),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("  Unclaimed Rewards: ", THEME.label()),
                        Span::styled(
                            format!("{} MON", validator.unclaimed_rewards as f64 / 1e18),
                            THEME.rewards(),
                        ),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("Consensus Stake: ", THEME.label()),
                        Span::styled(
                            format!("{} MON", validator.consensus_stake as f64 / 1e18),
                            THEME.metric_value(),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("Snapshot Stake: ", THEME.label()),
                        Span::styled(
                            format!("{} MON", validator.snapshot_stake as f64 / 1e18),
                            THEME.metric_value(),
                        ),
                    ]),
                ]
            }
            QueryValidatorResult::Error(msg) => {
                vec![
                    Line::from(vec![Span::styled(
                        "Query Failed",
                        THEME.status_error().bold(),
                    )]),
                    Line::from(""),
                    Line::from(vec![Span::styled(msg, THEME.status_error())]),
                ]
            }
        };

        let result_para = Paragraph::new(lines).wrap(Wrap { trim: false });
        frame.render_widget(result_para, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_validator_dialog_widget_creation() {
        let state = QueryValidatorState::new();
        let widget = QueryValidatorDialogWidget::new(&state);
        assert_eq!(widget.width_percent, 60);
    }

    #[test]
    fn test_query_validator_dialog_widget_with_width() {
        let state = QueryValidatorState::new();
        let widget = QueryValidatorDialogWidget::new(&state).with_width(80);
        assert_eq!(widget.width_percent, 80);
    }

    #[test]
    fn test_query_validator_dialog_widget_calculate_dialog_area() {
        let state = QueryValidatorState::new();
        let widget = QueryValidatorDialogWidget::new(&state).with_width(60);

        let frame_area = Rect::new(0, 0, 100, 30);
        let dialog_area = widget.calculate_dialog_area(frame_area);

        // Dialog should be centered and 60% width
        assert_eq!(dialog_area.width, 60);
    }
}
