use serde::Serialize;
use tree_sitter::Tree;

use crate::parser::positions::ts_point_to_position;
use crate::shared::language::LanguageId;
use crate::shared::position::Range;

use crate::safety::limits;

/// Represents a function definition found in source code.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstFunction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub kind: FunctionKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<AstParameter>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_type_text: Option<String>,
    pub range: Range,
    #[serde(rename = "async")]
    pub async_: bool,
    pub exported: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_text: Option<String>,
}

/// Classification of a function.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionKind {
    Function,
    Generator,
    Arrow,
    Method,
    Lambda,
    Unknown,
}

/// A single function parameter.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstParameter {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_text: Option<String>,
    pub optional: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value_text: Option<String>,
}

/// Options controlling function extraction.
#[derive(Debug, Clone)]
pub struct FunctionOptions {
    pub max_results: usize,
    pub include_anonymous: bool,
    pub include_parameters: bool,
    pub include_return_type: bool,
    pub include_signature: bool,
}

impl Default for FunctionOptions {
    fn default() -> Self {
        FunctionOptions {
            max_results: limits::MAX_RESULTS,
            include_anonymous: true,
            include_parameters: true,
            include_return_type: true,
            include_signature: true,
        }
    }
}

/// Find all function definitions in a parsed file.
pub fn find_functions(
    tree: &Tree,
    source: &str,
    lang: LanguageId,
    opts: &FunctionOptions,
) -> Vec<AstFunction> {
    match lang {
        LanguageId::TypeScript
        | LanguageId::TypeScriptReact
        | LanguageId::JavaScript
        | LanguageId::JavaScriptReact => find_functions_ts_js(tree, source, opts),
        LanguageId::Python => find_functions_python(tree, source, opts),
    }
}

// ---------------------------------------------------------------------------
// TS/JS
// ---------------------------------------------------------------------------

fn find_functions_ts_js(tree: &Tree, source: &str, opts: &FunctionOptions) -> Vec<AstFunction> {
    let mut results = Vec::new();
    let root = tree.root_node();

    // Walk top-level nodes for functions and exported functions.
    collect_ts_js_functions(&root, source, None, false, opts, &mut results);

    results.truncate(opts.max_results);
    results
}

fn collect_ts_js_functions(
    node: &tree_sitter::Node,
    source: &str,
    parent_name: Option<&str>,
    parent_exported: bool,
    opts: &FunctionOptions,
    results: &mut Vec<AstFunction>,
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
            "function_declaration" | "generator_function_declaration" => {
                let is_generator = child.kind() == "generator_function_declaration";
                extract_ts_js_function(
                    &child,
                    source,
                    if is_generator { FunctionKind::Generator } else { FunctionKind::Function },
                    parent_exported,
                    parent_name,
                    opts,
                    results,
                );
            }
            "arrow_function" => {
                if opts.include_anonymous {
                    extract_ts_js_function(
                        &child,
                        source,
                        FunctionKind::Arrow,
                        parent_exported,
                        parent_name,
                        opts,
                        results,
                    );
                }
            }
            "function_expression" => {
                if opts.include_anonymous {
                    extract_ts_js_function(
                        &child,
                        source,
                        FunctionKind::Function,
                        parent_exported,
                        parent_name,
                        opts,
                        results,
                    );
                }
            }
            "method_definition" => {
                extract_ts_js_function(
                    &child,
                    source,
                    FunctionKind::Method,
                    false,
                    parent_name,
                    opts,
                    results,
                );
            }
            "export_statement" => {
                // Check if this is exporting a function
                let exported = true;
                for j in 0..child.child_count() {
                    if let Some(inner) = child.child(j) {
                        if inner.kind() == "function_declaration"
                            || inner.kind() == "generator_function_declaration"
                        {
                            let is_gen = inner.kind() == "generator_function_declaration";
                            extract_ts_js_function(
                                &inner,
                                source,
                                if is_gen {
                                    FunctionKind::Generator
                                } else {
                                    FunctionKind::Function
                                },
                                exported,
                                parent_name,
                                opts,
                                results,
                            );
                        }
                    }
                }
                // Recurse into export_statement for nested functions
                collect_ts_js_functions(&child, source, parent_name, true, opts, results);
            }
            "class_declaration" | "class_expression" => {
                let class_name = extract_identifier_name(&child, source);
                let class_name_str = class_name.as_deref();
                // Recurse into class body for methods
                if let Some(body) = child.child_by_field_name("body") {
                    collect_ts_js_functions(&body, source, class_name_str, false, opts, results);
                }
            }
            _ => {
                // Recurse for nested functions
                if should_recurse_ts_js(child.kind()) {
                    collect_ts_js_functions(
                        &child,
                        source,
                        parent_name,
                        parent_exported,
                        opts,
                        results,
                    );
                }
            }
        }
    }
}

fn should_recurse_ts_js(kind: &str) -> bool {
    matches!(
        kind,
        "statement_block"
            | "lexical_declaration"
            | "variable_declaration"
            | "if_statement"
            | "for_statement"
            | "for_in_statement"
            | "while_statement"
            | "switch_statement"
            | "try_statement"
            | "catch_clause"
            | "block"
            | "program"
            | "module"
    )
}

