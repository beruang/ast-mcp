//! Central rewrite validation engine — enforces all V4 safety rules.

use std::collections::HashMap;

use tree_sitter::Parser;

use crate::config::defaults::{
    MAX_NEW_TEXT_BYTES_PER_EDIT, MAX_PARSE_AFTER_REWRITE_FILES, MAX_REWRITE_CHANGED_FILES,
    MAX_REWRITE_EDITS,
};
use crate::config::workspace::Workspace;
use crate::parser::registry;
use crate::rewrite::overlap::detect_overlaps;
use crate::safety::paths;
use crate::safety::violations;
use crate::shared::types_v4::{
    RewriteOperation, RewriteValidationResult, SafetyViolation, TextEdit,
};
use crate::text::position_encoding;

/// Limits for rewrite validation.
#[derive(Debug, Clone)]
pub struct RewriteLimits {
    pub max_changed_files: u32,
    pub max_edits: u32,
    pub max_new_text_per_edit: u64,
    pub max_parse_after_files: u32,
}

impl Default for RewriteLimits {
    fn default() -> Self {
        Self {
            max_changed_files: MAX_REWRITE_CHANGED_FILES,
            max_edits: MAX_REWRITE_EDITS,
            max_new_text_per_edit: MAX_NEW_TEXT_BYTES_PER_EDIT,
            max_parse_after_files: MAX_PARSE_AFTER_REWRITE_FILES,
        }
    }
}

/// Validate a list of `RewriteOperation`s without producing edits or diff.
/// Returns a `RewriteValidationResult` with any violations found.
pub fn validate_rewrite_operations(
    workspace: &Workspace,
    operations: &[RewriteOperation],
    limits: RewriteLimits,
) -> RewriteValidationResult {
    let mut violations: Vec<SafetyViolation> = Vec::new();
    let mut changed_files: Vec<String> = Vec::new();

    // Guard: empty operations
    if operations.is_empty() {
        return RewriteValidationResult {
            safe: false,
            changed_files: vec![],
            edit_count: 0,
            violations: vec![violations::internal_error("no operations provided")],
        };
    }

    // 1. Count files and edits
    let edit_count = operations.len() as u32;
    let mut seen_files = std::collections::HashSet::new();
    for op in operations {
        let path = op_file_path(op);
        seen_files.insert(path.to_string());
    }

    if seen_files.len() as u32 > limits.max_changed_files {
        violations
            .push(violations::too_many_files(seen_files.len() as u32, limits.max_changed_files));
    }

    if edit_count > limits.max_edits {
        violations.push(violations::too_many_edits(edit_count, limits.max_edits));
    }

    // 2. Validate each operation
    let mut sources: HashMap<String, String> = HashMap::new();
    let mut language_supported: HashMap<String, bool> = HashMap::new();

    for op in operations {
        let path = op_file_path(op);

        // Path validation
        let resolved = match paths::resolve_file(workspace, path) {
            Ok(r) => r,
            Err(_) => {
                violations.push(violations::outside_workspace(path));
                continue;
            }
        };

        // Read file (cache for reuse)
        if !sources.contains_key(path) {
            let content = match std::fs::read_to_string(&resolved.absolute) {
                Ok(c) => c,
                Err(_) => {
                    violations.push(violations::file_not_found(path));
                    continue;
                }
            };
            sources.insert(path.to_string(), content);
        }

        // Language support
        let supported = language_supported.entry(path.to_string()).or_insert_with(|| {
            let ext = resolved
                .absolute
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| format!(".{}", e))
                .unwrap_or_default();
            ext == ".ts"
                || ext == ".tsx"
                || ext == ".js"
                || ext == ".jsx"
                || ext == ".py"
                || ext == ".go"
                || ext == ".rs"
        });
        if !*supported {
            violations.push(violations::unsupported_language(path, "unknown"));
            continue;
        }

        // Range validation
        let Some(source) = sources.get(path) else {
            violations.push(violations::internal_error("source not available"));
            continue;
        };
        let range = op_range(op);
        let (byte_start, byte_end) =
            match position_encoding::validate_range_in_bounds(source, *range) {
                Ok(b) => b,
                Err(_) => {
                    violations.push(violations::invalid_range(path, "range out of bounds"));
                    continue;
                }
            };

        // New text size check (for operations with new_text)
        if let Some(new_text) = op_new_text(op) {
            if new_text.len() as u64 > limits.max_new_text_per_edit {
                violations.push(violations::new_text_too_large(
                    path,
                    new_text.len() as u64,
                    limits.max_new_text_per_edit,
                ));
            }
        }

        // Node operations: check node alignment and kind
        if let Some(expected_kind) = op_expected_kind(op) {
            let def = registry::for_extension(
                resolved
                    .absolute
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| format!(".{}", e))
                    .unwrap_or_default()
                    .as_str(),
            );
            if let Some(def) = def {
                let mut parser = Parser::new();
                if parser.set_language(&(def.tree_sitter_language)()).is_ok() {
                    if let Some(tree) = parser.parse(source, None) {
                        let root = tree.root_node();
                        // Find the node at the range
                        if let Some(node) = find_node_at_byte_range(&root, byte_start, byte_end) {
                            if node.kind() != expected_kind {
                                violations.push(violations::node_kind_mismatch(
                                    path,
                                    expected_kind,
                                    node.kind(),
                                ));
                            }
                        }
                    }
                }
            }
        }

        changed_files.push(path.to_string());
    }

    // 3. Check for overlapping edits (only if no critical violations so far)
    if violations.is_empty() {
        // Build temporary TextEdits for overlap detection
        let edits: Vec<TextEdit> = operations.iter().filter_map(build_text_edit_from_op).collect();

        let source_refs: HashMap<String, &str> =
            sources.iter().map(|(k, v)| (k.clone(), v.as_str())).collect();

        if let Err(e) = detect_overlaps(&source_refs, &edits) {
            violations.push(violations::overlapping_edits(&format!("{}", e)));
        }
    }

    let unique_files: Vec<String> = {
        let mut v: Vec<String> = seen_files.into_iter().collect();
        v.sort();
        v
    };

    RewriteValidationResult {
        safe: violations.is_empty(),
        changed_files: unique_files,
        edit_count,
        violations,
    }
}

