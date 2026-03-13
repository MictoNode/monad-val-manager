//! Transfer Screen - Native MON transfer interface
//!
//! This screen provides:
//! - Transfer dialog overlay (via TransferDialogWidget)
//! - Accessible via [T] shortcut from other screens
//! - Full transfer flow: Address -> Amount -> Confirm -> Processing -> Complete

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::{Screen, ScreenRender};
use crate::tui::state::AppState;
use crate::tui::theme::THEME;
use crate::tui::widgets::{NavMenuWidget, TransferDialogWidget};

/// Transfer screen for native MON transfers
pub struct TransferScreen;

impl TransferScreen {
    /// Create new transfer screen
    pub fn new() -> Self {
        Self
    }
}

impl Default for TransferScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenRender for TransferScreen {
    fn render(&self, frame: &mut Frame, state: &AppState) {
        let area = frame.area();

        // Create layout: nav menu, content, navigation hints
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Nav Menu (with branding)
                Constraint::Min(10),   // Content area
                Constraint::Length(2), // Navigation hints
            ])
            .split(area);

        // Navigation Menu
        let nav_menu = NavMenuWidget::new(Screen::Transfer, state.tick);
        nav_menu.render(frame, chunks[0]);

        // Render content (transfer dialog or empty state)
        if state.transfer.is_active {
            // Render transfer dialog overlay
            let transfer_widget = TransferDialogWidget::new(&state.transfer);
            transfer_widget.render(frame);
        } else {
            // Render empty state / instructions
            self.render_empty_state(frame, chunks[1]);
        }

        // Render navigation hints
        self.render_navigation(frame, chunks[2]);
    }
}

impl TransferScreen {
    /// Render empty state when dialog is not active
    fn render_empty_state(&self, frame: &mut Frame, area: Rect) {
        let content = vec![
            Line::from(vec![Span::styled("Transfer Screen", THEME.widget_title())]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press ", THEME.footer()),
                Span::styled("[t]", THEME.action_hint()),
                Span::styled(" to open the transfer dialog", THEME.footer()),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "The transfer dialog allows you to:",
                THEME.metric_label(),
            )]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("•", THEME.accent),
                Span::raw(" Send native MON to any address"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("•", THEME.accent),
                Span::raw(" Specify amount in MON (supports decimals)"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("•", THEME.accent),
                Span::raw(" Confirm transaction before broadcasting"),
            ]),
        ];

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(THEME.widget_border()),
            )
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    /// Render navigation hints
    fn render_navigation(&self, frame: &mut Frame, area: Rect) {
        let hints = Line::from(vec![
            Span::styled("[T]", THEME.action_hint()),
            Span::styled(" Open Transfer Dialog  ", THEME.footer()),
            Span::styled("[Tab]", THEME.status_warning()),
            Span::styled(" Next Screen", THEME.footer()),
        ]);

        let paragraph = Paragraph::new(hints).alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_screen_creation() {
        let screen = TransferScreen::new();
        let _ = &screen;
    }

    #[test]
    fn test_transfer_screen_default() {
        let screen = TransferScreen;
        let _ = &screen;
    }
}
