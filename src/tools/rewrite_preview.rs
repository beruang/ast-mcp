//! `ast_rewrite_preview` tool handler — generic structural rewrite preview.

use serde_json::{json, Value};

use crate::config::workspace::Workspace;
use crate::rewrite::preview::preview_edits;
use crate::rewrite::validate::RewriteLimits;
use crate::shared::types_v4::{AstRewritePreviewInput, PreviewOptions};

pub fn handle(workspace: &Workspace, arguments: Value) -> Value {
    let input: AstRewritePreviewInput = match serde_json::from_value(arguments) {
        Ok(v) => v,
        Err(e) => {
            return json!({
                "safe": false,
                "changed_files": [],
                "edit_count": 0,
                "diff": null,
                "edits": [],
                "parse_after_rewrite": null,
                "violations": [{
                    "violation_type": "invalid_input",
                    "message": format!("failed to parse input: {}", e)
                }]
            });
        }
    };

    let options = PreviewOptions {
        include_diff: input.include_diff.unwrap_or(true),
        parse_check: input.parse_check.unwrap_or(true),
        max_diff_bytes: 500_000,
    };

    let limits = RewriteLimits {
        max_changed_files: input.max_changed_files.unwrap_or(20),
        max_edits: input.max_edits.unwrap_or(200),
        max_new_text_per_edit: 100_000,
        max_parse_after_files: 20,
    };

    let result = preview_edits(workspace, &input.operations, options, limits);
    serde_json::to_value(result).unwrap_or_default()
}
