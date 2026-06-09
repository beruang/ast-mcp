use serde::Serialize;
use tree_sitter::Tree;

use crate::parser::positions::ts_point_to_position;
use crate::shared::language::LanguageId;
use crate::shared::position::Range;

use crate::safety::limits;

/// Represents a single import statement found in source code.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstImport {
    pub kind: ImportKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_path: Option<String>,
    pub names: Vec<Alias>,
    pub range: Range,
    pub is_type_only: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_text: Option<String>,
}

/// Classification of an import statement.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportKind {
    Import,
    FromImport,
    Require,
    DynamicImport,
    Unknown,
}

/// A name mapping within an import (imported name -> local name).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Alias {
    pub imported: String,
    pub local: String,
}

/// Find all import statements in a parsed file.
pub fn find_imports(tree: &Tree, source: &str, lang: LanguageId) -> Vec<AstImport> {
    match lang {
        LanguageId::TypeScript
        | LanguageId::TypeScriptReact
        | LanguageId::JavaScript
        | LanguageId::JavaScriptReact => find_imports_ts_js(tree, source),
        LanguageId::Python => find_imports_python(tree, source),
        LanguageId::Go | LanguageId::Rust => vec![],
    }
}

// ---------------------------------------------------------------------------
// TS/JS
// ---------------------------------------------------------------------------

fn find_imports_ts_js(tree: &Tree, source: &str) -> Vec<AstImport> {
    let mut results = Vec::new();
    collect_imports_ts_js(&tree.root_node(), source, &mut results);
    results.truncate(limits::MAX_RESULTS);
    results
}

fn collect_imports_ts_js(node: &tree_sitter::Node, source: &str, results: &mut Vec<AstImport>) {
    if results.len() >= limits::MAX_RESULTS {
        return;
    }

    match node.kind() {
        "import_statement" => {
            if let Some(import) = extract_ts_js_import_statement(node, source) {
                results.push(import);
            }
            return; // No need to recurse into import_statement children
        }
        "call_expression" => {
            if let Some(import) = extract_ts_js_call_import(node, source) {
                results.push(import);
            }
        }
        _ => {}
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_imports_ts_js(&child, source, results);
        }
    }
}

/// Extract information from a `import_statement` node.
fn extract_ts_js_import_statement(node: &tree_sitter::Node, source: &str) -> Option<AstImport> {
    let mut module_path = None;
    let mut names = Vec::new();
    let mut is_type_only = false;

    for i in 0..node.child_count() {
        let child = node.child(i)?;
        match child.kind() {
            "type" => is_type_only = true,
            "string" => {
                // Side-effect import: `import "mod"`
                module_path = strip_quotes_opt(&child, source);
            }
            "import_clause" => {
                extract_ts_js_import_clause(&child, source, &mut names);
            }
            "from_clause" => {
                // Extract source string from `from "mod"`
                for j in 0..child.child_count() {
                    if let Some(gc) = child.child(j) {
                        if gc.kind() == "string" {
                            module_path = strip_quotes_opt(&gc, source);
                        }
                    }
                }
                // Also check type inside import_clause (import type { ... })
                // Actually type is at import_statement level, not from_clause
            }
            _ => {}
        }
    }

    let range = make_range(node, source);
    let source_text = trimmed_node_text(node, source, limits::MAX_TEXT_BYTES);

    Some(AstImport {
        kind: ImportKind::Import,
        module_path,
        names,
        range,
        is_type_only,
        source_text: Some(source_text),
    })
}

/// Extract names from an `import_clause` node.
fn extract_ts_js_import_clause(node: &tree_sitter::Node, source: &str, names: &mut Vec<Alias>) {
    for i in 0..node.child_count() {
        let child = match node.child(i) {
            Some(c) => c,
            None => continue,
        };
        match child.kind() {
            "identifier" => {
                // Default import: `import React from "react"`
                if let Ok(text) = child.utf8_text(source.as_bytes()) {
                    names.push(Alias { imported: text.to_string(), local: text.to_string() });
                }
            }
            "named_imports" => {
                extract_named_imports(&child, source, names);
            }
            "namespace_import" => {
                // `import * as name from "mod"`
                extract_namespace_import(&child, source, names);
            }
            _ => {}
        }
    }
}

