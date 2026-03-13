//! Consensus Info Widget - Epoch, Round, Forkpoint display

use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::state::ConsensusData;
use crate::tui::theme::THEME;

/// Consensus info widget for displaying epoch, round, and forkpoint
#[derive(Debug)]
pub struct ConsensusWidget<'a> {
    data: &'a ConsensusData,
}

impl<'a> ConsensusWidget<'a> {
    /// Create new consensus widget
    pub fn new(data: &'a ConsensusData) -> Self {
        Self { data }
    }

    /// Render the consensus widget
    pub fn render(self, frame: &mut Frame, area: Rect) {
        let lines = vec![
            // Epoch - use epoch style (brand purple bold)
            Line::from(vec![
                Span::styled(" Epoch:   ", THEME.metric_label()),
                Span::styled(format!("{}", self.data.epoch), THEME.epoch()),
            ]),
            // Round - use info style (light blue)
            Line::from(vec![
                Span::styled(" Round:    ", THEME.metric_label()),
                Span::styled(format!("{}", self.data.round), THEME.metric_value()),
            ]),
            // Forkpoint
            Line::from(vec![
                Span::styled(" Forkpoint:", THEME.metric_label()),
                Span::styled(
                    format!(
                        " E{} R{}",
                        self.data.forkpoint_epoch, self.data.forkpoint_round
                    ),
                    THEME.muted(),
                ),
            ]),
            // Uptime removed - metric not reliably available
        ];

        let block = Block::default()
            .title(" Consensus ")
            .title_style(THEME.widget_title())
            .borders(Borders::ALL)
            .border_style(THEME.widget_border());

        let paragraph = Paragraph::new(lines).block(block);

        frame.render_widget(paragraph, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consensus_widget_creation() {
        let data = ConsensusData::default();
        let widget = ConsensusWidget::new(&data);
        let _ = widget;
    }
}
