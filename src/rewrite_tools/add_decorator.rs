//! `ast_add_decorator_preview` — insert a decorator/attribute before a target node.

use serde_json::Value;

use crate::config::workspace::Workspace;
use crate::rewrite::preview::preview_edits;
use crate::rewrite::validate::RewriteLimits;
use crate::safety::paths;
use crate::shared::types_v4::{
    AstAddDecoratorPreviewInput, PreviewOptions, RewriteOperation, RewritePreview, SafetyViolation,
};
use crate::text::indentation;

pub fn handle(workspace: &Workspace, arguments: Value) -> Value {
    let input: AstAddDecoratorPreviewInput = match serde_json::from_value(arguments) {
        Ok(v) => v,
        Err(e) => {
            return serde_json::json!({
                "safe": false, "changed_files": [], "edit_count": 0,
                "diff": null, "edits": [], "parse_after_rewrite": null,
                "violations": [{"violation_type": "invalid_input", "message": format!("{}", e)}]
            });
        }
    };

    let result = add_decorator(workspace, &input);
    serde_json::to_value(result).unwrap_or_default()
}

fn add_decorator(workspace: &Workspace, input: &AstAddDecoratorPreviewInput) -> RewritePreview {
    let resolved = match paths::resolve_file(workspace, &input.file_path) {
        Ok(r) => r,
        Err(e) => return make_error(&input.file_path, &e.to_string()),
    };
    let source = match std::fs::read_to_string(&resolved.absolute) {
        Ok(s) => s,
        Err(e) => return make_error(&input.file_path, &e.to_string()),
    };

    let (byte_start, _byte_end) =
        match crate::text::position_encoding::validate_range_in_bounds(&source, input.target_range)
        {
            Ok(b) => b,
            Err(e) => return make_error(&input.file_path, &e.to_string()),
        };

    let indent = indentation::indentation_string(&source, byte_start);

    let new_text = format!("{}\n{}", input.decorator_text, indent);

    let operation = RewriteOperation::InsertBeforeNode {
        file_path: input.file_path.clone(),
        range: input.target_range,
        expected_node_kind: input.expected_target_kind.clone(),
        new_text,
    };

    let options = PreviewOptions {
        include_diff: input.include_diff.unwrap_or(true),
        parse_check: input.parse_check.unwrap_or(true),
        max_diff_bytes: 500_000,
    };

    preview_edits(workspace, &[operation], options, RewriteLimits::default())
}

fn make_error(path: &str, msg: &str) -> RewritePreview {
    RewritePreview {
        safe: false,
        changed_files: vec![],
        edit_count: 0,
        diff: None,
        edits: vec![],
        parse_after_rewrite: None,
        violations: vec![SafetyViolation {
            violation_type: "invalid_range".into(),
            message: msg.to_string(),
            file_path: Some(path.to_string()),
            details: None,
        }],
    }
}
