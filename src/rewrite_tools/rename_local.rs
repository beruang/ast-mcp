//! `ast_rename_local_preview` — conservative local rename with Tree-sitter scope inference.

use serde_json::Value;

use crate::config::workspace::Workspace;
use crate::rewrite::preview::preview_edits;
use crate::rewrite::validate::RewriteLimits;
use crate::safety::paths;
use crate::shared::position::Range;
use crate::shared::types_v4::{
    AstRenameLocalPreviewInput, PreviewOptions, RewriteOperation, RewritePreview, SafetyViolation,
};
use crate::text::position_encoding;

pub fn handle(workspace: &Workspace, arguments: Value) -> Value {
    let input: AstRenameLocalPreviewInput = match serde_json::from_value(arguments) {
        Ok(v) => v,
        Err(e) => {
            return serde_json::json!({
                "safe": false, "changed_files": [], "edit_count": 0,
                "diff": null, "edits": [], "parse_after_rewrite": null,
                "violations": [{"violation_type": "invalid_input", "message": e.to_string()}]
            });
        }
    };
    let result = rename_local(workspace, &input);
    serde_json::to_value(result).unwrap_or_default()
}

fn rename_local(workspace: &Workspace, input: &AstRenameLocalPreviewInput) -> RewritePreview {
    let resolved = match paths::resolve_file(workspace, &input.file_path) {
        Ok(r) => r,
        Err(e) => return make_error(&input.file_path, "outside_workspace", &e.to_string()),
    };
    let source = match std::fs::read_to_string(&resolved.absolute) {
        Ok(s) => s,
        Err(e) => return make_error(&input.file_path, "file_not_found", &e.to_string()),
    };

    let ext = resolved.absolute.extension().and_then(|e| e.to_str()).unwrap_or("");

    // Parse with appropriate Tree-sitter parser
    let mut parser = tree_sitter::Parser::new();
    let lang_loaded = match ext {
        "ts" | "tsx" => parser.set_language(&tree_sitter_typescript::language_typescript()),
        "js" | "jsx" => parser.set_language(&tree_sitter_javascript::language()),
        "py" => parser.set_language(&tree_sitter_python::language()),
        "go" => parser.set_language(&tree_sitter_go::language()),
        "rs" => parser.set_language(&tree_sitter_rust::language()),
        _ => {
            return make_error(
                &input.file_path,
                "unsupported_language",
                &format!("unsupported: .{}", ext),
            );
        }
    };
    if lang_loaded.is_err() {
        return make_error(&input.file_path, "parse_failed", "failed to load language");
    }

    let Some(tree) = parser.parse(&source, None) else {
        return make_error(&input.file_path, "parse_failed", "tree-sitter returned None");
    };

    // Convert position to byte offset
    let target_byte = match position_encoding::position_to_byte(&source, input.position) {
        Ok(b) => b,
        Err(e) => return make_error(&input.file_path, "invalid_range", &e.to_string()),
    };

    // Find the identifier node at this position
    let Some(ident_node) = find_node_at_byte(&tree.root_node(), target_byte) else {
        return make_error(&input.file_path, "rewrite_identifier_not_found", "no node at position");
    };

    if ident_node.kind() != "identifier" && ident_node.kind() != "property_identifier" {
        return RewritePreview {
            safe: false,
            changed_files: vec![],
            edit_count: 0,
            diff: None,
            edits: vec![],
            parse_after_rewrite: None,
            violations: vec![SafetyViolation {
                violation_type: "rewrite_unsafe_local_rename".into(),
                message: format!("node is not an identifier (found {})", ident_node.kind()),
                file_path: Some(input.file_path.clone()),
                details: None,
            }],
        };
    }

    let old_name = match ident_node.utf8_text(source.as_bytes()) {
        Ok(n) => n.to_string(),
        Err(_) => {
            return make_error(&input.file_path, "internal_error", "failed to read identifier text")
        }
    };

    // Check if this is a property key or member access property → reject
    if let Some(parent) = ident_node.parent() {
        if parent.kind() == "member_expression" {
            let prop = parent.child_by_field_name("property");
            if prop.is_some_and(|p| p.id() == ident_node.id()) {
                return make_error(
                    &input.file_path,
                    "rewrite_unsafe_local_rename",
                    "cannot rename property access name; use LSP for semantic rename",
                );
            }
        }
        if parent.kind() == "pair" || parent.kind() == "object" {
            // Object key — could be shorthand, but too risky
            if !is_shorthand_property(&parent, &ident_node) {
                return make_error(
                    &input.file_path,
                    "rewrite_unsafe_local_rename",
                    "cannot rename object property key",
                );
            }
        }
    }

    // Check if it's an imported/exported name or top-level
    if is_imported_or_exported(&tree.root_node(), &ident_node, target_byte) {
        return make_error(
            &input.file_path,
            "rewrite_unsafe_local_rename",
            "identifier appears to be imported or exported; use LSP for semantic rename",
        );
    }

    // Determine scope: use provided scope_range, else find enclosing function/block
    let scope_node = if let Some(scope_range) = input.scope_range {
        // Use the provided range to find the scope node
        if let Ok((s_byte, e_byte)) =
            position_encoding::validate_range_in_bounds(&source, scope_range)
        {
            find_node_at_byte_range(&tree.root_node(), s_byte, e_byte)
        } else {
            None
        }
    } else {
        find_enclosing_scope(&ident_node)
    };

    // Find all occurrences of old_name within the scope
    let root_node = tree.root_node();
    let scope_ref: &tree_sitter::Node = scope_node.as_ref().unwrap_or(&root_node);
    let mut rename_ranges: Vec<Range> = Vec::new();
    collect_rename_targets(scope_ref, &source, &old_name, target_byte, &mut rename_ranges);

    if rename_ranges.is_empty() {
        return RewritePreview {
            safe: true,
            changed_files: vec![input.file_path.clone()],
            edit_count: 0,
            diff: None,
            edits: vec![],
            parse_after_rewrite: None,
            violations: vec![],
        };
    }

    let operations: Vec<RewriteOperation> = rename_ranges
        .into_iter()
        .map(|range| RewriteOperation::ReplaceRange {
            file_path: input.file_path.clone(),
            range,
            new_text: input.new_name.clone(),
        })
        .collect();

    let options = PreviewOptions {
        include_diff: input.include_diff.unwrap_or(true),
        parse_check: input.parse_check.unwrap_or(true),
        max_diff_bytes: 500_000,
    };

    preview_edits(workspace, &operations, options, RewriteLimits::default())
}

