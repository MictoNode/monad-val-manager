//! Navigation Menu Widget - Top menu buttons for all screens
//!
//! This widget provides consistent navigation menu across all screens:
//! - Numbered shortcuts (1-5) for quick screen access
//! - Visual feedback for current screen
//! - Monad theme styling
//! - Breathing animation on active tab

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::screens::Screen;
use crate::tui::theme::THEME;

/// Navigation menu widget
#[derive(Debug)]
pub struct NavMenuWidget {
    current_screen: Screen,
    tick: u64,
}

impl NavMenuWidget {
    /// Create new navigation menu widget
    pub fn new(current_screen: Screen, tick: u64) -> Self {
        Self {
            current_screen,
            tick,
        }
    }

    /// Calculate breathing background color for active tab
    /// Interpolates between #6E54FF and #8B6FFF based on tick % 60
    fn breathing_active_bg(&self) -> Color {
        // Monad purple: #6E54FF -> RGB(110, 84, 255)
        // Lighter purple: #8B6FFF -> RGB(139, 111, 255)
        let cycle = (self.tick % 60) as f64;
        let progress = (cycle / 60.0) * std::f64::consts::PI; // 0 to PI
        let sine = (progress).sin(); // -1 to 1
        let factor = (sine + 1.0) / 2.0; // 0 to 1

        let r = (110.0 + (139.0 - 110.0) * factor) as u8;
        let g = (84.0 + (111.0 - 84.0) * factor) as u8;
        let b = 255; // Blue stays constant

        Color::Rgb(r, g, b)
    }

    /// Render the navigation menu
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let menu_items = self.build_menu();

        // Outer block with border for the entire nav menu
        let outer_block = Block::default()
            .borders(Borders::ALL)
            .border_style(THEME.widget_border());

        // Render outer block first (creates the border)
        frame.render_widget(outer_block, area);

        // Inner area without borders (use Margin)
        let inner = area.inner(ratatui::layout::Margin::new(1, 1));

        // Layout: [Branding area (fixed)] | [Menu items...]
        let mut constraints = vec![Constraint::Length(22)]; // Branding area - fixed width
        for _ in &menu_items {
            constraints.push(Constraint::Min(12)); // Each menu item - minimum width
        }

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(inner);

        // Render branding in first slot
        self.render_branding(frame, layout[0]);

        // Render each menu item in remaining slots
        for (i, item) in menu_items.iter().enumerate() {
            self.render_menu_item(frame, layout[i + 1], item);
        }
    }

    /// Render the branding section
    fn render_branding(&self, frame: &mut Frame, area: Rect) {
        let content = Line::from(vec![Span::styled(
            " MonadNode Manager ",
            crate::tui::theme::THEME.header(),
        )]);

        let paragraph = Paragraph::new(content).alignment(Alignment::Left);

        frame.render_widget(paragraph, area);
    }

    /// Build menu items with styles
    fn build_menu(&self) -> Vec<MenuItem> {
        vec![
            MenuItem {
                key: '1',
                label: "Dashboard",
                is_active: self.current_screen == Screen::Dashboard,
            },
            MenuItem {
                key: '2',
                label: "Staking",
                is_active: self.current_screen == Screen::Staking,
            },
            MenuItem {
                key: '3',
                label: "Transfer",
                is_active: self.current_screen == Screen::Transfer,
            },
            MenuItem {
                key: '4',
                label: "Doctor",
                is_active: self.current_screen == Screen::Doctor,
            },
            MenuItem {
                key: '5',
                label: "Help",
                is_active: self.current_screen == Screen::Help,
            },
        ]
    }

    /// Render a single menu item (no individual borders)
    fn render_menu_item(&self, frame: &mut Frame, area: Rect, item: &MenuItem) {
        // Active item gets breathing background color, inactive items have no styling
        let (block_style, text_style) = if item.is_active {
            (
                Style::default().bg(self.breathing_active_bg()), // Breathing purple background
                THEME.tab_active(),                              // White bold text
            )
        } else {
            (
                Style::default(),     // No background
                THEME.tab_inactive(), // Gray text
            )
        };

        let content = Line::from(vec![
            Span::styled(format!("[{}]", item.key), THEME.tab_number()),
            Span::raw(" "),
            Span::styled(item.label, text_style),
        ]);

        let paragraph = Paragraph::new(content)
            .style(block_style)
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }
}

/// Menu item data
#[derive(Debug, Clone)]
struct MenuItem {
    key: char,
    label: &'static str,
    is_active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nav_menu_widget_creation() {
        let menu = NavMenuWidget::new(Screen::Dashboard, 0);
        assert_eq!(menu.current_screen, Screen::Dashboard);
        assert_eq!(menu.tick, 0);
    }

    #[test]
    fn test_build_menu_items() {
        let menu = NavMenuWidget::new(Screen::Doctor, 0);
        let items = menu.build_menu();

        assert_eq!(items.len(), 5); // After Query and Perf removal
        assert_eq!(items[0].key, '1');
        assert_eq!(items[0].label, "Dashboard");
        assert!(!items[0].is_active);

        assert_eq!(items[3].key, '4');
        assert_eq!(items[3].label, "Doctor");
        assert!(items[3].is_active);
    }

    #[test]
    fn test_menu_item_active_states() {
        let dashboard_menu = NavMenuWidget::new(Screen::Dashboard, 0);
        let items = dashboard_menu.build_menu();

        assert!(items[0].is_active); // Dashboard is active
        assert!(!items[1].is_active); // Staking is not
        assert!(!items[2].is_active); // Transfer is not
        assert!(!items[3].is_active); // Doctor is not
        assert!(!items[4].is_active); // Help is not
    }

    #[test]
    fn test_help_menu_item() {
        let help_menu = NavMenuWidget::new(Screen::Help, 0);
        let items = help_menu.build_menu();

        assert!(!items[0].is_active); // Dashboard is not
        assert!(!items[1].is_active); // Staking is not
        assert!(!items[2].is_active); // Transfer is not
        assert!(!items[3].is_active); // Doctor is not
        assert!(items[4].is_active); // Help is active

        assert_eq!(items[4].key, '5');
        assert_eq!(items[4].label, "Help");
    }

    #[test]
    fn test_menu_item_count_after_query_perf_removal() {
        let menu = NavMenuWidget::new(Screen::Dashboard, 0);
        let items = menu.build_menu();

        // Should have 5 items after Query and Perf removal
        assert_eq!(items.len(), 5);
    }

    #[test]
    fn test_breathing_active_bg() {
        let menu = NavMenuWidget::new(Screen::Dashboard, 0);
        let color = menu.breathing_active_bg();

        // Should return an Rgb color
        if let ratatui::style::Color::Rgb(r, g, b) = color {
            assert!((110..=139).contains(&r)); // Should be between 110 and 139
            assert!((84..=111).contains(&g)); // Should be between 84 and 111
            assert_eq!(b, 255);
        } else {
            panic!("Expected Rgb color");
        }
    }

    #[test]
    fn test_breathing_cycle() {
        // Test at different tick positions
        let menu_0 = NavMenuWidget::new(Screen::Dashboard, 0);
        let menu_30 = NavMenuWidget::new(Screen::Dashboard, 30);

        let color_0 = menu_0.breathing_active_bg();
        let color_30 = menu_30.breathing_active_bg();

        // Colors should be different at different points in the cycle
        if let (ratatui::style::Color::Rgb(r0, _, _), ratatui::style::Color::Rgb(r30, _, _)) =
            (color_0, color_30)
        {
            assert_ne!(r0, r30); // Should have different red values
        } else {
            panic!("Expected Rgb colors");
        }
    }
}
