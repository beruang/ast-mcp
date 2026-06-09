use std::time::Instant;

use crate::ops::liveness;
use serde_json::Value;

pub fn handle(started_at: Instant) -> Value {
    let result = liveness::check(started_at);
    serde_json::to_value(result).unwrap_or_else(
        |e| serde_json::json!({ "error": { "code": "internal_error", "message": e.to_string() } }),
    )
}