// ── Operation helpers ──

fn op_file_path(op: &RewriteOperation) -> &str {
    match op {
        RewriteOperation::ReplaceRange { file_path, .. }
        | RewriteOperation::ReplaceNode { file_path, .. }
        | RewriteOperation::InsertBeforeNode { file_path, .. }
        | RewriteOperation::InsertAfterNode { file_path, .. }
        | RewriteOperation::DeleteNode { file_path, .. } => file_path,
    }
}

fn op_range(op: &RewriteOperation) -> &crate::shared::position::Range {
    match op {
        RewriteOperation::ReplaceRange { range, .. }
        | RewriteOperation::ReplaceNode { range, .. }
        | RewriteOperation::InsertBeforeNode { range, .. }
        | RewriteOperation::InsertAfterNode { range, .. }
        | RewriteOperation::DeleteNode { range, .. } => range,
    }
}

fn op_new_text(op: &RewriteOperation) -> Option<&str> {
    match op {
        RewriteOperation::ReplaceRange { new_text, .. }
        | RewriteOperation::ReplaceNode { new_text, .. }
        | RewriteOperation::InsertBeforeNode { new_text, .. }
        | RewriteOperation::InsertAfterNode { new_text, .. } => Some(new_text),
        RewriteOperation::DeleteNode { .. } => None,
    }
}

fn op_expected_kind(op: &RewriteOperation) -> Option<&str> {
    match op {
        RewriteOperation::ReplaceRange { .. } => None,
        RewriteOperation::ReplaceNode { expected_node_kind, .. }
        | RewriteOperation::InsertBeforeNode { expected_node_kind, .. }
        | RewriteOperation::InsertAfterNode { expected_node_kind, .. }
        | RewriteOperation::DeleteNode { expected_node_kind, .. } => expected_node_kind.as_deref(),
    }
}

fn build_text_edit_from_op(op: &RewriteOperation) -> Option<TextEdit> {
    match op {
        RewriteOperation::ReplaceRange { file_path, range, new_text } => Some(TextEdit {
            file_path: file_path.clone(),
            range: *range,
            new_text: new_text.clone(),
        }),
        RewriteOperation::ReplaceNode { file_path, range, new_text, .. } => Some(TextEdit {
            file_path: file_path.clone(),
            range: *range,
            new_text: new_text.clone(),
        }),
        RewriteOperation::InsertBeforeNode { file_path, range, new_text, .. } => {
            // Insert before: zero-width range at start of node
            Some(TextEdit {
                file_path: file_path.clone(),
                range: crate::shared::position::Range { start: range.start, end: range.start },
                new_text: new_text.clone(),
            })
        }
        RewriteOperation::InsertAfterNode { file_path, range, new_text, .. } => {
            // Insert after: zero-width range at end of node
            Some(TextEdit {
                file_path: file_path.clone(),
                range: crate::shared::position::Range { start: range.end, end: range.end },
                new_text: new_text.clone(),
            })
        }
        RewriteOperation::DeleteNode { file_path, range, .. } => {
            Some(TextEdit { file_path: file_path.clone(), range: *range, new_text: String::new() })
        }
    }
}

/// Find the deepest node at a byte range within the tree.
fn find_node_at_byte_range<'a>(
    root: &tree_sitter::Node<'a>,
    start: usize,
    end: usize,
) -> Option<tree_sitter::Node<'a>> {
    let mut best: Option<tree_sitter::Node> = None;

    fn recurse<'a>(
        node: &tree_sitter::Node<'a>,
        start: usize,
        end: usize,
        best: &mut Option<tree_sitter::Node<'a>>,
    ) {
        if node.start_byte() <= start && node.end_byte() >= end {
            if best
                .is_none_or(|b| node.end_byte() - node.start_byte() < b.end_byte() - b.start_byte())
            {
                *best = Some(*node);
            }
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    recurse(&child, start, end, best);
                }
            }
        }
    }

    recurse(root, start, end, &mut best);
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_empty_operations() -> Result<(), Box<dyn std::error::Error>> {
        let w = Workspace::from_env()?;
        let result = validate_rewrite_operations(&w, &[], RewriteLimits::default());
        assert!(!result.safe);
        assert_eq!(result.edit_count, 0);
        Ok(())
    }
}
