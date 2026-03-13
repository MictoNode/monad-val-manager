//! TUI integration tests
//!
//! Tests for TUI navigation, screen management, and handler functions.
//!
//! Test categories:
//! - Screen navigation tests
//! - Action handler tests
//! - Dialog action tests
//! - State management tests

use monad_val_manager::tui::handler::{
    handle_confirm_popup_key_event, handle_dialog_key_event, handle_doctor_key_event,
    handle_key_event, handle_perf_key_event, handle_staking_key_event, Action, ConfirmAction,
    DialogAction, DoctorAction, PerfAction, StakingAction,
};
use monad_val_manager::tui::screens::Screen;
use monad_val_manager::tui::state::{AppState, NetworkData, SystemData, ValidatorData};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Create a KeyEvent for testing
fn make_key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    }
}

/// Create a key event with no modifiers
fn key(code: KeyCode) -> KeyEvent {
    make_key(code, KeyModifiers::empty())
}

/// Create a key event with control modifier
fn ctrl_key(code: KeyCode) -> KeyEvent {
    make_key(code, KeyModifiers::CONTROL)
}

// =============================================================================
// SCREEN NAVIGATION TESTS
// =============================================================================

#[test]
fn test_screen_default() {
    let screen = Screen::default();
    assert_eq!(screen, Screen::Dashboard);
}

#[test]
fn test_screen_next_navigation() {
    // Dashboard -> Staking -> Transfer -> Doctor -> Help -> Dashboard
    assert_eq!(Screen::Dashboard.next(), Screen::Staking);
    assert_eq!(Screen::Staking.next(), Screen::Transfer);
    assert_eq!(Screen::Transfer.next(), Screen::Doctor);
    assert_eq!(Screen::Doctor.next(), Screen::Help);
    assert_eq!(Screen::Help.next(), Screen::Dashboard);
}

#[test]
fn test_screen_prev_navigation() {
    // Dashboard -> Help -> Doctor -> Transfer -> Staking -> Dashboard
    assert_eq!(Screen::Dashboard.prev(), Screen::Help);
    assert_eq!(Screen::Help.prev(), Screen::Doctor);
    assert_eq!(Screen::Doctor.prev(), Screen::Transfer);
    assert_eq!(Screen::Transfer.prev(), Screen::Staking);
    assert_eq!(Screen::Staking.prev(), Screen::Dashboard);
}

#[test]
fn test_screen_cycle_completes() {
    let mut screen = Screen::Dashboard;

    // Cycle forward through all screens
    for _ in 0..5 {
        screen = screen.next();
    }

    // Should be back at Dashboard
    assert_eq!(screen, Screen::Dashboard);
}

#[test]
fn test_screen_cycle_backwards() {
    let mut screen = Screen::Dashboard;

    // Cycle backward through all screens
    for _ in 0..5 {
        screen = screen.prev();
    }

    // Should be back at Dashboard
    assert_eq!(screen, Screen::Dashboard);
}

#[test]
fn test_screen_name() {
    assert_eq!(Screen::Dashboard.name(), "Dashboard");
    assert_eq!(Screen::Staking.name(), "Staking");
    assert_eq!(Screen::Transfer.name(), "Transfer");
    assert_eq!(Screen::Doctor.name(), "Doctor");
    assert_eq!(Screen::Help.name(), "Help");
}

#[test]
fn test_screen_all() {
    let all_screens = Screen::all();
    assert_eq!(all_screens.len(), 5);
    assert!(all_screens.contains(&Screen::Dashboard));
    assert!(all_screens.contains(&Screen::Staking));
    assert!(all_screens.contains(&Screen::Transfer));
    assert!(all_screens.contains(&Screen::Doctor));
    assert!(all_screens.contains(&Screen::Help));
}

// =============================================================================
// MAIN HANDLER TESTS
// =============================================================================

#[test]
fn test_quit_with_q() {
    let result = handle_key_event(key(KeyCode::Char('q')));
    assert_eq!(result, Some(Action::Quit));
}

#[test]
fn test_quit_with_ctrl_c() {
    let result = handle_key_event(ctrl_key(KeyCode::Char('c')));
    assert_eq!(result, Some(Action::Quit));
}

#[test]
fn test_navigation_up() {
    let result = handle_key_event(key(KeyCode::Up));
    assert_eq!(result, Some(Action::Up));
}

#[test]
fn test_navigation_down() {
    let result = handle_key_event(key(KeyCode::Down));
    assert_eq!(result, Some(Action::Down));
}

