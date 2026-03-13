//! Monad Theme - Official Monad blockchain brand colors for TUI
//!
//! Brand Colors:
//! - Purple:      #6E54FF → RGB(110, 84, 255)   [main accent, titles, focus]
//! - Light Purple:#DDD7FE → RGB(221, 215, 254)   [secondary text, labels]
//! - Dark BG:     #0E091C → RGB(14, 9, 28)       [main background]
//! - Black:       #000000 → RGB(0, 0, 0)         [deep background]
//! - White:       #FFFFFF → RGB(255, 255, 255)   [primary text]
//!
//! Secondary:
//! - Cyan:        #85E6FF → RGB(133, 230, 255)   [status: synced, success, peers]
//! - Light Blue:  #B9E3F9 → RGB(185, 227, 249)   [info values, block numbers]
//! - Pink:        #FF8EE4 → RGB(255, 142, 228)   [warnings, highlights, selected]
//! - Orange:      #FFAE45 → RGB(255, 174, 69)    [errors, timeouts, alerts]

use ratatui::style::{Color, Modifier, Style};

/// Monad theme - Complete color palette for TUI
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    // Backgrounds
    /// #0E091C - Main background
    pub bg: Color,
    /// #000000 - Deep background
    pub bg_deep: Color,
    /// #1A1230 - Slightly lighter than bg for panels
    pub bg_panel: Color,

    // Text
    /// #FFFFFF - Primary text
    pub text_primary: Color,
    /// #DDD7FE - Secondary text, labels
    pub text_secondary: Color,
    /// #8888AA - Dimmed purple-white for hints
    pub text_muted: Color,

    // Accent
    /// #6E54FF - Main accent color
    pub accent: Color,
    /// #DDD7FE - Light accent
    pub accent_light: Color,

    // Semantic colors
    /// #85E6FF - Success, synced, peers
    pub success: Color,
    /// #B9E3F9 - Info values, block numbers
    pub info: Color,
    /// #FF8EE4 - Warnings, highlights, selected
    pub warning: Color,
    /// #FFAE45 - Errors, timeouts, alerts
    pub error: Color,

    // Borders
    /// #6E54FF dimmed - Default border
    pub border: Color,
    /// #6E54FF full - Focused border
    pub border_focused: Color,
    /// #85E6FF - Active/synced panels
    pub border_active: Color,

    // Interactive
    /// #6E54FF - Selected background
    pub selected_bg: Color,
    /// #FFFFFF - Selected foreground
    pub selected_fg: Color,
    /// #FF8EE4 - Keybind key color
    pub keybind_key: Color,
    /// #DDD7FE - Keybind description
    pub keybind_desc: Color,
    /// #6E54FF - Active tab background
    pub tab_active_bg: Color,
    /// #FFFFFF - Active tab foreground
    pub tab_active_fg: Color,
    /// #8888AA - Inactive tab foreground
    pub tab_inactive_fg: Color,
}

impl Theme {
    /// Create Monad theme with official brand colors
    pub const fn monad() -> Self {
        Theme {
            // Backgrounds
            bg: Color::Rgb(14, 9, 28),        // #0E091C
            bg_deep: Color::Rgb(0, 0, 0),     // #000000
            bg_panel: Color::Rgb(26, 18, 48), // #1A1230 (slightly lighter)

            // Text
            text_primary: Color::Rgb(255, 255, 255), // #FFFFFF
            text_secondary: Color::Rgb(221, 215, 254), // #DDD7FE
            text_muted: Color::Rgb(136, 136, 170),   // #8888AA

            // Accent
            accent: Color::Rgb(110, 84, 255),        // #6E54FF
            accent_light: Color::Rgb(221, 215, 254), // #DDD7FE

            // Semantic
            success: Color::Rgb(133, 230, 255), // #85E6FF
            info: Color::Rgb(185, 227, 249),    // #B9E3F9
            warning: Color::Rgb(255, 142, 228), // #FF8EE4
            error: Color::Rgb(255, 174, 69),    // #FFAE45

            // Borders
            border: Color::Rgb(110, 84, 255),         // #6E54FF
            border_focused: Color::Rgb(110, 84, 255), // #6E54FF
            border_active: Color::Rgb(133, 230, 255), // #85E6FF

            // Interactive
            selected_bg: Color::Rgb(110, 84, 255),    // #6E54FF
            selected_fg: Color::Rgb(255, 255, 255),   // #FFFFFF
            keybind_key: Color::Rgb(255, 142, 228),   // #FF8EE4
            keybind_desc: Color::Rgb(221, 215, 254),  // #DDD7FE
            tab_active_bg: Color::Rgb(110, 84, 255),  // #6E54FF
            tab_active_fg: Color::Rgb(255, 255, 255), // #FFFFFF
            tab_inactive_fg: Color::Rgb(136, 136, 170), // #8888AA
        }
    }

