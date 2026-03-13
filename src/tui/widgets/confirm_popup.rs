//! Transaction Confirmation Popup Widget - Modal dialog for confirming staking actions
//!
//! This widget provides a confirmation dialog that displays pending staking action
//! details and allows the user to confirm or cancel the transaction.
//!
//! Features:
//! - Modal overlay with centered popup
//! - Displays action type, amount, and validator info
//! - Confirm (Enter) and Cancel (Esc) actions
//! - Warning message about blockchain permanence

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::tui::staking::{PendingStakingAction, StakingActionType};
use crate::tui::theme::THEME;

/// State for the confirmation popup
#[derive(Debug, Clone, Default)]
pub struct ConfirmPopupState {
    /// Is the popup currently visible
    pub is_active: bool,
    /// The pending action to confirm
    pub pending_action: Option<PendingStakingAction>,
}

impl ConfirmPopupState {
    /// Create a new confirmation popup state
    pub fn new() -> Self {
        Self::default()
    }

    /// Open the popup with a pending action
    pub fn open(&mut self, action: PendingStakingAction) {
        self.pending_action = Some(action);
        self.is_active = true;
    }

    /// Close the popup and clear the pending action
    pub fn close(&mut self) {
        self.pending_action = None;
        self.is_active = false;
    }

    /// Check if the popup is active
    pub fn is_active(&self) -> bool {
        self.is_active && self.pending_action.is_some()
    }

    /// Get the pending action if any
    pub fn get_action(&self) -> Option<&PendingStakingAction> {
        self.pending_action.as_ref()
    }

    /// Take the pending action, consuming it
    pub fn take_action(&mut self) -> Option<PendingStakingAction> {
        self.is_active = false;
        self.pending_action.take()
    }
}

/// Widget for rendering the confirmation popup
pub struct ConfirmPopupWidget<'a> {
    /// Reference to popup state
    state: &'a ConfirmPopupState,
    /// Popup width as percentage (0-100)
    width_percent: u16,
}

impl<'a> ConfirmPopupWidget<'a> {
    /// Create a new confirmation popup widget
    pub fn new(state: &'a ConfirmPopupState) -> Self {
        Self {
            state,
            width_percent: 60,
        }
    }

    /// Set popup width as percentage
    pub fn with_width(mut self, percent: u16) -> Self {
        self.width_percent = percent.min(100);
        self
    }

    /// Calculate the centered popup area
    fn calculate_popup_area(&self, frame_area: Rect) -> Rect {
        let popup_width = (frame_area.width * self.width_percent / 100).max(50);
        let popup_height = 10; // Fixed height for confirmation

        let x = (frame_area.width.saturating_sub(popup_width)) / 2;
        let y = (frame_area.height.saturating_sub(popup_height)) / 2;

        Rect::new(x, y, popup_width, popup_height)
    }

    /// Get action type style
    fn get_action_style(action_type: StakingActionType) -> ratatui::style::Style {
        match action_type {
            StakingActionType::Delegate => THEME.amount_positive(),
            StakingActionType::Undelegate => THEME.status_warning(),
            StakingActionType::Withdraw => THEME.status_info(),
            StakingActionType::ClaimRewards => THEME.rewards(),
            StakingActionType::Compound => THEME.validator_id(),
        }
    }

