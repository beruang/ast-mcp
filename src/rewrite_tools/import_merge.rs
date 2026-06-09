//! Shared import parse/merge/dedup logic for TS/JS/Python/Go/Rust.

/// A parsed import from a source file.
#[derive(Debug, Clone)]
pub struct ParsedImport {
    pub source: String,
    pub default_import: Option<String>,
    pub named_imports: Vec<String>,
    pub namespace_import: Option<String>,
    pub is_type_only: bool,
    pub is_side_effect: bool,
    pub start_line: u32,
    pub end_line: u32,
    pub raw_text: String,
}

/// Check whether a named import already exists in a list.
pub fn has_named_import(existing: &[String], name: &str) -> bool {
    existing.iter().any(|n| n == name || n.trim() == name.trim())
}

/// Merge two lists of named imports, deduplicating.
pub fn merge_named_imports(existing: &[String], incoming: &[String]) -> Vec<String> {
    let mut merged: Vec<String> = existing.to_vec();
    for name in incoming {
        if !has_named_import(&merged, name) {
            merged.push(name.clone());
        }
    }
    merged
}

// ── TypeScript/JavaScript parsing ──

pub fn parse_ts_imports(source: &str) -> Vec<ParsedImport> {
    let mut imports = Vec::new();
    let mut parser = tree_sitter::Parser::new();
    let lang = tree_sitter_typescript::language_typescript();
    if parser.set_language(&lang).is_err() {
        return imports;
    }
    let Some(tree) = parser.parse(source, None) else {
        return imports;
    };
    walk_for_imports(&tree.root_node(), source, &mut imports, "import_statement");
    imports
}

fn walk_for_imports(
    node: &tree_sitter::Node,
    source: &str,
    imports: &mut Vec<ParsedImport>,
    target_kind: &str,
) {
    if node.kind() == target_kind {
        if let Some(import) = parse_ts_import_node(node, source) {
            imports.push(import);
        }
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            walk_for_imports(&child, source, imports, target_kind);
        }
    }
}

fn parse_ts_import_node(node: &tree_sitter::Node, source: &str) -> Option<ParsedImport> {
    let text = node.utf8_text(source.as_bytes()).ok()?.to_string();
    let start_line = node.start_position().row as u32;
    let end_line = node.end_position().row as u32;

    let source_str = node
        .child_by_field_name("source")
        .and_then(|s| s.utf8_text(source.as_bytes()).ok())
        .map(|s| s.trim_matches('"').trim_matches('\'').to_string())
        .unwrap_or_default();

    // Find the import_clause — first named child that's not a string
    let mut clause: Option<tree_sitter::Node> = None;
    {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.is_named() && child.kind() != "string" && child.kind() != "string_fragment" {
                clause = Some(child);
                break;
            }
        }
    }

    // Side-effect import: `import "mod"` (no import clause, source is the only named child)
    if clause.is_none() || source_str.is_empty() {
        return Some(ParsedImport {
            source: source_str,
            default_import: None,
            named_imports: vec![],
            namespace_import: None,
            is_type_only: false,
            is_side_effect: true,
            start_line,
            end_line,
            raw_text: text,
        });
    }

    let clause = clause?;
    let clause_text = clause.utf8_text(source.as_bytes()).ok()?;
    let is_type_only = clause_text.starts_with("type ");

    let mut default_import = None;
    let mut named_imports = Vec::new();
    let mut namespace_import = None;

    // The import_clause directly contains the default_import identifier and/or named_imports block
    // Walk all children (named and unnamed) to find them
    let mut cursor = clause.walk();
    for child in clause.children(&mut cursor) {
        if !child.is_named() {
            continue;
        }
        match child.kind() {
            "identifier" | "type_identifier" => {
                if default_import.is_none() {
                    default_import = child.utf8_text(source.as_bytes()).ok().map(String::from);
                }
            }
            "named_imports" => {
                let mut nc = child.walk();
                for spec in child.children(&mut nc) {
                    if spec.kind() == "import_specifier" {
                        if let Some(name_node) = spec.child_by_field_name("name") {
                            if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                                named_imports.push(name.to_string());
                            }
                        }
                    }
                }
            }
            "namespace_import" => {
                if let Some(name_node) = child.child_by_field_name("name") {
                    namespace_import =
                        name_node.utf8_text(source.as_bytes()).ok().map(String::from);
                }
            }
            _ => {}
        }
    }

    Some(ParsedImport {
        source: source_str,
        default_import,
        named_imports,
        namespace_import,
        is_type_only,
        is_side_effect: false,
        start_line,
        end_line,
        raw_text: text,
    })
}

// ── Python parsing ──

pub fn parse_python_imports(source: &str) -> Vec<ParsedImport> {
    let mut imports = Vec::new();
    let mut parser = tree_sitter::Parser::new();
    let lang = tree_sitter_python::language();
    if parser.set_language(&lang).is_err() {
        return imports;
    }
    let Some(tree) = parser.parse(source, None) else {
        return imports;
    };
    walk_python_imports(&tree.root_node(), source, &mut imports);
    imports
}

fn walk_python_imports(node: &tree_sitter::Node, source: &str, imports: &mut Vec<ParsedImport>) {
    match node.kind() {
        "import_statement" | "import_from_statement" => {
            if let Some(import) = parse_python_import_node(node, source) {
                imports.push(import);
                return; // Don't recurse into import body
            }
        }
        _ => {}
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            walk_python_imports(&child, source, imports);
        }
    }
}

