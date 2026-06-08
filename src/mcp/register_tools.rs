//! Tool registry — single dummy tool for phase 1.
use serde_json::{json, Value};

/// Tool specification used by the transport dispatcher.
pub struct ToolSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema: Value,
    pub handler: fn(Value) -> Value,
}

/// Return the V1 tool list (only `ast_health_check`).
pub fn tools() -> Vec<ToolSpec> {
    vec![ToolSpec {
        name: "ast_health_check",
        description: "Health-check stub. Returns ok=true when the server is operational.",
        input_schema: json!({
            "type": "object",
            "properties": {},
            "required": []
        }),
        handler: |_args| json!({ "ok": true }),
    }]
}

/// Dispatch a tool call by name, passing `arguments` to the handler.
/// Returns `None` if the tool name is not registered.
pub fn dispatch(name: &str, arguments: Value) -> Option<Value> {
    for tool in tools() {
        if tool.name == name {
            return Some((tool.handler)(arguments));
        }
    }
    None
}
