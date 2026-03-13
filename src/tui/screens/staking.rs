//! Staking Screen - Display staking information and operations
//!
//! This screen provides:
//! - Delegator address and balance display
//! - List of delegations to validators (via DelegationListWidget)
//! - Pending withdrawals status
//! - Action hints for staking operations
//! - Input dialog overlay for staking operations
//! - Confirmation popup for transaction confirmation
//! - Add Validator dialog overlay

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::{Screen, ScreenRender};
use crate::tui::state::AppState;
use crate::tui::theme::THEME;
use crate::tui::widgets::{
    AddValidatorDialogWidget, ChangeCommissionDialogWidget, ConfirmPopupWidget,
    DelegateDialogWidget, DelegationListWidget, InputDialogWidget, NavMenuWidget,
    QueryDelegatorDialogWidget, QueryValidatorDialogWidget, UndelegateDialogWidget,
    WithdrawDialogWidget,
};

/// Staking screen for staking operations
pub struct StakingScreen;

impl StakingScreen {
    /// Create new staking screen
    pub fn new() -> Self {
        Self
    }
}

impl Default for StakingScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenRender for StakingScreen {
    fn render(&self, frame: &mut Frame, state: &AppState) {
        let area = frame.area();

        // Create layout: nav menu, account info, delegations, footer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Nav Menu (with branding)
                Constraint::Length(3), // Account info (address + balance)
                Constraint::Min(10),   // Delegations list
                Constraint::Length(3), // Combined footer (actions + nav)
            ])
            .split(area);

        // Navigation Menu
        let nav_menu = NavMenuWidget::new(Screen::Staking, state.tick);
        nav_menu.render(frame, chunks[0]);

        // Render account info
        self.render_account_info(frame, chunks[1], state);

        // Render delegations list using the widget
        let delegation_widget = DelegationListWidget::new(&state.staking);
        delegation_widget.render(frame, chunks[2]);

        // Render combined footer
        self.render_footer(frame, chunks[3], state);

        // Render input dialog overlay if active
        if state.input_dialog.is_active() {
            let dialog_widget = InputDialogWidget::new(&state.input_dialog);
            dialog_widget.render(frame);
        }

        // Render Add Validator dialog overlay if active
        if state.add_validator.is_active() {
            let add_validator_widget = AddValidatorDialogWidget::new(&state.add_validator);
            add_validator_widget.render(frame);
        }

        // Render Change Commission dialog overlay if active
        if state.change_commission.is_active() {
            let change_commission_widget =
                ChangeCommissionDialogWidget::new(&state.change_commission);
            change_commission_widget.render(frame);
        }

        // Render Query Delegator dialog overlay if active
        if state.query_delegator.is_active() {
            let query_delegator_widget = QueryDelegatorDialogWidget::new(&state.query_delegator);
            query_delegator_widget.render(frame);
        }

        // Render Query Validator dialog overlay if active
        if state.query_validator.is_active() {
            let query_validator_widget = QueryValidatorDialogWidget::new(&state.query_validator);
            query_validator_widget.render(frame);
        }

        // Render Delegate dialog overlay if active
        if state.delegate.is_active() {
            let delegate_widget = DelegateDialogWidget::new(&state.delegate);
            delegate_widget.render(frame);
        }

        // Render Undelegate dialog overlay if active
        if state.undelegate.is_active() {
            let undelegate_widget = UndelegateDialogWidget::new(&state.undelegate);
            undelegate_widget.render(frame);
        }

        // Render Withdraw dialog overlay if active
        if state.withdraw.is_active() {
            let withdraw_widget = WithdrawDialogWidget::new(&state.withdraw);
            withdraw_widget.render(frame);
        }

        // Render confirmation popup overlay if active (renders on top of input dialog)
        if state.confirm_popup.is_active() {
            let confirm_widget = ConfirmPopupWidget::new(&state.confirm_popup);
            confirm_widget.render(frame);
        }
    }
}

