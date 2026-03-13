//! Doctor integration tests
//!
//! Tests for the diagnostic and troubleshooting functionality.
//!
//! Test categories:
//! - Check structure tests (pass/fail states)
//! - DoctorReport aggregation tests
//! - Doctor creation and configuration tests
//! - Diagnostic check behavior tests

use monad_val_manager::cli::Network;
use monad_val_manager::config::Config;
use monad_val_manager::doctor::{Check, Doctor, DoctorReport};

// =============================================================================
// TEST CONSTANTS
// =============================================================================

const TEST_RPC_ENDPOINT: &str = "http://localhost:8080";

// =============================================================================
// CHECK STRUCTURE TESTS
// =============================================================================

#[test]
fn test_check_passed_state() {
    let check = Check {
        name: "Test Check".to_string(),
        passed: true,
        message: "All good".to_string(),
    };

    assert_eq!(check.name, "Test Check");
    assert!(check.passed);
    assert_eq!(check.message, "All good");
}

#[test]
fn test_check_failed_state() {
    let check = Check {
        name: "Failed Check".to_string(),
        passed: false,
        message: "Something went wrong".to_string(),
    };

    assert_eq!(check.name, "Failed Check");
    assert!(!check.passed);
    assert_eq!(check.message, "Something went wrong");
}

#[test]
fn test_check_clone() {
    let original = Check {
        name: "Clone Test".to_string(),
        passed: true,
        message: "Original message".to_string(),
    };

    let cloned = original.clone();

    assert_eq!(original.name, cloned.name);
    assert_eq!(original.passed, cloned.passed);
    assert_eq!(original.message, cloned.message);
}

#[test]
fn test_check_debug_format() {
    let check = Check {
        name: "Debug Test".to_string(),
        passed: true,
        message: "Debug message".to_string(),
    };

    let debug_str = format!("{:?}", check);

    assert!(debug_str.contains("Debug Test"));
    assert!(debug_str.contains("true"));
    assert!(debug_str.contains("Debug message"));
}

// =============================================================================
// DOCTOR REPORT TESTS
// =============================================================================

#[test]
fn test_doctor_report_empty() {
    let report = DoctorReport {
        checks: vec![],
        issues: vec![],
        fixable_issues: vec![],
        passed: 0,
        failed: 0,
    };

    assert!(report.checks.is_empty());
    assert!(report.issues.is_empty());
    assert!(report.fixable_issues.is_empty());
    assert_eq!(report.passed, 0);
    assert_eq!(report.failed, 0);
}

#[test]
fn test_doctor_report_with_passed_checks() {
    let checks = vec![
        Check {
            name: "Check 1".to_string(),
            passed: true,
            message: "OK".to_string(),
        },
        Check {
            name: "Check 2".to_string(),
            passed: true,
            message: "OK".to_string(),
        },
    ];

    let report = DoctorReport {
        checks: checks.clone(),
        issues: vec![],
        fixable_issues: vec![],
        passed: 2,
        failed: 0,
    };

    assert_eq!(report.checks.len(), 2);
    assert_eq!(report.passed, 2);
    assert_eq!(report.failed, 0);
    assert!(report.issues.is_empty());
}

#[test]
fn test_doctor_report_with_failed_checks() {
    let checks = vec![
        Check {
            name: "Check 1".to_string(),
            passed: true,
            message: "OK".to_string(),
        },
        Check {
            name: "Check 2".to_string(),
            passed: false,
            message: "Failed".to_string(),
        },
        Check {
            name: "Check 3".to_string(),
            passed: false,
            message: "Error".to_string(),
        },
    ];

    let report = DoctorReport {
        checks,
        issues: vec!["Issue 1".to_string(), "Issue 2".to_string()],
        fixable_issues: vec!["Fixable 1".to_string()],
        passed: 1,
        failed: 2,
    };

    assert_eq!(report.checks.len(), 3);
    assert_eq!(report.passed, 1);
    assert_eq!(report.failed, 2);
    assert_eq!(report.issues.len(), 2);
    assert_eq!(report.fixable_issues.len(), 1);
}

#[test]
fn test_doctor_report_clone() {
    let report = DoctorReport {
        checks: vec![Check {
            name: "Test".to_string(),
            passed: true,
            message: "OK".to_string(),
        }],
        issues: vec!["Issue".to_string()],
        fixable_issues: vec![],
        passed: 1,
        failed: 0,
    };

    let cloned = report.clone();

    assert_eq!(report.checks.len(), cloned.checks.len());
    assert_eq!(report.passed, cloned.passed);
    assert_eq!(report.failed, cloned.failed);
}

#[test]
fn test_doctor_report_debug_format() {
    let report = DoctorReport {
        checks: vec![],
        issues: vec!["Test issue".to_string()],
        fixable_issues: vec![],
        passed: 5,
        failed: 2,
    };

    let debug_str = format!("{:?}", report);

    assert!(debug_str.contains("passed: 5"));
    assert!(debug_str.contains("failed: 2"));
}

// =============================================================================
// DOCTOR CREATION TESTS
// =============================================================================

#[test]
fn test_doctor_creation_mainnet() {
    let config = Config::create_default(Network::Mainnet).expect("Failed to create config");
    let doctor = Doctor::new(&config);

    // Doctor created successfully
    let _ = doctor;
}

