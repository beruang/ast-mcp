//! `ast_modify_function_signature_preview` — Tree-sitter-based signature modification.

use serde_json::Value;
use tree_sitter::Node;

use crate::config::workspace::Workspace;
use crate::rewrite::preview::preview_edits;
use crate::rewrite::validate::RewriteLimits;
use crate::safety::paths;
use crate::shared::position::Range;
use crate::shared::types_v4::{
    AstModifyFunctionSignaturePreviewInput, FunctionSignatureOperation, PreviewOptions,
    RewriteOperation, RewritePreview, SafetyViolation,
};
use crate::text::position_encoding;

pub fn handle(workspace: &Workspace, arguments: Value) -> Value {
    let input: AstModifyFunctionSignaturePreviewInput = match serde_json::from_value(arguments) {
        Ok(v) => v,
        Err(e) => {
            return serde_json::json!({
                "safe": false, "changed_files": [], "edit_count": 0,
                "diff": null, "edits": [], "parse_after_rewrite": null,
                "violations": [{"violation_type": "invalid_input", "message": e.to_string()}]
            });
        }
    };
    let result = modify_signature(workspace, &input);
    serde_json::to_value(result).unwrap_or_default()
}

fn modify_signature(
    workspace: &Workspace,
    input: &AstModifyFunctionSignaturePreviewInput,
) -> RewritePreview {
    let resolved = match paths::resolve_file(workspace, &input.file_path) {
        Ok(r) => r,
        Err(e) => return make_err(&input.file_path, &e.to_string()),
    };
    let source = match std::fs::read_to_string(&resolved.absolute) {
        Ok(s) => s,
        Err(e) => return make_err(&input.file_path, &e.to_string()),
    };

    let ext = resolved.absolute.extension().and_then(|e| e.to_str()).unwrap_or("");
    let mut parser = tree_sitter::Parser::new();
    let lang_ok = match ext {
        "ts" | "tsx" => parser.set_language(&tree_sitter_typescript::language_typescript()),
        "js" | "jsx" => parser.set_language(&tree_sitter_javascript::language()),
        "py" => parser.set_language(&tree_sitter_python::language()),
        "go" => parser.set_language(&tree_sitter_go::language()),
        "rs" => parser.set_language(&tree_sitter_rust::language()),
        _ => return make_err(&input.file_path, &format!("unsupported language: .{}", ext)),
    };
    if lang_ok.is_err() {
        return make_err(&input.file_path, "failed to load parser");
    }
    let Some(tree) = parser.parse(&source, None) else {
        return make_err(&input.file_path, "parse failed");
    };

    // Find the function node at the range
    let (byte_start, byte_end) =
        match position_encoding::validate_range_in_bounds(&source, input.function_range) {
            Ok(b) => b,
            Err(e) => return make_err(&input.file_path, &e.to_string()),
        };

    let Some(func_node) = find_function_node(&tree.root_node(), byte_start, byte_end) else {
        return make_err(&input.file_path, "no function found at range");
    };

    let operation = build_operation(&func_node, &source, &input.file_path, &input.operation);
    let Some(operation) = operation else {
        return make_err(&input.file_path, "could not build signature operation");
    };

    let options = PreviewOptions {
        include_diff: input.include_diff.unwrap_or(true),
        parse_check: input.parse_check.unwrap_or(true),
        max_diff_bytes: 500_000,
    };

    preview_edits(workspace, &[operation], options, RewriteLimits::default())
}

fn find_function_node<'a>(node: &Node<'a>, start: usize, end: usize) -> Option<Node<'a>> {
    let func_kinds = [
        "function_declaration",
        "function_expression",
        "arrow_function",
        "method_definition",
        "generator_function_declaration",
    ];
    if func_kinds.contains(&node.kind()) && node.start_byte() <= start && node.end_byte() >= end {
        return Some(*node);
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if let Some(found) = find_function_node(&child, start, end) {
                return Some(found);
            }
        }
    }
    None
}

