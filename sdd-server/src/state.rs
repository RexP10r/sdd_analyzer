use std::sync::Arc;

use tokio::sync::RwLock;

use sdd_core::models::{AnnotatedLocation, Requirement, Task};

/// @req SCS-API-002
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanStatus {
    Idle,
    Scanning,
    Completed,
    Failed(String),
}

/// @req SCS-API-002
pub struct AppStateInner {
    pub scan_status: ScanStatus,
    pub requirements: Vec<Requirement>,
    pub tasks: Vec<Task>,
    pub annotations: Vec<AnnotatedLocation>,
}

/// @req SCS-API-002
pub type AppState = Arc<RwLock<AppStateInner>>;

/// @req SCS-API-002
impl AppStateInner {
    pub fn new() -> Self {
        AppStateInner {
            scan_status: ScanStatus::Idle,
            requirements: Vec::new(),
            tasks: Vec::new(),
            annotations: Vec::new(),
        }
    }
}

/// @req SCS-API-002
impl Default for AppStateInner {
    fn default() -> Self {
        Self::new()
    }
}

/// @req SCS-API-002
pub fn new_app_state() -> AppState {
    Arc::new(RwLock::new(AppStateInner::new()))
}
