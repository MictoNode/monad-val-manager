//! Core TUI application functionality - creation and main event loop

use anyhow::{Context, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

use crate::rpc::RpcClient;
use crate::tui::screens::Screen;
use crate::tui::state::AppState;
use crate::tui::ToastManager;

use super::TuiApp;

impl TuiApp {
    /// Create new TUI application
    pub fn new(config: &crate::config::Config) -> Result<Self> {
        // BUG-008 FIX: Load environment variables from .env file BEFORE reading private key
        // This ensures that get_private_key() can find the PRIVATE_KEY environment variable
        crate::config::load_env().context("Failed to load .env file")?;

        let rpc_client = RpcClient::new(config.rpc_endpoint()).ok();

        let mut state = AppState::with_config(
            config.network.chain_id,
            config.network(),
            config.rpc_endpoint(),
        );

        // Load delegator address from private key if available
        if let Some(private_key_hex) = crate::config::get_private_key(config.network.network_type) {
            // Parse private key and derive address
            if let Ok(address) = Self::derive_address_from_private_key(&private_key_hex) {
                state.staking.set_address(address);
            }
        }

        Ok(Self {
            config: config.clone(),
            state,
            current_screen: Screen::default(),
            should_quit: false,
            last_update: std::time::Instant::now(),
            update_interval: Duration::from_secs(1),
            rpc_client,
            toast_manager: ToastManager::new(5),
            was_connected: false, // Track previous connection state for toasts
        })
    }

    /// Derive Ethereum address from private key hex string
    fn derive_address_from_private_key(private_key_hex: &str) -> Result<String> {
        use k256::ecdsa::SigningKey;
        use sha3::{Digest, Keccak256};

        // Remove 0x prefix if present
        let key_hex = private_key_hex
            .strip_prefix("0x")
            .unwrap_or(private_key_hex);

        // Parse private key
        let bytes = hex::decode(key_hex).context("Invalid private key hex")?;
        let signing_key = SigningKey::from_slice(&bytes).context("Invalid private key")?;

        // Get public key (SEC1 encoded point, uncompressed)
        let public_key = signing_key.verifying_key();
        let encoded_point = public_key.to_encoded_point(false);

        // Take last 20 bytes of Keccak256 hash of public key (without prefix byte)
        let public_key_bytes = encoded_point.as_bytes();
        let hash = Keccak256::digest(&public_key_bytes[1..]); // Skip 0x04 prefix byte

        let address_bytes = &hash[hash.len() - 20..];
        Ok(format!("0x{}", hex::encode(address_bytes)))
    }

    /// Run the TUI application
    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode().context("Failed to enable raw mode")?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
            .context("Failed to setup terminal")?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

        // Draw loading screen FIRST to avoid black screen
        terminal.draw(|f| {
            self.draw(f);
        })?;

        // Run the main loop
        let result = self.run_loop(&mut terminal).await;

        // Restore terminal
        disable_raw_mode().context("Failed to disable raw mode")?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .context("Failed to restore terminal")?;
        terminal.show_cursor().context("Failed to show cursor")?;

        result
    }

    /// Main event loop
    pub(crate) async fn run_loop<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<()>
    where
        B::Error: std::error::Error + Send + Sync + 'static,
    {
        loop {
            // Increment tick counter for animations
            self.state.tick = self.state.tick.wrapping_add(1);
            // Advance throbber animation state (modifies in-place)
            self.state.throbber_state.calc_next();

            // Draw UI
            if let Err(e) = terminal.draw(|f| {
                self.draw(f);
            }) {
                return Err(anyhow::anyhow!("Failed to draw UI: {}", e));
            }

            // Handle events with timeout for periodic updates
            match event::poll(Duration::from_millis(100)) {
                Ok(true) => {
                    // Event is available
                    match event::read() {
                        Ok(Event::Key(key)) => {
                            self.handle_key_event(key).await;
                        }
                        Ok(_) => {
                            // Ignore other events
                        }
                        Err(e) => {
                            // Log error but don't crash
                            tracing::warn!("Event read error: {}", e);
                        }
                    }
                }
                Ok(false) => {
                    // No event available (timeout)
                }
                Err(e) => {
                    tracing::warn!("Event poll error: {}", e);
                    // Don't return, keep the loop running
                }
            }

            // Periodic data refresh
            if self.last_update.elapsed() >= self.update_interval {
                self.refresh_data().await;
            }

            // Cleanup expired toasts
            self.toast_manager.cleanup();

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }
}
