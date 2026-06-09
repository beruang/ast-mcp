use serde::Serialize;
use tree_sitter::Tree;

use crate::parser::positions::ts_point_to_position;
use crate::shared::language::LanguageId;
use crate::shared::position::Range;

use crate::safety::limits;

/// Represents a single export statement found in source code.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstExport {
    pub kind: ExportKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub range: Range,
    pub is_default: bool,
    pub is_type_only: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub re_export_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_text: Option<String>,
}

/// Classification of an export statement.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExportKind {
    Function,
    Class,
    Const,
    Let,
    Var,
    Type,
    Interface,
    Enum,
    ReExport,
    Default,
    PythonPublicDefinition,
    PythonAll,
    Unknown,
}

/// Find all export statements in a parsed file.
///
/// For Python, `include_best_effort_python` enables detection of `__all__`
/// assignments and top-level public definitions (names not starting with `_`).
pub fn find_exports(
    tree: &Tree,
    source: &str,
    lang: LanguageId,
    include_best_effort_python: bool,
) -> Vec<AstExport> {
    match lang {
        LanguageId::TypeScript
        | LanguageId::TypeScriptReact
        | LanguageId::JavaScript
        | LanguageId::JavaScriptReact => find_exports_ts_js(tree, source),
        LanguageId::Python => find_exports_python(tree, source, include_best_effort_python),
        LanguageId::Go | LanguageId::Rust => vec![],
    }
}

// ---------------------------------------------------------------------------
// TS/JS
// ---------------------------------------------------------------------------

fn find_exports_ts_js(tree: &Tree, source: &str) -> Vec<AstExport> {
    let mut results = Vec::new();
    collect_exports_ts_js(&tree.root_node(), source, &mut results);
    results.truncate(limits::MAX_RESULTS);
    results
}

fn collect_exports_ts_js(node: &tree_sitter::Node, source: &str, results: &mut Vec<AstExport>) {
    if results.len() >= limits::MAX_RESULTS {
        return;
    }

    if node.kind() == "export_statement" {
        if let Some(export) = extract_ts_js_export(node, source) {
            results.push(export);
        }
        return; // Don't recurse into export_statement
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_exports_ts_js(&child, source, results);
        }
    }
}