    /// Get action icon
    fn get_action_icon(action_type: StakingActionType) -> &'static str {
        match action_type {
            StakingActionType::Delegate => "+",
            StakingActionType::Undelegate => "-",
            StakingActionType::Withdraw => ">",
            StakingActionType::ClaimRewards => "*",
            StakingActionType::Compound => "@",
        }
    }

    /// Render the popup
    pub fn render(&self, frame: &mut Frame) {
        if !self.state.is_active() {
            return;
        }

        let action = match &self.state.pending_action {
            Some(a) => a,
            None => return,
        };

        let frame_area = frame.area();
        let popup_area = self.calculate_popup_area(frame_area);

        // Clear the area where the popup will be rendered
        frame.render_widget(Clear, popup_area);

        // Create layout for popup content
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Title
                Constraint::Length(1), // Spacer
                Constraint::Length(1), // Action type
                Constraint::Length(1), // Amount (if applicable)
                Constraint::Length(1), // Validator info
                Constraint::Length(1), // Spacer
                Constraint::Length(1), // Warning
                Constraint::Length(1), // Spacer
                Constraint::Length(1), // Actions
            ])
            .split(popup_area);

        // Render border and title with Double border type
        let title_block = Block::default()
            .title(" Confirm Transaction ")
            .title_style(THEME.dialog_title())
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(THEME.dialog_border());

        frame.render_widget(title_block, popup_area);

        // Render action type with icon and color
        let action_style = Self::get_action_style(action.action_type);
        let icon = Self::get_action_icon(action.action_type);

        let action_line = Line::from(vec![
            Span::styled("  Action: ", THEME.footer()),
            Span::styled(
                format!("{} {}", icon, action.action_type.name()),
                action_style,
            ),
        ]);
        let action_para = Paragraph::new(action_line);
        frame.render_widget(action_para, chunks[2]);

        // Render amount if applicable
        if action.action_type.requires_amount() {
            if let Some(amount) = action.amount {
                let amount_str = format_mon_amount(amount);
                let amount_line = Line::from(vec![
                    Span::styled("  Amount: ", THEME.footer()),
                    Span::styled(format!("{} MON", amount_str), THEME.amount_positive()),
                ]);
                let amount_para = Paragraph::new(amount_line);
                frame.render_widget(amount_para, chunks[3]);
            }
        }

        // Render validator info
        let validator_line = Line::from(vec![
            Span::styled("  Validator: ", THEME.footer()),
            Span::styled(format!("#{}", action.validator_id), THEME.validator_id()),
        ]);
        let validator_para = Paragraph::new(validator_line);
        frame.render_widget(validator_para, chunks[4]);

        // Render warning message
        let warning_line = Line::styled(
            "  Warning: This action cannot be undone!",
            THEME.status_error(),
        );
        let warning_para = Paragraph::new(warning_line);
        frame.render_widget(warning_para, chunks[6]);

        // Render action hints with new theme styles
        let actions = Line::from(vec![
            Span::styled("[Enter]", THEME.keybind()),
            Span::styled(" Confirm  ", THEME.keybind_description()),
            Span::styled("[Esc]", THEME.keybind()),
            Span::styled(" Cancel", THEME.keybind_description()),
        ]);
        let actions_para = Paragraph::new(actions).alignment(Alignment::Center);
        frame.render_widget(actions_para, chunks[8]);
    }
}

