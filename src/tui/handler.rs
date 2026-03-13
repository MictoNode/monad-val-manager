//! TUI Event Handler

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle keyboard input for main application
pub fn handle_key_event(key: KeyEvent) -> Option<Action> {
    match (key.modifiers, key.code) {
        // Quit
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => Some(Action::Quit),
        (_, KeyCode::Char('q')) => Some(Action::Quit),

        // Navigation
        (_, KeyCode::Up) => Some(Action::Up),
        (_, KeyCode::Down) => Some(Action::Down),
        (_, KeyCode::Left) => Some(Action::Left),
        (_, KeyCode::Right) => Some(Action::Right),
        (_, KeyCode::Tab) => Some(Action::NextTab),
        (_, KeyCode::BackTab) => Some(Action::PrevTab),

        // Actions
        (_, KeyCode::Enter) => Some(Action::Select),
        (_, KeyCode::Char('r')) => Some(Action::Refresh),
        (_, KeyCode::Char('h')) => Some(Action::Help),
        (_, KeyCode::Esc) => Some(Action::Back),

        // Screen navigation via number keys
        (_, KeyCode::Char('1')) => Some(Action::GotoDashboard),
        (_, KeyCode::Char('2')) => Some(Action::GotoStaking),
        (_, KeyCode::Char('3')) => Some(Action::GotoTransfer),
        (_, KeyCode::Char('4')) => Some(Action::GotoDoctor),
        (_, KeyCode::Char('5')) => Some(Action::GotoHelp),

        _ => None,
    }
}

/// Handle keyboard input when input dialog is active
///
/// Returns Some(DialogAction) if the dialog should process this key,
/// None if the key should be handled by the main application.
pub fn handle_dialog_key_event(key: KeyEvent) -> Option<DialogAction> {
    match (key.modifiers, key.code) {
        // Confirm/Cancel
        (_, KeyCode::Enter) => Some(DialogAction::Confirm),
        (_, KeyCode::Esc) => Some(DialogAction::Cancel),

        // Tab navigation for multi-field dialogs
        (_, KeyCode::Tab) => Some(DialogAction::NextField),
        (KeyModifiers::SHIFT, KeyCode::BackTab) => Some(DialogAction::PrevField),

        // Cursor movement
        (_, KeyCode::Left) => Some(DialogAction::CursorLeft),
        (_, KeyCode::Right) => Some(DialogAction::CursorRight),
        (_, KeyCode::Home) => Some(DialogAction::CursorStart),
        (_, KeyCode::End) => Some(DialogAction::CursorEnd),

        // Editing (standard + alternative codes for terminal compatibility)
        (_, KeyCode::Backspace) | (KeyModifiers::NONE, KeyCode::Char('\x08')) => {
            Some(DialogAction::Backspace)
        }
        (_, KeyCode::Delete) => Some(DialogAction::Delete),

        // Shortcuts
        (KeyModifiers::CONTROL, KeyCode::Char('a')) => Some(DialogAction::SelectAll),
        (KeyModifiers::CONTROL, KeyCode::Char('w')) => Some(DialogAction::DeleteWord),

        // Character input (only without modifiers, after special chars to avoid catching control codes)
        (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(c)) => {
            Some(DialogAction::InputChar(c))
        }

        _ => None,
    }
}

/// Handle keyboard input when confirmation popup is active
///
/// Returns Some(ConfirmAction) if the popup should process this key,
/// None if the key should be handled by the main application.
pub fn handle_confirm_popup_key_event(key: KeyEvent) -> Option<ConfirmAction> {
    match key.code {
        // Confirm or Cancel
        KeyCode::Enter => Some(ConfirmAction::Confirm),
        KeyCode::Esc => Some(ConfirmAction::Cancel),
        _ => None,
    }
}

