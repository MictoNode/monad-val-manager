//! Doctor State - State management for Doctor screen diagnostics
//!
//! This module provides the state structure for the Doctor screen,
//! including check results, status tracking, and display formatting.

use std::time::Instant;

/// Status of an individual diagnostic check
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CheckStatus {
    /// Check passed successfully
    Pass,
    /// Check failed
    Fail,
    /// Check is currently running
    Running,
    /// Check has not been run yet
    #[default]
    Pending,
    /// Check encountered an error
    Error,
}

impl CheckStatus {
    /// Check if this status represents a completed check
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Pass | Self::Fail | Self::Error)
    }

    /// Get display symbol for this status
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Pass => "[PASS]",
            Self::Fail => "[FAIL]",
            Self::Running => "[....]",
            Self::Pending => "[    ]",
            Self::Error => "[ERR!]",
        }
    }
}

/// Individual diagnostic check result
#[derive(Debug, Clone)]
pub struct DoctorCheck {
    /// Check name/title
    pub name: String,
    /// Check category (System, Network, Service, etc.)
    pub category: CheckCategory,
    /// Current status
    pub status: CheckStatus,
    /// Human-readable message
    pub message: String,
    /// Optional fix suggestion
    pub fix_hint: Option<String>,
}

/// Category of diagnostic check
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckCategory {
    /// System resources (CPU, memory, disk)
    System,
    /// Network connectivity
    Network,
    /// Monad node services
    Service,
    /// Configuration validation
    Config,
    /// Consensus information
    Consensus,
}

impl CheckCategory {
    /// Get display name for this category
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Network => "Network",
            Self::Service => "Service",
            Self::Config => "Config",
            Self::Consensus => "Consensus",
        }
    }
}

/// State for the Doctor diagnostics screen
#[derive(Debug, Clone)]
pub struct DoctorState {
    /// All diagnostic checks
    pub checks: Vec<DoctorCheck>,
    /// Currently selected check index (for navigation)
    pub selected_index: usize,
    /// Are checks currently running?
    pub is_running: bool,
    /// Last run timestamp
    pub last_run: Option<Instant>,
    /// Summary: total passed
    pub passed_count: usize,
    /// Summary: total failed
    pub failed_count: usize,
}

impl Default for DoctorState {
    fn default() -> Self {
        Self::new()
    }
}

impl DoctorState {
    /// Create new doctor state with default checks
    pub fn new() -> Self {
        let checks = Self::default_checks();
        Self {
            checks,
            selected_index: 0,
            is_running: false,
            last_run: None,
            passed_count: 0,
            failed_count: 0,
        }
    }