impl StakingScreen {
    /// Render account info (address and balance)
    fn render_account_info(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let staking = &state.staking;

        let content = if staking.is_loading {
            vec![Line::from(vec![Span::styled(
                "Loading...",
                THEME.status_warning(),
            )])]
        } else if let Some(ref error) = staking.error {
            vec![Line::from(vec![
                Span::styled("Error: ", THEME.status_error()),
                Span::styled(error, THEME.status_error()),
            ])]
        } else {
            vec![Line::from(vec![
                Span::styled("Addr: ", THEME.label()),
                Span::styled(staking.format_address(), THEME.address()),
                Span::raw("  "),
                Span::styled("Bal: ", THEME.label()),
                Span::styled(staking.format_balance().to_string(), THEME.balance()),
                Span::styled(" MON", THEME.unit()),
            ])]
        };

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(THEME.widget_border()),
            )
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, area);
    }

    /// Render combined footer with action hints and navigation
    fn render_footer(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        // FIX: Don't render the main staking footer when any dialog is active
        // Dialogs have their own footers, and showing both creates visual clutter
        let any_dialog_active = state.input_dialog.is_active()
            || state.add_validator.is_active()
            || state.change_commission.is_active()
            || state.query_delegator.is_active()
            || state.delegate.is_active()
            || state.undelegate.is_active()
            || state.withdraw.is_active()
            || state.transfer.is_active
            || state.confirm_popup.is_active();

        if any_dialog_active {
            // Render empty footer when dialog is active (dialog has its own footer)
            let empty_content = Line::from(vec![Span::raw("")]);
            let footer = Paragraph::new(empty_content)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(THEME.widget_border()),
                )
                .alignment(Alignment::Center);
            frame.render_widget(footer, area);
            return;
        }

        let staking = &state.staking;

        // Build footer content with withdrawals summary and action hints
        let ready_count = staking.ready_withdrawal_count();
        let withdrawal_text = if staking.pending_withdrawals.is_empty() {
            String::new()
        } else {
            format!(
                " | Withdrawals: {}/{}",
                ready_count,
                staking.pending_withdrawals.len()
            )
        };

        let content = Line::from(vec![
            Span::styled("[v]", THEME.keybind()),
            Span::styled(" Q.Validator ", THEME.keybind_description()),
            Span::styled("[o]", THEME.keybind()),
            Span::styled(" Q.Delegator ", THEME.keybind_description()),
            Span::styled("[d]", THEME.keybind()),
            Span::styled(" Delegate ", THEME.keybind_description()),
            Span::styled("[u]", THEME.keybind()),
            Span::styled(" Undelegate ", THEME.keybind_description()),
            Span::styled("[w]", THEME.keybind()),
            Span::styled(" Withdraw ", THEME.keybind_description()),
            Span::styled("[c]", THEME.keybind()),
            Span::styled(" Claim ", THEME.keybind_description()),
            Span::styled("[m]", THEME.keybind()),
            Span::styled(" Compound ", THEME.keybind_description()),
            Span::styled("[a]", THEME.keybind()),
            Span::styled(" AddVal ", THEME.keybind_description()),
            Span::styled("[x]", THEME.keybind()),
            Span::styled(" ChgComm ", THEME.keybind_description()),
            Span::styled("[Tab]", THEME.keybind()),
            Span::styled(" Next ", THEME.keybind_description()),
            Span::styled("[r]", THEME.keybind()),
            Span::styled(" Refresh ", THEME.keybind_description()),
            Span::styled("[Esc]", THEME.keybind()),
            Span::styled(" Quit", THEME.keybind_description()),
            Span::styled(withdrawal_text.as_str(), THEME.muted()),
        ]);

        let footer = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(THEME.widget_border()),
            )
            .alignment(Alignment::Center);

        frame.render_widget(footer, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::staking::{DelegationInfo, PendingWithdrawal};

    #[test]
    fn test_staking_screen_creation() {
        let screen = StakingScreen::new();
        let _ = &screen;
    }

    #[test]
    fn test_staking_screen_default() {
        let screen = StakingScreen;
        let _ = &screen;
    }

    #[test]
    fn test_staking_state_has_staking_field() {
        let state = AppState::new();
        assert!(state.staking.delegator_address.is_none());
        assert!(state.staking.delegations.is_empty());
    }

    #[test]
    fn test_staking_state_with_delegations() {
        let mut state = AppState::new();
        let mut staking = state.staking.clone();
        staking.set_delegations(vec![DelegationInfo::new(1), DelegationInfo::new(2)]);
        state.update_staking(staking);

        assert_eq!(state.staking.delegations.len(), 2);
    }

    #[test]
    fn test_staking_state_with_withdrawals() {
        let mut state = AppState::new();
        let mut staking = state.staking.clone();
        staking.set_withdrawals(vec![PendingWithdrawal::new(1, 1000, 100, 0)]);
        state.update_staking(staking);

        assert_eq!(state.staking.pending_withdrawals.len(), 1);
    }

    #[test]
    fn test_staking_state_format_balance() {
        let mut state = AppState::new();
        let mut staking = state.staking.clone();
        staking.set_balance(1.5); // 1.5 MON (now stored as f64)
        state.update_staking(staking);

        assert_eq!(state.staking.format_balance(), "1.5");
    }

    #[test]
    fn test_staking_state_navigation() {
        let mut state = AppState::new();
        let mut staking = state.staking.clone();
        staking.set_delegations(vec![
            DelegationInfo::new(1),
            DelegationInfo::new(2),
            DelegationInfo::new(3),
        ]);
        state.update_staking(staking);

        // Test selection
        assert_eq!(state.staking.selected_index, 0);
    }

    #[test]
    fn test_delegation_widget_integration() {
        let mut state = AppState::new();
        let mut staking = state.staking.clone();
        staking.set_delegations(vec![DelegationInfo::new(1), DelegationInfo::new(2)]);
        state.update_staking(staking);

        // Create widget with state - widget creation succeeds
        let _widget = DelegationListWidget::new(&state.staking);
        // Verify delegations are accessible through state
        assert_eq!(state.staking.delegations.len(), 2);
    }
}
