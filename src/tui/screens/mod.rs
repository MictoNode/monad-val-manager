//! TUI Screens - Screen-based navigation infrastructure
//!
//! This module provides the screen navigation system for the TUI.
//! Each screen represents a distinct view with its own rendering and input handling.

mod dashboard;
mod doctor;
mod help;
mod staking;
mod transfer;

pub use dashboard::DashboardScreen;
pub use doctor::DoctorScreen;
pub use help::HelpScreen;
pub use staking::StakingScreen;
pub use transfer::TransferScreen;

use ratatui::Frame;

use super::state::AppState;

/// Available screens in the TUI application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    /// Main dashboard with system, network, and validator widgets
    #[default]
    Dashboard,
    /// Staking operations screen
    Staking,
    /// Transfer screen for native MON transfers
    Transfer,
    /// Doctor diagnostics screen
    Doctor,
    /// Help and keybindings screen
    Help,
}

impl Screen {
    /// Get all screens in navigation order
    pub const fn all() -> [Screen; 5] {
        [
            Screen::Dashboard,
            Screen::Staking,
            Screen::Transfer,
            Screen::Doctor,
            Screen::Help,
        ]
    }

    /// Get the next screen in the navigation cycle
    pub fn next(self) -> Self {
        let screens = Self::all();
        let current_index = screens.iter().position(|&s| s == self).unwrap_or(0);
        let next_index = (current_index + 1) % screens.len();
        screens[next_index]
    }

    /// Get the previous screen in the navigation cycle
    pub fn prev(self) -> Self {
        let screens = Self::all();
        let current_index = screens.iter().position(|&s| s == self).unwrap_or(0);
        let prev_index = if current_index == 0 {
            screens.len() - 1
        } else {
            current_index - 1
        };
        screens[prev_index]
    }

    /// Get the display name for this screen
    pub fn name(self) -> &'static str {
        match self {
            Screen::Dashboard => "Dashboard",
            Screen::Staking => "Staking",
            Screen::Transfer => "Transfer",
            Screen::Doctor => "Doctor",
            Screen::Help => "Help",
        }
    }
}

/// Trait for screen implementations
///
/// Each screen must be able to render itself and handle its specific behavior.
pub trait ScreenRender {
    /// Render the screen content
    fn render(&self, frame: &mut Frame, state: &AppState);
}

/// Get a renderer for the given screen type
pub fn get_renderer(screen: Screen) -> Box<dyn ScreenRender> {
    match screen {
        Screen::Dashboard => Box::new(DashboardScreen),
        Screen::Staking => Box::new(StakingScreen),
        Screen::Transfer => Box::new(TransferScreen),
        Screen::Doctor => Box::new(DoctorScreen),
        Screen::Help => Box::new(HelpScreen),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_default() {
        let screen = Screen::default();
        assert_eq!(screen, Screen::Dashboard);
    }

    #[test]
    fn test_screen_next_cycle() {
        assert_eq!(Screen::Dashboard.next(), Screen::Staking);
        assert_eq!(Screen::Staking.next(), Screen::Transfer);
        assert_eq!(Screen::Transfer.next(), Screen::Doctor);
        assert_eq!(Screen::Doctor.next(), Screen::Help);
        assert_eq!(Screen::Help.next(), Screen::Dashboard);
    }

    #[test]
    fn test_screen_prev_cycle() {
        assert_eq!(Screen::Dashboard.prev(), Screen::Help);
        assert_eq!(Screen::Help.prev(), Screen::Doctor);
        assert_eq!(Screen::Doctor.prev(), Screen::Transfer);
        assert_eq!(Screen::Transfer.prev(), Screen::Staking);
        assert_eq!(Screen::Staking.prev(), Screen::Dashboard);
    }

    #[test]
    fn test_screen_names() {
        assert_eq!(Screen::Dashboard.name(), "Dashboard");
        assert_eq!(Screen::Staking.name(), "Staking");
        assert_eq!(Screen::Transfer.name(), "Transfer");
        assert_eq!(Screen::Doctor.name(), "Doctor");
        assert_eq!(Screen::Help.name(), "Help");
    }

    #[test]
    fn test_all_screens_count() {
        assert_eq!(Screen::all().len(), 5);
    }

    #[test]
    fn test_get_renderer_dashboard() {
        let renderer = get_renderer(Screen::Dashboard);
        let _state = AppState::new();
        // Just verify we can create a frame - renderer exists
        let _ = &*renderer;
    }

    #[test]
    fn test_screen_nav_cycle_wraps_correctly() {
        // Test that navigation wraps correctly after Query and Perf removal
        assert_eq!(Screen::Help.next(), Screen::Dashboard);
        assert_eq!(Screen::Dashboard.prev(), Screen::Help);
    }
}
