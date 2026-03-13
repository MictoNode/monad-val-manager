//! Dashboard Screen - Main monitoring dashboard
//!
//! This is the default screen showing system metrics, network status,
//! and validator information using the existing widgets.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::{Screen, ScreenRender};
use crate::tui::state::AppState;
use crate::tui::theme::THEME;
use crate::tui::widgets::{
    ConsensusWidget, NavMenuWidget, NetworkWidget, PeerHealth, PeerHealthWidget, SystemWidget,
    ValidatorWidget,
};

/// Spinner characters for loading animation
const SPINNER_CHARS: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

/// Dashboard screen showing system, network, and validator widgets
pub struct DashboardScreen;

impl DashboardScreen {
    /// Create new dashboard screen
    pub fn new() -> Self {
        Self
    }
}

impl Default for DashboardScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenRender for DashboardScreen {
    fn render(&self, frame: &mut Frame, state: &AppState) {
        let area = frame.area();

        // Create main vertical layout
        // [0] NavMenuWidget    → Length(3)
        // [1] Main content     → Min(10)
        // [2] Footer           → Length(3)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // [0] Nav Menu (with branding)
                Constraint::Min(10),   // [1] Main content area
                Constraint::Length(3), // [2] Footer
            ])
            .split(area);

        // [0] Navigation Menu
        let nav_menu = NavMenuWidget::new(Screen::Dashboard, state.tick);
        nav_menu.render(frame, chunks[0]);

        // [1] Main content area - split into 3 ROWS vertically
        // ROW 1 → Length(7): 2 columns (NetworkWidget | ConsensusWidget)
        // ROW 2 → Length(7): Full width (SystemWidget)
        // ROW 3 → Min(5): 2 columns (ValidatorWidget | PeerHealthWidget)
        let main_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7), // ROW 1: Network | Consensus
                Constraint::Length(7), // ROW 2: System (full width)
                Constraint::Min(5),    // ROW 3: Validator | PeerHealth
            ])
            .split(chunks[1]);

        // === ROW 1: NetworkWidget (50%) | ConsensusWidget (50%) ===
        let row1_columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_rows[0]);

        NetworkWidget::new(&state.network, state.tick).render(frame, row1_columns[0]);
        ConsensusWidget::new(&state.consensus).render(frame, row1_columns[1]);

        // === ROW 2: SystemWidget (full width) ===
        SystemWidget::new(&state.system).render(frame, main_rows[1]);

        // === ROW 3: ValidatorWidget (50%) | PeerHealthWidget (50%) ===
        let row3_columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_rows[2]);

        ValidatorWidget::new(&state.validator).render(frame, row3_columns[0]);

        let peer_health = PeerHealth {
            active_peers: state.network.peer_count.saturating_sub(2) as usize,
            timeout_peers: if state.network.peer_count > 0 {
                2usize
            } else {
                0usize
            },
            total_peers: state.network.peer_count as usize,
        };
        PeerHealthWidget::new(&peer_health).render(frame, row3_columns[1]);

        // [2] Footer
        self.render_footer(frame, chunks[2], state);
    }
}

impl DashboardScreen {
    /// Render the footer with controls
    fn render_footer(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let seconds_ago = state
            .time_since_refresh()
            .map(|d| d.as_secs())
            .unwrap_or(u64::MAX);

        // Animated refresh dot: ● (#85E6FF) when tick % 2 == 0, ○ (muted) when tick % 2 == 1
        let refresh_dot = if state.tick.is_multiple_of(2) {
            Span::styled("●", Style::default().fg(Color::Rgb(133, 230, 255))) // Cyan #85E6FF
        } else {
            Span::styled("○", THEME.muted())
        };

        // Build the footer content
        let mut content_spans = vec![
            Span::styled("[q]", THEME.keybind()),
            Span::styled(" Quit ", THEME.keybind_description()),
            Span::styled("[r]", THEME.keybind()),
            Span::styled(" Refresh ", THEME.keybind_description()),
            Span::styled("[Tab]", THEME.keybind()),
            Span::styled(" Next Screen ", THEME.keybind_description()),
            Span::styled("[h]", THEME.keybind()),
            Span::styled(" Help ", THEME.keybind_description()),
            Span::raw("  "),
        ];

        // Add loading spinner or refresh info
        if state.is_loading {
            // Loading spinner using throbber state
            let spinner_idx = (state.tick % SPINNER_CHARS.len() as u64) as usize;
            let spinner_char = SPINNER_CHARS[spinner_idx];

            content_spans.push(Span::styled(
                spinner_char.to_string(),
                Style::default().fg(Color::Rgb(110, 84, 255)), // #6E54FF
            ));
            content_spans.push(Span::styled(" Loading...", THEME.status_warning()));
        } else if let Some(duration) = state.time_since_refresh() {
            content_spans.push(Span::styled(
                format!("Last update: {:.0}s ago", duration.as_secs()),
                THEME.last_update(seconds_ago),
            ));
        } else {
            content_spans.push(Span::styled("Not refreshed", THEME.muted()));
        }

        content_spans.push(Span::raw("  "));
        content_spans.push(refresh_dot);
        content_spans.push(Span::styled(
            format!(" Refresh: #{}", state.refresh_count),
            THEME.muted(),
        ));

        let footer = Paragraph::new(Line::from(content_spans)).block(
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
    fn test_dashboard_screen_creation() {
        let screen = DashboardScreen::new();
        let _ = &screen;
    }

    #[test]
    fn test_dashboard_screen_default() {
        let screen = DashboardScreen;
        let _ = &screen;
    }
}
