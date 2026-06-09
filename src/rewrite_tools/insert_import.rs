//! `ast_insert_import_preview` — insert or merge import statements with Tree-sitter parsing.

use serde_json::Value;

use crate::config::workspace::Workspace;
use crate::rewrite::preview::preview_edits;
use crate::rewrite::validate::RewriteLimits;
use crate::rewrite_tools::import_merge::{self, ParsedImport};
use crate::safety::paths;
use crate::shared::position::Range;
use crate::shared::types_v4::{
    AstInsertImportPreviewInput, ImportRequest, PreviewOptions, RewriteOperation, RewritePreview,
    SafetyViolation,
};

pub fn handle(workspace: &Workspace, arguments: Value) -> Value {
    let input: AstInsertImportPreviewInput = match serde_json::from_value(arguments) {
        Ok(v) => v,
        Err(e) => return json_error(&e.to_string()),
    };
    let result = insert_import(workspace, &input.file_path, &input.import, &input);
    serde_json::to_value(result).unwrap_or_default()
}

fn insert_import(
    workspace: &Workspace,
    file_path: &str,
    import: &ImportRequest,
    input: &AstInsertImportPreviewInput,
) -> RewritePreview {
    let resolved = match paths::resolve_file(workspace, file_path) {
        Ok(r) => r,
        Err(_) => return make_error(file_path, "outside_workspace", "path outside workspace"),
    };
    let source = match std::fs::read_to_string(&resolved.absolute) {
        Ok(s) => s,
        Err(_) => return make_error(file_path, "file_not_found", "file not found"),
    };

    let ext = resolved.absolute.extension().and_then(|e| e.to_str()).unwrap_or("");

    let (existing_imports, is_typescript, is_python, is_go) = match ext {
        "ts" | "tsx" | "js" | "jsx" => {
            (import_merge::parse_ts_imports(&source), true, false, false)
        }
        "py" => (import_merge::parse_python_imports(&source), false, true, false),
        "go" => (import_merge::parse_go_imports(&source), false, false, true),
        _ => {
            return make_error(
                file_path,
                "unsupported_language",
                &format!("unsupported: .{}", ext),
            );
        }
    };

    // Check if the import source already exists
    let existing = existing_imports.iter().find(|ei| ei.source == import.source);

    let operation = if let Some(ei) = existing {
        // Merge with existing import
        merge_import_operation(file_path, ei, import, is_typescript)
    } else {
        // Insert new import
        insert_new_import_operation(
            file_path,
            &source,
            &existing_imports,
            import,
            is_typescript,
            is_python,
            is_go,
        )
    };

    let options = PreviewOptions {
        include_diff: input.include_diff.unwrap_or(true),
        parse_check: input.parse_check.unwrap_or(true),
        max_diff_bytes: 500_000,
    };

    preview_edits(workspace, &[operation], options, RewriteLimits::default())
}

fn merge_import_operation(
    file_path: &str,
    existing: &ParsedImport,
    import: &ImportRequest,
    is_typescript: bool,
) -> RewriteOperation {
    let merged_names =
        import_merge::merge_named_imports(&existing.named_imports, &import.named_imports);

    // Check type-only compatibility
    if import.is_type_only.unwrap_or(false) != existing.is_type_only
        && !import.named_imports.is_empty()
        && !existing.named_imports.is_empty()
    {
        // Conflict: can't mix type-only and value imports from same source in TS
        // Return a range-replace with a note (we still merge structurally)
    }

    let new_text = if is_typescript {
        build_ts_import_line(
            import.source.as_str(),
            existing.default_import.as_deref().or(import.default_import.as_deref()),
            &merged_names,
            existing.namespace_import.as_deref().or(import.namespace_import.as_deref()),
            existing.is_type_only || import.is_type_only.unwrap_or(false),
        )
    } else {
        build_python_import_line(&import.source, &merged_names)
    };

    RewriteOperation::ReplaceRange {
        file_path: file_path.to_string(),
        range: Range {
            start: crate::shared::position::Position { line: existing.start_line, character: 0 },
            end: crate::shared::position::Position {
                line: existing.end_line + 1, // include trailing newline
                character: 0,
            },
        },
        new_text: format!("{}\n", new_text),
    }
}

fn insert_new_import_operation(
    file_path: &str,
    _source: &str,
    existing_imports: &[ParsedImport],
    import: &ImportRequest,
    _is_typescript: bool,
    is_python: bool,
    is_go: bool,
) -> RewriteOperation {
    let import_line = if is_go {
        build_go_import_line(&import.source)
    } else if is_python {
        build_python_import_line(&import.source, &import.named_imports)
    } else {
        build_ts_import_line(
            &import.source,
            import.default_import.as_deref(),
            &import.named_imports,
            import.namespace_import.as_deref(),
            import.is_type_only.unwrap_or(false),
        )
    };

    // Find insertion point: after the last import
    let insert_line = existing_imports.iter().map(|ei| ei.end_line + 1).max().unwrap_or(0);

    RewriteOperation::InsertBeforeNode {
        file_path: file_path.to_string(),
        range: Range {
            start: crate::shared::position::Position { line: insert_line, character: 0 },
            end: crate::shared::position::Position { line: insert_line, character: 0 },
        },
        expected_node_kind: None,
        new_text: format!("{}\n", import_line),
    }
}

fn build_ts_import_line(
    source: &str,
    default_import: Option<&str>,
    named_imports: &[String],
    namespace_import: Option<&str>,
    is_type_only: bool,
) -> String {
    let mut parts = Vec::new();

    if let Some(d) = default_import {
        parts.push(d.to_string());
    }
    if let Some(ns) = namespace_import {
        parts.push(format!("* as {}", ns));
    }
    if !named_imports.is_empty() {
        let type_prefix = if is_type_only { "type " } else { "" };
        parts.push(format!("{}{{ {} }}", type_prefix, named_imports.join(", ")));
    }

    let type_prefix = if is_type_only && parts.is_empty() { "type " } else { "" };

    if parts.is_empty() {
        format!("import \"{}\";", source)
    } else {
        format!("{}import {} from \"{}\";", type_prefix, parts.join(", "), source)
    }
}

fn build_python_import_line(source: &str, names: &[String]) -> String {
    if names.is_empty() {
        format!("import {}", source)
    } else {
        format!("from {} import {}", source, names.join(", "))
    }
}

fn build_go_import_line(source: &str) -> String {
    format!("import \"{}\"", source)
}

// ── Helpers ──

fn json_error(msg: &str) -> Value {
    serde_json::json!({
        "safe": false, "changed_files": [], "edit_count": 0,
        "diff": null, "edits": [], "parse_after_rewrite": null,
        "violations": [{"violation_type": "invalid_input", "message": msg}]
    })
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
