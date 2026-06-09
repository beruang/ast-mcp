//! `ast_remove_unused_import_preview` — syntax-level unused import removal with Tree-sitter.

use std::collections::HashSet;

use serde_json::Value;

use crate::config::workspace::Workspace;
use crate::rewrite::preview::preview_edits;
use crate::rewrite::validate::RewriteLimits;
use crate::rewrite_tools::import_merge::{self, ParsedImport};
use crate::safety::paths;
use crate::shared::position::Range;
use crate::shared::types_v4::{
    AstRemoveUnusedImportPreviewInput, PreviewOptions, RewriteOperation, RewritePreview,
    SafetyViolation,
};

pub fn handle(workspace: &Workspace, arguments: Value) -> Value {
    let input: AstRemoveUnusedImportPreviewInput = match serde_json::from_value(arguments) {
        Ok(v) => v,
        Err(e) => {
            return serde_json::json!({
                "safe": false, "changed_files": [], "edit_count": 0,
                "diff": null, "edits": [], "parse_after_rewrite": null,
                "violations": [{"violation_type": "invalid_input", "message": e.to_string()}]
            });
        }
    };

    let result = remove_unused(workspace, &input.file_path, input.import_names.as_deref(), &input);
    serde_json::to_value(result).unwrap_or_default()
}

fn remove_unused(
    workspace: &Workspace,
    file_path: &str,
    import_names: Option<&[String]>,
    input: &AstRemoveUnusedImportPreviewInput,
) -> RewritePreview {
    let resolved = match paths::resolve_file(workspace, file_path) {
        Ok(r) => r,
        Err(e) => return make_error(file_path, &e.to_string()),
    };
    let source = match std::fs::read_to_string(&resolved.absolute) {
        Ok(s) => s,
        Err(e) => return make_error(file_path, &e.to_string()),
    };

    let ext = resolved.absolute.extension().and_then(|e| e.to_str()).unwrap_or("");

    let imports = match ext {
        "ts" | "tsx" | "js" | "jsx" => import_merge::parse_ts_imports(&source),
        "py" => import_merge::parse_python_imports(&source),
        "go" => import_merge::parse_go_imports(&source),
        _ => {
            return RewritePreview {
                safe: true,
                changed_files: vec![file_path.to_string()],
                edit_count: 0,
                diff: None,
                edits: vec![],
                parse_after_rewrite: None,
                violations: vec![],
            };
        }
    };

    if imports.is_empty() {
        return RewritePreview {
            safe: true,
            changed_files: vec![file_path.to_string()],
            edit_count: 0,
            diff: None,
            edits: vec![],
            parse_after_rewrite: None,
            violations: vec![],
        };
    }

    // Build a set of all identifiers used in the source (non-import parts)
    let used_names = build_identifier_usage_set(&source, &imports);

    let mut operations: Vec<RewriteOperation> = Vec::new();

    for import in &imports {
        // Never remove side-effect imports
        if import.is_side_effect {
            continue;
        }

        // If import_names filter is provided, only consider those
        if let Some(names) = import_names {
            let has_target = names.iter().any(|n| {
                import.named_imports.contains(n)
                    || import.default_import.as_deref() == Some(n.as_str())
                    || import.namespace_import.as_deref() == Some(n.as_str())
            });
            if !has_target {
                continue;
            }
        }

        // Check if default import is used
        let default_unused =
            import.default_import.as_ref().is_some_and(|d| !used_names.contains(d.as_str()));

        // Check which named imports are unused
        let unused_named: Vec<&String> =
            import.named_imports.iter().filter(|n| !used_names.contains(n.as_str())).collect();

        // Check namespace import usage
        let namespace_unused =
            import.namespace_import.as_ref().is_some_and(|ns| !used_names.contains(ns.as_str()));

        // If everything is unused, remove the entire import line
        let all_unused = default_unused
            && unused_named.len() == import.named_imports.len()
            && (import.namespace_import.is_none() || namespace_unused);

        if all_unused && !import.named_imports.is_empty()
            || default_unused && import.named_imports.is_empty()
        {
            // Remove the entire import line
            operations.push(RewriteOperation::ReplaceRange {
                file_path: file_path.to_string(),
                range: Range {
                    start: crate::shared::position::Position {
                        line: import.start_line,
                        character: 0,
                    },
                    end: crate::shared::position::Position {
                        line: import.end_line + 1,
                        character: 0,
                    },
                },
                new_text: String::new(),
            });
        } else if !unused_named.is_empty() || default_unused || namespace_unused {
            // Partial removal — rewrite the import line with only used names
            let kept_named: Vec<String> = import
                .named_imports
                .iter()
                .filter(|n| used_names.contains(n.as_str()))
                .cloned()
                .collect();
            let kept_default = if default_unused { None } else { import.default_import.clone() };
            let kept_namespace =
                if namespace_unused { None } else { import.namespace_import.clone() };

            let new_line = if ext == "py" {
                if kept_named.is_empty() && import.named_imports.len() > kept_named.len() {
                    // All named imports removed — replace whole line
                    String::new()
                } else {
                    format!("from {} import {}", import.source, kept_named.join(", "))
                }
            } else {
                // TS/JS
                let mut parts = Vec::new();
                if let Some(ref d) = kept_default {
                    parts.push(d.clone());
                }
                if let Some(ref ns) = kept_namespace {
                    parts.push(format!("* as {}", ns));
                }
                if !kept_named.is_empty() {
                    let type_prefix = if import.is_type_only { "type " } else { "" };
                    parts.push(format!("{}{{ {} }}", type_prefix, kept_named.join(", ")));
                }
                if parts.is_empty() {
                    String::new()
                } else {
                    format!("import {} from \"{}\";", parts.join(", "), import.source)
                }
            };

            operations.push(RewriteOperation::ReplaceRange {
                file_path: file_path.to_string(),
                range: Range {
                    start: crate::shared::position::Position {
                        line: import.start_line,
                        character: 0,
                    },
                    end: crate::shared::position::Position {
                        line: import.end_line + 1,
                        character: 0,
                    },
                },
                new_text: if new_line.is_empty() {
                    String::new()
                } else {
                    format!("{}\n", new_line)
                },
            });
        }
    }

    if operations.is_empty() {
        return RewritePreview {
            safe: true,
            changed_files: vec![file_path.to_string()],
            edit_count: 0,
            diff: None,
            edits: vec![],
            parse_after_rewrite: None,
            violations: vec![],
        };
    }

    let options = PreviewOptions {
        include_diff: input.include_diff.unwrap_or(true),
        parse_check: input.parse_check.unwrap_or(true),
        max_diff_bytes: 500_000,
    };

    preview_edits(workspace, &operations, options, RewriteLimits::default())
}

