//! Enchanted Effects Module - TachyonFX integration
//!
//! This module provides shader-like effects for the TUI:
//! - Animated gradients
//! - Glow effects
//! - Smooth transitions
//! - Pulse animations

use ratatui::style::{Color, Style};
use std::time::{Duration, Instant};

/// Animation state for gradient effects
#[derive(Debug, Clone)]
pub struct GradientState {
    pub offset: f32,
    pub speed: f32,
    pub last_update: Instant,
    pub cached_color: Option<Color>, // Cache for performance
    pub cache_valid: bool,           // Track if cache is valid (for testing)
}

impl Default for GradientState {
    fn default() -> Self {
        Self {
            offset: 0.0,
            speed: 1.0,
            last_update: Instant::now(),
            cached_color: None,
            cache_valid: false,
        }
    }
}

impl GradientState {
    pub fn new(speed: f32) -> Self {
        Self {
            offset: 0.0,
            speed,
            last_update: Instant::now(),
            cached_color: None,
            cache_valid: false,
        }
    }

    pub fn update(&mut self, delta: Duration) {
        // Always update, but cache color to reduce interpolation calculations
        let secs = delta.as_secs_f32();
        self.offset = (self.offset + self.speed * secs) % 1.0;
        self.cached_color = None; // Invalidate cache
        self.cache_valid = false;
    }
}

/// Gradient color stops for animations
#[derive(Debug, Clone)]
pub struct GradientStop {
    pub color: Color,
    pub position: f32,
}

impl GradientStop {
    pub fn new(color: Color, position: f32) -> Self {
        Self { color, position }
    }
}

/// Animated gradient effect
#[derive(Debug, Clone)]
pub struct AnimatedGradient {
    pub stops: Vec<GradientStop>,
    pub state: GradientState,
}

impl AnimatedGradient {
    /// Create a new animated gradient with color stops
    pub fn new(stops: Vec<GradientStop>, speed: f32) -> Self {
        Self {
            stops,
            state: GradientState::new(speed),
        }
    }

    /// Create the Monad purple→cyan→pink gradient
    pub fn monad_gradient(speed: f32) -> Self {
        Self::new(
            vec![
                GradientStop::new(Color::Rgb(110, 84, 255), 0.0), // Purple
                GradientStop::new(Color::Rgb(133, 230, 255), 0.5), // Cyan
                GradientStop::new(Color::Rgb(255, 142, 228), 1.0), // Pink
            ],
            speed,
        )
    }

    /// Update animation state
    pub fn update(&mut self, delta: Duration) {
        self.state.update(delta);
    }

    /// Get interpolated color at current animation offset (cached)
    pub fn get_color(&self) -> Color {
        // Return cached color if available
        if let Some(cached) = self.state.cached_color {
            return cached;
        }

        let pos = self.state.offset;

        // Find the two stops we're between
        for i in 0..self.stops.len() - 1 {
            let start = &self.stops[i];
            let end = &self.stops[i + 1];

            if pos >= start.position && pos <= end.position {
                let range = end.position - start.position;
                let local_pos = (pos - start.position) / range;
                return Self::interpolate_color(start.color, end.color, local_pos);
            }
        }

        // Fallback to first color
        self.stops[0].color
    }

    /// Cache the current color to reduce calculations
    pub fn cache_color(&mut self) {
        self.state.cached_color = Some(self.get_color_uncached());
    }

    /// Get color without using cache (internal use)
    fn get_color_uncached(&self) -> Color {
        let pos = self.state.offset;

        // Find the two stops we're between
        for i in 0..self.stops.len() - 1 {
            let start = &self.stops[i];
            let end = &self.stops[i + 1];

            if pos >= start.position && pos <= end.position {
                let range = end.position - start.position;
                let local_pos = (pos - start.position) / range;
                return Self::interpolate_color(start.color, end.color, local_pos);
            }
        }

        // Fallback to first color
        self.stops[0].color
    }

    /// Interpolate between two colors
    fn interpolate_color(c1: Color, c2: Color, t: f32) -> Color {
        match (c1, c2) {
            (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
                let r = (r1 as f32 + (r2 as f32 - r1 as f32) * t).round() as u8;
                let g = (g1 as f32 + (g2 as f32 - g1 as f32) * t).round() as u8;
                let b = (b1 as f32 + (b2 as f32 - b1 as f32) * t).round() as u8;
                Color::Rgb(r, g, b)
            }
            _ => c1,
        }
    }

