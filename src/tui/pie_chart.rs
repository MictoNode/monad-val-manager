//! Pie Chart Widget - Visual data representation
//!
//! This module provides pie chart widgets for TUI:
//! - CPU usage pie chart
//! - Memory usage pie chart
//! - Disk usage pie chart
//! - Animated colors and labels

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Simple pie chart widget
#[derive(Debug, Clone)]
pub struct PieChart {
    pub title: String,
    pub percentage: f64,
    pub label: String,
    pub color: Color,
    pub size: PieChartSize,
}

/// Pie chart size options
#[derive(Debug, Clone, Copy)]
pub enum PieChartSize {
    Small,  // 10 chars width
    Medium, // 15 chars width
    Large,  // 20 chars width
}

impl PieChart {
    /// Create a new pie chart
    pub fn new(title: impl Into<String>, percentage: f64, label: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            percentage: percentage.clamp(0.0, 100.0),
            label: label.into(),
            color: Color::Rgb(110, 84, 255), // Default purple
            size: PieChartSize::Medium,
        }
    }

    /// Set the pie chart color
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set the pie chart size
    pub fn with_size(mut self, size: PieChartSize) -> Self {
        self.size = size;
        self
    }

    /// Get the width for this pie chart size
    fn get_width(&self) -> u16 {
        match self.size {
            PieChartSize::Small => 12,
            PieChartSize::Medium => 18,
            PieChartSize::Large => 24,
        }
    }

    /// Get the height for this pie chart
    fn get_height(&self) -> u16 {
        6 // Fixed height for all sizes
    }

    /// Generate pie chart ASCII art
    fn generate_pie(&self) -> Vec<String> {
        let pct = self.percentage;
        let width = self.get_width() as usize;

        // Simple pie representation using filled blocks
        let filled_width = (width as f64 * pct / 100.0) as usize;
        let empty_width = width.saturating_sub(filled_width);

        let filled_bar = "█".repeat(filled_width);
        let empty_bar = "░".repeat(empty_width);

        let top_border = "┌".to_string() + &"─".repeat(width.saturating_sub(2).max(0)) + "┐";
        let bottom_border = "└".to_string() + &"─".repeat(width.saturating_sub(2).max(0)) + "┘";

        vec![
            top_border,
            format!("│{}{}│", filled_bar, empty_bar),
            format!("│  {:.1}%  │", pct),
            format!("│{}{}│", filled_bar, empty_bar),
            bottom_border,
        ]
    }

    /// Render the pie chart
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let chart_width = self.get_width();
        let chart_height = self.get_height();

        // Center the chart in the area
        let x = if area.width > chart_width {
            (area.width - chart_width) / 2
        } else {
            0
        };

        let y = if area.height > chart_height {
            (area.height - chart_height) / 2
        } else {
            0
        };

        let chart_area = Rect {
            x: area.x + x,
            y: area.y + y,
            width: chart_width,
            height: chart_height,
        };

        // Generate pie chart lines
        let pie_lines = self.generate_pie();
        let lines: Vec<Line> = pie_lines
            .iter()
            .map(|line| Line::styled(line.as_str(), Style::default().fg(self.color)))
            .collect();

        let paragraph = Paragraph::new(lines).block(
            Block::default()
                .title(self.title.as_str())
                .title_style(Style::default().fg(self.color).bold())
                .borders(Borders::ALL)
                .border_style(Style::default().fg(self.color)),
        );

        frame.render_widget(paragraph, chart_area);
    }
}

/// CPU usage pie chart (green/yellow/red based on usage)
pub fn cpu_pie_chart(usage: f64) -> PieChart {
    let color = if usage < 50.0 {
        Color::Rgb(74, 222, 128) // Green
    } else if usage < 80.0 {
        Color::Rgb(255, 174, 69) // Orange
    } else {
        Color::Rgb(239, 68, 68) // Red
    };

    PieChart::new("CPU", usage, "Usage")
        .with_color(color)
        .with_size(PieChartSize::Medium)
}

/// Memory usage pie chart (purple gradient based on usage)
pub fn memory_pie_chart(usage: f64) -> PieChart {
    let color = if usage < 50.0 {
        Color::Rgb(110, 84, 255) // Purple
    } else if usage < 80.0 {
        Color::Rgb(133, 230, 255) // Cyan
    } else {
        Color::Rgb(255, 142, 228) // Pink
    };

    PieChart::new("MEM", usage, "Usage")
        .with_color(color)
        .with_size(PieChartSize::Medium)
}

/// Disk usage pie chart (blue gradient based on usage)
pub fn disk_pie_chart(usage: f64, label: &str) -> PieChart {
    let color = if usage < 50.0 {
        Color::Rgb(59, 130, 246) // Blue
    } else if usage < 80.0 {
        Color::Rgb(96, 165, 250) // Light Blue
    } else {
        Color::Rgb(147, 197, 253) // Lighter Blue
    };

    PieChart::new("DISK", usage, label)
        .with_color(color)
        .with_size(PieChartSize::Small)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pie_chart_creation() {
        let chart = PieChart::new("Test", 50.0, "Label");
        assert_eq!(chart.title, "Test");
        assert_eq!(chart.percentage, 50.0);
        assert_eq!(chart.label, "Label");
    }

    #[test]
    fn test_pie_chart_clamp() {
        let chart = PieChart::new("Test", 150.0, "Label");
        assert_eq!(chart.percentage, 100.0); // Clamped to 100

        let chart2 = PieChart::new("Test", -10.0, "Label");
        assert_eq!(chart2.percentage, 0.0); // Clamped to 0
    }

    #[test]
    fn test_pie_chart_sizes() {
        let small = PieChart::new("Test", 50.0, "Label").with_size(PieChartSize::Small);
        assert_eq!(small.get_width(), 12);

        let medium = PieChart::new("Test", 50.0, "Label").with_size(PieChartSize::Medium);
        assert_eq!(medium.get_width(), 18);

        let large = PieChart::new("Test", 50.0, "Label").with_size(PieChartSize::Large);
        assert_eq!(large.get_width(), 24);
    }

    #[test]
    fn test_cpu_pie_chart_colors() {
        let low = cpu_pie_chart(30.0);
        assert!(matches!(low.color, Color::Rgb(74, 222, 128)));

        let medium = cpu_pie_chart(60.0);
        assert!(matches!(medium.color, Color::Rgb(255, 174, 69)));

        let high = cpu_pie_chart(90.0);
        assert!(matches!(high.color, Color::Rgb(239, 68, 68)));
    }
}
