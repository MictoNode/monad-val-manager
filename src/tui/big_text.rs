//! Big Text Module - Giant MONAD Logo Display
//!
//! This module provides large text rendering:
//! - Giant MONAD logo for splash/header
//! - Custom fonts and styling
//! - Animated effects integration

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Paragraph, Widget},
};

/// Full MONAD logo - Complete MONAD text for splash screen
const MONAD_LOGO_FULL: &[&str] = &[
    "████  ████    █████  ███    ██  █████  ██████  ",
    "████  ████  ██    ██ ████   ██ ██   ██ ██   ██ ",
    "██ ████ ██  ██    ██ ██ ██  ██ ███████ ██   ██ ",
    "██  ██  ██  ██    ██ ██  ██ ██ ██   ██ ██   ██ ",
    "██      ██   ██████  ██   ████ ██   ██ ██████  ",
];

/// Giant M logo - Single M character for header
const GIANT_M_LOGO: &[&str] = &[
    "████   ███ ",
    "████  ████ ",
    "██ ████ ██ ",
    "██  ██  ██ ",
    "██      ██ ",
    "            ",
];

/// Compact M logo - Single M character for footer
const COMPACT_M_LOGO: &[&str] = &[
    "████   ███ ",
    "████  ████ ",
    "██ ████ ██ ",
    "██  ██  ██ ",
    "██      ██ ",
    "            ",
];

/// Big text configuration for different display modes
#[derive(Debug, Clone, Copy)]
pub enum BigTextMode {
    /// Full splash screen with animation
    Splash,
    /// Compact header mode
    Header,
    /// Minimal footer mode
    Footer,
}

impl BigTextMode {
    pub fn is_splash(&self) -> bool {
        matches!(self, BigTextMode::Splash)
    }

    pub fn is_header(&self) -> bool {
        matches!(self, BigTextMode::Header)
    }
}

/// MONAD logo renderer
#[derive(Debug, Clone)]
pub struct MonadLogo {
    pub mode: BigTextMode,
    pub show_subtitle: bool,
    pub style: Style,
}

impl MonadLogo {
    /// Create a new MONAD logo renderer
    pub fn new(mode: BigTextMode) -> Self {
        Self {
            mode,
            show_subtitle: true,
            style: Style::default()
                .fg(Color::Rgb(110, 84, 255)) // Monad purple
                .bg(Color::Rgb(14, 9, 28)), // Dark background
        }
    }

    /// Create a splash screen logo
    pub fn splash() -> Self {
        Self::new(BigTextMode::Splash)
    }

    /// Create a header logo
    pub fn header() -> Self {
        Self::new(BigTextMode::Header)
    }

    /// Set custom style
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set whether to show subtitle
    pub fn with_subtitle(mut self, show: bool) -> Self {
        self.show_subtitle = show;
        self
    }

    /// Get the logo text based on mode
    pub fn get_text(&self) -> String {
        // Return the MONAD brand name for all modes
        // This is the text representation of the logo
        "MONAD".to_string()
    }

    /// Get the subtitle text
    fn get_subtitle(&self) -> Option<String> {
        if self.show_subtitle {
            Some("Node Manager".to_string())
        } else {
            None
        }
    }
}

impl Widget for MonadLogo {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer)
    where
        Self: Sized,
    {
        // Select logo based on mode and available space
        let logo_lines = if self.mode.is_splash() && area.height >= 10 {
            MONAD_LOGO_FULL // Full MONAD logo for splash screen
        } else if self.mode.is_header() && area.height >= 6 {
            COMPACT_M_LOGO // Compact M for header
        } else if area.height >= 10 {
            MONAD_LOGO_FULL // Full MONAD logo when space permits
        } else if area.height >= 6 {
            GIANT_M_LOGO // Giant M when limited vertical space
        } else {
            COMPACT_M_LOGO // Compact M for very limited space
        };

        // Calculate logo dimensions
        let logo_height = logo_lines.len() as u16;
        let logo_width = logo_lines.first().map_or(0, |l| l.len() as u16);

        // Center the logo
        let x = if area.width > logo_width {
            (area.width - logo_width) / 2
        } else {
            0
        };

        let y = if area.height > logo_height + 2 {
            (area.height - logo_height - 2) / 2
        } else {
            0
        };

        let logo_area = Rect {
            x: area.x + x,
            y: area.y + y,
            width: logo_width.min(area.width),
            height: logo_height.min(area.height),
        };

        // Render each line of the logo
        for (i, line) in logo_lines.iter().enumerate() {
            let y_pos = logo_area.y + i as u16;
            if y_pos >= logo_area.bottom() {
                break;
            }

            let line_area = Rect {
                x: logo_area.x,
                y: y_pos,
                width: logo_area.width,
                height: 1,
            };

            let centered_line = if line.len() as u16 > logo_area.width {
                // Truncate if too long
                format!(
                    "{:<width$}",
                    &line[..logo_area.width.min(line.len() as u16) as usize],
                    width = logo_area.width as usize
                )
            } else {
                format!("{:^width$}", line, width = logo_area.width as usize)
            };

            Paragraph::new(Line::styled(
                centered_line,
                Style::default()
                    .fg(self.style.fg.unwrap_or(Color::Rgb(110, 84, 255)))
                    .add_modifier(ratatui::style::Modifier::BOLD),
            ))
            .render(line_area, buf);
        }

        // Render subtitle if enabled and there's space
        if let Some(subtitle) = self.get_subtitle() {
            let subtitle_y = logo_area.bottom().saturating_sub(1);
            if subtitle_y < area.bottom() && subtitle_y > logo_area.y {
                let subtitle_area = Rect {
                    x: area.x,
                    y: subtitle_y,
                    width: area.width,
                    height: 1,
                };

                let subtitle_style = Style::default()
                    .fg(Color::Rgb(221, 215, 254)) // Light purple
                    .bg(Color::Rgb(14, 9, 28));

                let centered_subtitle =
                    format!("{:^width$}", subtitle, width = area.width as usize);

                Line::styled(centered_subtitle, subtitle_style).render(subtitle_area, buf);
            }
        }
    }
}

