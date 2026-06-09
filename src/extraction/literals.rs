//! ast_find_literals — find literals (string, number, boolean, null, regex) in a file.
use serde::Deserialize;
use serde_json::json;

use crate::config::workspace::Workspace;
use crate::parser;
use crate::safety;
use crate::shared::errors::AstToolError;
use crate::shared::language::LanguageId;
use crate::shared::types_v2::LiteralMatch;
use crate::text::position_encoding;

#[derive(Deserialize)]
#[serde(default)]
pub struct AstFindLiteralsInput {
    pub file_path: String,
    pub literal_kind: Option<LiteralKind>,
    pub contains: Option<String>,
    pub exact: Option<String>,
    pub include_enclosing_scope: bool,
    pub max_results: usize,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiteralKind {
    String,
    Number,
    Boolean,
    Null,
    Regex,
    Unknown,
}

impl Default for AstFindLiteralsInput {
    fn default() -> Self {
        Self {
            file_path: String::new(),
            literal_kind: None,
            contains: None,
            exact: None,
            include_enclosing_scope: true,
            max_results: 200,
        }
    }
}

pub fn handle(workspace: &Workspace, args: serde_json::Value) -> serde_json::Value {
    let input: AstFindLiteralsInput = match serde_json::from_value(args) {
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
    let mut literals: Vec<LiteralMatch> = Vec::new();
    collect_literals(root, &source, &mut literals, &input, input.max_results);

    let returned = literals.len();
    let truncated = returned >= input.max_results;

    json!({
        "filePath": resolved.workspace_relative,
        "literals": literals,
        "returned": returned,
        "truncated": truncated,
    })
}

fn collect_literals(
    node: tree_sitter::Node,
    source: &str,
    literals: &mut Vec<LiteralMatch>,
    input: &AstFindLiteralsInput,
    max: usize,
) {
    if literals.len() >= max {
        return;
    }
    if let Some(lm) = build_literal(&node, source, input) {
        literals.push(lm);
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_literals(child, source, literals, input, max);
        }
    }
}

fn build_literal(
    node: &tree_sitter::Node,
    source: &str,
    input: &AstFindLiteralsInput,
) -> Option<LiteralMatch> {
    let (kind, raw_text) = match node.kind() {
        "string" | "string_literal" => {
            ("string", source[node.byte_range().start..node.byte_range().end].to_string())
        }
        "number" | "integer" | "float" => {
            ("number", source[node.byte_range().start..node.byte_range().end].to_string())
        }
        "true" | "false" => ("boolean", node.kind().to_string()),
        "null" | "none" | "nil" => ("null", node.kind().to_string()),
        "regex" | "regex_pattern" => {
            ("regex", source[node.byte_range().start..node.byte_range().end].to_string())
        }
        _ => return None,
    };

    if let Some(ref lk) = input.literal_kind {
        let expected = match lk {
            LiteralKind::String => "string",
            LiteralKind::Number => "number",
            LiteralKind::Boolean => "boolean",
            LiteralKind::Null => "null",
            LiteralKind::Regex => "regex",
            LiteralKind::Unknown => kind,
        };
        if kind != expected {
            return None;
        }
    }

    // Extract value text (strip quotes for strings)
    let value_text = if kind == "string" && raw_text.len() >= 2 {
        let inner = &raw_text[1..raw_text.len() - 1];
        Some(inner.to_string())
    } else {
        None
    };

    // Filter by contains/exact
    let search_text = value_text.as_deref().unwrap_or(&raw_text);
    if let Some(ref contains) = input.contains {
        if !search_text.contains(contains.as_str()) {
            return None;
        }
    }
    if let Some(ref exact) = input.exact {
        if search_text != exact.as_str() {
            return None;
        }
    }

    let br = node.byte_range();
    let range = position_encoding::byte_range_to_range(source, br.start, br.end);

    let enclosing_scope = if input.include_enclosing_scope {
        crate::extraction::calls::find_enclosing_scope(node, source)
    } else {
        None
    };

    Some(LiteralMatch { kind: kind.to_string(), raw_text, value_text, range, enclosing_scope })
}

fn extension_to_language(path: &str) -> Option<LanguageId> {
    let ext = std::path::Path::new(path).extension().and_then(|s| s.to_str())?;
    let dotted = format!(".{}", ext);
    parser::registry::for_extension(&dotted).map(|d| d.language)
}
