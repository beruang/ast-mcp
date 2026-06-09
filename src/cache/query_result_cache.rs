//! Query result cache.
//! Cache key: cache_key from parse_tree_cache + query_string

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn query_cache_key(parse_key: &str, query_string: &str) -> String {
    let mut hasher = DefaultHasher::new();
    parse_key.hash(&mut hasher);
    query_string.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