/// Extract information from an `export_statement` node.
fn extract_ts_js_export(node: &tree_sitter::Node, source: &str) -> Option<AstExport> {
    let mut kind = ExportKind::Unknown;
    let mut name: Option<String> = None;
    let mut is_default = false;
    let mut is_type_only = false;
    let mut re_export_source: Option<String> = None;

    for i in 0..node.child_count() {
        let child = match node.child(i) {
            Some(c) => c,
            None => continue,
        };

        match child.kind() {
            "default" => is_default = true,
            "type" => is_type_only = true,
            "function_declaration" | "generator_function_declaration" => {
                kind = ExportKind::Function;
                name = extract_identifier_name(&child, source);
            }
            "class_declaration" => {
                kind = ExportKind::Class;
                name = extract_identifier_name(&child, source);
            }
            "lexical_declaration" | "variable_declaration" => {
                // Determine const/let/var
                let (decl_kind, decl_name) = extract_lexical_info(&child, source);
                kind = decl_kind;
                name = decl_name;
            }
            "type_alias_declaration" => {
                kind = ExportKind::Type;
                name = extract_identifier_name(&child, source);
            }
            "interface_declaration" => {
                kind = ExportKind::Interface;
                name = extract_identifier_name(&child, source);
            }
            "enum_declaration" => {
                kind = ExportKind::Enum;
                name = extract_identifier_name(&child, source);
            }
            "export_clause" => {
                // `export { a, b }` or `export { a, b } from "mod"`
                kind = ExportKind::ReExport;
                name = extract_first_export_spec(&child, source);
            }
            "from_clause" => {
                // `export { ... } from "mod"` or `export * from "mod"`
                for j in 0..child.child_count() {
                    if let Some(gc) = child.child(j) {
                        if gc.kind() == "string" {
                            re_export_source = strip_quotes_opt(&gc, source);
                        }
                    }
                }
            }
            "namespace_export" => {
                // `export * as ns from "mod"`
                kind = ExportKind::ReExport;
                name = extract_identifier_name(&child, source);
            }
            "call_expression" | "identifier" if is_default && kind == ExportKind::Unknown => {
                // `export default <expr>` or `export default <id>`
                kind = ExportKind::Default;
                name = child.utf8_text(source.as_bytes()).ok().map(|s| s.to_string());
            }
            _ => {}
        }
    }

    // Handle `export * from "mod"` (wildcard re-export)
    if kind == ExportKind::Unknown {
        // Check for `*` child (anonymous)
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "*" || child.kind() == "namespace_export" {
                    kind = ExportKind::ReExport;
                    break;
                }
            }
        }
    }

    // Also try the "source" field (may be hoisted from from_clause).
    if re_export_source.is_none() {
        if let Some(src) = node.child_by_field_name("source") {
            re_export_source = strip_quotes_opt(&src, source);
        }
    }
    // Also check for direct string child (when from_clause is hidden/inlined).
    if re_export_source.is_none() {
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "string" {
                    // Check if preceded by "from" keyword
                    let from_idx = i.checked_sub(1);
                    let is_from = from_idx
                        .and_then(|idx| node.child(idx))
                        .is_some_and(|prev| prev.kind() == "from");
                    if is_from {
                        re_export_source = strip_quotes_opt(&child, source);
                        break;
                    }
                }
            }
        }
    }

    // If we found a from_clause but no export_clause, it's a wildcard re-export
    if kind == ExportKind::Unknown && re_export_source.is_some() {
        kind = ExportKind::ReExport;
    }

    // Default export without declaration (export default 42)
    if is_default && kind == ExportKind::Unknown {
        kind = ExportKind::Default;
    }

    let range = make_range(node, source);
    let source_text = trimmed_node_text(node, source, limits::MAX_TEXT_BYTES);

    Some(AstExport {
        kind,
        name,
        range,
        is_default,
        is_type_only,
        re_export_source,
        source_text: Some(source_text),
    })
}

/// Extract the first identifier from a node (used for function/class names).
fn extract_identifier_name(node: &tree_sitter::Node, source: &str) -> Option<String> {
    // First try the "name" field (used by class_declaration, function_declaration, etc.)
    if let Some(name_node) = node.child_by_field_name("name") {
        if let Ok(text) = name_node.utf8_text(source.as_bytes()) {
            return Some(text.to_string());
        }
    }
    // Fall back to iterating children for an identifier
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "identifier" {
                return child.utf8_text(source.as_bytes()).ok().map(|s| s.to_string());
            }
            // Recurse for decorated/async/unwrapped nodes
            if child.is_named() {
                if let Some(name) = extract_identifier_name(&child, source) {
                    return Some(name);
                }
            }
        }
    }
    None
}

