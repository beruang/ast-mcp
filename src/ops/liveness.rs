use std::time::Instant;

use crate::shared::types_v5::{LivenessResult, MemoryUsage};

pub fn check(started_at: Instant) -> LivenessResult {
    let uptime_ms = started_at.elapsed().as_millis() as u64;

    LivenessResult {
        alive: true,
        uptime_ms,
        started_at: format!("{:?}", started_at),
        memory: Some(MemoryUsage { rss_bytes: None, heap_bytes: None }),
    }
}
