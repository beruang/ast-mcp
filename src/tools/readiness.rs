use crate::config::workspace::Workspace;
use crate::ops::readiness;
use crate::shared::types_v5::ReadinessInput;
use serde_json::Value;

pub fn handle(workspace: &Workspace, arguments: Value) -> Value {
    let input: ReadinessInput =
        serde_json::from_value(arguments).unwrap_or(ReadinessInput { require_languages: None });

    let result = readiness::check(workspace, input.require_languages.as_deref());

    serde_json::to_value(result).unwrap_or_else(
        |e| serde_json::json!({ "error": { "code": "internal_error", "message": e.to_string() } }),
    )
}
