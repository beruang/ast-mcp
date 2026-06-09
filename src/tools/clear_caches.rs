use std::collections::BTreeMap;

use crate::cache::CacheManager;
use crate::observability::request_tracker::RequestTracker;
use crate::shared::types_v5::{AstCacheName, ClearCachesInput, ClearCachesResult};
use serde_json::Value;

pub fn handle(cache: &CacheManager, tracker: &RequestTracker, arguments: Value) -> Value {
    let input: ClearCachesInput =
        serde_json::from_value(arguments).unwrap_or(ClearCachesInput { caches: vec![] });

    let mut cleared = BTreeMap::new();

    let clear_all = input.caches.iter().any(|c| matches!(c, AstCacheName::All));

    for name in &input.caches {
        match name {
            AstCacheName::ParseTrees => {
                let count = cache.clear_parse_trees();
                cleared.insert("parse_trees".into(), count);
            }
            AstCacheName::QueryResults => {
                let count = cache.clear_query_results();
                cleared.insert("query_results".into(), count);
            }
            AstCacheName::FrameworkResults => {
                let count = cache.clear_framework_results();
                cleared.insert("framework_results".into(), count);
            }
            AstCacheName::RequestLog => {
                let count = tracker.clear(None);
                cleared.insert("request_log".into(), count);
            }
            AstCacheName::All => {
                // Handled below to avoid duplicate entries
            }
        }
    }

    if clear_all {
        let pc = cache.clear_parse_trees();
        cleared.insert("parse_trees".into(), pc);
        let rc = tracker.clear(None);
        cleared.insert("request_log".into(), rc);
    }

    serde_json::to_value(ClearCachesResult { cleared }).unwrap_or_else(
        |e| serde_json::json!({ "error": { "code": "internal_error", "message": e.to_string() } }),
    )
}