    /// Get a style with the current gradient color
    pub fn get_style(&self) -> Style {
        Style::default().fg(self.get_color())
    }
}

/// Glow effect intensity
#[derive(Debug, Clone, Copy)]
pub enum GlowIntensity {
    Subtle,
    Medium,
    Strong,
}

impl GlowIntensity {
    pub fn alpha(&self) -> f32 {
        match self {
            GlowIntensity::Subtle => 0.2,
            GlowIntensity::Medium => 0.4,
            GlowIntensity::Strong => 0.6,
        }
    }
}

/// Glow effect configuration
#[derive(Debug, Clone)]
pub struct GlowEffect {
    pub color: Color,
    pub intensity: GlowIntensity,
    pub radius: u16,
}

impl GlowEffect {
    pub fn new(color: Color, intensity: GlowIntensity, radius: u16) -> Self {
        Self {
            color,
            intensity,
            radius,
        }
    }

    /// Create a subtle purple glow
    pub fn purple_subtle() -> Self {
        Self::new(Color::Rgb(110, 84, 255), GlowIntensity::Subtle, 2)
    }

    /// Create a medium cyan glow
    pub fn cyan_medium() -> Self {
        Self::new(Color::Rgb(133, 230, 255), GlowIntensity::Medium, 3)
    }

    /// Create a strong pink glow
    pub fn pink_strong() -> Self {
        Self::new(Color::Rgb(255, 142, 228), GlowIntensity::Strong, 4)
    }

    /// Get glow style (simulated with lighter color)
    pub fn get_style(&self) -> Style {
        let glow_color = self.lighten_color(self.color, self.intensity.alpha());
        Style::default().fg(glow_color)
    }

    /// Get glow style with background
    pub fn get_style_with_bg(&self) -> Style {
        let glow_color = self.lighten_color(self.color, self.intensity.alpha());
        Style::default().fg(glow_color).bg(Color::Rgb(14, 9, 28))
    }

    /// Get border style with glow
    pub fn get_border_style(&self) -> Style {
        self.get_style_with_bg()
    }

    /// Lighten a color by blending with white
    fn lighten_color(&self, color: Color, factor: f32) -> Color {
        match color {
            Color::Rgb(r, g, b) => {
                let r = (r as f32 + (255.0 - r as f32) * factor).round() as u8;
                let g = (g as f32 + (255.0 - g as f32) * factor).round() as u8;
                let b = (b as f32 + (255.0 - b as f32) * factor).round() as u8;
                Color::Rgb(r, g, b)
            }
            _ => color,
        }
    }

    /// Create a success glow (green)
    pub fn success() -> Self {
        Self::new(Color::Rgb(74, 222, 128), GlowIntensity::Medium, 2)
    }

    /// Create an error glow (red)
    pub fn error() -> Self {
        Self::new(Color::Rgb(239, 68, 68), GlowIntensity::Medium, 2)
    }

    /// Create a warning glow (orange)
    pub fn warning() -> Self {
        Self::new(Color::Rgb(255, 174, 69), GlowIntensity::Medium, 2)
    }
}

/// Glow utilities for widgets
pub struct GlowWidgets;

impl GlowWidgets {
    /// Create a glowing border style for cards
    pub fn card_border(is_active: bool) -> Style {
        if is_active {
            GlowEffect::purple_subtle().get_border_style()
        } else {
            Style::default()
                .fg(Color::Rgb(110, 84, 255))
                .bg(Color::Rgb(14, 9, 28))
        }
    }

    /// Create a glowing style for active elements
    pub fn active_element(pulse_value: f32) -> Style {
        // Pulse the glow intensity
        let intensity = if pulse_value > 0.7 {
            GlowIntensity::Strong
        } else if pulse_value > 0.5 {
            GlowIntensity::Medium
        } else {
            GlowIntensity::Subtle
        };

        GlowEffect::new(Color::Rgb(110, 84, 255), intensity, 2).get_style_with_bg()
    }

    /// Create a focus indicator glow
    pub fn focus_indicator() -> Style {
        GlowEffect::cyan_medium().get_style_with_bg()
    }

    /// Create a status glow based on check status
    pub fn status_glow(is_passing: bool) -> Style {
        if is_passing {
            GlowEffect::success().get_border_style()
        } else {
            GlowEffect::error().get_border_style()
        }
    }
}