/// Extract const/let/var kind and variable name from a lexical/variable declaration.
fn extract_lexical_info(node: &tree_sitter::Node, source: &str) -> (ExportKind, Option<String>) {
    let mut kind = ExportKind::Unknown;
    let mut name = None;

    for i in 0..node.child_count() {
        let child = match node.child(i) {
            Some(c) => c,
            None => continue,
        };
        match child.kind() {
            "const" => kind = ExportKind::Const,
            "let" => kind = ExportKind::Let,
            "var" => kind = ExportKind::Var,
            "variable_declarator" => {
                // First identifier is the variable name
                for j in 0..child.child_count() {
                    if let Some(vc) = child.child(j) {
                        if vc.kind() == "identifier" {
                            name = vc.utf8_text(source.as_bytes()).ok().map(|s| s.to_string());
                            break;
                        }
                        // Could be destructuring pattern
                        if vc.kind() == "object_pattern" || vc.kind() == "array_pattern" {
                            if let Some(n) = extract_identifier_name(&vc, source) {
                                name = Some(n);
                            }
                            break;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    (kind, name)
}

/// Extract the first export specifier name from `export_clause` (for re-exports).
fn extract_first_export_spec(node: &tree_sitter::Node, source: &str) -> Option<String> {
    for i in 0..node.child_count() {
        let child = match node.child(i) {
            Some(c) => c,
            None => continue,
        };
        if child.kind() == "export_specifier" {
            // The export_specifier has name (exported) and optional alias (local)
            for j in 0..child.child_count() {
                if let Some(sc) = child.child(j) {
                    if sc.kind() == "identifier" {
                        return sc.utf8_text(source.as_bytes()).ok().map(|s| s.to_string());
                    }
                }
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Python (best-effort)
// ---------------------------------------------------------------------------

fn find_exports_python(tree: &Tree, source: &str, include_best_effort: bool) -> Vec<AstExport> {
    let mut results = Vec::new();

    if !include_best_effort {
        return results;
    }

    collect_exports_python(&tree.root_node(), source, &mut results);
    results.truncate(limits::MAX_RESULTS);
    results
}

fn collect_exports_python(node: &tree_sitter::Node, source: &str, results: &mut Vec<AstExport>) {
    if results.len() >= limits::MAX_RESULTS {
        return;
    }

    // Only process the top-level module node's children.
    // We stop recursing at class/function bodies.
    match node.kind() {
        "module" => {
            // Process top-level children
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    if !child.is_named() {
                        continue;
                    }
                    match child.kind() {
                        "expression_statement" => {
                            extract_py_all_assignment(&child, source, results);
                        }
                        "function_definition" => {
                            extract_py_public_def(&child, source, ExportKind::Function, results);
                        }
                        "class_definition" => {
                            extract_py_public_def(&child, source, ExportKind::Class, results);
                        }
                        "decorated_definition" => {
                            if let Some(inner) = child.child_by_field_name("definition") {
                                match inner.kind() {
                                    "function_definition" => {
                                        extract_py_public_def(
                                            &inner,
                                            source,
                                            ExportKind::Function,
                                            results,
                                        );
                                    }
                                    "class_definition" => {
                                        extract_py_public_def(
                                            &inner,
                                            source,
                                            ExportKind::Class,
                                            results,
                                        );
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        _ => {
            // Recurse only at module level; don't go into function/class bodies
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    collect_exports_python(&child, source, results);
                }
            }
        }
    }
}

/// Check if a node is a `__all__` assignment and add it as a PythonAll export.
fn extract_py_all_assignment(node: &tree_sitter::Node, source: &str, results: &mut Vec<AstExport>) {
    if results.len() >= limits::MAX_RESULTS {
        return;
    }

    // Look for assignment inside expression_statement
    for i in 0..node.child_count() {
        let child = match node.child(i) {
            Some(c) => c,
            None => continue,
        };
        if child.kind() == "assignment" {
            // Check left-hand side is `__all__`
            let left = match child.child_by_field_name("left") {
                Some(l) => l,
                None => continue,
            };
            if left.kind() == "identifier" {
                let name = left.utf8_text(source.as_bytes()).unwrap_or("");
                if name == "__all__" {
                    let range = make_range(node, source);
                    let source_text = trimmed_node_text(node, source, limits::MAX_TEXT_BYTES);
                    results.push(AstExport {
                        kind: ExportKind::PythonAll,
                        name: Some("__all__".to_string()),
                        range,
                        is_default: false,
                        is_type_only: false,
                        re_export_source: None,
                        source_text: Some(source_text),
                    });
                    return;
                }
            }
        }
    }
}

/// Add a public top-level function or class definition (name without leading _).
fn extract_py_public_def(
    node: &tree_sitter::Node,
    source: &str,
    export_kind: ExportKind,
    results: &mut Vec<AstExport>,
) {
    if results.len() >= limits::MAX_RESULTS {
        return;
    }

    let name = extract_identifier_name(node, source);
    if let Some(ref n) = name {
        if n.starts_with('_') && n != "__init__" {
            return; // Skip private definitions
        }
    }

    let range = make_range(node, source);
    let source_text = trimmed_node_text(node, source, limits::MAX_TEXT_BYTES);

    results.push(AstExport {
        kind: export_kind,
        name,
        range,
        is_default: false,
        is_type_only: false,
        re_export_source: None,
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
