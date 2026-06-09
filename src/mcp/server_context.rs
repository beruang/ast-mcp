//! Shared server context — bundles all state passed to tool handlers.
use std::sync::Arc;
use std::time::Instant;

use crate::cache::CacheManager;
use crate::config::runtime_config::RuntimeConfigStore;
use crate::config::workspace::Workspace;
use crate::observability::request_tracker::RequestTracker;
use crate::scan::ScanRegistry;

pub struct ServerContext {
    pub workspace: Arc<Workspace>,
    pub runtime_config: Arc<RuntimeConfigStore>,
    pub request_tracker: Arc<RequestTracker>,
    pub cache_manager: Arc<CacheManager>,
    pub scan_registry: Arc<ScanRegistry>,
    pub started_at: Instant,
}

impl ServerContext {
    /// Create a minimal context for testing.
    pub fn for_testing(workspace: Workspace) -> Self {
        let ws = Arc::new(workspace);
        let cfg = RuntimeConfigStore::new(ws.root().to_string_lossy().to_string());
        let c = cfg.current();
        ServerContext {
            workspace: ws,
            runtime_config: Arc::new(cfg),
            request_tracker: Arc::new(RequestTracker::new(c.caches.request_log_max_entries)),
            cache_manager: Arc::new(CacheManager::new(
                c.caches.parse_tree_ttl_ms,
                c.caches.query_result_ttl_ms,
                c.caches.framework_result_ttl_ms,
                c.caches.max_cached_files,
            )),
            scan_registry: Arc::new(ScanRegistry::new()),
            started_at: Instant::now(),
        }
    }
}
