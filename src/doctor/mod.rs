//! Doctor module - Smart diagnostics and troubleshooting

mod checks;

pub use checks::{parse_size_to_gb, Check, Doctor, DoctorReport};

/// Diagnostic check result
#[derive(Debug, Clone)]
pub struct DiagnosticCheck {
    pub name: String,
    pub passed: bool,
    pub message: String,
    pub severity: Severity,
}

/// Issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}
