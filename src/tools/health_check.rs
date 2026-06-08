use serde_json::json;

use crate::config::defaults;
use crate::config::workspace::Workspace;
use crate::parser;

pub fn handle(workspace: &Workspace, _args: serde_json::Value) -> serde_json::Value {
    let parsers: Vec<_> = parser::registry::registry()
        .iter()
        .map(|d| {
            json!({
                "language": d.language.as_str(),
                "extensions": d.extensions,
                "available": true,
                "parser": format!("tree-sitter-{}", d.language.as_str()),
            })
        })
        .collect();
    json!({
        "workspacePath": workspace.root().display().to_string(),
        "ok": true,
        "parsers": parsers,
        "limits": {
            "maxFileBytes": defaults::MAX_FILE_BYTES,
            "maxNodes": defaults::MAX_NODES,
            "maxResults": defaults::MAX_RESULTS,
        }
    })
}
