use crate::ops::parser_status;
use crate::shared::types_v5::ParserStatusInput;
use serde_json::Value;

pub fn handle(arguments: Value) -> Value {
    let input: ParserStatusInput =
        serde_json::from_value(arguments).unwrap_or(ParserStatusInput { language: None });

    let result = parser_status::status(input.language.as_deref());

    serde_json::to_value(result).unwrap_or_else(
        |e| serde_json::json!({ "error": { "code": "internal_error", "message": e.to_string() } }),
    )
}