    // ==================== BASE STYLES ====================

    /// Header style with brand background
    pub fn header(&self) -> Style {
        Style::default()
            .fg(self.text_primary)
            .bg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    /// Footer style
    pub fn footer(&self) -> Style {
        Style::default().fg(self.text_muted)
    }

    // ==================== TAB STYLES ====================

    /// Active tab style
    pub fn tab_active(&self) -> Style {
        Style::default()
            .fg(self.tab_active_fg)
            .bg(self.tab_active_bg)
            .add_modifier(Modifier::BOLD)
    }

    /// Inactive tab style
    pub fn tab_inactive(&self) -> Style {
        Style::default().fg(self.tab_inactive_fg)
    }

    /// Tab number style (pink)
    pub fn tab_number(&self) -> Style {
        Style::default()
            .fg(self.warning)
            .add_modifier(Modifier::BOLD)
    }

    // ==================== WIDGET STYLES ====================

    /// Widget title style (brand color, bold)
    pub fn widget_title(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    /// Default widget border
    pub fn widget_border(&self) -> Style {
        Style::default().fg(self.border)
    }

    /// Focused widget border
    pub fn widget_border_focused(&self) -> Style {
        Style::default()
            .fg(self.border_focused)
            .add_modifier(Modifier::BOLD)
    }

    /// Active/synced panel border
    pub fn widget_border_active(&self) -> Style {
        Style::default().fg(self.border_active)
    }

    /// Error state border
    pub fn widget_border_error(&self) -> Style {
        Style::default().fg(self.error).add_modifier(Modifier::BOLD)
    }

    // ==================== TEXT STYLES ====================

    /// Primary text style
    pub fn text(&self) -> Style {
        Style::default().fg(self.text_primary)
    }

    /// Secondary text/labels style
    pub fn label(&self) -> Style {
        Style::default().fg(self.text_secondary)
    }

    /// Muted/hint text style
    pub fn muted(&self) -> Style {
        Style::default().fg(self.text_muted)
    }

    /// Placeholder text style (italic)
    pub fn placeholder(&self) -> Style {
        Style::default()
            .fg(self.text_muted)
            .add_modifier(Modifier::ITALIC)
    }

    // ==================== SEMANTIC STYLES ====================

    /// Success status style (cyan, bold)
    pub fn status_success(&self) -> Style {
        Style::default()
            .fg(self.success)
            .add_modifier(Modifier::BOLD)
    }

    /// Info style (light blue)
    pub fn status_info(&self) -> Style {
        Style::default().fg(self.info).add_modifier(Modifier::BOLD)
    }

    /// Warning status style (pink, bold)
    pub fn status_warning(&self) -> Style {
        Style::default()
            .fg(self.warning)
            .add_modifier(Modifier::BOLD)
    }

    /// Error status style (orange, bold)
    pub fn status_error(&self) -> Style {
        Style::default().fg(self.error).add_modifier(Modifier::BOLD)
    }

    // ==================== METRIC STYLES ====================

    /// Metric label style
    pub fn metric_label(&self) -> Style {
        Style::default().fg(self.text_secondary)
    }

    /// Metric value style (light blue, bold)
    pub fn metric_value(&self) -> Style {
        Style::default().fg(self.info).add_modifier(Modifier::BOLD)
    }

    /// Block number style (light blue)
    pub fn block_number(&self) -> Style {
        Style::default().fg(self.info).add_modifier(Modifier::BOLD)
    }

    /// Epoch value style (brand purple, bold)
    pub fn epoch(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    /// Peers count style (cyan)
    pub fn peers_count(&self) -> Style {
        Style::default()
            .fg(self.success)
            .add_modifier(Modifier::BOLD)
    }

    /// Validator ID style (brand purple)
    pub fn validator_id(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    // ==================== STAKING STYLES ====================

    /// Balance value style (cyan, bold)
    pub fn balance(&self) -> Style {
        Style::default()
            .fg(self.success)
            .add_modifier(Modifier::BOLD)
    }

    /// MON unit style (brand purple)
    pub fn unit(&self) -> Style {
        Style::default().fg(self.accent)
    }

    /// Amount style (cyan)
    pub fn amount(&self) -> Style {
        Style::default()
            .fg(self.success)
            .add_modifier(Modifier::BOLD)
    }

    /// Rewards style (pink)
    pub fn rewards(&self) -> Style {
        Style::default()
            .fg(self.warning)
            .add_modifier(Modifier::BOLD)
    }

    /// Address style (light purple)
    pub fn address(&self) -> Style {
        Style::default().fg(self.text_secondary)
    }

    // ==================== KEYBIND STYLES ====================

    /// Keybind key style (pink, bold)
    pub fn keybind(&self) -> Style {
        Style::default()
            .fg(self.keybind_key)
            .add_modifier(Modifier::BOLD)
    }

    /// Keybind description style
    pub fn keybind_description(&self) -> Style {
        Style::default().fg(self.keybind_desc)
    }

    // ==================== SELECTION STYLES ====================

    /// Selected row style
    pub fn selected(&self) -> Style {
        Style::default().fg(self.selected_fg).bg(self.selected_bg)
    }

    /// Selected row bold style
    pub fn selected_bold(&self) -> Style {
        Style::default()
            .fg(self.selected_fg)
            .bg(self.selected_bg)
            .add_modifier(Modifier::BOLD)
    }

    // ==================== INPUT STYLES ====================

    /// Input field border
    pub fn input_border(&self) -> Style {
        Style::default().fg(self.border)
    }

    /// Input field focused border (cyan)
    pub fn input_border_focused(&self) -> Style {
        Style::default()
            .fg(self.success)
            .add_modifier(Modifier::BOLD)
    }

    /// Input text style
    pub fn input_text(&self) -> Style {
        Style::default().fg(self.text_primary)
    }

    /// Input cursor style (pink block)
    pub fn input_cursor(&self) -> Style {
        Style::default().fg(self.warning).bg(self.warning)
    }

    /// Input error style
    pub fn input_error(&self) -> Style {
        Style::default().fg(self.error).add_modifier(Modifier::BOLD)
    }

    // ==================== BUTTON STYLES ====================

    /// Button style (brand bg)
    pub fn button(&self) -> Style {
        Style::default()
            .fg(self.text_primary)
            .bg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    /// Button hover style
    pub fn button_hover(&self) -> Style {
        Style::default()
            .fg(self.bg)
            .bg(self.accent_light)
            .add_modifier(Modifier::BOLD)
    }

    // ==================== BADGE STYLES ====================

    /// Network badge style (brand bg)
    pub fn badge_network(&self) -> Style {
        Style::default()
            .fg(self.text_primary)
            .bg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    /// Validator type badge style (pink bg)
    pub fn badge_validator(&self) -> Style {
        Style::default()
            .fg(self.bg)
            .bg(self.warning)
            .add_modifier(Modifier::BOLD)
    }

    // ==================== PROGRESS STYLES ====================

    /// Progress bar fill
    pub fn progress_fill(&self) -> Style {
        Style::default().fg(self.accent)
    }

    /// Progress bar background
    pub fn progress_bg(&self) -> Style {
        Style::default().bg(self.bg_panel)
    }

    // ==================== GRADIENTS ====================

    /// Get color for CPU usage
    /// <50%: cyan, 50-80%: pink, >80%: orange
    pub fn cpu_gradient(&self, usage: f32) -> Color {
        if usage < 50.0 {
            self.success // #85E6FF - Good
        } else if usage < 80.0 {
            self.warning // #FF8EE4 - Medium
        } else {
            self.error // #FFAE45 - High
        }
    }

    /// Get color for memory usage
    pub fn memory_gradient(&self, usage: f64) -> Color {
        if usage < 50.0 {
            self.success
        } else if usage < 80.0 {
            self.warning
        } else {
            self.error
        }
    }

    /// Get color for disk usage
    pub fn disk_gradient(&self, usage: f64) -> Color {
        if usage < 70.0 {
            self.success
        } else if usage < 85.0 {
            self.warning
        } else {
            self.error
        }
    }

    /// Get status icon and style for connection
    pub fn connection_status(&self, connected: bool, syncing: bool) -> (&'static str, Style) {
        match (connected, syncing) {
            (true, false) => ("●", self.status_success()),
            (true, true) => ("●", self.status_warning()),
            (false, _) => ("●", self.status_error()),
        }
    }

    /// Get check status icon and style
    pub fn check_status(
        &self,
        status: crate::tui::doctor_state::CheckStatus,
    ) -> (&'static str, Style) {
        use crate::tui::doctor_state::CheckStatus;
        match status {
            CheckStatus::Pass => ("✓", self.status_success()),
            CheckStatus::Fail => ("✗", self.status_error()),
            CheckStatus::Error => ("⚠", self.status_warning()),
            CheckStatus::Pending => ("○", Style::default().fg(self.text_muted)),
            CheckStatus::Running => ("⟳", self.status_info()),
        }
    }

    /// Last update style - pink if >5s, cyan if fresh
    pub fn last_update(&self, seconds_ago: u64) -> Style {
        if seconds_ago > 5 {
            self.status_warning()
        } else {
            self.status_success()
        }
    }

    // ==================== DIALOG STYLES ====================

    /// Dialog border style
    pub fn dialog_border(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    /// Dialog title style
    pub fn dialog_title(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    // ==================== BACKWARD COMPATIBILITY ====================
    /// Legacy alias for tab_active
    pub fn active_tab(&self) -> Style {
        self.tab_active()
    }

    /// Legacy alias for tab_inactive
    pub fn inactive_tab(&self) -> Style {
        self.tab_inactive()
    }

    /// Legacy alias for keybind
    pub fn action_hint(&self) -> Style {
        self.keybind()
    }

    /// Legacy alias for muted
    pub fn action_hint_secondary(&self) -> Style {
        self.muted()
    }

    /// Legacy alias for amount
    pub fn amount_positive(&self) -> Style {
        self.amount()
    }

    /// Legacy method for negative amounts
    pub fn amount_negative(&self) -> Style {
        self.status_error()
    }

    /// Legacy accessor for brand color
    pub fn brand_color(&self) -> Color {
        self.accent
    }

    /// Legacy gradient method (uses network gradient)
    pub fn network_gradient(&self) -> Color {
        self.success
    }
}

/// Global theme instance
pub const THEME: Theme = Theme::monad();

/// Legacy alias for backward compatibility
#[deprecated(note = "Use THEME instead")]
pub const MONAD_THEME: Theme = THEME;

impl Default for Theme {
    fn default() -> Self {
        Self::monad()
    }
}

// ==================== TESTS ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_colors() {
        let theme = Theme::monad();
        assert_eq!(theme.bg, Color::Rgb(14, 9, 28));
        assert_eq!(theme.accent, Color::Rgb(110, 84, 255));
        assert_eq!(theme.success, Color::Rgb(133, 230, 255));
        assert_eq!(theme.error, Color::Rgb(255, 174, 69));
    }

    #[test]
    fn test_header_style() {
        let style = THEME.header();
        assert_eq!(style.fg.unwrap(), THEME.text_primary);
        assert_eq!(style.bg.unwrap(), THEME.accent);
    }

    #[test]
    fn test_tab_active() {
        let style = THEME.tab_active();
        assert_eq!(style.fg.unwrap(), THEME.tab_active_fg);
        assert_eq!(style.bg.unwrap(), THEME.tab_active_bg);
    }

    #[test]
    fn test_cpu_gradient() {
        let low = THEME.cpu_gradient(25.0);
        let mid = THEME.cpu_gradient(60.0);
        let high = THEME.cpu_gradient(90.0);
        assert_eq!(low, THEME.success);
        assert_eq!(mid, THEME.warning);
        assert_eq!(high, THEME.error);
    }

    #[test]
    fn test_default() {
        let theme = Theme::default();
        assert_eq!(theme.accent, THEME.accent);
    }

    #[test]
    fn test_global_theme() {
        assert_eq!(THEME.bg, Color::Rgb(14, 9, 28));
        assert_eq!(THEME.accent, Color::Rgb(110, 84, 255));
    }
}
