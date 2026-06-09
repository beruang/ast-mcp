use crate::observability::request_tracker::RequestTracker;
use crate::shared::types_v5::RequestLogInput;
use serde_json::Value;

pub fn handle(tracker: &RequestTracker, arguments: Value) -> Value {
    let input: RequestLogInput = serde_json::from_value(arguments).unwrap_or(RequestLogInput {
        tool: None,
        status: None,
        file_path: None,
        limit: None,
    });

    let limit = input.limit.unwrap_or(50);
    let result = tracker.query(
        input.tool.as_deref(),
        input.status.as_ref(),
        input.file_path.as_deref(),
        limit,
    );

    serde_json::to_value(result).unwrap_or_else(
        |e| serde_json::json!({ "error": { "code": "internal_error", "message": e.to_string() } }),
    )
}
