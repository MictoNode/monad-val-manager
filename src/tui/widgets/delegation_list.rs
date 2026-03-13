//! Delegation List Widget - Scrollable list of delegations for staking screen
//!
//! This widget displays the user's delegations in a scrollable list format
//! with selection highlighting and status indicators.

use ratatui::{
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, ListItem, Paragraph},
    Frame,
};

use crate::tui::staking::{DelegationInfo, StakingState};
use crate::tui::theme::THEME;

/// Delegation list widget for displaying user's delegations
///
/// Renders a scrollable list of delegations with:
/// - Validator ID and address
/// - Delegated amount in MON
/// - Commission rate
/// - Status indicator (active/inactive)
/// - Selection highlighting
#[derive(Debug)]
pub struct DelegationListWidget<'a> {
    /// Reference to staking state (contains delegations and selection)
    state: &'a StakingState,
    /// Optional scroll offset for large lists
    scroll_offset: usize,
    /// Maximum visible items (calculated from area height)
    visible_items: usize,
}

impl<'a> DelegationListWidget<'a> {
    /// Create a new delegation list widget
    pub fn new(state: &'a StakingState) -> Self {
        Self {
            state,
            scroll_offset: 0,
            visible_items: 10,
        }
    }

    /// Set scroll offset for the list
    pub fn with_scroll_offset(mut self, offset: usize) -> Self {
        self.scroll_offset = offset;
        self
    }

    /// Set maximum visible items
    pub fn with_visible_items(mut self, count: usize) -> Self {
        self.visible_items = count;
        self
    }

    /// Get status style for a delegation
    fn get_status_style(delegation: &DelegationInfo) -> Style {
        if delegation.is_active {
            THEME.status_success()
        } else {
            THEME.status_warning()
        }
    }

    /// Get the title for the widget
    fn get_title(&self) -> String {
        format!(" Your Delegations ({}) ", self.state.delegations.len())
    }

    /// Build a single delegation list item
    fn build_item(
        delegation: &DelegationInfo,
        index: usize,
        is_selected: bool,
    ) -> ListItem<'static> {
        let status_style = Self::get_status_style(delegation);
        let base_style = if is_selected {
            THEME.selected_bold()
        } else {
            Style::default().fg(THEME.text_secondary)
        };

        // Format: " #1 Validator 42  |  1,234.56 MON"
        let content = Line::from(vec![
            Span::styled(format!(" #{} ", index + 1), status_style),
            Span::styled(format!("Val {} ", delegation.validator_id), base_style),
            Span::styled("|", THEME.muted()),
            Span::styled(
                format!(" {} MON", delegation.format_delegated_amount()),
                THEME.amount(),
            ),
        ]);