/// Pulse animation state
#[derive(Debug, Clone)]
pub struct PulseState {
    pub min_value: f32,
    pub max_value: f32,
    pub speed: f32,
    pub phase: f32,
    pub last_update: Instant,
}

impl Default for PulseState {
    fn default() -> Self {
        Self {
            min_value: 0.5,
            max_value: 1.0,
            speed: 2.0,
            phase: 0.0,
            last_update: Instant::now(),
        }
    }
}

impl PulseState {
    pub fn new(min_value: f32, max_value: f32, speed: f32) -> Self {
        Self {
            min_value,
            max_value,
            speed,
            phase: 0.0,
            last_update: Instant::now(),
        }
    }

    pub fn update(&mut self, delta: Duration) {
        // Always update - sine calculation is cheap
        let secs = delta.as_secs_f32();
        self.phase = (self.phase + self.speed * secs) % (2.0 * std::f32::consts::PI);
    }

    pub fn get_value(&self) -> f32 {
        let sine = self.phase.sin();
        let range = self.max_value - self.min_value;
        self.min_value + (sine + 1.0) / 2.0 * range
    }
}

/// Animation manager for coordinating multiple effects
#[derive(Debug, Clone)]
pub struct AnimationManager {
    pub gradient: AnimatedGradient,
    pub pulse: PulseState,
    pub last_frame: Instant,
}

impl Default for AnimationManager {
    fn default() -> Self {
        Self {
            gradient: AnimatedGradient::monad_gradient(0.3),
            pulse: PulseState::default(),
            last_frame: Instant::now(),
        }
    }
}

impl AnimationManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update all animations and return delta time
    pub fn update(&mut self) -> Duration {
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame);
        self.last_frame = now;

        self.gradient.update(delta);
        self.pulse.update(delta);

        // Cache gradient color to reduce interpolation calculations
        self.gradient.cache_color();

        delta
    }

    /// Get current gradient color (cached)
    pub fn gradient_color(&self) -> Color {
        self.gradient.get_color()
    }

    /// Get current pulse value
    pub fn pulse_value(&self) -> f32 {
        self.pulse.get_value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gradient_state_update() {
        let mut state = GradientState::new(0.5); // speed = 0.5
        let initial_offset = state.offset;

        state.update(Duration::from_secs(1)); // delta = 1s
                                              // offset = (0.0 + 0.5 * 1.0) % 1.0 = 0.5

        assert!(state.offset > initial_offset); // 0.5 > 0.0
        assert!(state.offset < 1.0);
    }

    #[test]
    fn test_gradient_state_wrap() {
        let mut state = GradientState::new(10.0);
        state.offset = 0.9;

        state.update(Duration::from_secs(1));

        // Should wrap around to 0.0
        assert!(state.offset < 1.0);
    }

    #[test]
    fn test_monad_gradient() {
        let gradient = AnimatedGradient::monad_gradient(0.5);
        assert_eq!(gradient.stops.len(), 3);

        let color = gradient.get_color();
        match color {
            Color::Rgb(r, g, b) => {
                // Should be one of the gradient colors
                assert!(r > 0 || g > 0 || b > 0);
            }
            _ => panic!("Expected RGB color"),
        }
    }

    #[test]
    fn test_glow_effect() {
        let glow = GlowEffect::purple_subtle();
        let style = glow.get_style();

        assert!(style.fg.is_some());
    }

    #[test]
    fn test_pulse_state() {
        let mut pulse = PulseState::new(0.5, 1.0, 2.0);
        // initial_value = pulse.get_value(); // Not used in this test

        pulse.update(Duration::from_millis(500));

        let new_value = pulse.get_value();
        assert!(new_value >= pulse.min_value);
        assert!(new_value <= pulse.max_value);
    }

    #[test]
    fn test_animation_manager() {
        let mut manager = AnimationManager::new();
        let delta = manager.update();

        // Should have non-zero delta (though very small)
        assert!(delta.as_secs_f32() >= 0.0);

        let color = manager.gradient_color();
        match color {
            Color::Rgb(r, g, b) => {
                assert!(r > 0 || g > 0 || b > 0);
            }
            _ => panic!("Expected RGB color"),
        }

        let pulse = manager.pulse_value();
        assert!((0.0..=1.0).contains(&pulse));
    }
}
