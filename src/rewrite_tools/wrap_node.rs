//! `ast_wrap_node_preview` — wrap a syntax node with prefix/suffix, try/catch, or call expression.

use serde_json::Value;

use crate::config::workspace::Workspace;
use crate::rewrite::preview::preview_edits;
use crate::rewrite::validate::RewriteLimits;
use crate::safety::paths;
use crate::shared::types_v4::{
    AstWrapNodePreviewInput, PreviewOptions, RewriteOperation, RewritePreview, SafetyViolation,
    WrapRequest,
};
use crate::text::indentation;

pub fn handle(workspace: &Workspace, arguments: Value) -> Value {
    let input: AstWrapNodePreviewInput = match serde_json::from_value(arguments) {
        Ok(v) => v,
        Err(e) => {
            return serde_json::json!({
                "safe": false, "changed_files": [], "edit_count": 0,
                "diff": null, "edits": [], "parse_after_rewrite": null,
                "violations": [{"violation_type": "invalid_input", "message": format!("{}", e)}]
            });
        }
    };

    let result = wrap_node(workspace, &input);
    serde_json::to_value(result).unwrap_or_default()
}

fn wrap_node(workspace: &Workspace, input: &AstWrapNodePreviewInput) -> RewritePreview {
    let resolved = match paths::resolve_file(workspace, &input.file_path) {
        Ok(r) => r,
        Err(e) => return make_error(&input.file_path, &e.to_string()),
    };
    let source = match std::fs::read_to_string(&resolved.absolute) {
        Ok(s) => s,
        Err(e) => return make_error(&input.file_path, &e.to_string()),
    };

    // Get byte range for the target node
    let (byte_start, byte_end) =
        match crate::text::position_encoding::validate_range_in_bounds(&source, input.range) {
            Ok(b) => b,
            Err(e) => return make_error(&input.file_path, &e.to_string()),
        };

    let original_text = &source[byte_start..byte_end];
    let indent = indentation::indentation_string(&source, byte_start);

    let new_text = match &input.wrapper {
        WrapRequest::PrefixSuffix { prefix, suffix } => {
            format!("{}{}{}", prefix, original_text, suffix)
        }
        WrapRequest::TryCatch { catch_binding, catch_body } => {
            let binding = catch_binding.as_deref().unwrap_or("error");
            let indented_body = indentation::indent_text(original_text, &format!("{}  ", indent));
            let indented_catch = indentation::indent_text(catch_body, &format!("{}  ", indent));
            format!(
                "try {{\n{}\n{}}} catch ({}) {{\n{}\n{}}}",
                indented_body, indent, binding, indented_catch, indent
            )
        }
        WrapRequest::CallExpression { callee } => {
            format!("{}({})", callee, original_text)
        }
    };

    let operation = RewriteOperation::ReplaceNode {
        file_path: input.file_path.clone(),
        range: input.range,
        expected_node_kind: input.expected_node_kind.clone(),
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