/// Animated MONAD logo with gradient color
#[derive(Debug, Clone)]
pub struct AnimatedLogo {
    pub logo: MonadLogo,
    pub gradient_color: Color,
}

impl AnimatedLogo {
    /// Create a new animated logo
    pub fn new(mode: BigTextMode) -> Self {
        let logo = MonadLogo::new(mode);
        Self {
            logo,
            gradient_color: Color::Rgb(110, 84, 255),
        }
    }

    /// Update gradient color
    pub fn set_gradient(&mut self, color: Color) {
        self.gradient_color = color;
        self.logo = self
            .logo
            .clone()
            .with_style(Style::default().fg(color).bg(Color::Rgb(14, 9, 28)));
    }
}

impl Widget for AnimatedLogo {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer)
    where
        Self: Sized,
    {
        self.logo.render(area, buf);
    }
}

/// Welcome screen with MONAD logo
#[derive(Debug, Clone)]
pub struct WelcomeScreen {
    pub logo: AnimatedLogo,
    pub version: String,
}

impl WelcomeScreen {
    /// Create a new welcome screen
    pub fn new(version: &str) -> Self {
        Self {
            logo: AnimatedLogo::new(BigTextMode::Splash),
            version: format!("v{}", version),
        }
    }

    /// Update the logo gradient
    pub fn update_gradient(&mut self, color: Color) {
        self.logo.set_gradient(color);
    }
}

impl Widget for WelcomeScreen {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer)
    where
        Self: Sized,
    {
        // Clear background
        let _bg_style = Style::default().bg(Color::Rgb(14, 9, 28));
        ratatui::widgets::Clear.render(area, buf);

        // Render logo in center
        let logo_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: area.height.saturating_sub(2),
        };
        self.logo.render(logo_area, buf);

        // Render version at bottom
        let version_area = Rect {
            x: area.x,
            y: area.bottom().saturating_sub(1),
            width: area.width,
            height: 1,
        };

        let version_style = Style::default()
            .fg(Color::Rgb(136, 136, 136)) // Muted gray
            .bg(Color::Rgb(14, 9, 28));

        let centered_version = format!("{:^width$}", self.version, width = area.width as usize);
        ratatui::text::Line::styled(centered_version, version_style).render(version_area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monad_logo_creation() {
        let logo = MonadLogo::new(BigTextMode::Header);
        assert_eq!(logo.get_text(), "MONAD");
        assert!(logo.show_subtitle);
    }

    #[test]
    fn test_monad_logo_splash() {
        let logo = MonadLogo::splash();
        assert!(matches!(logo.mode, BigTextMode::Splash));
    }

    #[test]
    fn test_monad_logo_header() {
        let logo = MonadLogo::header();
        assert!(matches!(logo.mode, BigTextMode::Header));
    }

    #[test]
    fn test_monad_logo_with_style() {
        let logo =
            MonadLogo::new(BigTextMode::Footer).with_style(Style::default().fg(Color::White));
        assert_eq!(logo.style.fg, Some(Color::White));
    }

    #[test]
    fn test_monad_logo_with_subtitle() {
        let logo = MonadLogo::new(BigTextMode::Header).with_subtitle(false);
        assert!(!logo.show_subtitle);
        assert!(logo.get_subtitle().is_none());
    }

    #[test]
    fn test_animated_logo_creation() {
        let logo = AnimatedLogo::new(BigTextMode::Splash);
        assert_eq!(logo.gradient_color, Color::Rgb(110, 84, 255));
    }

    #[test]
    fn test_animated_logo_set_gradient() {
        let mut logo = AnimatedLogo::new(BigTextMode::Splash);
        logo.set_gradient(Color::Rgb(133, 230, 255));
        assert_eq!(logo.gradient_color, Color::Rgb(133, 230, 255));
    }

    #[test]
    fn test_welcome_screen_creation() {
        let screen = WelcomeScreen::new("1.0.0");
        assert_eq!(screen.version, "v1.0.0");
    }

    #[test]
    fn test_welcome_screen_update_gradient() {
        let mut screen = WelcomeScreen::new("1.0.0");
        screen.update_gradient(Color::Rgb(255, 142, 228));
        assert_eq!(screen.logo.gradient_color, Color::Rgb(255, 142, 228));
    }
}
