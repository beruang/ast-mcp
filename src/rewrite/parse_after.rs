//! Parse-after-rewrite: apply edits in memory, re-parse, report syntax errors.

use std::collections::HashMap;

use tree_sitter::Parser;

use crate::config::workspace::Workspace;
use crate::parser::registry;
use crate::rewrite::apply_edits::apply_edits;
use crate::shared::position::Range;
use crate::shared::types_v4::{ParseAfterRewriteSummary, SyntaxErrorSummary, TextEdit};

/// Apply edits in memory, re-parse each changed file, and return a summary
/// of any syntax errors found.
pub fn parse_after_edits(workspace: &Workspace, edits: &[TextEdit]) -> ParseAfterRewriteSummary {
    let mut files_with_errors: Vec<String> = Vec::new();
    let mut syntax_errors: Vec<SyntaxErrorSummary> = Vec::new();
    let mut changed_files_checked: u32 = 0;

    // Group edits by file
    let mut by_file: HashMap<&str, Vec<&TextEdit>> = HashMap::new();
    for edit in edits {
        by_file.entry(&edit.file_path).or_default().push(edit);
    }

    for (file_path, file_edits) in &by_file {
        changed_files_checked += 1;

        // Read original source
        let source = match read_file(workspace, file_path) {
            Ok(s) => s,
            Err(_) => {
                syntax_errors.push(SyntaxErrorSummary {
                    file_path: file_path.to_string(),
                    range: Range {
                        start: crate::shared::position::Position { line: 0, character: 0 },
                        end: crate::shared::position::Position { line: 0, character: 0 },
                    },
                    node_kind: "ERROR".into(),
                    message: "could not read file".into(),
                });
                files_with_errors.push(file_path.to_string());
                continue;
            }
        };

        // Check for pre-existing syntax errors
        let original_has_errors = check_parse_errors(&source, file_path);

        // Apply edits
        let edits_owned: Vec<TextEdit> = file_edits.iter().map(|&e| e.clone()).collect();
        let modified = match apply_edits(&source, &edits_owned) {
            Ok(m) => m,
            Err(e) => {
                syntax_errors.push(SyntaxErrorSummary {
                    file_path: file_path.to_string(),
                    range: Range {
                        start: crate::shared::position::Position { line: 0, character: 0 },
                        end: crate::shared::position::Position { line: 0, character: 0 },
                    },
                    node_kind: "ERROR".into(),
                    message: format!("edit application failed: {}", e),
                });
                files_with_errors.push(file_path.to_string());
                continue;
            }
        };

        // Parse modified text
        let parse_errors = collect_parse_errors(&modified, file_path);
        if !parse_errors.is_empty() || original_has_errors {
            files_with_errors.push(file_path.to_string());
            if original_has_errors && parse_errors.is_empty() {
                syntax_errors.push(SyntaxErrorSummary {
                    file_path: file_path.to_string(),
                    range: Range {
                        start: crate::shared::position::Position { line: 0, character: 0 },
                        end: crate::shared::position::Position { line: 0, character: 0 },
                    },
                    node_kind: "ERROR".into(),
                    message: "file had pre-existing syntax errors before rewrite".into(),
                });
            } else {
                syntax_errors.extend(parse_errors);
            }
        }
    }

    ParseAfterRewriteSummary {
        ok: files_with_errors.is_empty(),
        changed_files_checked,
        files_with_syntax_errors: files_with_errors,
        syntax_errors,
    }
}

/// Check if `source` has any ERROR nodes.
fn check_parse_errors(source: &str, file_path: &str) -> bool {
    let ext = std::path::Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e))
        .unwrap_or_default();

    if let Some(def) = registry::for_extension(&ext) {
        let mut parser = Parser::new();
        if parser.set_language(&(def.tree_sitter_language)()).is_ok() {
            if let Some(tree) = parser.parse(source, None) {
                return tree.root_node().has_error();
            }
        }
    }
    false
}

/// Walk the tree and collect ERROR node summaries.
fn collect_parse_errors(source: &str, file_path: &str) -> Vec<SyntaxErrorSummary> {
    let ext = std::path::Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e))
        .unwrap_or_default();

    let mut errors = Vec::new();

    if let Some(def) = registry::for_extension(&ext) {
        let mut parser = Parser::new();
        if parser.set_language(&(def.tree_sitter_language)()).is_ok() {
            if let Some(tree) = parser.parse(source, None) {
                collect_error_nodes(&tree.root_node(), source, file_path, &mut errors);
            }
        }
    }

    errors
}

fn collect_error_nodes(
    node: &tree_sitter::Node,
    _source: &str,
    file_path: &str,
    errors: &mut Vec<SyntaxErrorSummary>,
) {
    if node.kind() == "ERROR" || node.is_missing() {
        let start = node.start_position();
        let end = node.end_position();
        errors.push(SyntaxErrorSummary {
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
            node_kind: node.kind().to_string(),
            message: "Tree-sitter reported syntax error after rewrite.".into(),
        });
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_error_nodes(&child, _source, file_path, errors);
        }
    }
}

fn read_file(workspace: &Workspace, file_path: &str) -> Result<String, ()> {
    let resolved = crate::safety::paths::resolve_file(workspace, file_path).map_err(|_| ())?;
    std::fs::read_to_string(&resolved.absolute).map_err(|_| ())
}
