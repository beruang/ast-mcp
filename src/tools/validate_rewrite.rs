//! `ast_validate_rewrite` tool handler — cheap validation without diff generation.

use serde_json::{json, Value};

use crate::config::workspace::Workspace;
use crate::rewrite::validate::{validate_rewrite_operations, RewriteLimits};
use crate::shared::types_v4::AstValidateRewriteInput;

pub fn handle(workspace: &Workspace, arguments: Value) -> Value {
    let input: AstValidateRewriteInput = match serde_json::from_value(arguments) {
        Ok(v) => v,
        Err(e) => {
            return json!({
                "safe": false,
                "changed_files": [],
                "edit_count": 0,
                "violations": [{
                    "violation_type": "invalid_input",
                    "message": format!("failed to parse input: {}", e)
                }]
            });
        }
    };

    let limits = RewriteLimits {
        max_changed_files: input.max_changed_files.unwrap_or(20),
        max_edits: input.max_edits.unwrap_or(200),
        max_new_text_per_edit: 100_000,
        max_parse_after_files: 20,
    };

    let result = validate_rewrite_operations(workspace, &input.operations, limits);
    serde_json::to_value(result).unwrap_or_default()
}
