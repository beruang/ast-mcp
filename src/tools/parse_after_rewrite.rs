//! `ast_parse_after_rewrite` tool handler — apply edits in memory, re-parse, report errors.

use serde_json::{json, Value};

use crate::config::workspace::Workspace;
use crate::rewrite::parse_after::parse_after_edits;
use crate::shared::types_v4::AstParseAfterRewriteInput;

pub fn handle(workspace: &Workspace, arguments: Value) -> Value {
    let input: AstParseAfterRewriteInput = match serde_json::from_value(arguments) {
        Ok(v) => v,
        Err(e) => {
            return json!({
                "ok": false,
                "changed_files_checked": 0,
                "files_with_syntax_errors": [],
                "syntax_errors": [{
                    "file_path": "",
                    "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 0 } },
                    "node_kind": "ERROR",
                    "message": format!("failed to parse input: {}", e)
                }]
            });
        }
    };

    let result = parse_after_edits(workspace, &input.edits);
    serde_json::to_value(result).unwrap_or_default()
}
