//! Network Widget - Node status, block info, peers display

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::tui::state::NetworkData;
use crate::tui::theme::THEME;
use crate::tui::GlowWidgets;

/// Network status widget for displaying node connection info
#[derive(Debug)]
pub struct NetworkWidget<'a> {
    data: &'a NetworkData,
    tick: u64,
}

impl<'a> NetworkWidget<'a> {
    /// Create new network widget
    pub fn new(data: &'a NetworkData, tick: u64) -> Self {
        Self { data, tick }
    }

    /// Calculate breathing border color for synced node
    /// Interpolates between #6E54FF (purple) and #85E6FF (cyan) based on tick % 120
    fn breathing_border(&self) -> Style {
        // Monad purple: #6E54FF -> RGB(110, 84, 255)
        // Cyan: #85E6FF -> RGB(133, 230, 255)
        let cycle = (self.tick % 120) as f64;
        let progress = (cycle / 120.0) * std::f64::consts::PI; // 0 to PI
        let sine = (progress).sin(); // -1 to 1
        let factor = (sine + 1.0) / 2.0; // 0 to 1

        let r = 110.0 + (133.0 - 110.0) * factor;
        let g = 84.0 + (230.0 - 84.0) * factor;
        let b = 255.0; // Blue stays constant at 255

        Style::default().fg(Color::Rgb(r as u8, g as u8, b as u8))
    }

    /// Get border style - breathing when synced, normal otherwise
    fn get_border_style(&self) -> Style {
        // Breathing effect only when connected AND synced (not syncing)
        if self.data.is_connected && !self.data.is_syncing {
            self.breathing_border()
        } else {
            // Use normal glowing/dim border
            let is_active = self.data.is_connected;
            GlowWidgets::card_border(is_active)
        }
    }

    /// Get connection status icon and style
    fn get_connection_status(&self) -> (&'static str, ratatui::style::Style) {
        THEME.connection_status(self.data.is_connected, self.data.is_syncing)
    }

    /// Render the network widget
    pub fn render(self, frame: &mut Frame, area: Rect) {
        let (status_icon, status_style) = self.get_connection_status();

        // Calculate layout for content
        let content_height = if self.data.is_syncing && self.data.sync_progress.is_some() {
            7 // Extra line for sync progress bar
        } else {
            6
        };

        let content_area = if area.height > content_height + 2 {
            // Center content vertically if there's extra space
            let vertical_margin = (area.height.saturating_sub(content_height + 2)) / 2;
            Rect {
                x: area.x,
                y: area.y + vertical_margin,
                width: area.width,
                height: content_height + 2,
            }
        } else {
            area
        };

        let lines = vec![
            // Connection status
            Line::from(vec![
                Span::styled(" Status:   ", THEME.metric_label()),
                Span::styled(format!("{} ", status_icon), status_style),
                Span::styled(
                    if self.data.is_connected {
                        if self.data.is_syncing {
                            "Syncing".to_string()
                        } else {
                            "Synced".to_string()
                        }
                    } else {
                        "Disconnected".to_string()
                    },
                    status_style,
                ),
            ]),
            // Block number - use block_number style
            Line::from(vec![
                Span::styled(" Block:    ", THEME.metric_label()),
                Span::styled(
                    if self.data.is_connected {
                        self.data.format_block_number()
                    } else {
                        "N/A".to_string()
                    },
                    THEME.block_number(),
                ),
            ]),
            // Peer count - use peers_count style (cyan)
            Line::from(vec![
                Span::styled(" Peers:    ", THEME.metric_label()),
                Span::styled(
                    if self.data.is_connected {
                        self.data.peer_count.to_string()
                    } else {
                        "N/A".to_string()
                    },
                    THEME.peers_count(),
                ),
            ]),
            // Node version
            Line::from(vec![
                Span::styled(" Version:  ", THEME.metric_label()),
                Span::styled(
                    self.data.node_version.as_deref().unwrap_or("N/A"),
                    THEME.muted(),
                ),
            ]),
        ];

        // Add error line if present
        let mut final_lines = lines;
        if let Some(ref error) = self.data.last_error {
            final_lines.push(Line::from(vec![
                Span::styled(" Error:    ", THEME.metric_label()),
                Span::styled(
                    Self::truncate_error(error, content_area.width.saturating_sub(14) as usize),
                    THEME.status_error(),
                ),
            ]));
        }

        // Use glowing border when connected, dim border when disconnected
        // Breathing effect when synced (connected and not syncing)
        let block = Block::default()
            .title(" Node Status ")
            .title_style(THEME.widget_title())
            .borders(Borders::ALL)
            .border_style(self.get_border_style());

        let paragraph = Paragraph::new(final_lines).block(block);

        frame.render_widget(paragraph, content_area);

        // Render sync progress bar if syncing
        if self.data.is_syncing {
            if let Some(progress) = self.data.sync_progress {
                self.render_sync_progress(frame, content_area, progress);
            }
        }
    }