fn build_operation(
    func_node: &Node,
    source: &str,
    file_path: &str,
    op: &FunctionSignatureOperation,
) -> Option<RewriteOperation> {
    match op {
        FunctionSignatureOperation::ReplaceSignature { new_signature_text } => {
            // Get the signature portion (name + params, excluding body)
            let sig_range = extract_signature_range(func_node, source);
            Some(RewriteOperation::ReplaceRange {
                file_path: file_path.to_string(),
                range: sig_range,
                new_text: new_signature_text.clone(),
            })
        }
        FunctionSignatureOperation::AddParameter { parameter_text, position } => {
            let params_node = find_parameters_node(func_node)?;
            let params_text = params_node.utf8_text(source.as_bytes()).ok()?;

            // Build new parameter list
            let inner = params_text.trim().trim_start_matches('(').trim_end_matches(')').trim();
            let mut params: Vec<&str> = if inner.is_empty() {
                Vec::new()
            } else {
                inner.split(',').map(|s| s.trim()).collect()
            };

            let pos = position.unwrap_or(params.len() as u32) as usize;
            let pos = pos.min(params.len());
            params.insert(pos, parameter_text.as_str());

            let new_text = format!("({})", params.join(", "));
            let start = params_node.start_position();
            let end = params_node.end_position();

            Some(RewriteOperation::ReplaceRange {
                file_path: file_path.to_string(),
                range: Range {
                    start: crate::shared::position::Position {
                        line: start.row as u32,
                        character: start.column as u32,
                    },
                    end: crate::shared::position::Position {
                        line: end.row as u32,
                        character: end.column as u32,
                    },
                },
                new_text,
            })
        }
        FunctionSignatureOperation::RemoveParameter { parameter_name } => {
            let params_node = find_parameters_node(func_node)?;
            let params_text = params_node.utf8_text(source.as_bytes()).ok()?;
            let inner = params_text.trim().trim_start_matches('(').trim_end_matches(')').trim();

            let params: Vec<&str> = if inner.is_empty() {
                Vec::new()
            } else {
                inner.split(',').map(|s| s.trim()).collect()
            };

            let orig_len = params.len();
            let remaining: Vec<&str> =
                params.into_iter().filter(|p| !p.contains(parameter_name.as_str())).collect();

            if remaining.len() == orig_len {
                return None; // Parameter not found
            }

            let new_text = format!("({})", remaining.join(", "));
            let start = params_node.start_position();
            let end = params_node.end_position();

            Some(RewriteOperation::ReplaceRange {
                file_path: file_path.to_string(),
                range: Range {
                    start: crate::shared::position::Position {
                        line: start.row as u32,
                        character: start.column as u32,
                    },
                    end: crate::shared::position::Position {
                        line: end.row as u32,
                        character: end.column as u32,
                    },
                },
                new_text,
            })
        }
        FunctionSignatureOperation::RenameParameter {
            old_name,
            new_name,
            rename_body_occurrences,
        } => {
            let params_node = find_parameters_node(func_node)?;
            let params_text = params_node.utf8_text(source.as_bytes()).ok()?;
            let new_params_text = params_text.replace(old_name.as_str(), new_name.as_str());

            let start = params_node.start_position();
            let end = params_node.end_position();

            // Note: body occurrences would need a separate pass — for now just rename in params
            let _ = rename_body_occurrences;

            Some(RewriteOperation::ReplaceRange {
                file_path: file_path.to_string(),
                range: Range {
                    start: crate::shared::position::Position {
                        line: start.row as u32,
                        character: start.column as u32,
                    },
                    end: crate::shared::position::Position {
                        line: end.row as u32,
                        character: end.column as u32,
                    },
                },
                new_text: new_params_text,
            })
        }
    }
}

/// Extract the range from function name start to the end of parameters (before body/return type).
fn extract_signature_range(func_node: &Node<'_>, _source: &str) -> Range {
    let name_node = func_node.child_by_field_name("name");
    let params_node = find_parameters_node(func_node);

    let start = name_node.map(|n| n.start_position()).unwrap_or(func_node.start_position());

    let end = params_node.map(|p| p.end_position()).unwrap_or(func_node.end_position());

    Range {
        start: crate::shared::position::Position {
            line: start.row as u32,
            character: start.column as u32,
        },
        end: crate::shared::position::Position {
            line: end.row as u32,
            character: end.column as u32,
        },
    }
}

fn find_parameters_node<'a>(func_node: &Node<'a>) -> Option<Node<'a>> {
    if let Some(params) = func_node.child_by_field_name("parameters") {
        return Some(params);
    }
    // Search children for formal_parameters / parameters node
    for i in 0..func_node.child_count() {
        if let Some(child) = func_node.child(i) {
            if child.kind().contains("parameters") || child.kind() == "parameter_list" {
                return Some(child);
            }
            if let Some(grandchild) = find_parameters_node(&child) {
                return Some(grandchild);
            }
        }
    }
    None
}

fn make_err(path: &str, msg: &str) -> RewritePreview {
    RewritePreview {
        safe: false,
        changed_files: vec![],
        edit_count: 0,
        diff: None,
        edits: vec![],
        parse_after_rewrite: None,
        violations: vec![SafetyViolation {
            violation_type: "rewrite_parameter_not_found".into(),
            message: format!("{}: {}", path, msg),
            file_path: Some(path.to_string()),
            details: None,
        }],
    }
}
