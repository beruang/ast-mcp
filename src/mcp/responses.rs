//! JSON-RPC response envelope helpers.
use serde_json::json;

/// Wrap `payload` in the `{"content":[{"type":"text","text":...}]}` envelope.
pub fn text_envelope(payload: serde_json::Value) -> serde_json::Value {
    json!({
        "content": [{
            "type": "text",
            "text": payload.to_string()
        }]
    })
}

/// Build a JSON-RPC error envelope.
pub fn error_envelope(code: i32, message: &str) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "error": {
            "code": code,
            "message": message
        }
    })
}

/// Build a JSON-RPC success envelope with `result`.
pub fn success_envelope(result: serde_json::Value, id: serde_json::Value) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}