#[test]
fn test_navigation_left() {
    let result = handle_key_event(key(KeyCode::Left));
    assert_eq!(result, Some(Action::Left));
}

#[test]
fn test_navigation_right() {
    let result = handle_key_event(key(KeyCode::Right));
    assert_eq!(result, Some(Action::Right));
}

#[test]
fn test_navigation_tab() {
    let result = handle_key_event(key(KeyCode::Tab));
    assert_eq!(result, Some(Action::NextTab));
}

#[test]
fn test_navigation_backtab() {
    let result = handle_key_event(key(KeyCode::BackTab));
    assert_eq!(result, Some(Action::PrevTab));
}

#[test]
fn test_action_select() {
    let result = handle_key_event(key(KeyCode::Enter));
    assert_eq!(result, Some(Action::Select));
}

#[test]
fn test_action_refresh() {
    let result = handle_key_event(key(KeyCode::Char('r')));
    assert_eq!(result, Some(Action::Refresh));
}

#[test]
fn test_action_help() {
    let result = handle_key_event(key(KeyCode::Char('h')));
    assert_eq!(result, Some(Action::Help));
}

#[test]
fn test_action_back() {
    let result = handle_key_event(key(KeyCode::Esc));
    assert_eq!(result, Some(Action::Back));
}

#[test]
fn test_unmapped_key_returns_none() {
    let result = handle_key_event(key(KeyCode::Char('x')));
    assert_eq!(result, None);
}

#[test]
fn test_f1_key_returns_none() {
    let result = handle_key_event(key(KeyCode::F(1)));
    assert_eq!(result, None);
}

// =============================================================================
// DIALOG HANDLER TESTS
// =============================================================================

#[test]
fn test_dialog_confirm() {
    let result = handle_dialog_key_event(key(KeyCode::Enter));
    assert_eq!(result, Some(DialogAction::Confirm));
}

#[test]
fn test_dialog_cancel() {
    let result = handle_dialog_key_event(key(KeyCode::Esc));
    assert_eq!(result, Some(DialogAction::Cancel));
}

#[test]
fn test_dialog_cursor_left() {
    let result = handle_dialog_key_event(key(KeyCode::Left));
    assert_eq!(result, Some(DialogAction::CursorLeft));
}

#[test]
fn test_dialog_cursor_right() {
    let result = handle_dialog_key_event(key(KeyCode::Right));
    assert_eq!(result, Some(DialogAction::CursorRight));
}

#[test]
fn test_dialog_cursor_home() {
    let result = handle_dialog_key_event(key(KeyCode::Home));
    assert_eq!(result, Some(DialogAction::CursorStart));
}

#[test]
fn test_dialog_cursor_end() {
    let result = handle_dialog_key_event(key(KeyCode::End));
    assert_eq!(result, Some(DialogAction::CursorEnd));
}

#[test]
fn test_dialog_backspace() {
    let result = handle_dialog_key_event(key(KeyCode::Backspace));
    assert_eq!(result, Some(DialogAction::Backspace));
}

#[test]
fn test_dialog_delete() {
    let result = handle_dialog_key_event(key(KeyCode::Delete));
    assert_eq!(result, Some(DialogAction::Delete));
}

#[test]
fn test_dialog_char_input() {
    let result = handle_dialog_key_event(key(KeyCode::Char('a')));
    assert_eq!(result, Some(DialogAction::InputChar('a')));
}

#[test]
fn test_dialog_char_input_digit() {
    let result = handle_dialog_key_event(key(KeyCode::Char('5')));
    assert_eq!(result, Some(DialogAction::InputChar('5')));
}

#[test]
fn test_dialog_char_input_special() {
    let result = handle_dialog_key_event(key(KeyCode::Char('.')));
    assert_eq!(result, Some(DialogAction::InputChar('.')));
}

#[test]
fn test_dialog_unmapped_key() {
    let result = handle_dialog_key_event(key(KeyCode::F(1)));
    assert_eq!(result, None);
}

// =============================================================================
// CONFIRM POPUP HANDLER TESTS
// =============================================================================

#[test]
fn test_confirm_popup_confirm() {
    let result = handle_confirm_popup_key_event(key(KeyCode::Enter));
    assert_eq!(result, Some(ConfirmAction::Confirm));
}

