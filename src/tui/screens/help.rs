//! Help Screen - Keybindings and usage information

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::{Screen, ScreenRender};
use crate::tui::state::AppState;
use crate::tui::theme::THEME;
use crate::tui::widgets::NavMenuWidget;

/// Help screen showing keybindings and usage
pub struct HelpScreen;

impl HelpScreen {
    /// Create new help screen
    pub fn new() -> Self {
        Self
    }
}

impl Default for HelpScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenRender for HelpScreen {
    fn render(&self, frame: &mut Frame, state: &AppState) {
        let area = frame.area();

        // Create layout: nav menu, content, footer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Nav Menu (with branding)
                Constraint::Min(10),   // Help content
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Navigation Menu
        let nav_menu = NavMenuWidget::new(Screen::Help, state.tick);
        nav_menu.render(frame, chunks[0]);

        // Help content
        let content = vec![
            Line::from(""),
            Line::styled("  Navigation", THEME.metric_value()),
            Line::styled("    Tab        Next screen", THEME.footer()),
            Line::styled("    Shift+Tab  Previous screen", THEME.footer()),
            Line::styled("    Esc        Back / Dashboard", THEME.footer()),
            Line::from(""),
            Line::styled("  Quick Screen Access", THEME.metric_value()),
            Line::styled("    1          Dashboard", THEME.footer()),
            Line::styled("    2          Staking", THEME.footer()),
            Line::styled("    3          Transfer", THEME.footer()),
            Line::styled("    4          Doctor", THEME.footer()),
            Line::styled("    5          Help", THEME.footer()),
            Line::from(""),
            Line::styled("  Actions", THEME.metric_value()),
            Line::styled("    q / Ctrl+C Quit application", THEME.footer()),
            Line::styled("    r          Refresh data", THEME.footer()),
            Line::styled("    h          Show this help", THEME.footer()),
            Line::from(""),
            Line::styled("  Screen-Specific Actions", THEME.metric_value()),
            Line::styled(
                "    Staking:   d=Delegate u=Undelegate w=Withdraw",
                THEME.footer(),
            ),
            Line::styled(
                "               c=Claim m=Compound a=AddVal x=ChgComm",
                THEME.footer(),
            ),
            Line::styled(
                "               v=Q.Validator o=Q.Delegator (Auto-refresh)",
                THEME.footer(),
            ),
            Line::styled("    Transfer:  t=Open Transfer Dialog", THEME.footer()),
            Line::styled(
                "    Doctor:    r=Run Checks ↑↓=Navigate Enter=Run Selected",
                THEME.footer(),
            ),
        ];

        let content_block = Block::default()
            .borders(Borders::ALL)
            .border_style(THEME.widget_border());

        let paragraph = Paragraph::new(content)
            .block(content_block)
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, chunks[1]);

        // Footer
        self.render_footer(frame, chunks[2]);
    }
}

impl HelpScreen {
    /// Render the footer with controls
    fn render_footer(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let content = Line::from(vec![
            Span::styled("[q]", THEME.keybind()),
            Span::styled(" Quit ", THEME.keybind_description()),
            Span::styled("[Tab]", THEME.keybind()),
            Span::styled(" Next Screen ", THEME.keybind_description()),
            Span::styled("[Esc]", THEME.keybind()),
            Span::styled(" Back to Dashboard ", THEME.keybind_description()),
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
    fn test_help_screen_creation() {
        let screen = HelpScreen::new();
        let _ = &screen;
    }

    #[test]
    fn test_help_screen_default() {
        let screen = HelpScreen;
        let _ = &screen;
    }
}
