use serde::Serialize;
use tree_sitter::Tree;

use crate::parser::positions::ts_point_to_position;
use crate::shared::language::LanguageId;
use crate::shared::position::Range;

use crate::safety::limits;

/// Represents a class definition found in source code.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstClass {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implements_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decorators_text: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub methods: Option<Vec<AstClassMethod>>,
    pub range: Range,
    pub is_default_export: bool,
    pub is_abstract: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_text: Option<String>,
}

/// Classification of a class method.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClassMethodKind {
    Method,
    Constructor,
    Getter,
    Setter,
    Static,
    Abstract,
    Unknown,
}

/// A single method within a class.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstClassMethod {
    pub name: String,
    pub kind: ClassMethodKind,
    pub range: Range,
}

/// Options controlling class extraction.
#[derive(Debug, Clone)]
pub struct ClassOptions {
    pub max_results: usize,
    pub include_methods: bool,
    pub include_extends: bool,
    pub include_implements: bool,
    pub include_decorators: bool,
}

impl Default for ClassOptions {
    fn default() -> Self {
        ClassOptions {
            max_results: limits::MAX_RESULTS,
            include_methods: true,
            include_extends: true,
            include_implements: true,
            include_decorators: true,
        }
    }
}

/// Find all class definitions in a parsed file.
pub fn find_classes(
    tree: &Tree,
    source: &str,
    lang: LanguageId,
    opts: &ClassOptions,
) -> Vec<AstClass> {
    match lang {
        LanguageId::TypeScript
        | LanguageId::TypeScriptReact
        | LanguageId::JavaScript
        | LanguageId::JavaScriptReact => find_classes_ts_js(tree, source, opts),
        LanguageId::Python => find_classes_python(tree, source, opts),
    }
}

// ---------------------------------------------------------------------------
// TS/JS
// ---------------------------------------------------------------------------

fn find_classes_ts_js(tree: &Tree, source: &str, opts: &ClassOptions) -> Vec<AstClass> {
    let mut results = Vec::new();
    collect_ts_js_classes(&tree.root_node(), source, false, opts, &mut results);
    results.truncate(opts.max_results);
    results
}