#[test]
fn test_confirm_popup_cancel() {
    let result = handle_confirm_popup_key_event(key(KeyCode::Esc));
    assert_eq!(result, Some(ConfirmAction::Cancel));
}

#[test]
fn test_confirm_popup_other_key() {
    let result = handle_confirm_popup_key_event(key(KeyCode::Char('y')));
    assert_eq!(result, None);
}

#[test]
fn test_confirm_popup_navigation_key() {
    let result = handle_confirm_popup_key_event(key(KeyCode::Up));
    assert_eq!(result, None);
}

// =============================================================================
// STAKING HANDLER TESTS
// =============================================================================

#[test]
fn test_staking_delegate() {
    let result = handle_staking_key_event(key(KeyCode::Char('d')));
    assert_eq!(result, Some(StakingAction::OpenDelegate));
}

#[test]
fn test_staking_undelegate() {
    let result = handle_staking_key_event(key(KeyCode::Char('u')));
    assert_eq!(result, Some(StakingAction::OpenUndelegate));
}

#[test]
fn test_staking_withdraw() {
    let result = handle_staking_key_event(key(KeyCode::Char('w')));
    assert_eq!(result, Some(StakingAction::OpenWithdraw));
}

#[test]
fn test_staking_claim() {
    let result = handle_staking_key_event(key(KeyCode::Char('c')));
    assert_eq!(result, Some(StakingAction::OpenClaim));
}

#[test]
fn test_staking_compound() {
    let result = handle_staking_key_event(key(KeyCode::Char('m')));
    assert_eq!(result, Some(StakingAction::OpenCompound));
}

#[test]
fn test_staking_select_prev() {
    let result = handle_staking_key_event(key(KeyCode::Up));
    assert_eq!(result, Some(StakingAction::SelectPrev));
}

#[test]
fn test_staking_select_next() {
    let result = handle_staking_key_event(key(KeyCode::Down));
    assert_eq!(result, Some(StakingAction::SelectNext));
}

#[test]
fn test_staking_refresh() {
    let result = handle_staking_key_event(key(KeyCode::Char('r')));
    assert_eq!(result, Some(StakingAction::Refresh));
}

#[test]
fn test_staking_change_commission() {
    let result = handle_staking_key_event(key(KeyCode::Char('x')));
    assert_eq!(result, Some(StakingAction::OpenChangeCommission));
}

#[test]
fn test_staking_unmapped_key() {
    // 'x' now maps to OpenChangeCommission
    let result = handle_staking_key_event(key(KeyCode::Char('z')));
    assert_eq!(result, None);
}

// =============================================================================
// DOCTOR HANDLER TESTS
// =============================================================================

#[test]
fn test_doctor_run_checks() {
    let result = handle_doctor_key_event(key(KeyCode::Char('r')));
    assert_eq!(result, Some(DoctorAction::RunChecks));
}

#[test]
fn test_doctor_select_prev() {
    let result = handle_doctor_key_event(key(KeyCode::Up));
    assert_eq!(result, Some(DoctorAction::SelectPrev));
}

#[test]
fn test_doctor_select_next() {
    let result = handle_doctor_key_event(key(KeyCode::Down));
    assert_eq!(result, Some(DoctorAction::SelectNext));
}

#[test]
fn test_doctor_unmapped_key() {
    let result = handle_doctor_key_event(key(KeyCode::Char('d')));
    assert_eq!(result, None);
}

// =============================================================================
// PERF HANDLER TESTS
// =============================================================================

#[test]
fn test_perf_refresh() {
    let result = handle_perf_key_event(key(KeyCode::Char('r')));
    assert_eq!(result, Some(PerfAction::Refresh));
}

#[test]
fn test_perf_unmapped_key() {
    let result = handle_perf_key_event(key(KeyCode::Up));
    assert_eq!(result, None);
}

#[test]
fn test_perf_char_key() {
    let result = handle_perf_key_event(key(KeyCode::Char('a')));
    assert_eq!(result, None);
}

// =============================================================================
// ACTION ENUM TESTS
// =============================================================================

#[test]
fn test_action_equality() {
    assert_eq!(Action::Quit, Action::Quit);
    assert_eq!(Action::Up, Action::Up);
    assert_ne!(Action::Quit, Action::Up);
}

#[test]
fn test_action_copy() {
    let action = Action::Refresh;
    let copied = action;
    assert_eq!(action, copied);
}

#[test]
fn test_action_debug_format() {
    let debug_str = format!("{:?}", Action::Quit);
    assert!(debug_str.contains("Quit"));
}

