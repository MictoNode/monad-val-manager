//! Toast Notification Module - Discord/Slack-like Notifications
//!
//! This module provides toast notifications using ratatui-toaster:
//! - Transaction success/failure notifications
//! - Node status updates
//! - Warning and info messages
//! - Auto-dismiss with animations

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::Widget,
};
use std::time::{Duration, Instant};

/// Toast notification type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

impl ToastType {
    /// Get the default color for this toast type
    pub fn color(&self) -> Color {
        match self {
            ToastType::Success => Color::Rgb(74, 222, 128), // Green
            ToastType::Error => Color::Rgb(239, 68, 68),    // Red
            ToastType::Warning => Color::Rgb(255, 174, 69), // Orange
            ToastType::Info => Color::Rgb(133, 230, 255),   // Cyan
        }
    }

    /// Get the icon for this toast type
    pub fn icon(&self) -> &str {
        match self {
            ToastType::Success => "✓",
            ToastType::Error => "✗",
            ToastType::Warning => "⚠",
            ToastType::Info => "ℹ",
        }
    }

    /// Get the default duration for this toast type
    pub fn default_duration(&self) -> Duration {
        match self {
            ToastType::Success => Duration::from_secs(3),
            ToastType::Error => Duration::from_secs(5),
            ToastType::Warning => Duration::from_secs(4),
            ToastType::Info => Duration::from_secs(3),
        }
    }
}

/// A single toast notification
#[derive(Debug, Clone)]
pub struct Toast {
    pub id: usize,
    pub title: String,
    pub message: String,
    pub toast_type: ToastType,
    pub created_at: Instant,
    pub duration: Duration,
    pub is_dismissed: bool,
}

impl Toast {
    /// Create a new toast notification
    pub fn new(id: usize, title: String, message: String, toast_type: ToastType) -> Self {
        let duration = toast_type.default_duration();
        Self {
            id,
            title,
            message,
            toast_type,
            created_at: Instant::now(),
            duration,
            is_dismissed: false,
        }
    }

    /// Create a success toast
    pub fn success(id: usize, title: String, message: String) -> Self {
        Self::new(id, title, message, ToastType::Success)
    }

    /// Create an error toast
    pub fn error(id: usize, title: String, message: String) -> Self {
        Self::new(id, title, message, ToastType::Error)
    }

    /// Create a warning toast
    pub fn warning(id: usize, title: String, message: String) -> Self {
        Self::new(id, title, message, ToastType::Warning)
    }

    /// Create an info toast
    pub fn info(id: usize, title: String, message: String) -> Self {
        Self::new(id, title, message, ToastType::Info)
    }

    /// Check if this toast has expired
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() >= self.duration
    }

    /// Get the remaining time before expiration
    pub fn remaining_time(&self) -> Duration {
        self.duration.saturating_sub(self.created_at.elapsed())
    }

    /// Get the opacity based on remaining time (for fade-out effect)
    pub fn opacity(&self) -> f32 {
        let remaining = self.remaining_time();
        let fade_duration = Duration::from_millis(500);

        if remaining < fade_duration {
            remaining.as_secs_f32() / fade_duration.as_secs_f32()
        } else {
            1.0
        }
    }

    /// Mark this toast as dismissed
    pub fn dismiss(&mut self) {
        self.is_dismissed = true;
    }
}

impl Toast {
    /// Render toast by reference (no clone overhead)
    pub fn render_ref(&self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        // Calculate background color with opacity
        let bg_color = self.toast_type.color();
        let opacity = self.opacity();

        // Dim the color based on opacity
        let final_bg = if opacity < 1.0 {
            match bg_color {
                Color::Rgb(r, g, b) => {
                    let factor = opacity;
                    Color::Rgb(
                        (r as f32 * factor).round() as u8,
                        (g as f32 * factor).round() as u8,
                        (b as f32 * factor).round() as u8,
                    )
                }
                _ => bg_color,
            }
        } else {
            bg_color
        };

        // Background style
        let bg_style = Style::default().fg(Color::White).bg(final_bg);

        // Render background
        let block = ratatui::widgets::Block::default().style(bg_style);

        block.render(area, buf);

        // Calculate content area (with padding)
        let content_area = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };

        // Render title with icon
        let icon = self.toast_type.icon();
        let title_text = format!("{} {}", icon, self.title);
        let title_line = Line::styled(
            title_text,
            Style::default()
                .fg(Color::White)
                .add_modifier(ratatui::style::Modifier::BOLD),
        );

        // Render message
        let message_line = Line::styled(
            self.message.clone(),
            Style::default().fg(Color::Rgb(255, 255, 255)),
        );

