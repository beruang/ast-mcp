use serde_json::json;

use crate::parser;

pub fn handle(_args: serde_json::Value) -> serde_json::Value {
    let languages: Vec<_> = parser::registry::registry()
        .iter()
        .map(|d| {
            json!({
                "language": d.language.as_str(),
                "extensions": d.extensions,
                "parser": format!("tree-sitter-{}", d.language.as_str()),
                "available": true,
            })
        })
        .collect();
    json!({ "languages": languages })
}
