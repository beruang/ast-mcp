//! ast_find_member_access — find member/property access expressions.
use serde::Deserialize;
use serde_json::json;

use crate::config::workspace::Workspace;
use crate::parser;
use crate::safety;
use crate::shared::errors::AstToolError;
use crate::shared::language::LanguageId;
use crate::shared::types_v2::MemberAccess;
use crate::text::position_encoding;

#[derive(Deserialize)]
#[serde(default)]
pub struct AstFindMemberAccessInput {
    pub file_path: String,
    pub property: Option<String>,
    pub object_contains: Option<String>,
    pub full_text_contains: Option<String>,
    pub include_enclosing_scope: bool,
    pub max_results: usize,
}

impl Default for AstFindMemberAccessInput {
    fn default() -> Self {
        Self {
            file_path: String::new(),
            property: None,
            object_contains: None,
            full_text_contains: None,
            include_enclosing_scope: true,
            max_results: 200,
        }
    }
}

pub fn handle(workspace: &Workspace, args: serde_json::Value) -> serde_json::Value {
    let input: AstFindMemberAccessInput = match serde_json::from_value(args) {
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
    let mut members: Vec<MemberAccess> = Vec::new();
    collect_members(root, &source, &mut members, &input, input.max_results);

    let returned = members.len();
    let truncated = returned >= input.max_results;

    json!({
        "filePath": resolved.workspace_relative,
        "members": members,
        "returned": returned,
        "truncated": truncated,
    })
}

fn collect_members(
    node: tree_sitter::Node,
    source: &str,
    members: &mut Vec<MemberAccess>,
    input: &AstFindMemberAccessInput,
    max: usize,
) {
    if members.len() >= max {
        return;
    }
    if is_member_node(node.kind()) {
        if let Some(ma) = build_member(&node, source, input) {
            members.push(ma);
        }
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_members(child, source, members, input, max);
        }
    }
}

fn is_member_node(kind: &str) -> bool {
    matches!(
        kind,
        "member_expression"
            | "attribute"
            | "field_expression"
            | "selector_expression"
            | "subscript_expression"
    )
}

fn build_member(
    node: &tree_sitter::Node,
    source: &str,
    input: &AstFindMemberAccessInput,
) -> Option<MemberAccess> {
    let br = node.byte_range();
    let full_text = source[br.start..br.end].to_string();

    let object_text = node
        .child_by_field_name("object")
        .or_else(|| node.child_by_field_name("operand"))
        .map(|o| source[o.byte_range().start..o.byte_range().end].to_string())
        .unwrap_or_default();

    let property = node
        .child_by_field_name("property")
        .or_else(|| node.child_by_field_name("field"))
        .or_else(|| {
            // Fallback: look for last child that is a valid property identifier
            node.child(node.child_count().saturating_sub(1)).filter(|c| {
                matches!(c.kind(), "property_identifier" | "identifier" | "field_identifier")
            })
        })
        .map(|p| source[p.byte_range().start..p.byte_range().end].to_string())
        .unwrap_or_default();

    if let Some(ref prop) = input.property {
        if property != *prop {
            return None;
        }
    }
    if let Some(ref contains) = input.object_contains {
        if !object_text.contains(contains.as_str()) {
            return None;
        }
    }
    if let Some(ref contains) = input.full_text_contains {
        if !full_text.contains(contains.as_str()) {
            return None;
        }
    }

    let range = position_encoding::byte_range_to_range(source, br.start, br.end);

    let enclosing_scope = if input.include_enclosing_scope {
        crate::extraction::calls::find_enclosing_scope(node, source)
    } else {
        None
    };

    Some(MemberAccess { object_text, property, full_text, range, enclosing_scope })
}

fn extension_to_language(path: &str) -> Option<LanguageId> {
    let ext = std::path::Path::new(path).extension().and_then(|s| s.to_str())?;
    let dotted = format!(".{}", ext);
    parser::registry::for_extension(&dotted).map(|d| d.language)
}
