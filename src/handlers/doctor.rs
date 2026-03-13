//! Doctor command handler
//!
//! Run diagnostics and display health check results.

use crate::config::Config;
use crate::doctor::Doctor;
use anyhow::Result;
use colored::Colorize;

/// Execute doctor command - run diagnostics
pub async fn execute(config: &Config) -> Result<()> {
    println!("{}", "MonadNode Manager - Doctor".cyan().bold());
    println!("{}", "==========================".cyan());
    println!();

    let doctor = Doctor::new(config);
    let report = doctor.run_diagnostics().await?;

    // Print results
    for check in &report.checks {
        let status = if check.passed {
            "OK".green().to_string()
        } else {
            "X".red().to_string()
        };
        println!("{} {} - {}", status, check.name, check.message);
    }

    println!();
    println!(
        "Summary: {} passed, {} failed",
        report.passed, report.failed
    );

    if !report.issues.is_empty() {
        println!();
        println!("{}", "Issues Found:".yellow().bold());
        for issue in &report.issues {
            println!("  {} {}", "!".yellow(), issue);
        }
    }

    Ok(())
}
