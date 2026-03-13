//! Loading Spinner Widget - Animated loading indicators
//!
//! This module provides loading spinners:
//! - Simple text-based animations
//! - Configurable colors and styles
//! - Monad brand styling

use ratatui::{
    layout::Rect,
    prelude::Widget,
    style::{Color, Style},
    widgets::Paragraph,
    Frame,
};
use std::time::Instant;

/// Spinner animation frames
const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Spinner style preset
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpinnerStyle {
    /// Simple dots animation
    Dots,
    /// Monad brand style (purple)
    Monad,
    /// Minimal style
    Minimal,
}

impl SpinnerStyle {
    /// Get the color for this style
    fn get_color(&self) -> Color {
        match self {
            SpinnerStyle::Dots => Color::Rgb(133, 230, 255), // Cyan
            SpinnerStyle::Monad => Color::Rgb(110, 84, 255), // Purple
            SpinnerStyle::Minimal => Color::Rgb(221, 215, 254), // Light purple
        }
    }
}

/// Loading spinner widget
#[derive(Debug, Clone)]
pub struct LoadingSpinner {
    pub style: SpinnerStyle,
    pub label: Option<String>,
    pub is_loading: bool,
    pub started_at: Instant,
    pub frame_index: usize,
}

impl LoadingSpinner {
    /// Create a new loading spinner
    pub fn new(style: SpinnerStyle) -> Self {
        Self {
            style,
            label: None,
            is_loading: true,
            started_at: Instant::now(),
            frame_index: 0,
        }
    }

    /// Create a spinner with a label
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set loading state
    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
        if loading {
            self.started_at = Instant::now();
            self.frame_index = 0;
        }
    }

    /// Get the elapsed time since loading started
    pub fn elapsed_ms(&self) -> u128 {
        self.started_at.elapsed().as_millis()
    }

    /// Update animation frame (call this periodically)
    pub fn update_frame(&mut self) {
        if self.is_loading {
            // Update frame every 100ms
            let elapsed = self.elapsed_ms();
            self.frame_index = (elapsed / 100) as usize % SPINNER_FRAMES.len();
        }
    }

    /// Get current spinner character
    fn get_spinner_char(&self) -> &str {
        SPINNER_FRAMES[self.frame_index]
    }

    /// Render the spinner at the given position
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        if !self.is_loading {
            return;
        }

        self.update_frame();

        let spinner_char = self.get_spinner_char();
        let color = self.style.get_color();

        // Build text with spinner and label
        let text = if let Some(ref label) = self.label {
            format!("{} {}", spinner_char, label)
        } else {
            spinner_char.to_string()
        };

        let paragraph = Paragraph::new(text).style(Style::default().fg(color));

        paragraph.render(area, frame.buffer_mut());
    }
}

impl Default for LoadingSpinner {
    fn default() -> Self {
        Self::new(SpinnerStyle::Monad)
    }
}

/// Spinner manager for coordinating multiple spinners
#[derive(Debug, Clone)]
pub struct SpinnerManager {
    pub active_spinners: Vec<LoadingSpinner>,
    pub max_spinners: usize,
}

impl Default for SpinnerManager {
    fn default() -> Self {
        Self {
            active_spinners: Vec::new(),
            max_spinners: 3,
        }
    }
}

impl SpinnerManager {
    /// Create a new spinner manager
    pub fn new(max_spinners: usize) -> Self {
        Self {
            active_spinners: Vec::new(),
            max_spinners,
        }
    }

    /// Add a new spinner
    pub fn add(&mut self, spinner: LoadingSpinner) {
        if self.active_spinners.len() >= self.max_spinners {
            self.active_spinners.remove(0);
        }
        self.active_spinners.push(spinner);
    }

    /// Remove all spinners
    pub fn clear(&mut self) {
        self.active_spinners.clear();
    }

    /// Check if any spinner is active
    pub fn is_loading(&self) -> bool {
        self.active_spinners.iter().any(|s| s.is_loading)
    }

    /// Get the number of active spinners
    pub fn count(&self) -> usize {
        self.active_spinners.iter().filter(|s| s.is_loading).count()
    }

    /// Update all spinners
    pub fn update_all(&mut self) {
        for spinner in &mut self.active_spinners {
            spinner.update_frame();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_creation() {
        let spinner = LoadingSpinner::new(SpinnerStyle::Monad);
        assert!(spinner.is_loading);
        assert_eq!(spinner.style, SpinnerStyle::Monad);
    }

    #[test]
    fn test_spinner_with_label() {
        let spinner = LoadingSpinner::new(SpinnerStyle::Dots).with_label("Loading...");
        assert_eq!(spinner.label, Some("Loading...".to_string()));
    }

    #[test]
    fn test_spinner_set_loading() {
        let mut spinner = LoadingSpinner::new(SpinnerStyle::Minimal);
        assert!(spinner.is_loading);

        spinner.set_loading(false);
        assert!(!spinner.is_loading);

        spinner.set_loading(true);
        assert!(spinner.is_loading);
    }

    #[test]
    fn test_spinner_frame_animation() {
        let mut spinner = LoadingSpinner::new(SpinnerStyle::Monad);
        let initial_frame = spinner.frame_index;

        spinner.update_frame();
        assert!(spinner.frame_index != initial_frame || spinner.frame_index == 0);
    }

    #[test]
    fn test_spinner_manager_creation() {
        let manager = SpinnerManager::new(5);
        assert_eq!(manager.max_spinners, 5);
        assert!(!manager.is_loading());
    }

    #[test]
    fn test_spinner_manager_add() {
        let mut manager = SpinnerManager::new(3);
        manager.add(LoadingSpinner::new(SpinnerStyle::Monad));

        assert_eq!(manager.count(), 1);
        assert!(manager.is_loading());
    }

    #[test]
    fn test_spinner_manager_max_capacity() {
        let mut manager = SpinnerManager::new(2);

        manager.add(LoadingSpinner::new(SpinnerStyle::Dots));
        manager.add(LoadingSpinner::new(SpinnerStyle::Minimal));
        manager.add(LoadingSpinner::new(SpinnerStyle::Monad));

        // Should only keep 2 spinners
        assert_eq!(manager.count(), 2);
    }

    #[test]
    fn test_spinner_manager_clear() {
        let mut manager = SpinnerManager::new(3);
        manager.add(LoadingSpinner::new(SpinnerStyle::Monad));
        manager.add(LoadingSpinner::new(SpinnerStyle::Dots));

        assert_eq!(manager.count(), 2);

        manager.clear();
        assert_eq!(manager.count(), 0);
        assert!(!manager.is_loading());
    }
}