        // Position text based on area height
        if content_area.height >= 2 {
            title_line.render(
                Rect {
                    x: content_area.x,
                    y: content_area.y,
                    width: content_area.width,
                    height: 1,
                },
                buf,
            );

            message_line.render(
                Rect {
                    x: content_area.x,
                    y: content_area.y + 1,
                    width: content_area.width,
                    height: 1,
                },
                buf,
            );
        } else if content_area.height >= 1 {
            // Only show title if space is limited
            title_line.render(content_area, buf);
        }
    }
}

impl Widget for Toast {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer)
    where
        Self: Sized,
    {
        self.render_ref(area, buf);
    }
}

/// Toast manager for handling multiple toasts
#[derive(Debug, Clone)]
pub struct ToastManager {
    pub toasts: Vec<Toast>,
    pub next_id: usize,
    pub max_toasts: usize,
}

impl Default for ToastManager {
    fn default() -> Self {
        Self {
            toasts: Vec::new(),
            next_id: 0,
            max_toasts: 5,
        }
    }
}

impl ToastManager {
    /// Create a new toast manager
    pub fn new(max_toasts: usize) -> Self {
        Self {
            toasts: Vec::new(),
            next_id: 0,
            max_toasts,
        }
    }

    /// Add a new toast notification
    pub fn add(&mut self, toast: Toast) -> usize {
        let id = toast.id;

        // Remove oldest if at max capacity
        if self.toasts.len() >= self.max_toasts {
            self.toasts.remove(0);
        }

        self.toasts.push(toast);
        id
    }

    /// Add a success toast
    pub fn success(&mut self, title: String, message: String) -> usize {
        let toast = Toast::success(self.next_id(), title, message);
        self.add(toast)
    }

    /// Add an error toast
    pub fn error(&mut self, title: String, message: String) -> usize {
        let toast = Toast::error(self.next_id(), title, message);
        self.add(toast)
    }

    /// Add a warning toast
    pub fn warning(&mut self, title: String, message: String) -> usize {
        let toast = Toast::warning(self.next_id(), title, message);
        self.add(toast)
    }

    /// Add an info toast
    pub fn info(&mut self, title: String, message: String) -> usize {
        let toast = Toast::info(self.next_id(), title, message);
        self.add(toast)
    }

    /// Remove a toast by ID
    pub fn remove(&mut self, id: usize) -> bool {
        if let Some(pos) = self.toasts.iter().position(|t| t.id == id) {
            self.toasts.remove(pos);
            true
        } else {
            false
        }
    }

    /// Dismiss a toast by ID
    pub fn dismiss(&mut self, id: usize) -> bool {
        if let Some(toast) = self.toasts.iter_mut().find(|t| t.id == id) {
            toast.dismiss();
            true
        } else {
            false
        }
    }

    /// Clear all toasts
    pub fn clear(&mut self) {
        self.toasts.clear();
    }

    /// Remove expired toasts
    pub fn cleanup(&mut self) {
        self.toasts.retain(|t| !t.is_expired() && !t.is_dismissed);
    }

    /// Get the next toast ID
    pub fn next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Get the number of active toasts
    pub fn count(&self) -> usize {
        self.toasts.len()
    }

    /// Check if there are any active toasts
    pub fn is_empty(&self) -> bool {
        self.toasts.is_empty()
    }
}

impl Widget for ToastManager {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer)
    where
        Self: Sized,
    {
        Self::render_ref(&self, area, buf);
    }
}

impl ToastManager {
    /// Render toasts by reference (no clone overhead)
    pub fn render_ref(&self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let toast_count = self.toasts.len();
        if toast_count == 0 {
            return;
        }

        let toast_height = 4; // Height per toast
        let toast_width = area.width.min(60); // Max width

        // Render toasts from bottom up
        for (i, toast) in self.toasts.iter().enumerate() {
            let reverse_index = toast_count - 1 - i;

            // Calculate toast position
            let y = if reverse_index == 0 {
                area.bottom() - toast_height
            } else {
                area.bottom() - (reverse_index as u16 * toast_height) - 2 // 2px gap
            };

            // Skip if out of bounds
            if y < area.top() {
                break;
            }

            let toast_area = Rect {
                x: area.right() - toast_width - 2, // 2px from right
                y,
                width: toast_width,
                height: toast_height,
            };

            toast.render_ref(toast_area, buf);
        }
    }
}

/// Helper functions for common toast scenarios
pub struct ToastHelpers;

impl ToastHelpers {
    /// Create a transaction success toast
    pub fn tx_success(tx_hash: &str, operation: &str) -> (String, String) {
        let short_hash = if tx_hash.len() > 10 {
            format!("{}...{}", &tx_hash[..6], &tx_hash[tx_hash.len() - 4..])
        } else {
            tx_hash.to_string()
        };

        (
            format!("{} Successful", operation),
            format!("Tx: {}", short_hash),
        )
    }

