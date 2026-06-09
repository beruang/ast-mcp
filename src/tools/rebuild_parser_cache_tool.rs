use crate::ops::rebuild_parser_cache;
use crate::shared::types_v5::RebuildParserCacheInput;
use serde_json::Value;

pub fn handle(arguments: Value) -> Value {
    let input: RebuildParserCacheInput =
        serde_json::from_value(arguments).unwrap_or(RebuildParserCacheInput { languages: None });

    let result = rebuild_parser_cache::rebuild(input.languages.as_deref());

    serde_json::to_value(result).unwrap_or_else(
        |e| serde_json::json!({ "error": { "code": "internal_error", "message": e.to_string() } }),
    )
}
