//! Utility functions for CLI handlers
//!
//! Common formatting and parsing functions used across command handlers.

/// Parse amount from MON (default) or wei string
///
/// # Arguments
/// * `amount_str` - Amount string (MON by default, supports "100MON", "100 MON", "100" formats)
/// * `wei_str` - Optional wei amount string (takes precedence if provided)
///
/// # Returns
/// Amount in wei as u128
pub fn parse_amount(amount_str: &str, wei_str: Option<&str>) -> anyhow::Result<u128> {
    const WEI_PER_MON: u128 = 1_000_000_000_000_000_000; // 10^18

    if let Some(wei) = wei_str {
        // Parse as wei (raw value)
        let wei_value: u128 = wei
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid wei amount format: {}", e))?;
        Ok(wei_value)
    } else {
        // BUG-005: Support MON suffix (e.g., "100MON", "100 MON", "100")
        let cleaned = amount_str.trim().to_lowercase();

        // Remove "mon" suffix if present (with or without space)
        let mon_str = if cleaned.ends_with("mon") {
            cleaned[..cleaned.len() - 3].trim()
        } else {
            &cleaned
        };

        // Parse as MON (human-readable format)
        let mon_f64: f64 = mon_str
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid MON amount format: {}", e))?;
        if mon_f64 < 0.0 {
            return Err(anyhow::anyhow!("Amount cannot be negative"));
        }
        let wei = (mon_f64 * WEI_PER_MON as f64) as u128;
        Ok(wei)
    }
}

/// Format wei amount as MON (human-readable)
pub fn format_mon(wei: u128) -> String {
    const WEI_PER_MON: u128 = 1_000_000_000_000_000_000; // 10^18

    if wei == 0 {
        return "0".to_string();
    }

    let mon = wei as f64 / WEI_PER_MON as f64;

    if mon >= 1_000_000.0 {
        format!("{:.2}M", mon / 1_000_000.0)
    } else if mon >= 1_000.0 {
        format!("{:.2}K", mon / 1_000.0)
    } else if mon >= 1.0 {
        format!("{:.4}", mon)
    } else {
        format!("{:.8}", mon)
    }
}

/// Format validator status flags to human-readable string
pub fn format_validator_status(flags: u64) -> String {
    use crate::staking::types::ValidatorStatus;

    let status = ValidatorStatus::from_flags(flags);
    let mut parts = Vec::new();

    if status.is_active() {
        parts.push("Active");
    }
    if status.is_slashed() {
        parts.push("Slashed");
    }
    if status.is_in_cooldown() {
        parts.push("Cooldown");
    }
    if status.is_opted_out() {
        parts.push("OptedOut");
    }

    if parts.is_empty() {
        "Unknown".to_string()
    } else {
        parts.join(", ")
    }
}

/// Format uptime in seconds to human-readable string
pub fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_amount_mon() {
        let result = parse_amount("1.5", None).unwrap();
        assert_eq!(result, 1_500_000_000_000_000_000);
    }

    // BUG-005: Test MON suffix support
    #[test]
    fn test_parse_amount_mon_suffix() {
        let result = parse_amount("100MON", None).unwrap();
        assert_eq!(result, 100_000_000_000_000_000_000);

        let result2 = parse_amount("100 MON", None).unwrap();
        assert_eq!(result2, 100_000_000_000_000_000_000);

        let result3 = parse_amount("50mon", None).unwrap();
        assert_eq!(result3, 50_000_000_000_000_000_000);

        let result4 = parse_amount("50 Mon", None).unwrap();
        assert_eq!(result4, 50_000_000_000_000_000_000);
    }

    #[test]
    fn test_parse_amount_without_suffix() {
        let result = parse_amount("100", None).unwrap();
        assert_eq!(result, 100_000_000_000_000_000_000);
    }

    #[test]
    fn test_parse_amount_wei() {
        let result = parse_amount("0", Some("1000000000000000000")).unwrap();
        assert_eq!(result, 1_000_000_000_000_000_000);
    }

    #[test]
    fn test_parse_amount_negative() {
        let result = parse_amount("-1.0", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_mon_zero() {
        assert_eq!(format_mon(0), "0");
    }

    #[test]
    fn test_format_mon_millions() {
        let wei = 2_500_000_000_000_000_000_000_000u128; // 2.5M MON
        assert_eq!(format_mon(wei), "2.50M");
    }

    #[test]
    fn test_format_mon_thousands() {
        let wei = 1_500_000_000_000_000_000_000u128; // 1.5K MON
        assert_eq!(format_mon(wei), "1.50K");
    }

    #[test]
    fn test_format_mon_regular() {
        let wei = 1_234_567_890_123_456_789u128; // ~1.23 MON
        assert_eq!(format_mon(wei), "1.2346");
    }

    #[test]
    fn test_format_mon_small() {
        let wei = 123_456_789_012_345u128; // ~0.00012 MON
        let result = format_mon(wei);
        assert!(result.starts_with("0.0001"));
    }

    #[test]
    fn test_format_uptime_days() {
        assert_eq!(format_uptime(90061), "1d 1h 1m");
    }

    #[test]
    fn test_format_uptime_hours() {
        assert_eq!(format_uptime(3661), "1h 1m");
    }

    #[test]
    fn test_format_uptime_minutes() {
        assert_eq!(format_uptime(61), "1m");
    }

    #[test]
    fn test_format_uptime_zero() {
        assert_eq!(format_uptime(0), "0m");
    }

    // BUG-003: Test hex wei to MON conversion
    #[test]
    fn test_format_hex_wei_to_mon() {
        // Test "0x1a784379d99db42000000 wei" -> "2.00M MON"
        let hex_wei = "0x1a784379d99db42000000";

        // Parse hex string to u128
        let wei_value = u128::from_str_radix(hex_wei.strip_prefix("0x").unwrap_or(hex_wei), 16)
            .expect("Invalid hex");

        // Format as MON
        let result = format_mon(wei_value);

        // Should be approximately 2.00M MON
        assert!(
            result.contains("2.00"),
            "Expected ~2.00M MON, got {}",
            result
        );
    }

    #[test]
    fn test_format_zero_wei() {
        let result = format_mon(0);
        assert_eq!(result, "0");
    }

    #[test]
    fn test_format_one_mon() {
        let wei = 1_000_000_000_000_000_000u128; // 1 MON
        let result = format_mon(wei);
        assert!(result.contains("1"), "Expected 1 MON, got {}", result);
    }
}