fn collect_ts_js_classes(
    node: &tree_sitter::Node,
    source: &str,
    parent_exported: bool,
    opts: &ClassOptions,
    results: &mut Vec<AstClass>,
) {
    if results.len() >= opts.max_results {
        return;
    }

    for i in 0..node.child_count() {
        let child = match node.child(i) {
            Some(c) => c,
            None => continue,
        };
        if !child.is_named() {
            continue;
        }

        match child.kind() {
            "class_declaration" | "class_expression" => {
                extract_ts_js_class(&child, source, parent_exported, false, opts, results);
            }
            "abstract_class_declaration" => {
                // abstract_class_declaration directly contains name, body, heritage etc.
                // It does NOT wrap a separate class_declaration child.
                extract_ts_js_class(&child, source, parent_exported, true, opts, results);
            }
            "export_statement" => {
                // Check for exported class inside
                for j in 0..child.child_count() {
                    if let Some(inner) = child.child(j) {
                        match inner.kind() {
                            "class_declaration" | "class_expression" => {
                                extract_ts_js_class(&inner, source, true, false, opts, results);
                            }
                            "abstract_class_declaration" => {
                                extract_ts_js_class(&inner, source, true, true, opts, results);
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn extract_ts_js_class(
    node: &tree_sitter::Node,
    source: &str,
    is_default_export: bool,
    is_abstract: bool,
    opts: &ClassOptions,
    results: &mut Vec<AstClass>,
) {
    if results.len() >= opts.max_results {
        return;
    }

    let name = extract_identifier_name(node, source);

    let extends_text =
        if opts.include_extends { extract_ts_js_extends(node, source) } else { None };

    let implements_text =
        if opts.include_implements { extract_ts_js_implements(node, source) } else { None };

    let decorators_text = if opts.include_decorators {
        let decs = extract_ts_js_decorators(node, source);
        if decs.is_empty() {
            None
        } else {
            Some(decs)
        }
    } else {
        None
    };

    let methods = if opts.include_methods {
        let meths = extract_ts_js_methods(node, source);
        if meths.is_empty() {
            None
        } else {
            Some(meths)
        }
    } else {
        None
    };

    let range = make_range(node, source);
    let source_text = trimmed_node_text(node, source, limits::MAX_TEXT_BYTES);

    results.push(AstClass {
        name,
        extends_text,
        implements_text,
        decorators_text,
        methods,
        range,
        is_default_export,
        is_abstract,
        source_text: Some(source_text),
    });
}

fn extract_ts_js_extends(node: &tree_sitter::Node, source: &str) -> Option<String> {
    // Look for class_heritage node directly
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "class_heritage" {
                // Look for extends_clause inside
                for j in 0..child.child_count() {
                    if let Some(ec) = child.child(j) {
                        if ec.kind() == "extends_clause" {
                            // Get the first named child after "extends"
                            for k in 0..ec.child_count() {
                                if let Some(inner) = ec.child(k) {
                                    if inner.is_named() {
                                        return inner
                                            .utf8_text(source.as_bytes())
                                            .ok()
                                            .map(|s| s.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn extract_ts_js_implements(node: &tree_sitter::Node, source: &str) -> Option<String> {
    if let Some(heritage) = node.child_by_field_name("heritage") {
        for i in 0..heritage.child_count() {
            if let Some(child) = heritage.child(i) {
                if child.kind() == "implements_clause" {
                    for j in 0..child.child_count() {
                        if let Some(ic) = child.child(j) {
                            if ic.is_named() {
                                return ic.utf8_text(source.as_bytes()).ok().map(|s| s.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn extract_ts_js_decorators(node: &tree_sitter::Node, source: &str) -> Vec<String> {
    let mut decs = Vec::new();
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "decorator" {
                if let Ok(text) = child.utf8_text(source.as_bytes()) {
                    decs.push(text.to_string());
                }
            }
        }
    }
    decs
}

fn extract_ts_js_methods(node: &tree_sitter::Node, source: &str) -> Vec<AstClassMethod> {
    let body = match node.child_by_field_name("body") {
        Some(b) => b,
        None => return Vec::new(),
    };

    let mut methods = Vec::new();

    for i in 0..body.child_count() {
        let child = match body.child(i) {
            Some(c) => c,
            None => continue,
        };
        if !child.is_named() {
            continue;
        }

        match child.kind() {
            "method_definition" => {
                let (method_name, method_kind) = extract_ts_js_method_info(&child, source);
                let range = make_range(&child, source);
                methods.push(AstClassMethod { name: method_name, kind: method_kind, range });
            }
            "public_field_definition" | "field_definition" => {
                // Property/field — not a method per se
                if let Some(name) = extract_identifier_name(&child, source) {
                    let range = make_range(&child, source);
                    methods.push(AstClassMethod { name, kind: ClassMethodKind::Unknown, range });
                }
            }
            _ => {}
        }
    }

    methods
}

fn extract_ts_js_method_info(node: &tree_sitter::Node, source: &str) -> (String, ClassMethodKind) {
    let name = extract_identifier_name(node, source).unwrap_or_else(|| "<anonymous>".to_string());

    let kind = if name == "constructor" {
        ClassMethodKind::Constructor
    } else if has_anonymous_child(node, "get") {
        ClassMethodKind::Getter
    } else if has_anonymous_child(node, "set") {
        ClassMethodKind::Setter
    } else if has_anonymous_child(node, "static") {
        ClassMethodKind::Static
    } else if has_anonymous_child(node, "abstract") {
        ClassMethodKind::Abstract
    } else {
        ClassMethodKind::Method
    };

    (name, kind)
}

// ---------------------------------------------------------------------------
// Python
// ---------------------------------------------------------------------------

fn find_classes_python(tree: &Tree, source: &str, opts: &ClassOptions) -> Vec<AstClass> {
    let mut results = Vec::new();
    collect_py_classes(&tree.root_node(), source, opts, &mut results);
    results.truncate(opts.max_results);
    results
}

fn collect_py_classes(
    node: &tree_sitter::Node,
    source: &str,
    opts: &ClassOptions,
    results: &mut Vec<AstClass>,
) {
    if results.len() >= opts.max_results {
        return;
    }

    for i in 0..node.child_count() {
        let child = match node.child(i) {
            Some(c) => c,
            None => continue,
        };
        if !child.is_named() {
            continue;
        }

        match child.kind() {
            "class_definition" => {
                extract_py_class(&child, source, false, opts, results);
            }
            "decorated_definition" => {
                if let Some(inner) = child.child_by_field_name("definition") {
                    if inner.kind() == "class_definition" {
                        extract_py_class(&inner, source, false, opts, results);
                    }
                }
            }
            _ => {}
        }
    }
}

fn extract_py_class(
    node: &tree_sitter::Node,
    source: &str,
    is_default_export: bool,
    opts: &ClassOptions,
    results: &mut Vec<AstClass>,
) {
    if results.len() >= opts.max_results {
        return;
    }

    let name = extract_identifier_name(node, source);
    let is_abstract = false; // Python doesn't have abstract keyword

    let extends_text = if opts.include_extends { extract_py_bases(node, source) } else { None };

    let decorators_text: Option<Vec<String>> = None; // Decorators are on outer decorated_definition

    let methods = if opts.include_methods {
        let meths = extract_py_methods(node, source);
        if meths.is_empty() {
            None
        } else {
            Some(meths)
        }
    } else {
        None
    };

    let range = make_range(node, source);
    let source_text = trimmed_node_text(node, source, limits::MAX_TEXT_BYTES);

    results.push(AstClass {
        name,
        extends_text,
        implements_text: None,
        decorators_text,
        methods,
        range,
        is_default_export,
        is_abstract,
        source_text: Some(source_text),
    });
}

fn extract_py_bases(node: &tree_sitter::Node, source: &str) -> Option<String> {
    // Python uses `argument_list` for base classes in class definition
    // Look for `argument_list` after the class name
    let mut past_name = false;
    for i in 0..node.child_count() {
        let child = match node.child(i) {
            Some(c) => c,
            None => continue,
        };
        if child.kind() == "identifier" {
            past_name = true;
            continue;
        }
        if past_name && child.kind() == "argument_list" {
            return child.utf8_text(source.as_bytes()).ok().map(|s| s.to_string());
        }
    }
    None
}

fn extract_py_methods(node: &tree_sitter::Node, source: &str) -> Vec<AstClassMethod> {
    let body = match node.child_by_field_name("body") {
        Some(b) => b,
        None => return Vec::new(),
    };

    let mut methods = Vec::new();

    for i in 0..body.child_count() {
        let child = match body.child(i) {
            Some(c) => c,
            None => continue,
        };
        if !child.is_named() {
            continue;
        }

        match child.kind() {
            "function_definition" => {
                let name = extract_identifier_name(&child, source)
                    .unwrap_or_else(|| "<anonymous>".to_string());
                let kind = if name == "__init__" {
                    ClassMethodKind::Constructor
                } else {
                    ClassMethodKind::Method
                };
                let range = make_range(&child, source);
                methods.push(AstClassMethod { name, kind, range });
            }
            "decorated_definition" => {
                if let Some(inner) = child.child_by_field_name("definition") {
                    if inner.kind() == "function_definition" {
                        let name = extract_identifier_name(&inner, source)
                            .unwrap_or_else(|| "<anonymous>".to_string());
                        let kind = if name == "__init__" {
                            ClassMethodKind::Constructor
                        } else {
                            ClassMethodKind::Method
                        };
                        let range = make_range(&inner, source);
                        methods.push(AstClassMethod { name, kind, range });
                    }
                }
            }
            _ => {}
        }
    }

    methods
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn make_range(node: &tree_sitter::Node, source: &str) -> Range {
    let start = ts_point_to_position(node.start_position(), source);
    let end = ts_point_to_position(node.end_position(), source);
    Range { start, end }
}

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

fn extract_identifier_name(node: &tree_sitter::Node, source: &str) -> Option<String> {
    if let Some(name_node) = node.child_by_field_name("name") {
        if let Ok(text) = name_node.utf8_text(source.as_bytes()) {
            return Some(text.to_string());
        }
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "identifier" {
                return child.utf8_text(source.as_bytes()).ok().map(|s| s.to_string());
            }
        }
    }
    None
}

fn has_anonymous_child(node: &tree_sitter::Node, target: &str) -> bool {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == target {
                return true;
            }
        }
    }
    false
}
