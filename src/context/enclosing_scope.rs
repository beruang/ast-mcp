//! ast_enclosing_scope — return the scope chain enclosing a position.
use serde::Deserialize;
use serde_json::json;

use crate::config::workspace::Workspace;
use crate::parser;
use crate::safety;
use crate::shared::errors::AstToolError;
use crate::shared::language::LanguageId;
use crate::shared::position::{Position, Range};
use crate::shared::types_v2::{ScopeKind, ScopeSummary};
use crate::text::position_encoding;

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct AstEnclosingScopeInput {
    pub file_path: String,
    pub position: Option<Position>,
    pub include_block_scopes: bool,
}

pub fn handle(workspace: &Workspace, args: serde_json::Value) -> serde_json::Value {
    let input: AstEnclosingScopeInput = match serde_json::from_value(args) {
        Ok(v) => v,
        Err(_) => return AstToolError::InvalidPosition("invalid input".into()).payload(),
    };

    let pos = match input.position {
        Some(p) => p,
        None => {
            return AstToolError::InvalidPosition("position is required".into()).payload();
        }
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

    let byte_offset = match position_encoding::position_to_byte(&source, pos) {
        Ok(b) => b,
        Err(e) => return e.payload(),
    };

    let root = tree.root_node();

    // Find the leaf node at the byte offset, then walk ancestors keeping scopes.
    let leaf = find_leaf_at(root, byte_offset);
    let scopes = if let Some(lf) = leaf {
        ancestor_scopes(lf, &source, input.include_block_scopes)
    } else {
        vec![]
    };

    json!({
        "filePath": resolved.workspace_relative,
        "position": { "line": pos.line, "character": pos.character },
        "scopes": scopes,
    })
}

fn find_leaf_at<'t>(start: tree_sitter::Node<'t>, byte: usize) -> Option<tree_sitter::Node<'t>> {
    let mut cursor = start;
    loop {
        let mut next: Option<tree_sitter::Node<'t>> = None;
        for i in 0..cursor.child_count() {
            if let Some(child) = cursor.child(i) {
                let br = child.byte_range();
                if br.start <= byte && br.end >= byte {
                    next = Some(child);
                    break;
                }
            }
        }
        match next {
            Some(n) => cursor = n,
            None => return Some(cursor),
        }
    }
}

pub(crate) fn ancestor_scopes(
    leaf: tree_sitter::Node,
    source: &str,
    include_blocks: bool,
) -> Vec<ScopeSummary> {
    let mut scopes: Vec<ScopeSummary> = Vec::new();
    let mut cursor = Some(leaf);
    while let Some(node) = cursor {
        if let Some(scope) = classify_scope(&node, source, include_blocks) {
            scopes.push(scope);
        }
        cursor = node.parent();
    }
    scopes.reverse(); // outermost first
    scopes
}

fn classify_scope(
    node: &tree_sitter::Node,
    source: &str,
    include_blocks: bool,
) -> Option<ScopeSummary> {
    let kind_str = node.kind();
    let scope_kind = match kind_str {
        "program" | "module" => ScopeKind::Module,
        "class_declaration" | "class_definition" => ScopeKind::Class,
        "interface_declaration" => ScopeKind::Interface,
        "function_declaration" | "function_definition" => ScopeKind::Function,
        "method_definition" => ScopeKind::Method,
        "constructor" | "constructor_declaration" => ScopeKind::Constructor,
        "arrow_function" => ScopeKind::ArrowFunction,
        "lambda" | "lambda_expression" => ScopeKind::Lambda,
        "block" | "statement_block" | "_block" | "block_statement" if include_blocks => {
            ScopeKind::Block
        }
        "impl_item" | "trait_item" => ScopeKind::Class,
        _ => return None,
    };

    let br = node.byte_range();
    let range = position_encoding::byte_range_to_range(source, br.start, br.end);

    let name = extract_scope_name(node, source);

    // selection_range = range of the body (excluding signature for functions)
    let selection_range = node_body_range(node, source);

    Some(ScopeSummary {
        kind: scope_kind,
        name,
        node_kind: kind_str.to_string(),
        range,
        selection_range,
    })
}

fn extract_scope_name(node: &tree_sitter::Node, source: &str) -> Option<String> {
    if let Some(name_node) = node.child_by_field_name("name") {
        let br = name_node.byte_range();
        return Some(source[br.start..br.end].to_string());
    }
    // For Python class/func: first "identifier" child (not "def"/"class")
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "identifier" {
                let br = child.byte_range();
                return Some(source[br.start..br.end].to_string());
            }
        }
    }
    None
}

fn node_body_range(node: &tree_sitter::Node, source: &str) -> Option<Range> {
    if let Some(body) = node.child_by_field_name("body") {
        let br = body.byte_range();
        Some(position_encoding::byte_range_to_range(source, br.start, br.end))
    } else {
        // For nodes without explicit body field, use the node itself
        None
    }
}

fn extension_to_language(path: &str) -> Option<LanguageId> {
    let ext = std::path::Path::new(path).extension().and_then(|s| s.to_str())?;
    let dotted = format!(".{}", ext);
    parser::registry::for_extension(&dotted).map(|d| d.language)
}