/// Format MON amount from smallest unit (18 decimals) to display string
fn format_mon_amount(amount: u128) -> String {
    const DECIMALS: u32 = 18;
    let whole = amount / 10u128.pow(DECIMALS);
    let fractional = amount % 10u128.pow(DECIMALS);

    if fractional == 0 {
        format!("{}", whole)
    } else {
        // Format with up to 6 decimal places, trimming trailing zeros
        let frac_str = format!("{:018}", fractional);
        let trimmed = frac_str.trim_end_matches('0');
        let decimals = trimmed.chars().take(6).collect::<String>();
        if decimals.is_empty() {
            format!("{}", whole)
        } else {
            format!("{}.{}", whole, decimals)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirm_popup_state_default() {
        let state = ConfirmPopupState::default();
        assert!(!state.is_active);
        assert!(state.pending_action.is_none());
    }

    #[test]
    fn test_confirm_popup_state_new() {
        let state = ConfirmPopupState::new();
        assert!(!state.is_active());
        assert!(state.get_action().is_none());
    }

    #[test]
    fn test_confirm_popup_state_open() {
        let mut state = ConfirmPopupState::new();
        let action = PendingStakingAction::delegate(1, 1_000_000_000_000_000_000);
        state.open(action.clone());

        assert!(state.is_active());
        assert!(state.get_action().is_some());
        let stored = state.get_action().unwrap();
        assert_eq!(stored.action_type, StakingActionType::Delegate);
        assert_eq!(stored.validator_id, 1);
    }

    #[test]
    fn test_confirm_popup_state_close() {
        let mut state = ConfirmPopupState::new();
        let action = PendingStakingAction::delegate(1, 1000);
        state.open(action);
        state.close();

        assert!(!state.is_active());
        assert!(state.pending_action.is_none());
    }

    #[test]
    fn test_confirm_popup_state_take_action() {
        let mut state = ConfirmPopupState::new();
        let action = PendingStakingAction::delegate(42, 5000);
        state.open(action.clone());

        let taken = state.take_action();
        assert!(taken.is_some());
        assert_eq!(taken.unwrap().validator_id, 42);
        assert!(!state.is_active());
        assert!(state.pending_action.is_none());
    }

    #[test]
    fn test_confirm_popup_state_is_active_requires_both() {
        let mut state = ConfirmPopupState::new();

        // Neither active nor action
        assert!(!state.is_active());

        // Set active but no action
        state.is_active = true;
        assert!(!state.is_active());

        // Set action but not active
        state.is_active = false;
        state.pending_action = Some(PendingStakingAction::delegate(1, 100));
        assert!(!state.is_active());

        // Both set
        state.is_active = true;
        assert!(state.is_active());
    }

    #[test]
    fn test_confirm_popup_widget_creation() {
        let state = ConfirmPopupState::new();
        let widget = ConfirmPopupWidget::new(&state);
        assert_eq!(widget.width_percent, 60);
    }

    #[test]
    fn test_confirm_popup_widget_with_width() {
        let state = ConfirmPopupState::new();
        let widget = ConfirmPopupWidget::new(&state).with_width(80);
        assert_eq!(widget.width_percent, 80);
    }

    #[test]
    fn test_confirm_popup_widget_width_capped_at_100() {
        let state = ConfirmPopupState::new();
        let widget = ConfirmPopupWidget::new(&state).with_width(150);
        assert_eq!(widget.width_percent, 100);
    }

    #[test]
    fn test_get_action_icon() {
        assert_eq!(
            ConfirmPopupWidget::get_action_icon(StakingActionType::Delegate),
            "+"
        );
        assert_eq!(
            ConfirmPopupWidget::get_action_icon(StakingActionType::Undelegate),
            "-"
        );
        assert_eq!(
            ConfirmPopupWidget::get_action_icon(StakingActionType::Withdraw),
            ">"
        );
        assert_eq!(
            ConfirmPopupWidget::get_action_icon(StakingActionType::ClaimRewards),
            "*"
        );
        assert_eq!(
            ConfirmPopupWidget::get_action_icon(StakingActionType::Compound),
            "@"
        );
    }

    #[test]
    fn test_format_mon_amount_whole() {
        assert_eq!(format_mon_amount(1_000_000_000_000_000_000), "1");
        assert_eq!(format_mon_amount(10_000_000_000_000_000_000), "10");
    }

    #[test]
    fn test_format_mon_amount_fractional() {
        assert_eq!(format_mon_amount(1_500_000_000_000_000_000), "1.5");
        assert_eq!(format_mon_amount(1_234_567_000_000_000_000), "1.234567");
    }

    #[test]
    fn test_format_mon_amount_zero() {
        assert_eq!(format_mon_amount(0), "0");
    }

    #[test]
    fn test_confirm_popup_with_delegate_action() {
        let mut state = ConfirmPopupState::new();
        let action = PendingStakingAction::delegate(5, 2_500_000_000_000_000_000);
        state.open(action);

        assert!(state.is_active());
        let stored = state.get_action().unwrap();
        assert_eq!(stored.action_type, StakingActionType::Delegate);
        assert_eq!(stored.validator_id, 5);
        assert_eq!(stored.amount, Some(2_500_000_000_000_000_000));
    }

    #[test]
    fn test_confirm_popup_with_undelegate_action() {
        let mut state = ConfirmPopupState::new();
        let action = PendingStakingAction::undelegate(3, 1_000_000_000_000_000_000, 0);
        state.open(action);

        let stored = state.get_action().unwrap();
        assert_eq!(stored.action_type, StakingActionType::Undelegate);
        assert_eq!(stored.amount, Some(1_000_000_000_000_000_000));
        assert_eq!(stored.withdrawal_index, Some(0));
    }

    #[test]
    fn test_confirm_popup_with_claim_action() {
        let mut state = ConfirmPopupState::new();
        let action = PendingStakingAction::claim_rewards(10);
        state.open(action);

        let stored = state.get_action().unwrap();
        assert_eq!(stored.action_type, StakingActionType::ClaimRewards);
        assert_eq!(stored.validator_id, 10);
        assert!(stored.amount.is_none());
    }

    #[test]
    fn test_confirm_popup_with_compound_action() {
        let mut state = ConfirmPopupState::new();
        let action = PendingStakingAction::compound(7);
        state.open(action);

        let stored = state.get_action().unwrap();
        assert_eq!(stored.action_type, StakingActionType::Compound);
        assert_eq!(stored.validator_id, 7);
        assert!(stored.amount.is_none());
        assert!(stored.auth_address.is_none());
    }

    #[test]
    fn test_confirm_popup_with_withdraw_action() {
        let mut state = ConfirmPopupState::new();
        let action = PendingStakingAction::withdraw(2, 1);
        state.open(action);

        let stored = state.get_action().unwrap();
        assert_eq!(stored.action_type, StakingActionType::Withdraw);
        assert_eq!(stored.validator_id, 2);
        assert_eq!(stored.withdrawal_index, Some(1));
        assert!(stored.amount.is_none());
    }
}