// ── Tree-sitter helpers ──

fn find_node_at_byte<'a>(
    node: &tree_sitter::Node<'a>,
    byte: usize,
) -> Option<tree_sitter::Node<'a>> {
    if !node.byte_range().contains(&byte) {
        return None;
    }
    if node.child_count() == 0 {
        return Some(*node);
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.byte_range().contains(&byte) {
                return find_node_at_byte(&child, byte);
            }
        }
    }
    Some(*node)
}

fn find_node_at_byte_range<'a>(
    node: &tree_sitter::Node<'a>,
    start: usize,
    end: usize,
) -> Option<tree_sitter::Node<'a>> {
    let mut best: Option<tree_sitter::Node> = None;
    let mut best_size = usize::MAX;

    fn recurse<'a>(
        node: &tree_sitter::Node<'a>,
        start: usize,
        end: usize,
        best: &mut Option<tree_sitter::Node<'a>>,
        best_size: &mut usize,
    ) {
        if node.start_byte() <= start && node.end_byte() >= end {
            let size = node.end_byte() - node.start_byte();
            if size < *best_size {
                *best = Some(*node);
                *best_size = size;
            }
        }
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                recurse(&child, start, end, best, best_size);
            }
        }
    }

    recurse(node, start, end, &mut best, &mut best_size);
    best
}

