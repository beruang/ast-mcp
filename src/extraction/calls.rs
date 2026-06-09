//! ast_find_calls — find call expressions in a file with optional callee filters.
use serde::Deserialize;
use serde_json::json;

use crate::config::workspace::Workspace;
use crate::parser;
use crate::safety;
use crate::shared::errors::AstToolError;
use crate::shared::language::LanguageId;
use crate::shared::types_v2::{CallExpression, ScopeKind, ScopeSummary};
use crate::text::position_encoding;

#[derive(Deserialize)]
#[serde(default)]
pub struct AstFindCallsInput {
    pub file_path: String,
    pub callee: Option<String>,
    pub callee_contains: Option<String>,
    pub include_arguments: bool,
    pub include_enclosing_scope: bool,
    pub max_results: usize,
}

impl Default for AstFindCallsInput {
    fn default() -> Self {
        Self {
            file_path: String::new(),
            callee: None,
            callee_contains: None,
            include_arguments: true,
            include_enclosing_scope: true,
            max_results: 200,
        }
    }
}

pub fn handle(workspace: &Workspace, args: serde_json::Value) -> serde_json::Value {
    let input: AstFindCallsInput = match serde_json::from_value(args) {
        Ok(v) => v,
        Err(e) => return AstToolError::InvalidPosition(e.to_string()).payload(),
    };

    let resolved = match safety::paths::resolve_file(workspace, &input.file_path) {
        Ok(r) => r,
        Err(e) => return e.payload(),
    };
    let meta = match std::fs::metadata(&resolved.absolute) {
        Ok(m) => m,
        Err(e) => return AstToolError::FileNotFound(e.to_string()).payload(),
    };
    if let Err(e) = safety::paths::ensure_under_size(meta.len()) {
        return e.payload();
    }
    let source = match std::fs::read_to_string(&resolved.absolute) {
        Ok(s) => s,
        Err(e) => return AstToolError::FileNotFound(e.to_string()).payload(),
    };
    let lang = match extension_to_language(&resolved.workspace_relative) {
        Some(l) => l,
        None => {
            let ext = std::path::Path::new(&resolved.workspace_relative)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            return AstToolError::UnsupportedLanguage(ext.to_string()).payload();
        }
    };

    let (tree, _status) = match parser::parse::parse_source(&source, lang) {
        Ok(t) => t,
        Err(e) => return e.payload(),
    };

    let root = tree.root_node();
    let mut calls: Vec<CallExpression> = Vec::new();
    collect_calls(root, &source, &mut calls, &input, input.max_results);

    let returned = calls.len();
    let truncated = returned >= input.max_results;

    json!({
        "filePath": resolved.workspace_relative,
        "calls": calls,
        "returned": returned,
        "truncated": truncated,
    })
}

pub(crate) fn find_enclosing_scope(node: &tree_sitter::Node, source: &str) -> Option<ScopeSummary> {
    let mut cursor = node.parent();
    while let Some(p) = cursor {
        let k = p.kind();
        if matches!(
            k,
            "function_declaration"
                | "function_definition"
                | "method_definition"
                | "arrow_function"
                | "class_declaration"
                | "class_definition"
                | "lambda"
                | "program"
                | "module"
        ) {
            let br = p.byte_range();
            let range = position_encoding::byte_range_to_range(source, br.start, br.end);
            let name = crate::context::node_at_range_helpers::extract_name(&p, source);
            let kind = match k {
                "program" | "module" => ScopeKind::Module,
                "class_declaration" | "class_definition" => ScopeKind::Class,
                "method_definition" => ScopeKind::Method,
                "arrow_function" => ScopeKind::ArrowFunction,
                "lambda" => ScopeKind::Lambda,
                _ => ScopeKind::Function,
            };
            return Some(ScopeSummary {
                kind,
                name,
                node_kind: k.to_string(),
                range,
                selection_range: None,
            });
        }
        cursor = p.parent();
    }
    None
}

fn collect_calls(
    node: tree_sitter::Node,
    source: &str,
    calls: &mut Vec<CallExpression>,
    input: &AstFindCallsInput,
    max: usize,
) {
    if calls.len() >= max {
        return;
    }
    if is_call_node(node.kind()) {
        if let Some(ce) = build_call(&node, source, input) {
            calls.push(ce);
        }
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_calls(child, source, calls, input, max);
        }
    }
}

fn is_call_node(kind: &str) -> bool {
    matches!(kind, "call_expression" | "new_expression" | "method_invocation")
}

fn build_call(
    node: &tree_sitter::Node,
    source: &str,
    input: &AstFindCallsInput,
) -> Option<CallExpression> {
    let callee_text = extract_callee(node, source);

    if let Some(ref exact) = input.callee {
        if callee_text.as_deref() != Some(exact.as_str()) {
            return None;
        }
    }
    if let Some(ref contains) = input.callee_contains {
        match &callee_text {
            Some(ct) if ct.contains(contains.as_str()) => {}
            _ => return None,
        }
    }

    let arguments_text: Vec<String> =
        if input.include_arguments { extract_arguments(node, source) } else { vec![] };

    let br = node.byte_range();
    let range = position_encoding::byte_range_to_range(source, br.start, br.end);

    let enclosing_scope =
        if input.include_enclosing_scope { find_enclosing_scope(node, source) } else { None };

    Some(CallExpression {
        callee_text: callee_text.unwrap_or_default(),
        arguments_text,
        range,
        enclosing_scope,
    })
}

fn extract_callee(node: &tree_sitter::Node, source: &str) -> Option<String> {
    if let Some(func) = node.child_by_field_name("function") {
        let br = func.byte_range();
        return Some(source[br.start..br.end].to_string());
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.is_named() {
                let br = child.byte_range();
                return Some(source[br.start..br.end].to_string());
            }
        }
    }
    None
}

fn extract_arguments(node: &tree_sitter::Node, source: &str) -> Vec<String> {
    if let Some(args) = node.child_by_field_name("arguments") {
        let mut out = Vec::new();
        for i in 0..args.child_count() {
            if let Some(arg) = args.child(i) {
                if arg.is_named() {
                    let br = arg.byte_range();
                    out.push(source[br.start..br.end].to_string());
                }
            }
        }
        return out;
    }
    vec![]
}

fn extension_to_language(path: &str) -> Option<LanguageId> {
    let ext = std::path::Path::new(path).extension().and_then(|s| s.to_str())?;
    let dotted = format!(".{}", ext);
    parser::registry::for_extension(&dotted).map(|d| d.language)
}