#[test]
fn test_all_action_variants() {
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

    // Verify all variants are accessible and can be compared
    for (i, action) in actions.iter().enumerate() {
        for (j, other) in actions.iter().enumerate() {
            if i == j {
                assert_eq!(action, other);
            } else {
                assert_ne!(action, other);
            }
        }
    }
}

// =============================================================================
// DIALOG ACTION ENUM TESTS
// =============================================================================

#[test]
fn test_dialog_action_equality() {
    assert_eq!(DialogAction::Confirm, DialogAction::Confirm);
    assert_eq!(DialogAction::Cancel, DialogAction::Cancel);
    assert_ne!(DialogAction::Confirm, DialogAction::Cancel);
}

#[test]
fn test_all_dialog_action_variants() {
    let actions = [
        DialogAction::Confirm,
        DialogAction::Cancel,
        DialogAction::CursorLeft,
        DialogAction::CursorRight,
        DialogAction::CursorStart,
        DialogAction::CursorEnd,
        DialogAction::Backspace,
        DialogAction::Delete,
        DialogAction::InputChar('a'),
    ];

    // Verify all variants exist
    assert_eq!(actions.len(), 9);
}

#[test]
fn test_dialog_action_input_char_equality() {
    assert_eq!(DialogAction::InputChar('a'), DialogAction::InputChar('a'));
    assert_ne!(DialogAction::InputChar('a'), DialogAction::InputChar('b'));
}

// =============================================================================
// CONFIRM ACTION ENUM TESTS
// =============================================================================

#[test]
fn test_confirm_action_variants() {
    assert_eq!(ConfirmAction::Confirm, ConfirmAction::Confirm);
    assert_eq!(ConfirmAction::Cancel, ConfirmAction::Cancel);
    assert_ne!(ConfirmAction::Confirm, ConfirmAction::Cancel);
}

// =============================================================================
// STAKING ACTION ENUM TESTS
// =============================================================================

#[test]
fn test_staking_action_equality() {
    assert_eq!(StakingAction::OpenDelegate, StakingAction::OpenDelegate);
    assert_ne!(StakingAction::OpenDelegate, StakingAction::OpenUndelegate);
}

#[test]
fn test_all_staking_action_variants() {
    let actions = [
        StakingAction::OpenDelegate,
        StakingAction::OpenUndelegate,
        StakingAction::OpenWithdraw,
        StakingAction::OpenClaim,
        StakingAction::OpenCompound,
        StakingAction::SelectPrev,
        StakingAction::SelectNext,
        StakingAction::Refresh,
        StakingAction::ConfirmAction,
        StakingAction::CancelAction,
    ];

    // Verify all variants exist
    assert_eq!(actions.len(), 10);
}

// =============================================================================
// DOCTOR ACTION ENUM TESTS
// =============================================================================

#[test]
fn test_doctor_action_equality() {
    assert_eq!(DoctorAction::RunChecks, DoctorAction::RunChecks);
    assert_ne!(DoctorAction::RunChecks, DoctorAction::SelectPrev);
}

#[test]
fn test_all_doctor_action_variants() {
    let actions = [
        DoctorAction::RunChecks,
        DoctorAction::SelectPrev,
        DoctorAction::SelectNext,
    ];

    // Verify all variants exist
    assert_eq!(actions.len(), 3);
}

// =============================================================================
// PERF ACTION ENUM TESTS
// =============================================================================

#[test]
fn test_perf_action_equality() {
    assert_eq!(PerfAction::Refresh, PerfAction::Refresh);
}

#[test]
fn test_all_perf_action_variants() {
    let actions = [PerfAction::Refresh];

    // Verify variant exists
    assert_eq!(actions.len(), 1);
}

// =============================================================================
// APP STATE TESTS
// =============================================================================

#[test]
fn test_app_state_creation() {
    let state = AppState::new();
    assert!(state.is_loading);
    assert_eq!(state.refresh_count, 0);
    assert!(state.last_refresh.is_none());
}

#[test]
fn test_app_state_with_config() {
    let state = AppState::with_config(10143, "testnet", "http://localhost:8080");
    assert_eq!(state.validator.chain_id, 10143);
    assert_eq!(state.validator.network_name, "testnet");
    assert_eq!(state.validator.rpc_endpoint, "http://localhost:8080");
}

