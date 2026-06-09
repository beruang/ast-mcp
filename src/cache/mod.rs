pub mod framework_result_cache;
pub mod parse_tree_cache;
pub mod query_result_cache;

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::shared::types_v5::{AstCacheStatus, CacheSectionStatus, RequestLogCacheStatus};
use parse_tree_cache::CachedTree;

/// A simple TTL cache with a max entry count.
pub struct TtlCache<K, V> {
    entries: HashMap<K, (V, Instant)>,
    max_entries: usize,
    ttl: Duration,
}

impl<K: std::hash::Hash + Eq + Clone, V> TtlCache<K, V> {
    pub fn new(max_entries: usize, ttl_ms: u64) -> Self {
        TtlCache { entries: HashMap::new(), max_entries, ttl: Duration::from_millis(ttl_ms) }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        if let Some((_v, inserted_at)) = self.entries.get(key) {
            if inserted_at.elapsed() < self.ttl {
                return self.entries.get(key).map(|(v, _)| v);
            }
            self.entries.remove(key);
        }
        None
    }

    pub fn insert(&mut self, key: K, value: V) {
        if self.entries.len() >= self.max_entries {
            // Simple eviction: remove a random entry
            if let Some(k) = self.entries.keys().next().cloned() {
                self.entries.remove(&k);
            }
        }
        self.entries.insert(key, (value, Instant::now()));
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.entries.remove(key).map(|(v, _)| v)
    }

    pub fn clear(&mut self) -> usize {
        let count = self.entries.len();
        self.entries.clear();
        count
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn ttl_ms(&self) -> u64 {
        self.ttl.as_millis() as u64
    }

    pub fn max_entries(&self) -> usize {
        self.max_entries
    }

    pub fn estimated_bytes(&self) -> usize {
        self.entries.len() * std::mem::size_of::<(K, V, Instant)>()
    }
}

pub struct CacheManager {
    pub parse_trees: Mutex<TtlCache<String, CachedTree>>,
    pub query_results: Mutex<TtlCache<String, serde_json::Value>>,
    pub framework_results: Mutex<TtlCache<String, serde_json::Value>>,
    parse_ttl_ms: u64,
    query_ttl_ms: u64,
    framework_ttl_ms: u64,
    max_cached_files: usize,
}

impl CacheManager {
    pub fn new(
        parse_ttl_ms: u64,
        query_ttl_ms: u64,
        framework_ttl_ms: u64,
        max_cached_files: usize,
    ) -> Self {
        CacheManager {
            parse_trees: Mutex::new(TtlCache::new(max_cached_files, parse_ttl_ms)),
            query_results: Mutex::new(TtlCache::new(max_cached_files * 2, query_ttl_ms)),
            framework_results: Mutex::new(TtlCache::new(max_cached_files, framework_ttl_ms)),
            parse_ttl_ms,
            query_ttl_ms,
            framework_ttl_ms,
            max_cached_files,
        }
    }

    pub fn status(&self) -> AstCacheStatus {
        let parse = self.parse_trees.lock().unwrap();
        let query = self.query_results.lock().unwrap();
        let framework = self.framework_results.lock().unwrap();

        AstCacheStatus {
            parse_trees: CacheSectionStatus {
                entries: parse.len(),
                max_entries: Some(self.max_cached_files),
                ttl_ms: self.parse_ttl_ms,
                estimated_bytes: Some(parse.estimated_bytes()),
            },
            query_results: CacheSectionStatus {
                entries: query.len(),
                max_entries: Some(self.max_cached_files * 2),
                ttl_ms: self.query_ttl_ms,
                estimated_bytes: Some(query.estimated_bytes()),
            },
            framework_results: CacheSectionStatus {
                entries: framework.len(),
                max_entries: Some(self.max_cached_files),
                ttl_ms: self.framework_ttl_ms,
                estimated_bytes: Some(framework.estimated_bytes()),
            },
            request_log: RequestLogCacheStatus {
                entries: 0, // populated by caller
                max_entries: 0,
            },
        }
    }

    pub fn clear_parse_trees(&self) -> usize {
        let count = self.parse_trees.lock().unwrap().clear();
        // Cascading: clear dependent caches
        self.query_results.lock().unwrap().clear();
        self.framework_results.lock().unwrap().clear();
        count
    }

    pub fn clear_query_results(&self) -> usize {
        self.query_results.lock().unwrap().clear()
    }

    pub fn clear_framework_results(&self) -> usize {
        self.framework_results.lock().unwrap().clear()
    }
}
