//! TUI Application module - Main dashboard with real-time updates and screen navigation
//!
//! This module provides the main TUI application with feature-based organization:
//! - `core` - Application creation and main event loop
//! - `handlers` - User input event handlers
//! - `doctor` - Doctor diagnostic check execution
//! - `refresh` - Data refresh operations
//! - `render` - UI rendering

mod core;
mod doctor;
mod handlers;
mod refresh;
mod render;

#[cfg(test)]
mod handlers_tests;

use std::time::{Duration, Instant};

use crate::config::Config;
use crate::rpc::RpcClient;
use crate::tui::screens::Screen;
use crate::tui::state::AppState;
use crate::tui::ToastManager;

/// TUI Application with screen navigation
pub struct TuiApp {
    #[allow(dead_code)] // Config stored for future use
    pub(crate) config: Config,
    pub(crate) state: AppState,
    pub(crate) current_screen: Screen,
    pub(crate) should_quit: bool,
    pub(crate) last_update: Instant,
    pub(crate) update_interval: Duration,
    pub(crate) rpc_client: Option<RpcClient>,
    pub(crate) toast_manager: ToastManager,
    // Track previous connection state for toast notifications
    pub(crate) was_connected: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Network as CliNetwork;

    #[test]
    fn test_tui_app_creation() {
        let config = Config::create_default(CliNetwork::Mainnet).unwrap();
        let app = TuiApp::new(&config);
        assert!(app.is_ok());
    }

    #[test]
    fn test_tui_app_state_initialized() {
        let config = Config::create_default(CliNetwork::Mainnet).unwrap();
        let app = TuiApp::new(&config).unwrap();
        assert!(app.state.is_loading);
        assert_eq!(app.state.validator.chain_id, 143);
    }

    #[test]
    fn test_tui_app_testnet() {
        let config = Config::create_default(CliNetwork::Testnet).unwrap();
        let app = TuiApp::new(&config).unwrap();
        assert_eq!(app.state.validator.chain_id, 10143);
    }

    #[test]
    fn test_update_interval() {
        let config = Config::create_default(CliNetwork::Mainnet).unwrap();
        let app = TuiApp::new(&config).unwrap();
        assert_eq!(app.update_interval, Duration::from_secs(1));
    }

    #[test]
    fn test_default_screen() {
        let config = Config::create_default(CliNetwork::Mainnet).unwrap();
        let app = TuiApp::new(&config).unwrap();
        assert_eq!(app.current_screen, Screen::Dashboard);
    }

    #[test]
    fn test_screen_navigation_next() {
        let config = Config::create_default(CliNetwork::Mainnet).unwrap();
        let mut app = TuiApp::new(&config).unwrap();
        assert_eq!(app.current_screen, Screen::Dashboard);

        // Simulate NextTab action
        app.current_screen = app.current_screen.next();
        assert_eq!(app.current_screen, Screen::Staking);
    }

    #[test]
    fn test_screen_navigation_prev() {
        let config = Config::create_default(CliNetwork::Mainnet).unwrap();
        let mut app = TuiApp::new(&config).unwrap();

        // Simulate PrevTab action - Dashboard prev should be Help (last screen after Account removal)
        app.current_screen = app.current_screen.prev();
        assert_eq!(app.current_screen, Screen::Help);
    }

    #[test]
    fn test_screen_cycle_completes() {
        let config = Config::create_default(CliNetwork::Mainnet).unwrap();
        let mut app = TuiApp::new(&config).unwrap();

        // Cycle through all screens (now 5 screens after Query and Perf removal)
        for expected in [
            Screen::Dashboard,
            Screen::Staking,
            Screen::Transfer,
            Screen::Doctor,
            Screen::Help,
        ] {
            assert_eq!(app.current_screen, expected);
            app.current_screen = app.current_screen.next();
        }

        // Should be back to Dashboard after cycling through all 5 screens
        assert_eq!(app.current_screen, Screen::Dashboard);
    }
}
