use crate::observability::request_tracker::RequestTracker;
use crate::shared::types_v5::{ClearRequestLogInput, ClearRequestLogResult};
use serde_json::Value;

pub fn handle(tracker: &RequestTracker, arguments: Value) -> Value {
    let input: ClearRequestLogInput =
        serde_json::from_value(arguments).unwrap_or(ClearRequestLogInput { tool: None });

    let cleared = tracker.clear(input.tool.as_deref());

    serde_json::to_value(ClearRequestLogResult { cleared }).unwrap_or_else(
        |e| serde_json::json!({ "error": { "code": "internal_error", "message": e.to_string() } }),
    )
}
