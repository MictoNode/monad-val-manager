//! Validator Widget - Chain info and validator status

use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::state::ValidatorData;
use crate::tui::theme::THEME;
use crate::tui::GlowWidgets;

/// Validator info widget for displaying chain and validator status
#[derive(Debug)]
pub struct ValidatorWidget<'a> {
    data: &'a ValidatorData,
}

impl<'a> ValidatorWidget<'a> {
    /// Create new validator widget
    pub fn new(data: &'a ValidatorData) -> Self {
        Self { data }
    }

    /// Get network display style based on chain ID
    fn get_network_style(&self) -> ratatui::style::Style {
        match self.data.chain_id {
            143 => THEME.badge_network(),   // Mainnet
            10143 => THEME.badge_network(), // Testnet
            _ => THEME.muted(),
        }
    }

    /// Get validator badge text and style
    fn get_validator_badge(&self) -> (&'static str, ratatui::style::Style) {
        if self.data.is_validator {
            ("VALIDATOR", THEME.badge_validator())
        } else {
            ("FULL NODE", THEME.status_info())
        }
    }

    /// Render the validator widget
    pub fn render(self, frame: &mut Frame, area: Rect) {
        let network_style = self.get_network_style();
        let (badge_text, badge_style) = self.get_validator_badge();

        let lines = vec![
            // Network
            Line::from(vec![
                Span::styled(" Network:  ", THEME.metric_label()),
                Span::styled(self.data.network_name.to_uppercase(), network_style),
            ]),
            // Chain ID
            Line::from(vec![
                Span::styled(" Chain ID: ", THEME.metric_label()),
                Span::styled(self.data.chain_id.to_string(), THEME.metric_value()),
            ]),
            // RPC Endpoint
            Line::from(vec![
                Span::styled(" RPC:      ", THEME.metric_label()),
                Span::styled(
                    Self::truncate_endpoint(
                        &self.data.rpc_endpoint,
                        area.width.saturating_sub(14) as usize,
                    ),
                    THEME.muted(),
                ),
            ]),
            // Node Type Badge
            Line::from(vec![
                Span::styled(" Type:     ", THEME.metric_label()),
                Span::styled(format!("[ {} ]", badge_text), badge_style),
            ]),
        ];

        // Use glowing border for active widget
        let is_active = true; // Validator widget is always active (static data)
        let block = Block::default()
            .title(" Validator ")
            .title_style(THEME.widget_title())
            .borders(Borders::ALL)
            .border_style(GlowWidgets::card_border(is_active));

        let paragraph = Paragraph::new(lines).block(block);

        frame.render_widget(paragraph, area);
    }

    /// Truncate endpoint URL to fit display
    fn truncate_endpoint(endpoint: &str, max_len: usize) -> String {
        if endpoint.len() <= max_len {
            endpoint.to_string()
        } else if max_len > 3 {
            format!("{}...", &endpoint[..max_len.saturating_sub(3)])
        } else {
            "...".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_widget_creation() {
        let data = ValidatorData::new();
        let widget = ValidatorWidget::new(&data);
        assert!(format!("{:?}", widget).contains("ValidatorWidget"));
    }

    #[test]
    fn test_network_style_mainnet() {
        let data = ValidatorData::from_config(143, "mainnet", "http://localhost:8080");
        let widget = ValidatorWidget::new(&data);
        let style = widget.get_network_style();
        // badge_network() has fg=text_primary, bg=accent
        assert_eq!(style.fg, Some(THEME.text_primary));
        assert_eq!(style.bg, Some(THEME.accent));
    }

    #[test]
    fn test_network_style_testnet() {
        let data = ValidatorData::from_config(10143, "testnet", "http://localhost:8080");
        let widget = ValidatorWidget::new(&data);
        let style = widget.get_network_style();
        // badge_network() has fg=text_primary, bg=accent
        assert_eq!(style.fg, Some(THEME.text_primary));
        assert_eq!(style.bg, Some(THEME.accent));
    }

    #[test]
    fn test_network_style_unknown() {
        let data = ValidatorData::from_config(999, "unknown", "http://localhost:8080");
        let widget = ValidatorWidget::new(&data);
        let style = widget.get_network_style();
        assert_eq!(style.fg, Some(THEME.text_muted));
    }

    #[test]
    fn test_validator_badge_validator() {
        let mut data = ValidatorData::new();
        data.is_validator = true;

        let widget = ValidatorWidget::new(&data);
        let (text, style) = widget.get_validator_badge();
        assert_eq!(text, "VALIDATOR");
        // badge_validator() has fg=bg, bg=warning
        assert_eq!(style.fg, Some(THEME.bg));
        assert_eq!(style.bg, Some(THEME.warning));
    }

    #[test]
    fn test_validator_badge_full_node() {
        let data = ValidatorData::new();
        let widget = ValidatorWidget::new(&data);
        let (text, style) = widget.get_validator_badge();
        assert_eq!(text, "FULL NODE");
        assert_eq!(style.fg, Some(THEME.info));
    }

    #[test]
    fn test_truncate_endpoint_short() {
        let endpoint = "http://localhost:8080";
        let truncated = ValidatorWidget::truncate_endpoint(endpoint, 50);
        assert_eq!(truncated, "http://localhost:8080");
    }

    #[test]
    fn test_truncate_endpoint_long() {
        let endpoint = "http://very-long-hostname.example.com:8080/path/to/rpc";
        let truncated = ValidatorWidget::truncate_endpoint(endpoint, 25);
        assert_eq!(truncated.len(), 25);
        assert!(truncated.ends_with("..."));
    }
}