fn extract_ts_js_function(
    node: &tree_sitter::Node,
    source: &str,
    kind: FunctionKind,
    exported: bool,
    parent_name: Option<&str>,
    opts: &FunctionOptions,
    results: &mut Vec<AstFunction>,
) {
    if results.len() >= opts.max_results {
        return;
    }

    let name = extract_identifier_name(node, source);

    // Skip anonymous functions unless requested
    if name.is_none() && !opts.include_anonymous {
        return;
    }

    let async_ = has_anonymous_child(node, "async");
    let parameters = if opts.include_parameters {
        let params = extract_ts_js_parameters(node, source);
        if params.is_empty() {
            None
        } else {
            Some(params)
        }
    } else {
        None
    };

    let return_type_text =
        if opts.include_return_type { extract_ts_js_return_type(node, source) } else { None };

    let signature_text = if opts.include_signature {
        Some(trimmed_node_text(node, source, limits::MAX_TEXT_BYTES))
    } else {
        None
    };

    let range = make_range(node, source);

    results.push(AstFunction {
        name,
        kind,
        parameters,
        return_type_text,
        range,
        async_,
        exported,
        parent_name: parent_name.map(|s| s.to_string()),
        signature_text,
    });
}

/// Extract parameters from TS/JS function node.
fn extract_ts_js_parameters(node: &tree_sitter::Node, source: &str) -> Vec<AstParameter> {
    let params_node = node.child_by_field_name("parameters");
    let params_node = match params_node {
        Some(p) => p,
        None => return Vec::new(),
    };

    let mut params = Vec::new();

    for i in 0..params_node.child_count() {
        let child = match params_node.child(i) {
            Some(c) => c,
            None => continue,
        };
        if child.kind() != "required_parameter" && child.kind() != "optional_parameter" {
            continue;
        }

        let is_optional = child.kind() == "optional_parameter";

        // Extract parameter name from the pattern
        let name = extract_ts_js_param_name(&child, source);
        let type_text = extract_ts_js_param_type(&child, source);
        let default_value_text =
            if is_optional { extract_ts_js_default_value(&child, source) } else { None };

        // Also check for field "value" which may contain the default
        let default_value_text = default_value_text.or_else(|| {
            child
                .child_by_field_name("value")
                .and_then(|v| v.utf8_text(source.as_bytes()).ok().map(|s| s.to_string()))
        });

        params.push(AstParameter { name, type_text, optional: is_optional, default_value_text });
    }

    params
}

fn extract_ts_js_param_name(node: &tree_sitter::Node, source: &str) -> String {
    // Look for pattern > identifier, or direct identifier
    if let Some(pattern) = node.child_by_field_name("pattern") {
        return extract_identifier_name(&pattern, source)
            .unwrap_or_else(|| "<unknown>".to_string());
    }
    // Direct identifier child
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "identifier" {
                return child.utf8_text(source.as_bytes()).unwrap_or("<unknown>").to_string();
            }
        }
    }
    "<unknown>".to_string()
}

fn extract_ts_js_param_type(node: &tree_sitter::Node, source: &str) -> Option<String> {
    if let Some(type_node) = node.child_by_field_name("type") {
        type_node.utf8_text(source.as_bytes()).ok().map(|s| s.to_string())
    } else {
        None
    }
}

fn extract_ts_js_default_value(node: &tree_sitter::Node, source: &str) -> Option<String> {
    if let Some(value) = node.child_by_field_name("value") {
        value.utf8_text(source.as_bytes()).ok().map(|s| s.to_string())
    } else {
        None
    }
}

