//! Staking Helpers - Utility functions for staking display
//!
//! This module provides helper functions for formatting amounts and addresses.

/// Format balance from f64 (already in MON) to display string
/// Used for account balance display which is stored as f64
pub fn format_balance(amount: f64) -> String {
    if amount == 0.0 {
        return "0".to_string();
    }

    // Format with up to 4 decimal places for small amounts, 2 for larger
    if amount >= 1_000.0 {
        format!("{:.2}", amount)
    } else if amount >= 1.0 {
        format!("{:.4}", amount)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    } else {
        format!("{:.6}", amount)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

/// Format MON amount from smallest unit (18 decimals) to display string
pub fn format_mon_amount(amount: u128) -> String {
    const DECIMALS: u32 = 18;
    let whole = amount / 10u128.pow(DECIMALS);
    let fractional = amount % 10u128.pow(DECIMALS);

    if fractional == 0 {
        format!("{}", whole)
    } else {
        // Format with up to 6 decimal places, trimming trailing zeros
        let frac_str = format!("{:018}", fractional);
        let trimmed = frac_str.trim_end_matches('0');
        let decimals = trimmed.chars().take(6).collect::<String>();
        if decimals.is_empty() {
            format!("{}", whole)
        } else {
            format!("{}.{}", whole, decimals)
        }
    }
}

/// Truncate an Ethereum address for display (0x1234...5678)
pub fn truncate_address(address: &str) -> String {
    if address.len() > 12 {
        format!("{}...{}", &address[..6], &address[address.len() - 4..])
    } else {
        address.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_balance_zero() {
        assert_eq!(format_balance(0.0), "0");
    }

    #[test]
    fn test_format_balance_whole() {
        assert_eq!(format_balance(1.0), "1");
        assert_eq!(format_balance(10.0), "10");
    }

    #[test]
    fn test_format_balance_fractional() {
        assert_eq!(format_balance(1.5), "1.5");
        assert_eq!(format_balance(1.234567), "1.2346");
    }

    #[test]
    fn test_format_mon_amount_whole() {
        assert_eq!(format_mon_amount(1_000_000_000_000_000_000), "1");
        assert_eq!(format_mon_amount(10_000_000_000_000_000_000), "10");
    }

    #[test]
    fn test_format_mon_amount_fractional() {
        assert_eq!(format_mon_amount(1_500_000_000_000_000_000), "1.5");
        assert_eq!(format_mon_amount(1_234_567_000_000_000_000), "1.234567");
    }

    #[test]
    fn test_format_mon_amount_zero() {
        assert_eq!(format_mon_amount(0), "0");
    }

    #[test]
    fn test_truncate_address() {
        assert_eq!(
            truncate_address("0x1234567890abcdef1234567890abcdef12345678"),
            "0x1234...5678"
        );
        assert_eq!(truncate_address("0x1234"), "0x1234");
    }
}
