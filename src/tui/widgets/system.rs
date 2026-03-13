//! System Metrics Widget - CPU, Memory, Disk display

use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::state::SystemData;
use crate::tui::theme::THEME;

/// System metrics widget for displaying CPU, memory, and disk usage
#[derive(Debug)]
pub struct SystemWidget<'a> {
    data: &'a SystemData,
}

impl<'a> SystemWidget<'a> {
    /// Create new system widget
    pub fn new(data: &'a SystemData) -> Self {
        Self { data }
    }

    /// Render the system widget
    pub fn render(self, frame: &mut Frame, area: Rect) {
        // Create layout for metrics
        let lines = vec![
            // CPU usage - use cpu_gradient for color coding
            Line::from(vec![
                Span::styled(" CPU:      ", THEME.metric_label()),
                Span::styled(
                    format!("{:>5.1}%", self.data.cpu_usage),
                    ratatui::style::Style::default().fg(THEME.cpu_gradient(self.data.cpu_usage)),
                ),
            ]),
            // Memory usage - use memory_gradient for color coding
            Line::from(vec![
                Span::styled(" Memory:   ", THEME.metric_label()),
                Span::styled(
                    format!("{:>5.1}%", self.data.memory_usage),
                    ratatui::style::Style::default()
                        .fg(THEME.memory_gradient(self.data.memory_usage)),
                ),
                Span::styled(format!(" ({})", self.data.format_memory()), THEME.muted()),
            ]),
            // Disk usage - use disk_gradient for color coding
            Line::from(vec![
                Span::styled(" Disk:     ", THEME.metric_label()),
                Span::styled(
                    format!("{:>5.1}%", self.data.disk_usage),
                    ratatui::style::Style::default().fg(THEME.disk_gradient(self.data.disk_usage)),
                ),
                Span::styled(format!(" ({})", self.data.format_disk()), THEME.muted()),
            ]),
        ];

        let block = Block::default()
            .title(" System ")
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
    fn test_system_widget_creation() {
        let data = SystemData::new();
        let widget = SystemWidget::new(&data);
        assert!(format!("{:?}", widget).contains("SystemWidget"));
    }

    #[test]
    fn test_cpu_gradient() {
        // Low CPU usage (< 50%) → success color
        let low = THEME.cpu_gradient(25.0);
        assert_eq!(low, THEME.success);

        // Medium CPU usage (50-80%) → warning color
        let medium = THEME.cpu_gradient(60.0);
        assert_eq!(medium, THEME.warning);

        // High CPU usage (> 80%) → error color
        let high = THEME.cpu_gradient(90.0);
        assert_eq!(high, THEME.error);
    }

    #[test]
    fn test_memory_gradient() {
        let low = THEME.memory_gradient(40.0);
        assert_eq!(low, THEME.success);

        let medium = THEME.memory_gradient(70.0);
        assert_eq!(medium, THEME.warning);

        let high = THEME.memory_gradient(90.0);
        assert_eq!(high, THEME.error);
    }

    #[test]
    fn test_disk_gradient() {
        let low = THEME.disk_gradient(60.0);
        assert_eq!(low, THEME.success);

        let medium = THEME.disk_gradient(80.0);
        assert_eq!(medium, THEME.warning);

        let high = THEME.disk_gradient(95.0);
        assert_eq!(high, THEME.error);
    }

    #[test]
    fn test_system_widget_with_data() {
        let mut data = SystemData::new();
        data.cpu_usage = 65.0;
        data.memory_usage = 45.0;
        data.disk_usage = 80.0;

        let widget = SystemWidget::new(&data);
        assert!((widget.data.cpu_usage - 65.0).abs() < 0.1);
    }
}
