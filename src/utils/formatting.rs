//! Formatting utilities

use chrono::{DateTime, Utc};
use std::time::Duration;

/// Format bytes to human readable string
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format duration to human readable string
pub fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();

    if total_secs < 60 {
        format!("{}s", total_secs)
    } else if total_secs < 3600 {
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        format!("{}m {}s", mins, secs)
    } else if total_secs < 86400 {
        let hours = total_secs / 3600;
        let mins = (total_secs % 3600) / 60;
        format!("{}h {}m", hours, mins)
    } else {
        let days = total_secs / 86400;
        let hours = (total_secs % 86400) / 3600;
        format!("{}d {}h", days, hours)
    }
}

/// Format timestamp to human readable string
pub fn format_timestamp(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// Format percentage
pub fn format_percentage(value: f64) -> String {
    format!("{:.1}%", value)
}

/// Format large numbers with thousand separators
pub fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }
    result
}

/// Truncate string with ellipsis
pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Format hex string (add 0x prefix if missing)
pub fn format_hex(s: &str) -> String {
    if s.starts_with("0x") {
        s.to_string()
    } else {
        format!("0x{}", s)
    }
}

/// Strip hex prefix
pub fn strip_hex_prefix(s: &str) -> &str {
    s.strip_prefix("0x").unwrap_or(s)
}

/// Count zeros in a number
///
/// This helper function counts the number of '0' characters in the decimal
/// representation of a number. It's useful for validating amounts and
/// providing user feedback about the scale of numbers.
///
/// # Arguments
/// * `amount` - The number to count zeros in
///
/// # Returns
/// The count of '0' characters in the decimal representation
///
/// # Examples
/// ```
/// use monad_val_manager::utils::formatting::count_zeros;
///
/// assert_eq!(count_zeros(100), 2);
/// assert_eq!(count_zeros(1000), 3);
/// assert_eq!(count_zeros(123), 0);
/// ```
pub fn count_zeros(amount: u64) -> usize {
    amount.to_string().chars().filter(|&c| c == '0').count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
        assert_eq!(format_bytes(1073741824), "1.00 GB");
        assert_eq!(format_bytes(1099511627776), "1.00 TB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration(Duration::from_secs(3661)), "1h 1m");
        assert_eq!(format_duration(Duration::from_secs(90061)), "1d 1h");
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(100), "100");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1000000), "1,000,000");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 8), "hello...");
    }

    #[test]
    fn test_format_hex() {
        assert_eq!(format_hex("abc123"), "0xabc123");
        assert_eq!(format_hex("0xabc123"), "0xabc123");
    }

    #[test]
    fn test_strip_hex_prefix() {
        assert_eq!(strip_hex_prefix("0xabc"), "abc");
        assert_eq!(strip_hex_prefix("abc"), "abc");
    }

    #[test]
    fn test_count_zeros() {
        // Test basic cases - counts ALL zeros in the string
        assert_eq!(count_zeros(0), 1); // "0" has one zero
        assert_eq!(count_zeros(10), 1); // "10" has one zero
        assert_eq!(count_zeros(100), 2); // "100" has two zeros
        assert_eq!(count_zeros(1000), 3); // "1000" has three zeros
        assert_eq!(count_zeros(10000), 4); // "10000" has four zeros

        // Test amounts with zeros in the middle
        assert_eq!(count_zeros(1001), 2); // "1001" has two zeros
        assert_eq!(count_zeros(1010), 2); // "1010" has two zeros

        // Test no zeros
        assert_eq!(count_zeros(1), 0); // "1" has no zeros
        assert_eq!(count_zeros(123), 0); // "123" has no zeros
        assert_eq!(count_zeros(99999), 0); // "99999" has no zeros

        // Test large numbers (typical wei amounts)
        assert_eq!(count_zeros(1_000_000_000_000_000_000), 18); // "1000000000000000000"
        assert_eq!(count_zeros(10_000_000_000_000_000_000), 19); // "10000000000000000000"
    }
}