/// Extract individual specifiers from a `named_imports` node.
fn extract_named_imports(node: &tree_sitter::Node, source: &str, names: &mut Vec<Alias>) {
    for i in 0..node.child_count() {
        let child = match node.child(i) {
            Some(c) => c,
            None => continue,
        };
        if child.kind() == "import_specifier" {
            let mut imported = String::new();
            let mut local = String::new();
            let mut seen_alias = false;

            for j in 0..child.child_count() {
                let spec_child = match child.child(j) {
                    Some(c) => c,
                    None => continue,
                };
                if spec_child.kind() == "identifier" {
                    let text = spec_child.utf8_text(source.as_bytes()).unwrap_or("");
                    if seen_alias {
                        local = text.to_string();
                    } else {
                        imported = text.to_string();
                        local = text.to_string(); // default: local = imported
                    }
                } else if spec_child.kind() == "as" {
                    // The next identifier will be the alias
                    seen_alias = true;
                }
            }

            if !imported.is_empty() {
                names.push(Alias { imported, local });
            }
        }
    }
}

/// Extract name from a `namespace_import` node.
fn extract_namespace_import(node: &tree_sitter::Node, source: &str, names: &mut Vec<Alias>) {
    for i in 0..node.child_count() {
        let child = match node.child(i) {
            Some(c) => c,
            None => continue,
        };
        if child.kind() == "identifier" {
            if let Ok(text) = child.utf8_text(source.as_bytes()) {
                names.push(Alias { imported: "*".to_string(), local: text.to_string() });
                return;
            }
        }
    }
}

/// Extract require() and dynamic import() calls.
fn extract_ts_js_call_import(node: &tree_sitter::Node, source: &str) -> Option<AstImport> {
    let func = node.child_by_field_name("function")?;

    let (kind, is_type_only) = match func.kind() {
        "identifier" => {
            let name = func.utf8_text(source.as_bytes()).unwrap_or("");
            if name == "require" {
                (ImportKind::Require, false)
            } else {
                return None;
            }
        }
        "import" => {
            // dynamic import()
            (ImportKind::DynamicImport, false)
        }
        _ => return None,
    };

    // Extract string argument
    let args = node.child_by_field_name("arguments")?;
    let mut module_path = None;
    for i in 0..args.child_count() {
        if let Some(arg) = args.child(i) {
            if arg.kind() == "string" {
                module_path = strip_quotes_opt(&arg, source);
                break;
            }
        }
    }

    let range = make_range(node, source);
    let source_text = trimmed_node_text(node, source, limits::MAX_TEXT_BYTES);

    Some(AstImport {
        kind,
        module_path,
        names: Vec::new(),
        range,
        is_type_only,
        source_text: Some(source_text),
    })
}

// ---------------------------------------------------------------------------
// Python
// ---------------------------------------------------------------------------

fn find_imports_python(tree: &Tree, source: &str) -> Vec<AstImport> {
    let mut results = Vec::new();
    collect_imports_python(&tree.root_node(), source, &mut results);
    results.truncate(limits::MAX_RESULTS);
    results
}

fn collect_imports_python(node: &tree_sitter::Node, source: &str, results: &mut Vec<AstImport>) {
    if results.len() >= limits::MAX_RESULTS {
        return;
    }

    match node.kind() {
        "import_statement" => {
            if let Some(import) = extract_py_import_statement(node, source) {
                results.push(import);
            }
            return;
        }
        "import_from_statement" => {
            if let Some(import) = extract_py_import_from_statement(node, source) {
                results.push(import);
            }
            return;
        }
        _ => {}
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_imports_python(&child, source, results);
        }
    }
}