/// Handle staking-specific keyboard shortcuts
///
/// Returns the appropriate staking action or None.
pub fn handle_staking_key_event(key: KeyEvent) -> Option<StakingAction> {
    match (key.modifiers, key.code) {
        // Staking operations
        (KeyModifiers::NONE, KeyCode::Char('o')) => Some(StakingAction::OpenQueryDelegator),
        (KeyModifiers::NONE, KeyCode::Char('v')) => Some(StakingAction::OpenQueryValidator),
        (_, KeyCode::Char('d')) => Some(StakingAction::OpenDelegate),
        (_, KeyCode::Char('u')) => Some(StakingAction::OpenUndelegate),
        (_, KeyCode::Char('w')) => Some(StakingAction::OpenWithdraw),
        (_, KeyCode::Char('c')) => Some(StakingAction::OpenClaim),
        (_, KeyCode::Char('m')) => Some(StakingAction::OpenCompound),
        (_, KeyCode::Char('a')) => Some(StakingAction::OpenAddValidator),
        (KeyModifiers::NONE, KeyCode::Char('x')) => Some(StakingAction::OpenChangeCommission),

        // Navigation within delegation list
        (_, KeyCode::Up) => Some(StakingAction::SelectPrev),
        (_, KeyCode::Down) => Some(StakingAction::SelectNext),

        // Refresh staking data
        (_, KeyCode::Char('r')) => Some(StakingAction::Refresh),

        _ => None,
    }
}

/// Actions that can be triggered by keyboard input
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    Up,
    Down,
    Left,
    Right,
    NextTab,
    PrevTab,
    Select,
    Refresh,
    Help,
    Back,
    /// Navigate to Dashboard screen
    GotoDashboard,
    /// Navigate to Staking screen
    GotoStaking,
    /// Navigate to Transfer screen
    GotoTransfer,
    /// Navigate to Doctor screen
    GotoDoctor,
    /// Navigate to Help screen
    GotoHelp,
}

/// Dialog-specific actions for input handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogAction {
    /// Confirm the dialog (Enter)
    Confirm,
    /// Cancel the dialog (Esc)
    Cancel,
    /// Move to next field (Tab)
    NextField,
    /// Move to previous field (Shift+Tab)
    PrevField,
    /// Move cursor left
    CursorLeft,
    /// Move cursor right
    CursorRight,
    /// Move cursor to start
    CursorStart,
    /// Move cursor to end
    CursorEnd,
    /// Delete character before cursor
    Backspace,
    /// Delete character at cursor
    Delete,
    /// Select all (Ctrl+A)
    SelectAll,
    /// Delete word before cursor (Ctrl+W)
    DeleteWord,
    /// Input a character
    InputChar(char),
}

/// Confirmation popup actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmAction {
    /// Confirm the transaction (Enter)
    Confirm,
    /// Cancel the transaction (Esc)
    Cancel,
}

/// Staking screen-specific actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StakingAction {
    /// Open query delegator dialog
    OpenQueryDelegator,
    /// Open query validator dialog
    OpenQueryValidator,
    /// Open delegate dialog
    OpenDelegate,
    /// Open undelegate dialog
    OpenUndelegate,
    /// Open withdraw dialog
    OpenWithdraw,
    /// Open claim dialog
    OpenClaim,
    /// Open compound dialog
    OpenCompound,
    /// Open add validator dialog
    OpenAddValidator,
    /// Open change commission dialog
    OpenChangeCommission,
    /// Open transfer dialog
    OpenTransfer,
    /// Select previous delegation
    SelectPrev,
    /// Select next delegation
    SelectNext,
    /// Refresh staking data
    Refresh,
    /// Confirm dialog and execute action
    ConfirmAction,
    /// Cancel dialog
    CancelAction,
}

/// Doctor screen-specific actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoctorAction {
    /// Run diagnostics
    RunChecks,
    /// Select previous check
    SelectPrev,
    /// Select next check
    SelectNext,
}

/// Perf screen-specific actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerfAction {
    /// Refresh performance data
    Refresh,
}

/// Query screen-specific actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryAction {
    /// Select previous query type
    SelectPrev,
    /// Select next query type
    SelectNext,
    /// Execute selected query
    ExecuteQuery,
    /// Refresh query result
    Refresh,
    /// Clear result and return to menu
    ClearResult,
}

/// Transfer screen-specific actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferAction {
    /// Open transfer dialog
    OpenDialog,
}

/// Handle doctor-specific keyboard shortcuts
///
/// Returns the appropriate doctor action or None.
pub fn handle_doctor_key_event(key: KeyEvent) -> Option<DoctorAction> {
    match (key.modifiers, key.code) {
        // Run diagnostics
        (_, KeyCode::Char('r')) => Some(DoctorAction::RunChecks),

        // Navigation within check list
        (_, KeyCode::Up) => Some(DoctorAction::SelectPrev),
        (_, KeyCode::Down) => Some(DoctorAction::SelectNext),

        _ => None,
    }
}

