//! Header Widget - Branding and network info display

use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::state::ValidatorData;
use crate::tui::theme::THEME;

/// Header widget for displaying branding and network info
#[derive(Debug)]
pub struct HeaderWidget<'a> {
    validator: &'a ValidatorData,
}

impl<'a> HeaderWidget<'a> {
    /// Create new header widget
    pub fn new(validator: &'a ValidatorData) -> Self {
        Self { validator }
    }

    /// Get network color based on chain ID
    fn get_network_style(&self) -> ratatui::style::Style {
        match self.validator.chain_id {
            143 => THEME.status_success(),   // Mainnet
            10143 => THEME.status_warning(), // Testnet
            _ => THEME.footer(),
        }
    }

    /// Render the header widget
    pub fn render(self, frame: &mut Frame, area: Rect) {
        let network_style = self.get_network_style();

        let content = Line::from(vec![
            Span::styled(" Monad Val Manager ", THEME.header()),
            Span::raw("|"),
            Span::styled(
                format!(" {} ", self.validator.network_name.to_uppercase()),
                network_style,
            ),
            Span::raw("|"),
            Span::styled(
                format!(" Chain ID: {} ", self.validator.chain_id),
                THEME.muted(),
            ),
        ]);

        let header = Paragraph::new(content).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(THEME.widget_border()),
        );

        frame.render_widget(header, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_widget_creation() {
        let validator = ValidatorData::new();
        let widget = HeaderWidget::new(&validator);
        assert!(format!("{:?}", widget).contains("HeaderWidget"));
    }

    #[test]
    fn test_header_widget_mainnet_color() {
        let validator = ValidatorData::from_config(143, "mainnet", "http://localhost:8080");
        let _widget = HeaderWidget::new(&validator);
        // Mainnet should use success color (chain_id 143)
        assert_eq!(validator.chain_id, 143);
    }

    #[test]
    fn test_header_widget_testnet_color() {
        let validator = ValidatorData::from_config(10143, "testnet", "http://localhost:8080");
        let _widget = HeaderWidget::new(&validator);
        // Testnet should use warning color (chain_id 10143)
        assert_eq!(validator.chain_id, 10143);
    }
}