    /// Create default set of diagnostic checks
    fn default_checks() -> Vec<DoctorCheck> {
        vec![
            // System checks
            DoctorCheck {
                name: "CPU Cores".to_string(),
                category: CheckCategory::System,
                status: CheckStatus::Pending,
                message: "Not checked".to_string(),
                fix_hint: None,
            },
            DoctorCheck {
                name: "Memory (RAM)".to_string(),
                category: CheckCategory::System,
                status: CheckStatus::Pending,
                message: "Not checked".to_string(),
                fix_hint: None,
            },
            // PHASE-5-FIX: Renamed from "Disk Space" to "Disk Space (OS)" to match CLI doctor
            DoctorCheck {
                name: "Disk Space (OS)".to_string(),
                category: CheckCategory::System,
                status: CheckStatus::Pending,
                message: "Not checked".to_string(),
                fix_hint: None,
            },
            // PHASE-5-FIX: Added MPT Storage check (Linux only, like CLI doctor)
            #[cfg(target_os = "linux")]
            DoctorCheck {
                name: "MPT Storage (Triedb)".to_string(),
                category: CheckCategory::System,
                status: CheckStatus::Pending,
                message: "Not checked".to_string(),
                fix_hint: None,
            },
            // Network checks
            DoctorCheck {
                name: "RPC Connection".to_string(),
                category: CheckCategory::Network,
                status: CheckStatus::Pending,
                message: "Not checked".to_string(),
                fix_hint: None,
            },
            DoctorCheck {
                name: "RPC Port".to_string(),
                category: CheckCategory::Network,
                status: CheckStatus::Pending,
                message: "Not checked".to_string(),
                fix_hint: None,
            },
            DoctorCheck {
                name: "Sync Status".to_string(),
                category: CheckCategory::Network,
                status: CheckStatus::Pending,
                message: "Not checked".to_string(),
                fix_hint: None,
            },
            // Service checks
            DoctorCheck {
                name: "Monad BFT Service".to_string(),
                category: CheckCategory::Service,
                status: CheckStatus::Pending,
                message: "Not checked".to_string(),
                fix_hint: None,
            },
            DoctorCheck {
                name: "Monad Execution Service".to_string(),
                category: CheckCategory::Service,
                status: CheckStatus::Pending,
                message: "Not checked".to_string(),
                fix_hint: None,
            },
            DoctorCheck {
                name: "Monad RPC Service".to_string(),
                category: CheckCategory::Service,
                status: CheckStatus::Pending,
                message: "Not checked".to_string(),
                fix_hint: None,
            },
            DoctorCheck {
                name: "Monad Archiver Service".to_string(),
                category: CheckCategory::Service,
                status: CheckStatus::Pending,
                message: "Not checked".to_string(),
                fix_hint: None,
            },
            // Config checks
            DoctorCheck {
                name: "Chain ID Config".to_string(),
                category: CheckCategory::Config,
                status: CheckStatus::Pending,
                message: "Not checked".to_string(),
                fix_hint: None,
            },
            // Consensus checks (NEW)
            DoctorCheck {
                name: "Current Epoch".to_string(),
                category: CheckCategory::Consensus,
                status: CheckStatus::Pending,
                message: "Not checked".to_string(),
                fix_hint: None,
            },
            DoctorCheck {
                name: "Current Round".to_string(),
                category: CheckCategory::Consensus,
                status: CheckStatus::Pending,
                message: "Not checked".to_string(),
                fix_hint: None,
            },
            DoctorCheck {
                name: "Forkpoint Info".to_string(),
                category: CheckCategory::Consensus,
                status: CheckStatus::Pending,
                message: "Not checked".to_string(),
                fix_hint: None,
            },
            DoctorCheck {
                name: "StateSync Status".to_string(),
                category: CheckCategory::Consensus,
                status: CheckStatus::Pending,
                message: "Not checked".to_string(),
                fix_hint: None,
            },
        ]
    }

    /// Update a check's result
    pub fn update_check(
        &mut self,
        name: &str,
        status: CheckStatus,
        message: &str,
        fix_hint: Option<&str>,
    ) {
        if let Some(check) = self.checks.iter_mut().find(|c| c.name == name) {
            check.status = status;
            check.message = message.to_string();
            check.fix_hint = fix_hint.map(|s| s.to_string());
        }
    }

    /// Recalculate pass/fail counts
    pub fn recalculate_summary(&mut self) {
        self.passed_count = self
            .checks
            .iter()
            .filter(|c| c.status == CheckStatus::Pass)
            .count();
        self.failed_count = self
            .checks
            .iter()
            .filter(|c| c.status == CheckStatus::Fail || c.status == CheckStatus::Error)
            .count();
    }

    /// Get currently selected check
    pub fn selected_check(&self) -> Option<&DoctorCheck> {
        self.checks.get(self.selected_index)
    }

    /// Move selection up
    pub fn select_prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else {
            // Wrap to bottom
            self.selected_index = self.checks.len().saturating_sub(1);
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if self.selected_index < self.checks.len().saturating_sub(1) {
            self.selected_index += 1;
        } else {
            // Wrap to top
            self.selected_index = 0;
        }
    }

    /// Start running checks (set all to running state)
    pub fn start_checks(&mut self) {
        self.is_running = true;
        for check in &mut self.checks {
            check.status = CheckStatus::Running;
            check.message = "Checking...".to_string();
        }
    }

    /// Mark checks as complete
    pub fn finish_checks(&mut self) {
        self.is_running = false;
        self.last_run = Some(Instant::now());
        self.recalculate_summary();
    }

    /// Get checks by category
    pub fn checks_by_category(&self, category: CheckCategory) -> Vec<&DoctorCheck> {
        self.checks
            .iter()
            .filter(|c| c.category == category)
            .collect()
    }

    /// Check if all checks have passed
    pub fn all_passed(&self) -> bool {
        !self.is_running && self.failed_count == 0 && self.passed_count > 0
    }