        ListItem::new(content)
    }

    /// Build empty state content
    fn build_empty_state(&self) -> Vec<Line<'static>> {
        vec![
            Line::from(""),
            Line::styled("No delegations found", THEME.text_secondary),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press ", THEME.label()),
                Span::styled("[v]", THEME.keybind()),
                Span::styled(" to query validator", THEME.label()),
                Span::styled(" then ", THEME.label()),
                Span::styled("[o]", THEME.keybind()),
                Span::styled(" to query your delegations", THEME.label()),
            ]),
        ]
    }

    /// Calculate scroll offset to ensure selected item is visible
    fn calculate_scroll_offset(&self, area_height: usize) -> usize {
        if self.state.delegations.is_empty() {
            return 0;
        }

        let visible = area_height.saturating_sub(2); // Account for borders
        let selected = self.state.selected_index;

        // If selected is before scroll offset, scroll up
        if selected < self.scroll_offset {
            return selected;
        }

        // If selected is after visible area, scroll down
        if selected >= self.scroll_offset + visible {
            return selected.saturating_sub(visible) + 1;
        }

        self.scroll_offset
    }

    /// Render the widget
    pub fn render(self, frame: &mut Frame, area: Rect) {
        let title = self.get_title();

        // Handle empty state
        if self.state.delegations.is_empty() {
            let empty_content = self.build_empty_state();
            let paragraph = Paragraph::new(empty_content)
                .block(
                    Block::default()
                        .title(title)
                        .title_style(THEME.widget_title())
                        .borders(Borders::ALL)
                        .border_style(THEME.widget_border()),
                )
                .alignment(Alignment::Center);

            frame.render_widget(paragraph, area);
            return;
        }

        // Calculate scroll offset
        let scroll = self.calculate_scroll_offset(area.height as usize);

        // Build list items with selection highlighting
        let items: Vec<ListItem> = self
            .state
            .delegations
            .iter()
            .enumerate()
            .skip(scroll)
            .take(area.height.saturating_sub(2) as usize)
            .map(|(idx, delegation)| {
                let is_selected = idx == self.state.selected_index;
                Self::build_item(delegation, idx, is_selected)
            })
            .collect();

        let list = ratatui::widgets::List::new(items)
            .block(
                Block::default()
                    .title(title)
                    .title_style(THEME.widget_title())
                    .borders(Borders::ALL)
                    .border_style(THEME.widget_border()),
            )
            .highlight_style(THEME.selected_bold());

        frame.render_widget(list, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_delegation(id: u64, amount: u128, active: bool) -> DelegationInfo {
        let mut info = DelegationInfo::new(id);
        info.delegated_amount = amount;
        info.is_active = active;
        info.commission = 500; // 5%
        info
    }

    fn create_test_state(delegations: Vec<DelegationInfo>) -> StakingState {
        let mut state = StakingState::new();
        state.set_delegations(delegations);
        state
    }

    #[test]
    fn test_delegation_list_widget_creation() {
        let state = StakingState::new();
        let widget = DelegationListWidget::new(&state);
        assert!(format!("{:?}", widget).contains("DelegationListWidget"));
    }

    #[test]
    fn test_widget_with_scroll_offset() {
        let state = StakingState::new();
        let widget = DelegationListWidget::new(&state).with_scroll_offset(5);
        assert_eq!(widget.scroll_offset, 5);
    }

    #[test]
    fn test_widget_with_visible_items() {
        let state = StakingState::new();
        let widget = DelegationListWidget::new(&state).with_visible_items(20);
        assert_eq!(widget.visible_items, 20);
    }

    #[test]
    fn test_get_status_style_active() {
        let delegation = create_test_delegation(1, 1000, true);
        let style = DelegationListWidget::get_status_style(&delegation);
        assert_eq!(style.fg, Some(THEME.success));
    }

    #[test]
    fn test_get_status_style_inactive() {
        let delegation = create_test_delegation(1, 1000, false);
        let style = DelegationListWidget::get_status_style(&delegation);
        assert_eq!(style.fg, Some(THEME.warning));
    }

    #[test]
    fn test_get_title_empty() {
        let state = StakingState::new();
        let widget = DelegationListWidget::new(&state);
        assert_eq!(widget.get_title(), " Your Delegations (0) ");
    }

    #[test]
    fn test_get_title_with_delegations() {
        let state = create_test_state(vec![
            create_test_delegation(1, 1000, true),
            create_test_delegation(2, 2000, true),
        ]);
        let widget = DelegationListWidget::new(&state);
        assert_eq!(widget.get_title(), " Your Delegations (2) ");
    }

    #[test]
    fn test_build_empty_state() {
        let state = StakingState::new();
        let widget = DelegationListWidget::new(&state);
        let empty = widget.build_empty_state();
        assert_eq!(empty.len(), 4);
        assert!(empty[1].to_string().contains("No delegations found"));
    }

    #[test]
    fn test_calculate_scroll_offset_empty() {
        let state = StakingState::new();
        let widget = DelegationListWidget::new(&state);
        let offset = widget.calculate_scroll_offset(10);
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_calculate_scroll_offset_first_visible() {
        let delegations: Vec<DelegationInfo> = (1..=5)
            .map(|id| create_test_delegation(id, 1000, true))
            .collect();
        let state = create_test_state(delegations);
        let widget = DelegationListWidget::new(&state);
        let offset = widget.calculate_scroll_offset(10);
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_calculate_scroll_offset_selected_before_offset() {
        let delegations: Vec<DelegationInfo> = (1..=20)
            .map(|id| create_test_delegation(id, 1000, true))
            .collect();
        let mut state = create_test_state(delegations);
        state.selected_index = 2; // Selected item 2

        let widget = DelegationListWidget::new(&state).with_scroll_offset(5);
        let offset = widget.calculate_scroll_offset(10);
        assert_eq!(offset, 2); // Should scroll to show selected
    }

    #[test]
    fn test_calculate_scroll_offset_selected_after_visible() {
        let delegations: Vec<DelegationInfo> = (1..=20)
            .map(|id| create_test_delegation(id, 1000, true))
            .collect();
        let mut state = create_test_state(delegations);
        state.selected_index = 15; // Selected item 15

        let widget = DelegationListWidget::new(&state).with_scroll_offset(0);
        let offset = widget.calculate_scroll_offset(10);
        // With visible = 8 (10 - 2 borders), offset should be 15 - 8 + 1 = 8
        assert_eq!(offset, 8);
    }

    #[test]
    fn test_build_item_selected() {
        let delegation = create_test_delegation(42, 1_000_000_000_000_000_000, true);
        let item = DelegationListWidget::build_item(&delegation, 0, true);
        // Verify item is created (content check would require parsing Line)
        let _ = item;
    }

    #[test]
    fn test_build_item_not_selected() {
        let delegation = create_test_delegation(42, 1_000_000_000_000_000_000, true);
        let item = DelegationListWidget::build_item(&delegation, 0, false);
        let _ = item;
    }

    #[test]
    fn test_widget_with_pending_amount() {
        let mut delegation = create_test_delegation(1, 1_000_000_000_000_000_000, true);
        delegation.pending_amount = 500_000_000_000_000_000; // 0.5 MON pending

        let state = create_test_state(vec![delegation]);
        let widget = DelegationListWidget::new(&state);
        assert_eq!(
            widget.state.delegations[0].pending_amount,
            500_000_000_000_000_000
        );
    }

    #[test]
    fn test_widget_with_rewards() {
        let mut delegation = create_test_delegation(1, 1_000_000_000_000_000_000, true);
        delegation.rewards = 100_000_000_000_000_000; // 0.1 MON rewards

        let state = create_test_state(vec![delegation]);
        let widget = DelegationListWidget::new(&state);
        assert_eq!(widget.state.delegations[0].rewards, 100_000_000_000_000_000);
    }
}