fn extract_ts_js_return_type(node: &tree_sitter::Node, source: &str) -> Option<String> {
    if let Some(ret) = node.child_by_field_name("return_type") {
        ret.utf8_text(source.as_bytes()).ok().map(|s| s.to_string())
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Python
// ---------------------------------------------------------------------------

fn find_functions_python(tree: &Tree, source: &str, opts: &FunctionOptions) -> Vec<AstFunction> {
    let mut results = Vec::new();
    let root = tree.root_node();
    collect_py_functions(&root, source, None, opts, &mut results);
    results.truncate(opts.max_results);
    results
}

fn collect_py_functions(
    node: &tree_sitter::Node,
    source: &str,
    parent_name: Option<&str>,
    opts: &FunctionOptions,
    results: &mut Vec<AstFunction>,
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
            "function_definition" => {
                let async_ = has_anonymous_child(&child, "async");
                let kind = FunctionKind::Function;
                extract_py_function(
                    &child,
                    source,
                    kind,
                    async_,
                    false,
                    parent_name,
                    opts,
                    results,
                );
            }
            "decorated_definition" => {
                if let Some(inner) = child.child_by_field_name("definition") {
                    match inner.kind() {
                        "function_definition" => {
                            let async_ = has_anonymous_child(&inner, "async");
                            extract_py_function(
                                &inner,
                                source,
                                FunctionKind::Function,
                                async_,
                                false,
                                parent_name,
                                opts,
                                results,
                            );
                        }
                        "class_definition" => {
                            let cls_name = extract_identifier_name(&inner, source);
                            let cls_name_str = cls_name.as_deref();
                            if let Some(body) = inner.child_by_field_name("body") {
                                collect_py_functions(&body, source, cls_name_str, opts, results);
                            }
                        }
                        _ => {}
                    }
                }
            }
            "class_definition" => {
                let cls_name = extract_identifier_name(&child, source);
                let cls_name_str = cls_name.as_deref();
                if let Some(body) = child.child_by_field_name("body") {
                    collect_py_functions(&body, source, cls_name_str, opts, results);
                }
            }
            _ => {
                // Recurse into blocks
                if child.kind() == "block" || child.kind() == "module" {
                    collect_py_functions(&child, source, parent_name, opts, results);
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn extract_py_function(
    node: &tree_sitter::Node,
    source: &str,
    kind: FunctionKind,
    async_: bool,
    exported: bool,
    parent_name: Option<&str>,
    opts: &FunctionOptions,
    results: &mut Vec<AstFunction>,
) {
    if results.len() >= opts.max_results {
        return;
    }

    let name = extract_identifier_name(node, source);
    if name.is_none() && !opts.include_anonymous {
        return;
    }

    let parameters = if opts.include_parameters {
        let params = extract_py_parameters(node, source);
        if params.is_empty() {
            None
        } else {
            Some(params)
        }
    } else {
        None
    };

    let return_type_text =
        if opts.include_return_type { extract_py_return_type(node, source) } else { None };

    let signature_text = if opts.include_signature {
        Some(trimmed_node_text(node, source, limits::MAX_TEXT_BYTES))
    } else {
        None
    };

    let range = make_range(node, source);

    results.push(AstFunction {
        name,
        kind,
        parameters,
        return_type_text,
        range,
        async_,
        exported,
        parent_name: parent_name.map(|s| s.to_string()),
        signature_text,
    });
}

fn extract_py_parameters(node: &tree_sitter::Node, source: &str) -> Vec<AstParameter> {
    let params_node = node.child_by_field_name("parameters");
    let params_node = match params_node {
        Some(p) => p,
        None => return Vec::new(),
    };

    let mut params = Vec::new();

    for i in 0..params_node.child_count() {
        let child = match params_node.child(i) {
            Some(c) => c,
            None => continue,
        };

        match child.kind() {
            "identifier" => {
                // Simple parameter without type/default
                params.push(AstParameter {
                    name: child.utf8_text(source.as_bytes()).unwrap_or("<unknown>").to_string(),
                    type_text: None,
                    optional: false,
                    default_value_text: None,
                });
            }
            "default_parameter" => {
                let name = extract_py_param_name(&child, source);
                let default_text = extract_py_default_value(&child, source);
                params.push(AstParameter {
                    name,
                    type_text: None,
                    optional: true,
                    default_value_text: default_text,
                });
            }
            "typed_parameter" => {
                let name = extract_py_param_name(&child, source);
                let type_text = extract_py_param_type(&child, source);
                params.push(AstParameter {
                    name,
                    type_text,
                    optional: false,
                    default_value_text: None,
                });
            }
            "typed_default_parameter" => {
                let name = extract_py_param_name(&child, source);
                let type_text = extract_py_param_type(&child, source);
                let default_text = extract_py_default_value(&child, source);
                params.push(AstParameter {
                    name,
                    type_text,
                    optional: true,
                    default_value_text: default_text,
                });
            }
            _ => {}
        }
    }

    params
}

fn extract_py_param_name(node: &tree_sitter::Node, source: &str) -> String {
    // First identifier child is the parameter name
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "identifier" {
                return child.utf8_text(source.as_bytes()).unwrap_or("<unknown>").to_string();
            }
        }
    }
    "<unknown>".to_string()
}

fn extract_py_param_type(node: &tree_sitter::Node, source: &str) -> Option<String> {
    // Look for ":" followed by type expression
    let mut colon_seen = false;
    for i in 0..node.child_count() {
        let child = match node.child(i) {
            Some(c) => c,
            None => continue,
        };
        if child.kind() == ":" {
            colon_seen = true;
            continue;
        }
        if colon_seen && child.is_named() {
            return child.utf8_text(source.as_bytes()).ok().map(|s| s.to_string());
        }
    }
    None
}

fn extract_py_default_value(node: &tree_sitter::Node, source: &str) -> Option<String> {
    // Look for "=" followed by value
    let mut eq_seen = false;
    for i in 0..node.child_count() {
        let child = match node.child(i) {
            Some(c) => c,
            None => continue,
        };
        if child.kind() == "=" {
            eq_seen = true;
            continue;
        }
        if eq_seen && child.is_named() {
            return child.utf8_text(source.as_bytes()).ok().map(|s| s.to_string());
        }
    }
    None
}

fn extract_py_return_type(node: &tree_sitter::Node, source: &str) -> Option<String> {
    if let Some(ret) = node.child_by_field_name("return_type") {
        ret.utf8_text(source.as_bytes()).ok().map(|s| s.to_string())
    } else {
        None
    }
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
