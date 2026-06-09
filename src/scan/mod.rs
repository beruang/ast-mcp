pub mod cancel;

use std::collections::HashMap;
use std::sync::{atomic::AtomicUsize, Arc, Mutex};
use std::time::Instant;

use cancel::Cancellable;

use crate::shared::types_v5::WorkspaceScanInfo;

pub struct ScanHandle {
    pub scan_id: String,
    pub tool: String,
    pub status: Mutex<String>,
    pub started_at: Instant,
    pub completed_at: Mutex<Option<Instant>>,
    pub files_discovered: AtomicUsize,
    pub files_processed: AtomicUsize,
    pub results_found: AtomicUsize,
    pub cancellable: Cancellable,
    pub error: Mutex<Option<String>>,
}

impl ScanHandle {
    pub fn to_info(&self) -> WorkspaceScanInfo {
        let status = self.status.lock().unwrap().clone();
        WorkspaceScanInfo {
            scan_id: self.scan_id.clone(),
            tool: self.tool.clone(),
            status,
            started_at: format!("{:?}", self.started_at),
            completed_at: self.completed_at.lock().unwrap().map(|t| format!("{:?}", t)),
            files_discovered: self.files_discovered.load(std::sync::atomic::Ordering::Relaxed),
            files_processed: self.files_processed.load(std::sync::atomic::Ordering::Relaxed),
            results_found: self.results_found.load(std::sync::atomic::Ordering::Relaxed),
            error: self.error.lock().unwrap().clone(),
        }
    }
}

pub struct ScanRegistry {
    active: Mutex<HashMap<String, Arc<ScanHandle>>>,
}

impl Default for ScanRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ScanRegistry {
    pub fn new() -> Self {
        ScanRegistry { active: Mutex::new(HashMap::new()) }
    }

    pub fn register(&self, handle: Arc<ScanHandle>) {
        self.active.lock().unwrap().insert(handle.scan_id.clone(), handle);
    }

    pub fn remove(&self, scan_id: &str) {
        self.active.lock().unwrap().remove(scan_id);
    }

    pub fn get(&self, scan_id: &str) -> Option<Arc<ScanHandle>> {
        self.active.lock().unwrap().get(scan_id).cloned()
    }

    pub fn list_all(&self) -> Vec<WorkspaceScanInfo> {
        self.active.lock().unwrap().values().map(|h| h.to_info()).collect()
    }
}

pub fn new_scan(tool: &str) -> Arc<ScanHandle> {
    Arc::new(ScanHandle {
        scan_id: uuid::Uuid::new_v4().to_string(),
        tool: tool.to_string(),
        status: Mutex::new("running".to_string()),
        started_at: Instant::now(),
        completed_at: Mutex::new(None),
        files_discovered: AtomicUsize::new(0),
        files_processed: AtomicUsize::new(0),
        results_found: AtomicUsize::new(0),
        cancellable: Cancellable::new(),
        error: Mutex::new(None),
    })
}
