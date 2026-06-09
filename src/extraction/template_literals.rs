//! ast_find_template_literals — find template/tagged template literals.
use serde::Deserialize;
use serde_json::json;

use crate::config::workspace::Workspace;
use crate::parser;
use crate::safety;
use crate::shared::errors::AstToolError;
use crate::shared::language::LanguageId;
use crate::shared::types_v2::TemplateLiteralMatch;
use crate::text::position_encoding;

#[derive(Deserialize)]
#[serde(default)]
pub struct AstFindTemplateLiteralsInput {
    pub file_path: String,
    pub tag: Option<String>,
    pub contains: Option<String>,
    pub include_untagged: bool,
    pub include_enclosing_scope: bool,
    pub max_results: usize,
}

impl Default for AstFindTemplateLiteralsInput {
    fn default() -> Self {
        Self {
            file_path: String::new(),
            tag: None,
            contains: None,
            include_untagged: true,
            include_enclosing_scope: true,
            max_results: 100,
        }
    }
}

pub fn handle(workspace: &Workspace, args: serde_json::Value) -> serde_json::Value {
    let input: AstFindTemplateLiteralsInput = match serde_json::from_value(args) {
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
    let mut templates: Vec<TemplateLiteralMatch> = Vec::new();
    collect_templates(root, &source, &mut templates, &input, input.max_results);

    let returned = templates.len();
    let truncated = returned >= input.max_results;

    json!({
        "filePath": resolved.workspace_relative,
        "templates": templates,
        "returned": returned,
        "truncated": truncated,
    })
}

fn collect_templates(
    node: tree_sitter::Node,
    source: &str,
    templates: &mut Vec<TemplateLiteralMatch>,
    input: &AstFindTemplateLiteralsInput,
    max: usize,
) {
    if templates.len() >= max {
        return;
    }
    if node.kind() == "template_string" || node.kind() == "template_literal" {
        if let Some(tm) = build_template(&node, source, input) {
            templates.push(tm);
        }
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_templates(child, source, templates, input, max);
        }
    }
}

fn build_template(
    node: &tree_sitter::Node,
    source: &str,
    input: &AstFindTemplateLiteralsInput,
) -> Option<TemplateLiteralMatch> {
    let br = node.byte_range();
    let raw_text = source[br.start..br.end].to_string();

    // Tag detection: parent is a call_expression with a function that matches
    let tag = node.parent().and_then(|p| {
        if p.kind() == "call_expression" {
            p.child_by_field_name("function")
                .map(|f| source[f.byte_range().start..f.byte_range().end].to_string())
        } else {
            None
        }
    });

    if let Some(ref req_tag) = input.tag {
        if tag.as_deref() != Some(req_tag.as_str()) {
            return None;
        }
    }

    if !input.include_untagged && tag.is_none() {
        return None;
    }

    if let Some(ref contains) = input.contains {
        if !raw_text.contains(contains.as_str()) {
            return None;
        }
    }

    // Count interpolations
    let mut interpolation_count: usize = 0;
    count_interpolations(node, &mut interpolation_count);

    let range = position_encoding::byte_range_to_range(source, br.start, br.end);

    let enclosing_scope = if input.include_enclosing_scope {
        crate::extraction::calls::find_enclosing_scope(node, source)
    } else {
        None
    };

    Some(TemplateLiteralMatch { tag, raw_text, range, interpolation_count, enclosing_scope })
}

fn count_interpolations(node: &tree_sitter::Node, count: &mut usize) {
    if node.kind() == "template_substitution" || node.kind() == "interpolation" {
        *count += 1;
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            count_interpolations(&child, count);
        }
    }
}

fn extension_to_language(path: &str) -> Option<LanguageId> {
    let ext = std::path::Path::new(path).extension().and_then(|s| s.to_str())?;
    let dotted = format!(".{}", ext);
    parser::registry::for_extension(&dotted).map(|d| d.language)
}
