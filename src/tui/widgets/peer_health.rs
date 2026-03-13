//! Peer Health Widget - Displays peer connectivity status
//!
//! Shows active peers, timeout count, and total peer count
//! in a compact, visually appealing format.

use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::theme::THEME;

/// Peer health information
#[derive(Debug, Clone, Default)]
pub struct PeerHealth {
    pub active_peers: usize,
    pub timeout_peers: usize,
    pub total_peers: usize,
}

/// Peer Health Widget
pub struct PeerHealthWidget {
    health: PeerHealth,
}

impl PeerHealthWidget {
    /// Create new peer health widget
    pub fn new(health: &PeerHealth) -> Self {
        Self {
            health: health.clone(),
        }
    }

    /// Render the widget
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Calculate peer counts
        let active_count = self.health.active_peers;
        let timeout_count = self.health.timeout_peers;
        let total_count = self.health.total_peers;

        // Create active peers dots (green circles)
        let mut active_dots = String::new();
        for _ in 0..active_count.min(10) {
            active_dots.push('●');
        }
        if active_count > 10 {
            active_dots.push_str(&format!("(+{})", active_count - 10));
        }

        // Create timeout dots (red circles)
        let mut timeout_dots = String::new();
        for _ in 0..timeout_count.min(10) {
            timeout_dots.push('●');
        }
        if timeout_count > 10 {
            timeout_dots.push_str(&format!("(+{})", timeout_count - 10));
        }

        // Build content lines
        let content = vec![
            Line::from(vec![
                Span::styled("Active: ", THEME.label()),
                Span::styled(
                    active_dots,
                    ratatui::style::Style::default().fg(THEME.success),
                ),
                Span::styled(format!(" ({})", active_count), THEME.muted()),
            ]),
            Line::from(vec![
                Span::styled("Timeouts: ", THEME.label()),
                Span::styled(
                    timeout_dots,
                    ratatui::style::Style::default().fg(THEME.error),
                ),
                Span::styled(format!(" ({})", timeout_count), THEME.muted()),
            ]),
            Line::from(vec![
                Span::styled("Total: ", THEME.label()),
                Span::styled(format!("{} peers", total_count), THEME.peers_count()),
            ]),
        ];

        // Create paragraph
        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .title(" Peers ")
                    .borders(Borders::ALL)
                    .border_style(THEME.widget_border()),
            )
            .wrap(ratatui::widgets::Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_health_default() {
        let health = PeerHealth::default();
        assert_eq!(health.active_peers, 0);
        assert_eq!(health.timeout_peers, 0);
        assert_eq!(health.total_peers, 0);
    }

    #[test]
    fn test_peer_health_creation() {
        let health = PeerHealth {
            active_peers: 8,
            timeout_peers: 2,
            total_peers: 10,
        };
        assert_eq!(health.active_peers, 8);
        assert_eq!(health.timeout_peers, 2);
        assert_eq!(health.total_peers, 10);
    }

    #[test]
    fn test_peer_health_widget_creation() {
        let health = PeerHealth {
            active_peers: 5,
            timeout_peers: 1,
            total_peers: 6,
        };
        let _widget = PeerHealthWidget::new(&health);
        // Widget created successfully
    }

    #[test]
    fn test_peer_health_with_zero_peers() {
        let health = PeerHealth {
            active_peers: 0,
            timeout_peers: 0,
            total_peers: 0,
        };
        let _widget = PeerHealthWidget::new(&health);
        // Should handle zero peers gracefully
    }

    #[test]
    fn test_peer_health_with_many_timeouts() {
        let health = PeerHealth {
            active_peers: 2,
            timeout_peers: 15,
            total_peers: 17,
        };
        let _widget = PeerHealthWidget::new(&health);
        // Should handle >10 timeouts with "+n" notation
    }
}
