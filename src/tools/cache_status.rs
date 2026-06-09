use crate::cache::CacheManager;
use crate::observability::request_tracker::RequestTracker;
use crate::shared::types_v5::CacheStatusResult;
use serde_json::Value;

pub fn handle(cache: &CacheManager, tracker: &RequestTracker) -> Value {
    let mut status = cache.status();
    status.request_log.entries = tracker.len();
    status.request_log.max_entries = 500; // default, configurable in future

    serde_json::to_value(CacheStatusResult { caches: status }).unwrap_or_else(
        |e| serde_json::json!({ "error": { "code": "internal_error", "message": e.to_string() } }),
    )
}