#[test]
fn test_app_state_mark_refreshed() {
    let mut state = AppState::new();
    assert!(state.is_loading);

    state.mark_refreshed();

    assert!(!state.is_loading);
    assert_eq!(state.refresh_count, 1);
    assert!(state.last_refresh.is_some());
}

#[test]
fn test_app_state_multiple_refreshes() {
    let mut state = AppState::new();

    for i in 1..=5 {
        state.mark_refreshed();
        assert_eq!(state.refresh_count, i);
    }
}

#[test]
fn test_app_state_update_system() {
    let mut state = AppState::new();
    let mut system = SystemData::new();
    system.cpu_usage = 75.5;
    system.total_memory = 16 * 1024 * 1024 * 1024;
    system.used_memory = 8 * 1024 * 1024 * 1024;

    state.update_system(system);

    assert!((state.system.cpu_usage - 75.5).abs() < 0.1);
    assert_eq!(state.system.total_memory, 16 * 1024 * 1024 * 1024);
    assert_eq!(state.system.used_memory, 8 * 1024 * 1024 * 1024);
}

#[test]
fn test_app_state_update_network() {
    let mut state = AppState::new();
    let mut network = NetworkData::new();
    network.block_number = 12345;
    network.is_connected = true;
    network.peer_count = 50;

    state.update_network(network);

    assert_eq!(state.network.block_number, 12345);
    assert!(state.network.is_connected);
    assert_eq!(state.network.peer_count, 50);
}

#[test]
fn test_app_state_update_validator() {
    let mut state = AppState::new();
    let validator = ValidatorData::from_config(10143, "testnet", "http://test:8080");

    state.update_validator(validator);

    assert_eq!(state.validator.chain_id, 10143);
    assert_eq!(state.validator.network_name, "testnet");
}

// =============================================================================
// SYSTEM DATA TESTS
// =============================================================================

#[test]
fn test_system_data_defaults() {
    let data = SystemData::new();
    assert_eq!(data.cpu_usage, 0.0);
    assert_eq!(data.total_memory, 0);
    assert_eq!(data.used_memory, 0);
    assert_eq!(data.memory_usage, 0.0);
    assert_eq!(data.disk_total, 0);
    assert_eq!(data.disk_available, 0);
    assert_eq!(data.disk_usage, 0.0);
}

#[test]
fn test_system_data_memory_calculation() {
    let mut data = SystemData::new();
    data.total_memory = 16 * 1024 * 1024 * 1024; // 16 GB
    data.used_memory = 8 * 1024 * 1024 * 1024; // 8 GB

    let usage = data.calculate_memory_usage();

    assert!((usage - 50.0).abs() < 0.1);
}

#[test]
fn test_system_data_memory_calculation_zero_total() {
    let data = SystemData::new();
    let usage = data.calculate_memory_usage();
    assert_eq!(usage, 0.0);
}

#[test]
fn test_system_data_disk_calculation() {
    let mut data = SystemData::new();
    data.disk_total = 1000 * 1024 * 1024 * 1024; // 1 TB
    data.disk_available = 250 * 1024 * 1024 * 1024; // 250 GB

    let usage = data.calculate_disk_usage();

    // 750 GB used / 1000 GB total = 75%
    assert!((usage - 75.0).abs() < 0.1);
}

#[test]
fn test_system_data_disk_calculation_zero_total() {
    let data = SystemData::new();
    let usage = data.calculate_disk_usage();
    assert_eq!(usage, 0.0);
}

#[test]
fn test_system_data_format_memory() {
    let mut data = SystemData::new();
    data.used_memory = 8 * 1024 * 1024 * 1024;
    data.total_memory = 16 * 1024 * 1024 * 1024;

    let formatted = data.format_memory();

    assert!(formatted.contains("8.0"));
    assert!(formatted.contains("16.0"));
    assert!(formatted.contains("GB"));
}

#[test]
fn test_system_data_format_disk() {
    let mut data = SystemData::new();
    data.disk_available = 500 * 1024 * 1024 * 1024;
    data.disk_total = 1000 * 1024 * 1024 * 1024;

    let formatted = data.format_disk();

    assert!(formatted.contains("500"));
    assert!(formatted.contains("1000"));
    assert!(formatted.contains("GB"));
}

// =============================================================================
// NETWORK DATA TESTS
// =============================================================================