/// Walk the entire source tree collecting all identifier names that are referenced
/// (not in import declarations).
fn build_identifier_usage_set(source: &str, imports: &[ParsedImport]) -> HashSet<String> {
    let mut used = HashSet::new();

    // Collect import line ranges to exclude from usage scan
    let import_ranges: Vec<(usize, usize)> = imports
        .iter()
        .map(|i| {
            let start_byte = find_line_start_byte(source, i.start_line as usize);
            let end_byte = find_line_start_byte(source, i.end_line as usize + 1);
            (start_byte, end_byte)
        })
        .collect();

    let is_in_import =
        |byte: usize| -> bool { import_ranges.iter().any(|(s, e)| byte >= *s && byte < *e) };

    // Walk the tree and collect identifiers outside import ranges
    let mut parser = tree_sitter::Parser::new();
    let lang = tree_sitter_typescript::language_typescript();
    // Try TS first, then Python, then Go
    if parser.set_language(&lang).is_ok() {
        if let Some(tree) = parser.parse(source, None) {
            collect_identifiers(&tree.root_node(), source, &mut used, &is_in_import);
        }
    }
    // If no identifiers found with TS parser, also try Python
    if used.is_empty() {
        let mut parser = tree_sitter::Parser::new();
        if parser.set_language(&tree_sitter_python::language()).is_ok() {
            if let Some(tree) = parser.parse(source, None) {
                collect_identifiers(&tree.root_node(), source, &mut used, &is_in_import);
            }
        }
    }

    used
}

fn collect_identifiers(
    node: &tree_sitter::Node,
    source: &str,
    used: &mut HashSet<String>,
    is_in_import: &dyn Fn(usize) -> bool,
) {
    // Collect leaf identifiers that are not inside import statements
    if node.child_count() == 0 && node.kind() == "identifier" && !is_in_import(node.start_byte()) {
        if let Ok(text) = node.utf8_text(source.as_bytes()) {
            used.insert(text.to_string());
        }
    }

    // Also collect property identifiers for member access (e.g., `Foo.bar`)
    if node.child_count() == 0
        && node.kind() == "property_identifier"
        && !is_in_import(node.start_byte())
    {
        if let Ok(text) = node.utf8_text(source.as_bytes()) {
            used.insert(text.to_string());
        }
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_identifiers(&child, source, used, is_in_import);
        }
    }
}

fn find_line_start_byte(source: &str, line: usize) -> usize {
    source.lines().take(line).map(|l| l.len() + 1).sum()
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
            violation_type: "file_error".into(),
            message: format!("{}: {}", path, msg),
            file_path: Some(path.to_string()),
            details: None,
        }],
    }
}