/// Handle perf screen-specific keyboard shortcuts
///
/// Returns the appropriate perf action or None.
pub fn handle_perf_key_event(key: KeyEvent) -> Option<PerfAction> {
    match (key.modifiers, key.code) {
        // Refresh performance data
        (_, KeyCode::Char('r')) => Some(PerfAction::Refresh),

        _ => None,
    }
}

/// Handle query screen-specific keyboard shortcuts
///
/// Returns the appropriate query action or None.
pub fn handle_query_key_event(key: KeyEvent) -> Option<QueryAction> {
    match (key.modifiers, key.code) {
        // Navigation within query menu
        (_, KeyCode::Up) => Some(QueryAction::SelectPrev),
        (_, KeyCode::Down) => Some(QueryAction::SelectNext),

        // Execute query or refresh result
        (_, KeyCode::Enter) => Some(QueryAction::ExecuteQuery),
        (_, KeyCode::Char('r')) => Some(QueryAction::Refresh),

        // Clear result and return to menu
        (_, KeyCode::Esc) => Some(QueryAction::ClearResult),

        _ => None,
    }
}

/// Handle transfer screen-specific keyboard shortcuts
///
/// Returns the appropriate transfer action or None.
pub fn handle_transfer_key_event(key: KeyEvent) -> Option<TransferAction> {
    match (key.modifiers, key.code) {
        // Open transfer dialog
        (_, KeyCode::Char('t')) | (_, KeyCode::Char('T')) => Some(TransferAction::OpenDialog),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyEventKind;

    fn make_key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        }
    }

    #[test]
    fn test_quit_keys() {
        let ctrl_c = make_key(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(handle_key_event(ctrl_c), Some(Action::Quit));

        let q = make_key(KeyCode::Char('q'), KeyModifiers::empty());
        assert_eq!(handle_key_event(q), Some(Action::Quit));
    }

    #[test]
    fn test_navigation_keys() {
        assert_eq!(
            handle_key_event(make_key(KeyCode::Up, KeyModifiers::empty())),
            Some(Action::Up)
        );
        assert_eq!(
            handle_key_event(make_key(KeyCode::Down, KeyModifiers::empty())),
            Some(Action::Down)
        );
        assert_eq!(
            handle_key_event(make_key(KeyCode::Tab, KeyModifiers::empty())),
            Some(Action::NextTab)
        );
        assert_eq!(
            handle_key_event(make_key(KeyCode::BackTab, KeyModifiers::empty())),
            Some(Action::PrevTab)
        );
    }

    #[test]
    fn test_action_keys() {
        assert_eq!(
            handle_key_event(make_key(KeyCode::Enter, KeyModifiers::empty())),
            Some(Action::Select)
        );
        assert_eq!(
            handle_key_event(make_key(KeyCode::Char('r'), KeyModifiers::empty())),
            Some(Action::Refresh)
        );
        assert_eq!(
            handle_key_event(make_key(KeyCode::Char('h'), KeyModifiers::empty())),
            Some(Action::Help)
        );
        assert_eq!(
            handle_key_event(make_key(KeyCode::Esc, KeyModifiers::empty())),
            Some(Action::Back)
        );
    }

    #[test]
    fn test_dialog_actions() {
        assert_eq!(
            handle_dialog_key_event(make_key(KeyCode::Enter, KeyModifiers::empty())),
            Some(DialogAction::Confirm)
        );
        assert_eq!(
            handle_dialog_key_event(make_key(KeyCode::Esc, KeyModifiers::empty())),
            Some(DialogAction::Cancel)
        );
        assert_eq!(
            handle_dialog_key_event(make_key(KeyCode::Left, KeyModifiers::empty())),
            Some(DialogAction::CursorLeft)
        );
        assert_eq!(
            handle_dialog_key_event(make_key(KeyCode::Right, KeyModifiers::empty())),
            Some(DialogAction::CursorRight)
        );
        assert_eq!(
            handle_dialog_key_event(make_key(KeyCode::Backspace, KeyModifiers::empty())),
            Some(DialogAction::Backspace)
        );
        assert_eq!(
            handle_dialog_key_event(make_key(KeyCode::Delete, KeyModifiers::empty())),
            Some(DialogAction::Delete)
        );
    }

    #[test]
    fn test_dialog_char_input() {
        let result = handle_dialog_key_event(make_key(KeyCode::Char('a'), KeyModifiers::empty()));
        assert_eq!(result, Some(DialogAction::InputChar('a')));
    }

    #[test]
    fn test_staking_actions() {
        assert_eq!(
            handle_staking_key_event(make_key(KeyCode::Char('d'), KeyModifiers::empty())),
            Some(StakingAction::OpenDelegate)
        );
        assert_eq!(
            handle_staking_key_event(make_key(KeyCode::Char('u'), KeyModifiers::empty())),
            Some(StakingAction::OpenUndelegate)
        );
        assert_eq!(
            handle_staking_key_event(make_key(KeyCode::Char('w'), KeyModifiers::empty())),
            Some(StakingAction::OpenWithdraw)
        );
        assert_eq!(
            handle_staking_key_event(make_key(KeyCode::Char('c'), KeyModifiers::empty())),
            Some(StakingAction::OpenClaim)
        );
        assert_eq!(
            handle_staking_key_event(make_key(KeyCode::Char('m'), KeyModifiers::empty())),
            Some(StakingAction::OpenCompound)
        );
        assert_eq!(
            handle_staking_key_event(make_key(KeyCode::Char('a'), KeyModifiers::empty())),
            Some(StakingAction::OpenAddValidator)
        );
        assert_eq!(
            handle_staking_key_event(make_key(KeyCode::Char('o'), KeyModifiers::empty())),
            Some(StakingAction::OpenQueryDelegator)
        );
    }

    #[test]
    fn test_staking_navigation() {
        assert_eq!(
            handle_staking_key_event(make_key(KeyCode::Up, KeyModifiers::empty())),
            Some(StakingAction::SelectPrev)
        );
        assert_eq!(
            handle_staking_key_event(make_key(KeyCode::Down, KeyModifiers::empty())),
            Some(StakingAction::SelectNext)
        );
    }

    #[test]
    fn test_staking_refresh() {
        assert_eq!(
            handle_staking_key_event(make_key(KeyCode::Char('r'), KeyModifiers::empty())),
            Some(StakingAction::Refresh)
        );
    }

    #[test]
    fn test_staking_change_commission() {
        assert_eq!(
            handle_staking_key_event(make_key(KeyCode::Char('x'), KeyModifiers::empty())),
            Some(StakingAction::OpenChangeCommission)
        );
    }

    #[test]
    fn test_staking_action_variants() {
        // Ensure all variants exist and are Debug + Clone + Copy + PartialEq + Eq
        let actions = [
            StakingAction::OpenDelegate,
            StakingAction::OpenUndelegate,
            StakingAction::OpenWithdraw,
            StakingAction::OpenClaim,
            StakingAction::OpenCompound,
            StakingAction::OpenAddValidator,
            StakingAction::OpenChangeCommission,
            StakingAction::OpenTransfer,
            StakingAction::SelectPrev,
            StakingAction::SelectNext,
            StakingAction::Refresh,
            StakingAction::ConfirmAction,
            StakingAction::CancelAction,
        ];

        for action in actions {
            let cloned = action; // Copy types don't need clone
            assert_eq!(action, cloned);
            // Ensure Debug is implemented
            let _ = format!("{:?}", action);
        }
    }

    #[test]
    fn test_dialog_action_variants() {
        let actions = [
            DialogAction::Confirm,
            DialogAction::Cancel,
            DialogAction::CursorLeft,
            DialogAction::CursorRight,
            DialogAction::CursorStart,
            DialogAction::CursorEnd,
            DialogAction::Backspace,
            DialogAction::Delete,
            DialogAction::InputChar('x'),
        ];

        for action in actions {
            let cloned = action; // Copy types don't need clone
            assert_eq!(action, cloned);
            let _ = format!("{:?}", action);
        }
    }

    #[test]
    fn test_action_variants() {
        let actions = [
            Action::Quit,
            Action::Up,
            Action::Down,
            Action::Left,
            Action::Right,
            Action::NextTab,
            Action::PrevTab,
            Action::Select,
            Action::Refresh,
            Action::Help,
            Action::Back,
        ];

        for action in actions {
            let cloned = action; // Copy types don't need clone
            assert_eq!(action, cloned);
            let _ = format!("{:?}", action);
        }
    }

    #[test]
    fn test_confirm_popup_actions() {
        // Test Enter confirms
        assert_eq!(
            handle_confirm_popup_key_event(make_key(KeyCode::Enter, KeyModifiers::empty())),
            Some(ConfirmAction::Confirm)
        );

        // Test Esc cancels
        assert_eq!(
            handle_confirm_popup_key_event(make_key(KeyCode::Esc, KeyModifiers::empty())),
            Some(ConfirmAction::Cancel)
        );

        // Test other keys return None
        assert_eq!(
            handle_confirm_popup_key_event(make_key(KeyCode::Char('a'), KeyModifiers::empty())),
            None
        );
        assert_eq!(
            handle_confirm_popup_key_event(make_key(KeyCode::Up, KeyModifiers::empty())),
            None
        );
    }

    #[test]
    fn test_confirm_action_variants() {
        let actions = [ConfirmAction::Confirm, ConfirmAction::Cancel];

        for action in actions {
            let cloned = action; // Copy types don't need clone
            assert_eq!(action, cloned);
            let _ = format!("{:?}", action);
        }
    }

    #[test]
    fn test_doctor_actions() {
        // Test Run checks
        assert_eq!(
            handle_doctor_key_event(make_key(KeyCode::Char('r'), KeyModifiers::empty())),
            Some(DoctorAction::RunChecks)
        );

        // Test navigation
        assert_eq!(
            handle_doctor_key_event(make_key(KeyCode::Up, KeyModifiers::empty())),
            Some(DoctorAction::SelectPrev)
        );
        assert_eq!(
            handle_doctor_key_event(make_key(KeyCode::Down, KeyModifiers::empty())),
            Some(DoctorAction::SelectNext)
        );

        // Test other keys return None
        assert_eq!(
            handle_doctor_key_event(make_key(KeyCode::Char('a'), KeyModifiers::empty())),
            None
        );
    }

    #[test]
    fn test_doctor_action_variants() {
        let actions = [
            DoctorAction::RunChecks,
            DoctorAction::SelectPrev,
            DoctorAction::SelectNext,
        ];

        for action in actions {
            let cloned = action; // Copy types don't need clone
            assert_eq!(action, cloned);
            let _ = format!("{:?}", action);
        }
    }

    #[test]
    fn test_perf_actions() {
        // Test Refresh
        assert_eq!(
            handle_perf_key_event(make_key(KeyCode::Char('r'), KeyModifiers::empty())),
            Some(PerfAction::Refresh)
        );

        // Test other keys return None
        assert_eq!(
            handle_perf_key_event(make_key(KeyCode::Char('a'), KeyModifiers::empty())),
            None
        );
        assert_eq!(
            handle_perf_key_event(make_key(KeyCode::Up, KeyModifiers::empty())),
            None
        );
    }

    #[test]
    fn test_perf_action_variants() {
        let actions = [PerfAction::Refresh];

        for action in actions {
            let cloned = action; // Copy types don't need clone
            assert_eq!(action, cloned);
            let _ = format!("{:?}", action);
        }
    }

    #[test]
    fn test_unbound_numeric_keys_return_none() {
        // After Account removal, '8' and '9' keys should return None
        assert_eq!(
            handle_key_event(make_key(KeyCode::Char('8'), KeyModifiers::empty())),
            None
        );
        assert_eq!(
            handle_key_event(make_key(KeyCode::Char('9'), KeyModifiers::empty())),
            None
        );
    }

    #[test]
    fn test_transfer_action_open_dialog() {
        // Test lowercase t
        assert_eq!(
            handle_transfer_key_event(make_key(KeyCode::Char('t'), KeyModifiers::empty())),
            Some(TransferAction::OpenDialog)
        );

        // Test uppercase T
        assert_eq!(
            handle_transfer_key_event(make_key(KeyCode::Char('T'), KeyModifiers::empty())),
            Some(TransferAction::OpenDialog)
        );
    }

    #[test]
    fn test_transfer_other_keys_return_none() {
        assert_eq!(
            handle_transfer_key_event(make_key(KeyCode::Char('x'), KeyModifiers::empty())),
            None
        );
        assert_eq!(
            handle_transfer_key_event(make_key(KeyCode::Up, KeyModifiers::empty())),
            None
        );
    }

    #[test]
    fn test_transfer_action_variants() {
        let actions = [TransferAction::OpenDialog];

        for action in actions {
            let cloned = action;
            assert_eq!(action, cloned);
            let _ = format!("{:?}", action);
        }
    }
}
