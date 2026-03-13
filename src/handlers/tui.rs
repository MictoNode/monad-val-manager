//! TUI command handler
//!
//! Launch the terminal user interface dashboard.

use crate::config::Config;
use crate::tui::TuiApp;
use anyhow::Result;

/// Execute TUI dashboard
pub async fn execute(config: &Config) -> Result<()> {
    tracing::info!("Starting TUI dashboard");
    let mut app = TuiApp::new(config)?;
    app.run().await?;

    Ok(())
}