    /// Create a transaction error toast
    pub fn tx_error(error: &str, operation: &str) -> (String, String) {
        (format!("{} Failed", operation), error.to_string())
    }

    /// Create a node status toast
    pub fn node_status(connected: bool, syncing: bool) -> (String, String, ToastType) {
        match (connected, syncing) {
            (true, false) => (
                "Node Connected".to_string(),
                "RPC connection established".to_string(),
                ToastType::Success,
            ),
            (true, true) => (
                "Node Syncing".to_string(),
                "Catching up with network".to_string(),
                ToastType::Info,
            ),
            (false, _) => (
                "Node Disconnected".to_string(),
                "Unable to reach RPC endpoint".to_string(),
                ToastType::Error,
            ),
        }
    }

    /// Create a rewards claim toast
    pub fn rewards_claimed(amount: &str) -> (String, String) {
        (
            "Rewards Claimed".to_string(),
            format!("{} MON added to balance", amount),
        )
    }

    /// Create a delegation toast
    pub fn delegated(amount: &str, validator_id: u64) -> (String, String) {
        (
            "Delegation Successful".to_string(),
            format!("{} MON to validator #{}", amount, validator_id),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toast_creation() {
        let toast = Toast::new(
            0,
            "Test".to_string(),
            "Message".to_string(),
            ToastType::Info,
        );
        assert_eq!(toast.id, 0);
        assert_eq!(toast.title, "Test");
        assert!(!toast.is_dismissed);
    }

    #[test]
    fn test_toast_success() {
        let toast = Toast::success(0, "Success".to_string(), "It worked!".to_string());
        assert_eq!(toast.toast_type, ToastType::Success);
        assert_eq!(toast.duration, Duration::from_secs(3));
    }

    #[test]
    fn test_toast_error() {
        let toast = Toast::error(0, "Error".to_string(), "It failed!".to_string());
        assert_eq!(toast.toast_type, ToastType::Error);
        assert_eq!(toast.duration, Duration::from_secs(5));
    }

    #[test]
    fn test_toast_type_colors() {
        assert_eq!(ToastType::Success.color(), Color::Rgb(74, 222, 128));
        assert_eq!(ToastType::Error.color(), Color::Rgb(239, 68, 68));
        assert_eq!(ToastType::Warning.color(), Color::Rgb(255, 174, 69));
        assert_eq!(ToastType::Info.color(), Color::Rgb(133, 230, 255));
    }

    #[test]
    fn test_toast_type_icons() {
        assert_eq!(ToastType::Success.icon(), "✓");
        assert_eq!(ToastType::Error.icon(), "✗");
        assert_eq!(ToastType::Warning.icon(), "⚠");
        assert_eq!(ToastType::Info.icon(), "ℹ");
    }

    #[test]
    fn test_toast_manager_creation() {
        let manager = ToastManager::new(5);
        assert_eq!(manager.max_toasts, 5);
        assert!(manager.is_empty());
    }

    #[test]
    fn test_toast_manager_add() {
        let mut manager = ToastManager::new(5);
        let _id = manager.success("Test".to_string(), "Message".to_string());
        assert_eq!(manager.count(), 1);
        assert!(!manager.is_empty());
    }

    #[test]
    fn test_toast_manager_max_capacity() {
        let mut manager = ToastManager::new(3);

        manager.success("Test1".to_string(), "Msg1".to_string());
        manager.success("Test2".to_string(), "Msg2".to_string());
        manager.success("Test3".to_string(), "Msg3".to_string());
        manager.success("Test4".to_string(), "Msg4".to_string());

        // Should only keep 3 toasts
        assert_eq!(manager.count(), 3);
    }

    #[test]
    fn test_toast_manager_remove() {
        let mut manager = ToastManager::new(5);
        let id = manager.success("Test".to_string(), "Message".to_string());
        assert_eq!(manager.count(), 1);

        manager.remove(id);
        assert!(manager.is_empty());
    }

    #[test]
    fn test_toast_manager_clear() {
        let mut manager = ToastManager::new(5);
        manager.success("Test1".to_string(), "Msg1".to_string());
        manager.success("Test2".to_string(), "Msg2".to_string());

        manager.clear();
        assert!(manager.is_empty());
    }

    #[test]
    fn test_toast_helpers_tx_success() {
        let (title, msg) = ToastHelpers::tx_success("0xdef9abc123", "Delegate");
        assert_eq!(title, "Delegate Successful");
        assert!(msg.contains("0xdef9"));
    }

    #[test]
    fn test_toast_helpers_delegated() {
        let (title, msg) = ToastHelpers::delegated("100.5", 123);
        assert_eq!(title, "Delegation Successful");
        assert!(msg.contains("100.5 MON"));
        assert!(msg.contains("123"));
    }
}