#[test]
fn test_doctor_creation_testnet() {
    let config = Config::create_default(Network::Testnet).expect("Failed to create config");
    let doctor = Doctor::new(&config);

    // Doctor created successfully
    let _ = doctor;
}

#[test]
fn test_doctor_with_custom_config() {
    let mut config = Config::create_default(Network::Mainnet).expect("Failed to create config");
    config.rpc.http_url = TEST_RPC_ENDPOINT.to_string();

    let doctor = Doctor::new(&config);

    // Doctor created successfully with custom config
    let _ = doctor;
}

// =============================================================================
// DIAGNOSTIC CHECK COUNT TESTS
// =============================================================================

#[test]
fn test_check_counts_consistency() {
    // Verify that passed + failed equals total checks
    let checks = [
        Check {
            name: "A".to_string(),
            passed: true,
            message: "".to_string(),
        },
        Check {
            name: "B".to_string(),
            passed: false,
            message: "".to_string(),
        },
        Check {
            name: "C".to_string(),
            passed: true,
            message: "".to_string(),
        },
        Check {
            name: "D".to_string(),
            passed: true,
            message: "".to_string(),
        },
        Check {
            name: "E".to_string(),
            passed: false,
            message: "".to_string(),
        },
    ];

    let passed = checks.iter().filter(|c| c.passed).count();
    let failed = checks.iter().filter(|c| !c.passed).count();

    assert_eq!(passed, 3);
    assert_eq!(failed, 2);
    assert_eq!(passed + failed, checks.len());
}

#[test]
fn test_all_checks_passed() {
    let checks = [
        Check {
            name: "A".to_string(),
            passed: true,
            message: "OK".to_string(),
        },
        Check {
            name: "B".to_string(),
            passed: true,
            message: "OK".to_string(),
        },
        Check {
            name: "C".to_string(),
            passed: true,
            message: "OK".to_string(),
        },
    ];

    let all_passed = checks.iter().all(|c| c.passed);
    assert!(all_passed);
}

#[test]
fn test_all_checks_failed() {
    let checks = [
        Check {
            name: "A".to_string(),
            passed: false,
            message: "Error".to_string(),
        },
        Check {
            name: "B".to_string(),
            passed: false,
            message: "Error".to_string(),
        },
    ];

    let all_failed = checks.iter().all(|c| !c.passed);
    assert!(all_failed);
}

// =============================================================================
// ISSUE TRACKING TESTS
// =============================================================================

#[test]
fn test_issues_list_population() {
    let checks = [
        Check {
            name: "Pass".to_string(),
            passed: true,
            message: "OK".to_string(),
        },
        Check {
            name: "Fail 1".to_string(),
            passed: false,
            message: "Error 1".to_string(),
        },
        Check {
            name: "Fail 2".to_string(),
            passed: false,
            message: "Error 2".to_string(),
        },
    ];

    let issues: Vec<String> = checks
        .iter()
        .filter(|c| !c.passed)
        .map(|c| format!("{} failed", c.name))
        .collect();

    assert_eq!(issues.len(), 2);
    assert!(issues[0].contains("Fail 1"));
    assert!(issues[1].contains("Fail 2"));
}

#[test]
fn test_fixable_issues_tracking() {
    let fixable_issues = ["rpc_connection".to_string(), "disk_space".to_string()];

    assert_eq!(fixable_issues.len(), 2);
    assert!(fixable_issues.contains(&"rpc_connection".to_string()));
    assert!(fixable_issues.contains(&"disk_space".to_string()));
}

// =============================================================================
// CHECK MESSAGE FORMAT TESTS
// =============================================================================

#[test]
fn test_check_message_format_values() {
    let check = Check {
        name: "Memory".to_string(),
        passed: true,
        message: "16.5 GB / 32.0 GB (51.5%)".to_string(),
    };

    assert!(check.message.contains("GB"));
    assert!(check.message.contains("%"));
}

#[test]
fn test_check_message_format_status() {
    let running_check = Check {
        name: "Service".to_string(),
        passed: true,
        message: "running".to_string(),
    };

    let stopped_check = Check {
        name: "Service".to_string(),
        passed: false,
        message: "not running (inactive)".to_string(),
    };

    assert!(running_check.passed);
    assert!(!stopped_check.passed);
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[test]
fn test_check_with_empty_message() {
    let check = Check {
        name: "Empty Message Check".to_string(),
        passed: true,
        message: "".to_string(),
    };

    assert!(check.message.is_empty());
    assert!(check.passed);
}

#[test]
fn test_check_with_long_name() {
    let long_name = "A".repeat(100);
    let check = Check {
        name: long_name.clone(),
        passed: true,
        message: "OK".to_string(),
    };

    assert_eq!(check.name.len(), 100);
}

#[test]
fn test_report_with_many_checks() {
    let checks: Vec<Check> = (0..100)
        .map(|i| Check {
            name: format!("Check {}", i),
            passed: i % 2 == 0,
            message: format!("Message {}", i),
        })
        .collect();

    let report = DoctorReport {
        checks: checks.clone(),
        issues: vec![],
        fixable_issues: vec![],
        passed: checks.iter().filter(|c| c.passed).count(),
        failed: checks.iter().filter(|c| !c.passed).count(),
    };

    assert_eq!(report.checks.len(), 100);
    assert_eq!(report.passed, 50);
    assert_eq!(report.failed, 50);
}