/// Extract `import x` or `import x as y`.
fn extract_py_import_statement(node: &tree_sitter::Node, source: &str) -> Option<AstImport> {
    let mut names = Vec::new();

    for i in 0..node.child_count() {
        let child = node.child(i)?;
        match child.kind() {
            "dotted_name" => {
                let text = child.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                names.push(Alias { imported: text.clone(), local: text });
            }
            "aliased_import" => {
                let mut imported = String::new();
                let mut local = String::new();
                let mut past_as = false;
                for j in 0..child.child_count() {
                    if let Some(ac) = child.child(j) {
                        match ac.kind() {
                            "dotted_name" => {
                                let t = ac.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                                if past_as {
                                    local = t;
                                } else {
                                    imported = t;
                                }
                            }
                            "identifier" => {
                                let t = ac.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                                if past_as {
                                    local = t;
                                } else {
                                    imported = t.clone();
                                    local = t;
                                }
                            }
                            "as" => past_as = true,
                            _ => {}
                        }
                    }
                }
                if imported.is_empty() {
                    imported = local.clone();
                }
                if local.is_empty() {
                    local = imported.clone();
                }
                names.push(Alias { imported, local });
            }
            _ => {}
        }
    }

    let range = make_range(node, source);
    let source_text = trimmed_node_text(node, source, limits::MAX_TEXT_BYTES);

    Some(AstImport {
        kind: ImportKind::Import,
        module_path: None,
        names,
        range,
        is_type_only: false,
        source_text: Some(source_text),
    })
}

/// Extract `from X import Y` or `from X import Y as Z`.
fn extract_py_import_from_statement(node: &tree_sitter::Node, source: &str) -> Option<AstImport> {
    let mut module_path = None;
    let mut names = Vec::new();
    let mut is_wildcard = false;

    for i in 0..node.child_count() {
        let child = node.child(i)?;
        match child.kind() {
            "dotted_name" => {
                let text = child.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                if module_path.is_none() {
                    // First dotted_name is the module (after "from")
                    module_path = Some(text);
                } else {
                    // Subsequent dotted_names are imported names
                    names.push(Alias { imported: text.clone(), local: text });
                }
            }
            "aliased_import" => {
                let mut imported = String::new();
                let mut local = String::new();
                let mut past_as = false;
                for j in 0..child.child_count() {
                    if let Some(ac) = child.child(j) {
                        match ac.kind() {
                            "dotted_name" | "identifier" => {
                                let t = ac.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                                if past_as {
                                    local = t;
                                } else {
                                    imported = t;
                                }
                            }
                            "as" => past_as = true,
                            _ => {}
                        }
                    }
                }
                if imported.is_empty() {
                    imported = local.clone();
                }
                if local.is_empty() {
                    local = imported.clone();
                }
                names.push(Alias { imported, local });
            }
            "wildcard_import" => {
                is_wildcard = true;
            }
            _ => {}
        }
    }

    if is_wildcard {
        names.push(Alias { imported: "*".to_string(), local: "*".to_string() });
    }

    let range = make_range(node, source);
    let source_text = trimmed_node_text(node, source, limits::MAX_TEXT_BYTES);

    Some(AstImport {
        kind: ImportKind::FromImport,
        module_path,
        names,
        range,
        is_type_only: false,
        source_text: Some(source_text),
    })
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Build a Range from a tree-sitter node.
fn make_range(node: &tree_sitter::Node, source: &str) -> Range {
    let start = ts_point_to_position(node.start_position(), source);
    let end = ts_point_to_position(node.end_position(), source);
    Range { start, end }
}

/// Get trimmed text from a node, capped at `max_bytes`.
fn trimmed_node_text(node: &tree_sitter::Node, source: &str, max_bytes: usize) -> String {
    let start = node.start_byte();
    let end = node.end_byte();
    let raw = &source[start..end];
    let trimmed = raw.trim();
    if trimmed.len() > max_bytes {
        let mut bound = max_bytes;
        while bound > 0 && !trimmed.is_char_boundary(bound) {
            bound -= 1;
        }
        trimmed[..bound].to_string()
    } else {
        trimmed.to_string()
    }
}

/// Strip surrounding quotes from a string literal node, returning the inner text.
fn strip_quotes_opt(node: &tree_sitter::Node, source: &str) -> Option<String> {
    let raw = node.utf8_text(source.as_bytes()).ok()?;
    let trimmed = raw.trim();
    if trimmed.len() >= 2 {
        let first = trimmed.chars().next()?;
        let last = trimmed.chars().last()?;
        if (first == '"' && last == '"')
            || (first == '\'' && last == '\'')
            || (first == '`' && last == '`')
        {
            Some(trimmed[1..trimmed.len() - 1].to_string())
        } else {
            Some(trimmed.to_string())
        }
    } else {
        Some(trimmed.to_string())
    }
}
