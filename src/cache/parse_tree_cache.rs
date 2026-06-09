//! Parse tree cache — wraps TtlCache for parsed Tree-sitter trees.
//! Cache key: file_path + language + file_mtime + file_size

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A cached parse result.
pub struct CachedTree {
    // Tree-sitter trees are not Send+Sync by default, but we store source
    // and metadata for re-parsing if needed.
    pub source: String,
    pub root_kind: String,
    pub node_count: usize,
    pub parse_time_ms: u64,
}

/// Build a cache key from file metadata.
pub fn cache_key(
    workspace_path: &str,
    file_path: &str,
    mtime: u64,
    size: u64,
    language: &str,
) -> String {
    let mut hasher = DefaultHasher::new();
    workspace_path.hash(&mut hasher);
    file_path.hash(&mut hasher);
    mtime.hash(&mut hasher);
    size.hash(&mut hasher);
    language.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