fn find_enclosing_scope<'a>(node: &tree_sitter::Node<'a>) -> Option<tree_sitter::Node<'a>> {
    let mut current = *node;
    loop {
        match current.kind() {
            "function_declaration"
            | "function_expression"
            | "arrow_function"
            | "method_definition"
            | "class_declaration"
            | "block"
            | "module" => return Some(current),
            _ => {}
        }
        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            return Some(current);
        }
    }
}

fn is_imported_or_exported(
    root: &tree_sitter::Node,
    node: &tree_sitter::Node,
    target_byte: usize,
) -> bool {
    // Walk ancestors — if any is an import_statement or export_statement, reject
    let mut current = *node;
    loop {
        match current.kind() {
            "import_statement" | "export_statement" | "export_specifier" => return true,
            _ => {}
        }
        // Also check: is this node at the top level of the module?
        if current.parent().is_none() {
            break;
        }
        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            break;
        }
    }
    // Check if it's a top-level declaration by looking at depth
    let depth = node_depth(root, target_byte);
    depth <= 3 // Module → statement → declaration → identifier ≈ depth 3-4
}

fn node_depth(root: &tree_sitter::Node, byte: usize) -> usize {
    fn depth(node: &tree_sitter::Node, byte: usize, current: usize) -> usize {
        if !node.byte_range().contains(&byte) {
            return 0;
        }
        if node.child_count() == 0 {
            return current;
        }
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.byte_range().contains(&byte) {
                    return depth(&child, byte, current + 1);
                }
            }
        }
        current
    }
    depth(root, byte, 1)
}

fn is_shorthand_property(pair: &tree_sitter::Node, _node: &tree_sitter::Node) -> bool {
    // In shorthand `{ foo }`, the key and value are the same node
    pair.kind() == "pair" && pair.child_count() == 1
}

/// Collect all identifiers matching `old_name` within the scope subtree.
fn collect_rename_targets(
    scope: &tree_sitter::Node,
    source: &str,
    old_name: &str,
    original_byte: usize,
    results: &mut Vec<Range>,
) {
    if scope.child_count() == 0
        && scope.kind() == "identifier"
        && !scope.byte_range().contains(&original_byte)
    // skip the original if already in results
    {
        // Actually, we want ALL occurrences including the original
        if let Ok(text) = scope.utf8_text(source.as_bytes()) {
            if text == old_name {
                // Don't rename property keys or import names
                if let Some(parent) = scope.parent() {
                    if parent.kind() == "import_specifier"
                        || parent.kind() == "import_statement"
                        || parent.kind() == "export_statement"
                    {
                        return;
                    }
                    if parent.kind() == "member_expression" {
                        if let Some(prop) = parent.child_by_field_name("property") {
                            if prop.id() == scope.id() {
                                return;
                            }
                        }
                    }
                }
                let start = scope.start_position();
                let end = scope.end_position();
                results.push(Range {
                    start: crate::shared::position::Position {
                        line: start.row as u32,
                        character: start.column as u32,
                    },
                    end: crate::shared::position::Position {
                        line: end.row as u32,
                        character: end.column as u32,
                    },
                });
            }
        }
    }

    for i in 0..scope.child_count() {
        if let Some(child) = scope.child(i) {
            collect_rename_targets(&child, source, old_name, original_byte, results);
        }
    }
}

fn make_error(path: &str, violation_type: &str, msg: &str) -> RewritePreview {
    RewritePreview {
        safe: false,
        changed_files: vec![],
        edit_count: 0,
        diff: None,
        edits: vec![],
        parse_after_rewrite: None,
        violations: vec![SafetyViolation {
            violation_type: violation_type.into(),
            message: msg.to_string(),
            file_path: Some(path.to_string()),
            details: None,
        }],
    }
}