fn parse_python_import_node(node: &tree_sitter::Node, source: &str) -> Option<ParsedImport> {
    let text = node.utf8_text(source.as_bytes()).ok()?.to_string();
    let start_line = node.start_position().row as u32;
    let end_line = node.end_position().row as u32;

    if node.kind() == "import_from_statement" {
        let source_str = node
            .child_by_field_name("module_name")
            .and_then(|s| s.utf8_text(source.as_bytes()).ok())
            .map(|s| s.to_string())
            .unwrap_or_default();

        let named_imports = extract_python_import_names(node, source);

        return Some(ParsedImport {
            source: source_str,
            default_import: None,
            named_imports,
            namespace_import: None,
            is_type_only: false,
            is_side_effect: false,
            start_line,
            end_line,
            raw_text: text,
        });
    }

    // Plain `import module` or `import module as alias`
    if node.kind() == "import_statement" {
        let source_str = node
            .child_by_field_name("name")
            .and_then(|s| s.utf8_text(source.as_bytes()).ok())
            .map(|s| s.to_string())
            .unwrap_or_default();

        return Some(ParsedImport {
            source: source_str,
            default_import: None,
            named_imports: vec![],
            namespace_import: None,
            is_type_only: false,
            is_side_effect: false,
            start_line,
            end_line,
            raw_text: text,
        });
    }

    None
}

fn extract_python_import_names(node: &tree_sitter::Node, source: &str) -> Vec<String> {
    let mut names = Vec::new();
    for i in 0..node.child_count() {
        let Some(child) = node.child(i) else { continue };
        match child.kind() {
            "dotted_name" | "identifier" => {
                if let Ok(text) = child.utf8_text(source.as_bytes()) {
                    let t = text.trim().trim_end_matches(',');
                    if t != "from" && t != "import" && !t.is_empty() {
                        names.push(t.to_string());
                    }
                }
            }
            "aliased_import" => {
                // `name as alias` — take the alias
                if let Some(alias) = child.child_by_field_name("alias") {
                    if let Ok(text) = alias.utf8_text(source.as_bytes()) {
                        names.push(text.to_string());
                    }
                }
            }
            _ => {}
        }
    }
    names
}

// ── Go parsing ──

pub fn parse_go_imports(source: &str) -> Vec<ParsedImport> {
    let mut imports = Vec::new();
    let mut parser = tree_sitter::Parser::new();
    let lang = tree_sitter_go::language();
    if parser.set_language(&lang).is_err() {
        return imports;
    }
    let Some(tree) = parser.parse(source, None) else {
        return imports;
    };
    walk_go_imports(&tree.root_node(), source, &mut imports);
    imports
}

fn walk_go_imports(node: &tree_sitter::Node, source: &str, imports: &mut Vec<ParsedImport>) {
    if node.kind() == "import_declaration" {
        // Single import: `import "fmt"`
        if let Some(path) = node.child_by_field_name("path") {
            if let Ok(p) = path.utf8_text(source.as_bytes()) {
                let source_str = p.trim_matches('"').trim_matches('\'').to_string();
                let is_side = source_str.is_empty() || source_str == "_";
                imports.push(ParsedImport {
                    source: source_str,
                    default_import: None,
                    named_imports: vec![],
                    namespace_import: None,
                    is_type_only: false,
                    is_side_effect: is_side,
                    start_line: node.start_position().row as u32,
                    end_line: node.end_position().row as u32,
                    raw_text: node
                        .utf8_text(source.as_bytes())
                        .ok()
                        .unwrap_or_default()
                        .to_string(),
                });
            }
        }
        // Grouped import: `import ( ... )`
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "import_spec_list" {
                    for j in 0..child.child_count() {
                        if let Some(spec) = child.child(j) {
                            if spec.kind() == "import_spec" {
                                if let Some(path) = spec.child_by_field_name("path") {
                                    if let Ok(p) = path.utf8_text(source.as_bytes()) {
                                        let source_str =
                                            p.trim_matches('"').trim_matches('\'').to_string();
                                        imports.push(ParsedImport {
                                            source: source_str,
                                            default_import: None,
                                            named_imports: vec![],
                                            namespace_import: None,
                                            is_type_only: false,
                                            is_side_effect: false,
                                            start_line: spec.start_position().row as u32,
                                            end_line: spec.end_position().row as u32,
                                            raw_text: spec
                                                .utf8_text(source.as_bytes())
                                                .ok()
                                                .unwrap_or_default()
                                                .to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        return; // Don't recurse deeper
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            walk_go_imports(&child, source, imports);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_dedup() {
        let merged = merge_named_imports(&["a".into(), "b".into()], &["c".into(), "a".into()]);
        assert_eq!(merged, vec!["a", "b", "c"]);
    }

    #[test]
    fn parse_ts_default() {
        let src = "import React from 'react';\n";
        let i = parse_ts_imports(src);
        assert_eq!(i.len(), 1);
        assert_eq!(i[0].default_import.as_deref(), Some("React"));
        assert_eq!(i[0].source, "react");
    }

    #[test]
    fn parse_ts_named() {
        let src = "import { useState, useEffect } from 'react';\n";
        let i = parse_ts_imports(src);
        assert_eq!(i.len(), 1);
        assert!(i[0].named_imports.contains(&"useState".into()));
        assert_eq!(i[0].source, "react");
    }

    #[test]
    fn parse_ts_side_effect() {
        let src = "import 'reflect-metadata';\n";
        let i = parse_ts_imports(src);
        assert!(i[0].is_side_effect);
    }

    #[test]
    fn parse_python_from() {
        let src = "from app.models import User, Profile\n";
        let i = parse_python_imports(src);
        assert_eq!(i.len(), 1);
        assert_eq!(i[0].source, "app.models");
    }
}
