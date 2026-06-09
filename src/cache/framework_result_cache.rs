//! Framework extraction result cache.
//! Cache key: parse_key + extractor_name + extractor_version

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn framework_cache_key(parse_key: &str, extractor_name: &str, version: &str) -> String {
    let mut hasher = DefaultHasher::new();
    parse_key.hash(&mut hasher);
    extractor_name.hash(&mut hasher);
    version.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