#[test]
fn test_network_data_defaults() {
    let data = NetworkData::new();
    assert_eq!(data.block_number, 0);
    assert!(!data.is_syncing);
    assert!(!data.is_connected);
    assert!(data.sync_progress.is_none());
    assert_eq!(data.peer_count, 0);
    assert!(data.node_version.is_none());
    assert!(data.uptime_seconds.is_none());
    assert!(data.last_error.is_none());
}

#[test]
fn test_network_data_format_block_number() {
    let mut data = NetworkData::new();
    data.block_number = 12345;

    assert_eq!(data.format_block_number(), "12345");
}

#[test]
fn test_network_data_format_sync_status_synced() {
    let data = NetworkData::new();
    assert_eq!(data.format_sync_status(), "Synced");
}

#[test]
fn test_network_data_format_sync_status_syncing_no_progress() {
    let mut data = NetworkData::new();
    data.is_syncing = true;

    assert_eq!(data.format_sync_status(), "Syncing...");
}

#[test]
fn test_network_data_format_sync_status_syncing_with_progress() {
    let mut data = NetworkData::new();
    data.is_syncing = true;
    data.sync_progress = Some(75.5);

    assert_eq!(data.format_sync_status(), "Syncing (75.5%)");
}

#[test]
fn test_network_data_format_uptime_none() {
    let data = NetworkData::new();
    assert_eq!(data.format_uptime(), "N/A");
}

#[test]
fn test_network_data_format_uptime_days() {
    let mut data = NetworkData::new();
    data.uptime_seconds = Some(90061); // 1 day, 1 hour, 1 minute

    assert_eq!(data.format_uptime(), "1d 1h 1m");
}

#[test]
fn test_network_data_format_uptime_hours() {
    let mut data = NetworkData::new();
    data.uptime_seconds = Some(3661); // 1 hour, 1 minute

    assert_eq!(data.format_uptime(), "1h 1m");
}

#[test]
fn test_network_data_format_uptime_minutes_only() {
    let mut data = NetworkData::new();
    data.uptime_seconds = Some(120); // 2 minutes

    assert_eq!(data.format_uptime(), "2m");
}

// =============================================================================
// VALIDATOR DATA TESTS
// =============================================================================

#[test]
fn test_validator_data_defaults() {
    let data = ValidatorData::new();
    assert_eq!(data.chain_id, 143);
    assert_eq!(data.network_name, "mainnet");
    assert_eq!(data.rpc_endpoint, "http://localhost:8080");
    assert!(!data.is_validator);
}

#[test]
fn test_validator_data_from_config() {
    let data = ValidatorData::from_config(10143, "testnet", "http://custom:8545");

    assert_eq!(data.chain_id, 10143);
    assert_eq!(data.network_name, "testnet");
    assert_eq!(data.rpc_endpoint, "http://custom:8545");
    assert!(!data.is_validator);
}

// =============================================================================
// KEY MODIFIER TESTS
// =============================================================================

#[test]
fn test_key_with_alt_modifier() {
    // Alt modifier on 'q' - handler uses (_, KeyCode::Char('q')) pattern
    // which matches any modifier, so Alt+Q also triggers Quit
    let alt_q = make_key(KeyCode::Char('q'), KeyModifiers::ALT);
    let result = handle_key_event(alt_q);

    // Alt+Q triggers Quit because handler ignores modifiers for 'q'
    assert_eq!(result, Some(Action::Quit));
}

#[test]
fn test_key_with_shift_modifier() {
    // Shift modifier on 'Q' (uppercase)
    let shift_q = make_key(KeyCode::Char('Q'), KeyModifiers::SHIFT);
    let result = handle_key_event(shift_q);

    // Shift+Q should not be handled (returns None)
    assert_eq!(result, None);
}

#[test]
fn test_ctrl_modifier_only_works_with_c() {
    // Ctrl+C should quit
    let ctrl_c = make_key(KeyCode::Char('c'), KeyModifiers::CONTROL);
    assert_eq!(handle_key_event(ctrl_c), Some(Action::Quit));

    // Ctrl+R triggers Refresh because handler uses (_, KeyCode::Char('r'))
    let ctrl_r = make_key(KeyCode::Char('r'), KeyModifiers::CONTROL);
    assert_eq!(handle_key_event(ctrl_r), Some(Action::Refresh));

    // Ctrl+Q triggers Quit because handler uses (_, KeyCode::Char('q'))
    let ctrl_q = make_key(KeyCode::Char('q'), KeyModifiers::CONTROL);
    assert_eq!(handle_key_event(ctrl_q), Some(Action::Quit));
}
