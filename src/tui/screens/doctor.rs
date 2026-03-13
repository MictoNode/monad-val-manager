//! Doctor Screen - Diagnostics and health checks display
//!
//! This screen provides diagnostic tools:
//! - System health checks
//! - Network connectivity tests
//! - Configuration validation
//! - Performance diagnostics

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::{Screen, ScreenRender};
use crate::tui::state::AppState;
use crate::tui::theme::THEME;
use crate::tui::widgets::{ConsensusWidget, DoctorResultsWidget, NavMenuWidget};

/// Doctor screen for diagnostics
pub struct DoctorScreen;

impl DoctorScreen {
    /// Create new doctor screen
    pub fn new() -> Self {
        Self
    }
}

impl Default for DoctorScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenRender for DoctorScreen {
    fn render(&self, frame: &mut Frame, state: &AppState) {
        let area = frame.area();

        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Nav Menu (with branding)
                Constraint::Length(4), // Consensus (Epoch/Round)
                Constraint::Min(10),   // Main content
                Constraint::Length(6), // Detail panel
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Navigation Menu
        let nav_menu = NavMenuWidget::new(Screen::Doctor, state.tick);
        nav_menu.render(frame, chunks[0]);

        // Consensus info (Epoch/Round)
        ConsensusWidget::new(&state.consensus).render(frame, chunks[1]);

        // Main content - diagnostic checks list
        let doctor_widget = DoctorResultsWidget::new(&state.doctor);
        if state.doctor.is_running {
            doctor_widget.with_tick(state.tick).render(frame, chunks[2]);
        } else {
            doctor_widget.render(frame, chunks[2]);
        }

        // Detail panel for selected check
        DoctorResultsWidget::new(&state.doctor).render_detail_panel(frame, chunks[3]);

        // Footer with controls and summary
        self.render_footer(frame, chunks[4], state);
    }
}

impl DoctorScreen {
    /// Render the footer with controls and summary
    fn render_footer(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let doctor = &state.doctor;

        // Build summary text
        #[cfg(target_os = "linux")]
        let total_checks = 16;
        #[cfg(not(target_os = "linux"))]
        let total_checks = 15;

        let summary = if doctor.is_running {
            "Running diagnostics...".to_string()
        } else if doctor.passed_count == 0 && doctor.failed_count == 0 {
            "Press [r] to run diagnostics".to_string()
        } else {
            format!(
                "Passed: {} | Failed: {} | Total: {}",
                doctor.passed_count, doctor.failed_count, total_checks,
            )
        };

        let summary_style = if doctor.is_running {
            THEME.status_warning()
        } else if doctor.passed_count == 0 && doctor.failed_count == 0 {
            THEME.label()
        } else {
            THEME.muted()
        };

        let content = Line::from(vec![
            Span::styled("[r]", THEME.keybind()),
            Span::styled(" Run ", THEME.keybind_description()),
            Span::styled("[↑/↓]", THEME.keybind()),
            Span::styled(" Navigate ", THEME.keybind_description()),
            Span::styled("[Esc]", THEME.keybind()),
            Span::styled(" Back ", THEME.keybind_description()),
            Span::raw("  "),
            Span::styled(summary, summary_style),
        ]);

        let footer = Paragraph::new(content).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(THEME.widget_border()),
        );

        frame.render_widget(footer, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doctor_screen_creation() {
        let screen = DoctorScreen::new();
        let _ = &screen;
    }

    #[test]
    fn test_doctor_screen_default() {
        let screen = DoctorScreen;
        let _ = &screen;
    }

    #[test]
    fn test_doctor_screen_with_state() {
        let _screen = DoctorScreen::new();
        let state = AppState::new();

        // Verify doctor state is initialized
        assert!(!state.doctor.checks.is_empty());
        assert_eq!(state.doctor.selected_index, 0);
    }

    #[test]
    fn test_doctor_state_in_app_state() {
        let state = AppState::new();

        // Verify doctor state exists
        // Linux: 16 checks (with MPT Storage), Other: 15 checks (without MPT Storage, removed Node Uptime)
        #[cfg(target_os = "linux")]
        assert_eq!(state.doctor.checks.len(), 16);
        #[cfg(not(target_os = "linux"))]
        assert_eq!(state.doctor.checks.len(), 15);

        // All checks should be pending initially
        for check in &state.doctor.checks {
            assert!(format!("{:?}", check.status).contains("Pending"));
        }
    }

    #[test]
    fn test_app_state_with_doctor() {
        let mut state = AppState::new();

        // Simulate running checks
        state.doctor.start_checks();
        assert!(state.doctor.is_running);

        // Simulate completing checks
        state.doctor.update_check(
            "CPU Cores",
            crate::tui::doctor_state::CheckStatus::Pass,
            "16 cores",
            None,
        );
        state.doctor.finish_checks();

        assert!(!state.doctor.is_running);
        assert!(state.doctor.last_run.is_some());
    }
}