    /// Get overall health status
    pub fn overall_status(&self) -> CheckStatus {
        if self.is_running {
            return CheckStatus::Running;
        }
        if self.passed_count == 0 && self.failed_count == 0 {
            return CheckStatus::Pending;
        }
        if self.failed_count > 0 {
            return CheckStatus::Fail;
        }
        CheckStatus::Pass
    }

    /// Format summary for display
    pub fn format_summary(&self) -> String {
        if self.is_running {
            "Running diagnostics...".to_string()
        } else if self.passed_count == 0 && self.failed_count == 0 {
            "Press 'r' to run diagnostics".to_string()
        } else {
            format!(
                "Passed: {} | Failed: {} | Total: {}",
                self.passed_count,
                self.failed_count,
                self.checks.len()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_status_default() {
        let status = CheckStatus::default();
        assert_eq!(status, CheckStatus::Pending);
    }

    #[test]
    fn test_check_status_is_complete() {
        assert!(CheckStatus::Pass.is_complete());
        assert!(CheckStatus::Fail.is_complete());
        assert!(CheckStatus::Error.is_complete());
        assert!(!CheckStatus::Running.is_complete());
        assert!(!CheckStatus::Pending.is_complete());
    }

    #[test]
    fn test_check_status_symbols() {
        assert_eq!(CheckStatus::Pass.symbol(), "[PASS]");
        assert_eq!(CheckStatus::Fail.symbol(), "[FAIL]");
        assert_eq!(CheckStatus::Running.symbol(), "[....]");
        assert_eq!(CheckStatus::Pending.symbol(), "[    ]");
        assert_eq!(CheckStatus::Error.symbol(), "[ERR!]");
    }

    #[test]
    fn test_check_category_display_names() {
        assert_eq!(CheckCategory::System.display_name(), "System");
        assert_eq!(CheckCategory::Network.display_name(), "Network");
        assert_eq!(CheckCategory::Service.display_name(), "Service");
        assert_eq!(CheckCategory::Config.display_name(), "Config");
    }

    #[test]
    fn test_doctor_state_creation() {
        let state = DoctorState::new();
        assert!(!state.is_running);
        assert!(state.last_run.is_none());
        assert_eq!(state.passed_count, 0);
        assert_eq!(state.failed_count, 0);
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_doctor_state_default_checks() {
        let state = DoctorState::new();
        // Should have default checks
        assert!(!state.checks.is_empty());
        // All checks should be pending initially
        for check in &state.checks {
            assert_eq!(check.status, CheckStatus::Pending);
        }
    }

    #[test]
    fn test_doctor_state_update_check() {
        let mut state = DoctorState::new();
        state.update_check("CPU Cores", CheckStatus::Pass, "16 cores available", None);

        let cpu_check = state.checks.iter().find(|c| c.name == "CPU Cores");
        assert!(cpu_check.is_some());
        let cpu_check = cpu_check.unwrap();
        assert_eq!(cpu_check.status, CheckStatus::Pass);
        assert_eq!(cpu_check.message, "16 cores available");
    }

    #[test]
    fn test_doctor_state_update_check_with_hint() {
        let mut state = DoctorState::new();
        state.update_check(
            "Memory (RAM)",
            CheckStatus::Fail,
            "8GB available",
            Some("Upgrade to 32GB RAM"),
        );

        let mem_check = state.checks.iter().find(|c| c.name == "Memory (RAM)");
        assert!(mem_check.is_some());
        let mem_check = mem_check.unwrap();
        assert_eq!(mem_check.status, CheckStatus::Fail);
        assert_eq!(mem_check.fix_hint, Some("Upgrade to 32GB RAM".to_string()));
    }

    #[test]
    fn test_doctor_state_recalculate_summary() {
        let mut state = DoctorState::new();
        state.update_check("CPU Cores", CheckStatus::Pass, "OK", None);
        state.update_check("Memory (RAM)", CheckStatus::Pass, "OK", None);
        // PHASE-5-FIX: Renamed to "Disk Space (OS)"
        state.update_check("Disk Space (OS)", CheckStatus::Fail, "Low", None);

        state.recalculate_summary();

        assert_eq!(state.passed_count, 2);
        assert_eq!(state.failed_count, 1);
    }

    #[test]
    fn test_doctor_state_selected_check() {
        let state = DoctorState::new();
        let selected = state.selected_check();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "CPU Cores");
    }

    #[test]
    fn test_doctor_state_select_next() {
        let mut state = DoctorState::new();
        assert_eq!(state.selected_index, 0);

        state.select_next();
        assert_eq!(state.selected_index, 1);

        // Test wrapping
        state.selected_index = state.checks.len() - 1;
        state.select_next();
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_doctor_state_select_prev() {
        let mut state = DoctorState::new();
        state.selected_index = 2;

        state.select_prev();
        assert_eq!(state.selected_index, 1);

        // Test wrapping
        state.selected_index = 0;
        state.select_prev();
        assert_eq!(state.selected_index, state.checks.len() - 1);
    }

    #[test]
    fn test_doctor_state_start_checks() {
        let mut state = DoctorState::new();
        state.start_checks();

        assert!(state.is_running);
        for check in &state.checks {
            assert_eq!(check.status, CheckStatus::Running);
        }
    }

    #[test]
    fn test_doctor_state_finish_checks() {
        let mut state = DoctorState::new();
        state.start_checks();
        state.update_check("CPU Cores", CheckStatus::Pass, "OK", None);
        state.update_check("Memory (RAM)", CheckStatus::Fail, "Low", None);

        state.finish_checks();

        assert!(!state.is_running);
        assert!(state.last_run.is_some());
        assert_eq!(state.passed_count, 1);
        assert_eq!(state.failed_count, 1);
    }

    #[test]
    fn test_doctor_state_checks_by_category() {
        let state = DoctorState::new();
        let system_checks = state.checks_by_category(CheckCategory::System);

        assert!(!system_checks.is_empty());
        for check in system_checks {
            assert_eq!(check.category, CheckCategory::System);
        }
    }

    #[test]
    fn test_doctor_state_all_passed() {
        let mut state = DoctorState::new();

        // Initially not all passed (no checks run)
        assert!(!state.all_passed());

        // Pass all checks
        for check in &mut state.checks {
            check.status = CheckStatus::Pass;
        }
        state.recalculate_summary();

        assert!(state.all_passed());

        // Fail one check
        state.checks[0].status = CheckStatus::Fail;
        state.recalculate_summary();

        assert!(!state.all_passed());
    }

    #[test]
    fn test_doctor_state_overall_status() {
        let mut state = DoctorState::new();

        // Pending when nothing run
        assert_eq!(state.overall_status(), CheckStatus::Pending);

        // Running when in progress
        state.is_running = true;
        assert_eq!(state.overall_status(), CheckStatus::Running);

        // Fail when any failed
        state.is_running = false;
        state.failed_count = 1;
        assert_eq!(state.overall_status(), CheckStatus::Fail);

        // Pass when all passed
        state.failed_count = 0;
        state.passed_count = state.checks.len();
        assert_eq!(state.overall_status(), CheckStatus::Pass);
    }

    #[test]
    fn test_doctor_state_format_summary() {
        let mut state = DoctorState::new();

        // Initial state
        assert_eq!(state.format_summary(), "Press 'r' to run diagnostics");

        // Running state
        state.is_running = true;
        assert_eq!(state.format_summary(), "Running diagnostics...");

        // Completed state
        state.is_running = false;
        state.passed_count = 8;
        state.failed_count = 2;
        // PHASE-5-FIX: Updated total to 16 (removed Node Uptime check)
        // Linux: 16 checks (with MPT Storage), Other: 15 checks (without MPT Storage)
        #[cfg(target_os = "linux")]
        let total = 16;
        #[cfg(not(target_os = "linux"))]
        let total = 15;
        assert_eq!(
            state.format_summary(),
            format!("Passed: 8 | Failed: 2 | Total: {}", total)
        );
    }

    #[test]
    fn test_doctor_check_clone() {
        let check = DoctorCheck {
            name: "Test Check".to_string(),
            category: CheckCategory::System,
            status: CheckStatus::Pass,
            message: "All good".to_string(),
            fix_hint: Some("No fix needed".to_string()),
        };

        let cloned = check.clone();
        assert_eq!(check.name, cloned.name);
        assert_eq!(check.status, cloned.status);
    }

    #[test]
    fn test_doctor_state_clone() {
        let state = DoctorState::new();
        let cloned = state.clone();

        assert_eq!(state.selected_index, cloned.selected_index);
        assert_eq!(state.is_running, cloned.is_running);
        assert_eq!(state.checks.len(), cloned.checks.len());
    }
}