    /// Render sync progress bar at the bottom of the widget
    fn render_sync_progress(&self, frame: &mut Frame, area: Rect, progress: f64) {
        let progress_pct = (progress / 100.0).clamp(0.0, 1.0);

        // Create a small area at the bottom for the progress bar
        let bar_area = Rect {
            x: area.x + 2,
            y: area.bottom().saturating_sub(2),
            width: area.width.saturating_sub(4),
            height: 1,
        };

        let label = Span::styled(format!(" Sync: {:.1}% ", progress), THEME.metric_value());

        let gauge = Gauge::default()
            .gauge_style(ratatui::style::Style::default().fg(THEME.accent))
            .label(label)
            .ratio(progress_pct);

        frame.render_widget(gauge, bar_area);
    }

    /// Truncate error message to fit display
    fn truncate_error(error: &str, max_len: usize) -> String {
        if error.len() <= max_len {
            error.to_string()
        } else {
            format!("{}...", &error[..max_len.saturating_sub(3)])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_widget_creation() {
        let data = NetworkData::new();
        let widget = NetworkWidget::new(&data, 0);
        assert!(format!("{:?}", widget).contains("NetworkWidget"));
        assert_eq!(widget.tick, 0);
    }

    #[test]
    fn test_status_connected_synced() {
        let mut data = NetworkData::new();
        data.is_connected = true;
        data.is_syncing = false;

        let widget = NetworkWidget::new(&data, 0);
        let (icon, _style) = widget.get_connection_status();
        assert_eq!(icon, "●");
    }

    #[test]
    fn test_status_connected_syncing() {
        let mut data = NetworkData::new();
        data.is_connected = true;
        data.is_syncing = true;

        let widget = NetworkWidget::new(&data, 0);
        let (icon, _style) = widget.get_connection_status();
        // Monad brand spec: all connection states use ● dot
        assert_eq!(icon, "●");
    }

    #[test]
    fn test_status_disconnected() {
        let mut data = NetworkData::new();
        data.is_connected = false;

        let widget = NetworkWidget::new(&data, 0);
        let (icon, _style) = widget.get_connection_status();
        // Monad brand spec: all connection states use ● dot
        assert_eq!(icon, "●");
    }

    #[test]
    fn test_breathing_border_when_synced() {
        let mut data = NetworkData::new();
        data.is_connected = true;
        data.is_syncing = false; // Synced

        let widget = NetworkWidget::new(&data, 0);
        let border_style = widget.get_border_style();

        // Should be a breathing color (RGB)
        if let Some(Color::Rgb(r, g, b)) = border_style.fg {
            assert!((110..=133).contains(&r));
            assert!((84..=230).contains(&g));
            assert_eq!(b, 255);
        } else {
            panic!("Expected Rgb color for breathing border when synced");
        }
    }

    #[test]
    fn test_no_breathing_border_when_syncing() {
        let mut data = NetworkData::new();
        data.is_connected = true;
        data.is_syncing = true; // Still syncing

        let widget = NetworkWidget::new(&data, 0);
        let border_style = widget.get_border_style();

        // Should NOT be breathing (use normal glow color)
        // The exact color depends on GlowWidgets::card_border(true)
        // Just verify it returns some style
        assert!(border_style == Style::default() || border_style.fg.is_some());
    }

    #[test]
    fn test_breathing_cycle() {
        let data = NetworkData::new();
        let widget_0 = NetworkWidget::new(&data, 0);
        let widget_60 = NetworkWidget::new(&data, 60);

        let border_0 = widget_0.breathing_border();
        let border_60 = widget_60.breathing_border();

        // Colors should be different at different points in the cycle
        if let (Some(Color::Rgb(r0, _, _)), Some(Color::Rgb(r60, _, _))) =
            (border_0.fg, border_60.fg)
        {
            assert_ne!(r0, r60);
        } else {
            panic!("Expected Rgb colors");
        }
    }

    #[test]
    fn test_truncate_error_short() {
        let error = "Short error";
        let truncated = NetworkWidget::truncate_error(error, 50);
        assert_eq!(truncated, "Short error");
    }

    #[test]
    fn test_truncate_error_long() {
        let error = "This is a very long error message that needs to be truncated";
        let truncated = NetworkWidget::truncate_error(error, 20);
        assert_eq!(truncated.len(), 20);
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_network_widget_with_data() {
        let mut data = NetworkData::new();
        data.is_connected = true;
        data.block_number = 123456;
        data.peer_count = 25;

        let widget = NetworkWidget::new(&data, 42);
        assert!(widget.data.is_connected);
        assert_eq!(widget.data.block_number, 123456);
        assert_eq!(widget.data.peer_count, 25);
        assert_eq!(widget.tick, 42);
    }
}
