//! RPC method definitions and types

/// Block information
#[derive(Debug, Clone)]
pub struct BlockInfo {
    pub number: u64,
    pub hash: String,
    pub parent_hash: String,
    pub timestamp: u64,
    pub gas_used: u64,
    pub gas_limit: u64,
    pub transaction_count: usize,
}

/// Peer information
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub id: String,
    pub address: String,
    pub latency_ms: Option<u64>,
}

/// Node information
#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub version: String,
    pub chain_id: u64,
    pub block_number: u64,
    pub peer_count: u64,
    pub sync_status: SyncStatus,
}

/// Sync status
#[derive(Debug, Clone)]
pub enum SyncStatus {
    Synced,
    Syncing {
        starting_block: u64,
        current_block: u64,
        highest_block: u64,
    },
}

impl SyncStatus {
    /// Get sync progress percentage (0-100)
    pub fn progress_percent(&self) -> Option<f64> {
        match self {
            SyncStatus::Synced => Some(100.0),
            SyncStatus::Syncing {
                current_block,
                highest_block,
                ..
            } => {
                if *highest_block > 0 {
                    Some((*current_block as f64 / *highest_block as f64) * 100.0)
                } else {
                    None
                }
            }
        }
    }

    /// Check if node is synced
    pub fn is_synced(&self) -> bool {
        matches!(self, SyncStatus::Synced)
    }
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheck {
    pub is_healthy: bool,
    pub block_number: Option<u64>,
    pub peer_count: Option<u64>,
    pub is_synced: bool,
    pub response_time_ms: u64,
    pub issues: Vec<String>,
}

impl HealthCheck {
    /// Create a new health check
    pub fn new() -> Self {
        Self {
            is_healthy: false,
            block_number: None,
            peer_count: None,
            is_synced: false,
            response_time_ms: 0,
            issues: Vec::new(),
        }
    }
}

impl Default for HealthCheck {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_status_progress() {
        let synced = SyncStatus::Synced;
        assert_eq!(synced.progress_percent(), Some(100.0));
        assert!(synced.is_synced());

        let syncing = SyncStatus::Syncing {
            starting_block: 0,
            current_block: 50,
            highest_block: 100,
        };
        assert_eq!(syncing.progress_percent(), Some(50.0));
        assert!(!syncing.is_synced());
    }

    #[test]
    fn test_health_check_default() {
        let check = HealthCheck::default();
        assert!(!check.is_healthy);
        assert!(check.issues.is_empty());
    }
}
