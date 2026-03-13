//! Doctor Results Widget - Display diagnostic check results
//!
//! This widget renders the list of diagnostic checks with their status,
//! organized by category with visual indicators for pass/fail states.

use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, ListItem, Paragraph},
    Frame,
};

use crate::tui::doctor_state::{CheckCategory, CheckStatus, DoctorCheck, DoctorState};
use crate::tui::theme::THEME;
use crate::tui::{GlowEffect, GlowWidgets};

/// Spinner characters for loading animation
const SPINNER_CHARS: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

/// Doctor results widget for displaying diagnostic check results
#[derive(Debug)]
pub struct DoctorResultsWidget<'a> {
    state: &'a DoctorState,
    tick: Option<u64>,
}

impl<'a> DoctorResultsWidget<'a> {
    /// Create new doctor results widget
    pub fn new(state: &'a DoctorState) -> Self {
        Self { state, tick: None }
    }

    /// Set the tick for loading animation
    pub fn with_tick(mut self, tick: u64) -> Self {
        self.tick = Some(tick);
        self
    }

    /// Render the doctor results widget
    pub fn render(self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .state
            .checks
            .iter()
            .enumerate()
            .map(|(idx, check)| {
                let is_selected = idx == self.state.selected_index;
                self.render_check_item(check, is_selected)
            })
            .collect();

        // Build title with optional spinner when running
        let title = if self.state.is_running {
            if let Some(tick) = self.tick {
                let spinner_idx = (tick % SPINNER_CHARS.len() as u64) as usize;
                let spinner_char = SPINNER_CHARS[spinner_idx];
                format!(" {} Diagnostics ", spinner_char)
            } else {
                " Diagnostics ".to_string()
            }
        } else {
            " Diagnostics ".to_string()
        };

        // Use glowing border for active diagnostics panel
        let has_failures = self.state.failed_count > 0;
        let list = ratatui::widgets::List::new(items).block(
            Block::default()
                .title(title)
                .title_style(THEME.widget_title())
                .borders(Borders::ALL)
                .border_style(if has_failures {
                    GlowEffect::error().get_border_style()
                } else {
                    GlowWidgets::card_border(true)
                }),
        );

        frame.render_widget(list, area);
    }

    /// Render a single check item
    fn render_check_item(&self, check: &DoctorCheck, is_selected: bool) -> ListItem<'_> {
        let (status_symbol, status_style) = THEME.check_status(check.status);

        // Build the line with status symbol and check name
        let name_style = if is_selected {
            THEME.selected_bold()
        } else {
            ratatui::style::Style::default().fg(THEME.text_primary)
        };

        let mut spans = vec![
            Span::styled(format!("{} ", status_symbol), status_style),
            Span::styled(check.name.clone(), name_style),
        ];

        // Add message if check is complete
        if check.status.is_complete() {
            spans.push(Span::styled(format!(" - {}", check.message), THEME.muted()));
        } else if check.status == CheckStatus::Running {
            spans.push(Span::styled(
                " - Checking...".to_string(),
                THEME.status_warning(),
            ));
        }

        ListItem::new(Line::from(spans))
    }

    /// Render selected check details panel
    pub fn render_detail_panel(&self, frame: &mut Frame, area: Rect) {
        if let Some(check) = self.state.selected_check() {
            let category_color = Self::category_style(check.category);

            let mut lines = vec![
                Line::from(vec![
                    Span::styled("Check: ", THEME.label()),
                    Span::styled(check.name.clone(), THEME.text()),
                ]),
                Line::from(vec![
                    Span::styled("Category: ", THEME.label()),
                    Span::styled(check.category.display_name(), category_color),
                ]),
                Line::from(vec![
                    Span::styled("Status: ", THEME.label()),
                    Span::styled(
                        THEME.check_status(check.status).0,
                        THEME.check_status(check.status).1,
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Message: ", THEME.label()),
                    Span::styled(check.message.clone(), THEME.text()),
                ]),
            ];

            // Add fix hint if available
            if let Some(ref hint) = check.fix_hint {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("Suggested Fix: ", THEME.status_warning()),
                    Span::styled(hint.clone(), THEME.status_success()),
                ]));
            }

            let paragraph = Paragraph::new(lines).block(
                Block::default()
                    .title(" Details ")
                    .title_style(THEME.widget_title())
                    .borders(Borders::ALL)
                    .border_style(THEME.widget_border()),
            );

            frame.render_widget(paragraph, area);
        }
    }

    /// Get category style
    fn category_style(category: CheckCategory) -> ratatui::style::Style {
        match category {
            CheckCategory::System => THEME.metric_value(),
            CheckCategory::Network => THEME.status_info(),
            CheckCategory::Service => THEME.status_warning(),
            CheckCategory::Config => THEME.rewards(),
            CheckCategory::Consensus => THEME.status_success(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_state() -> DoctorState {
        let mut state = DoctorState::new();
        state.update_check("CPU Cores", CheckStatus::Pass, "16 cores", None);
        state.update_check(
            "Memory (RAM)",
            CheckStatus::Fail,
            "8GB (need 32GB)",
            Some("Upgrade RAM"),
        );
        state.update_check("RPC Connection", CheckStatus::Running, "Checking...", None);
        state.recalculate_summary();
        state
    }

    #[test]
    fn test_widget_creation() {
        let state = DoctorState::new();
        let widget = DoctorResultsWidget::new(&state);
        assert!(format!("{:?}", widget).contains("DoctorResultsWidget"));
    }

    #[test]
    fn test_widget_with_state() {
        let state = create_test_state();
        let widget = DoctorResultsWidget::new(&state);

        // Linux: 16 checks, Other: 15 checks (removed Node Uptime)
        #[cfg(target_os = "linux")]
        assert_eq!(widget.state.checks.len(), 16);
        #[cfg(not(target_os = "linux"))]
        assert_eq!(widget.state.checks.len(), 15);
        assert_eq!(widget.state.passed_count, 1);
        assert_eq!(widget.state.failed_count, 1);
    }

    #[test]
    fn test_selected_check_access() {
        let mut state = DoctorState::new();
        state.update_check("CPU Cores", CheckStatus::Pass, "16 cores", None);

        let widget = DoctorResultsWidget::new(&state);
        let selected = widget.state.selected_check();

        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "CPU Cores");
    }

    #[test]
    fn test_navigation_selection() {
        let mut state = DoctorState::new();
        state.select_next();

        let widget = DoctorResultsWidget::new(&state);
        let selected = widget.state.selected_check();

        assert!(selected.is_some());
        // Second check should be selected
        assert_eq!(widget.state.selected_index, 1);
    }
}
