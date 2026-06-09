//! High-level rewrite preview pipeline — assembles validation, edit application, diff, and parse-after.

use std::collections::HashMap;

use crate::config::workspace::Workspace;
use crate::rewrite::apply_edits::apply_edits;
use crate::rewrite::diff::generate_diff;
use crate::rewrite::parse_after::parse_after_edits;
use crate::rewrite::validate::{validate_rewrite_operations, RewriteLimits};
use crate::safety::paths;
use crate::safety::violations;
use crate::shared::types_v4::{
    PreviewOptions, RewriteOperation, RewritePreview, SafetyViolation, TextEdit,
};

/// Build a list of `TextEdit` values from `RewriteOperation`s.
/// This resolves node ranges and generates concrete text edits.
pub fn build_text_edits(
    workspace: &Workspace,
    operations: &[RewriteOperation],
) -> Result<Vec<TextEdit>, Vec<SafetyViolation>> {
    let mut edits = Vec::new();
    let mut violations = Vec::new();

    for op in operations {
        let path = op_file_path(op);
        // Validate path exists (already validated, but defense in depth)
        if paths::resolve_file(workspace, path).is_err() {
            violations.push(violations::outside_workspace(path));
            continue;
        }

        let edit = match op {
            RewriteOperation::ReplaceRange { file_path, range, new_text } => {
                TextEdit { file_path: file_path.clone(), range: *range, new_text: new_text.clone() }
            }
            RewriteOperation::ReplaceNode { file_path, range, new_text, .. } => {
                TextEdit { file_path: file_path.clone(), range: *range, new_text: new_text.clone() }
            }
            RewriteOperation::InsertBeforeNode { file_path, range, new_text, .. } => {
                let start = range.start;
                TextEdit {
                    file_path: file_path.clone(),
                    range: crate::shared::position::Range { start, end: start },
                    new_text: new_text.clone(),
                }
            }
            RewriteOperation::InsertAfterNode { file_path, range, new_text, .. } => {
                let end = range.end;
                TextEdit {
                    file_path: file_path.clone(),
                    range: crate::shared::position::Range { start: end, end },
                    new_text: new_text.clone(),
                }
            }
            RewriteOperation::DeleteNode { file_path, range, .. } => {
                TextEdit { file_path: file_path.clone(), range: *range, new_text: String::new() }
            }
        };
        edits.push(edit);
    }

    if !violations.is_empty() {
        return Err(violations);
    }

    Ok(edits)
}

/// Main preview pipeline: validate → build edits → apply → diff → parse-after.
pub fn preview_edits(
    workspace: &Workspace,
    operations: &[RewriteOperation],
    options: PreviewOptions,
    limits: RewriteLimits,
) -> RewritePreview {
    // 1. Validate
    let validation = validate_rewrite_operations(workspace, operations, limits);
    if !validation.safe && !validation.violations.is_empty() {
        return RewritePreview {
            safe: false,
            changed_files: validation.changed_files,
            edit_count: validation.edit_count,
            diff: None,
            edits: vec![],
            parse_after_rewrite: None,
            violations: validation.violations,
        };
    }

    // 2. Build text edits
    let edits = match build_text_edits(workspace, operations) {
        Ok(e) => e,
        Err(violations) => {
            return RewritePreview {
                safe: false,
                changed_files: validation.changed_files,
                edit_count: validation.edit_count,
                diff: None,
                edits: vec![],
                parse_after_rewrite: None,
                violations,
            };
        }
    };

    // 3. Read sources and apply edits
    let mut sources: HashMap<String, (String, String)> = HashMap::new(); // path -> (original, modified)
    let mut file_violations = Vec::new();

    let by_file: HashMap<&str, Vec<&TextEdit>> = {
        let mut m = HashMap::new();
        for edit in &edits {
            m.entry(edit.file_path.as_str()).or_insert_with(Vec::new).push(edit);
        }
        m
    };

    for (file_path, file_edits) in &by_file {
        let resolved = match paths::resolve_file(workspace, file_path) {
            Ok(r) => r,
            Err(_) => {
                file_violations.push(violations::file_not_found(file_path));
                continue;
            }
        };
        let original = match std::fs::read_to_string(&resolved.absolute) {
            Ok(s) => s,
            Err(_) => {
                file_violations.push(violations::file_not_found(file_path));
                continue;
            }
        };
        let edits_owned: Vec<TextEdit> = file_edits.iter().map(|&e| e.clone()).collect();
        let modified = match apply_edits(&original, &edits_owned) {
            Ok(m) => m,
            Err(_) => {
                file_violations.push(violations::internal_error("failed to apply edits"));
                continue;
            }
        };
        sources.insert(file_path.to_string(), (original, modified));
    }

    if !file_violations.is_empty() {
        return RewritePreview {
            safe: false,
            changed_files: validation.changed_files,
            edit_count: validation.edit_count,
            diff: None,
            edits,
            parse_after_rewrite: None,
            violations: file_violations,
        };
    }

    // 4. Generate diff
    let diff = if options.include_diff {
        let mut combined = String::new();
        for (path, (orig, modified)) in &sources {
            if !combined.is_empty() {
                combined.push('\n');
            }
            match generate_diff(path, orig, modified, options.max_diff_bytes) {
                Ok(d) => combined.push_str(&d),
                Err(_) => {
                    return RewritePreview {
                        safe: false,
                        changed_files: validation.changed_files,
                        edit_count: validation.edit_count,
                        diff: None,
                        edits,
                        parse_after_rewrite: None,
                        violations: vec![violations::diff_too_large(0, options.max_diff_bytes)],
                    };
                }
            }
        }
        Some(combined)
    } else {
        None
    };

    // 5. Parse after rewrite
    let parse_result =
        if options.parse_check { Some(parse_after_edits(workspace, &edits)) } else { None };

    let safe = parse_result.as_ref().is_none_or(|p| p.ok);

    RewritePreview {
        safe,
        changed_files: sources.keys().cloned().collect(),
        edit_count: edits.len() as u32,
        diff,
        edits,
        parse_after_rewrite: parse_result,
        violations: vec![],
    }
}

fn op_file_path(op: &RewriteOperation) -> &str {
    match op {
        RewriteOperation::ReplaceRange { file_path, .. }
        | RewriteOperation::ReplaceNode { file_path, .. }
        | RewriteOperation::InsertBeforeNode { file_path, .. }
        | RewriteOperation::InsertAfterNode { file_path, .. }
        | RewriteOperation::DeleteNode { file_path, .. } => file_path,
    }
}
