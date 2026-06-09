//! ast_context_for_range — return structural context around a source range.
use serde::Deserialize;
use serde_json::json;

use crate::config::defaults::MAX_CONTEXT_BYTES;
use crate::config::workspace::Workspace;
use crate::parser;
use crate::safety;
use crate::shared::ast_node::AstNodeSummary;
use crate::shared::errors::AstToolError;
use crate::shared::language::LanguageId;
use crate::shared::position::Range;
use crate::shared::types_v2::ContextBlock;
use crate::text::{position_encoding, text_budget};

#[derive(Deserialize)]
#[serde(default)]
pub struct AstContextForRangeInput {
    pub file_path: String,
    pub range: Option<Range>,
    pub include_parents: bool,
    pub include_siblings: bool,
    pub max_parent_depth: usize,
    pub max_context_bytes: usize,
}

impl Default for AstContextForRangeInput {
    fn default() -> Self {
        Self {
            file_path: String::new(),
            range: None,
            include_parents: true,
            include_siblings: false,
            max_parent_depth: 4,
            max_context_bytes: MAX_CONTEXT_BYTES,
        }
    }
}

pub fn handle(workspace: &Workspace, args: serde_json::Value) -> serde_json::Value {
    let input: AstContextForRangeInput = match serde_json::from_value(args) {
        Ok(v) => v,
        Err(_) => return AstToolError::InvalidRange.payload(),
    };

    let range = match input.range {
        Some(r) => r,
        None => return AstToolError::InvalidRange.payload(),
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

    let (start_byte, end_byte) = match position_encoding::validate_range_in_bounds(&source, range) {
        Ok(v) => v,
        Err(e) => return e.payload(),
    };

    let root = tree.root_node();

    // Find the target node (smallest containing the range)
    let target =
        crate::context::node_at_range_helpers::find_smallest_containing(root, start_byte, end_byte);

    let target_summary = target.as_ref().map(|n| summary(n, &source, true, 12000));

    // Walk parents
    let mut parent_summaries: Vec<AstNodeSummary> = Vec::new();
    let mut parent_nodes: Vec<tree_sitter::Node> = Vec::new();
    if input.include_parents {
        let mut cursor = target.as_ref().and_then(|n| n.parent());
        let mut depth = 0;
        while let Some(p) = cursor {
            if depth >= input.max_parent_depth {
                break;
            }
            parent_summaries.push(summary(&p, &source, true, 8000));
            // We can't store the node ref long-term, so store positions for block generation
            parent_nodes.push(p);
            depth += 1;
            cursor = p.parent();
        }
    }

    // Siblings
    let siblings: Vec<AstNodeSummary> = if input.include_siblings {
        if let Some(ref t) = target {
            if let Some(parent) = t.parent() {
                sibling_summaries(&parent, &source, t.id())
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    // Build context blocks respecting budget
    let mut budget = text_budget::TextBudget::new(input.max_context_bytes);
    let mut blocks: Vec<ContextBlock> = Vec::new();

    if let Some(ref t) = target {
        add_block(
            &mut blocks,
            &mut budget,
            "target_node",
            t,
            &source,
            &resolved.workspace_relative,
        );
    }
    for (i, p) in parent_nodes.iter().enumerate() {
        let label = if i == 0 { "parent".to_string() } else { format!("parent_{}", i + 1) };
        add_block(&mut blocks, &mut budget, &label, p, &source, &resolved.workspace_relative);
    }

    let truncated = budget.exceeded;

    json!({
        "filePath": resolved.workspace_relative,
        "target": target_summary,
        "parents": parent_summaries,
        "siblings": siblings,
        "contextBlocks": blocks,
        "truncated": truncated,
    })
}

fn sibling_summaries(
    parent: &tree_sitter::Node,
    source: &str,
    skip_id: usize,
) -> Vec<AstNodeSummary> {
    let mut out = Vec::new();
    for i in 0..parent.child_count() {
        if let Some(child) = parent.child(i) {
            if child.id() != skip_id {
                out.push(summary(&child, source, false, 0));
            }
        }
    }
    out
}

fn summary(
    node: &tree_sitter::Node,
    source: &str,
    include_text: bool,
    max_text: usize,
) -> AstNodeSummary {
    let br = node.byte_range();
    let text = if include_text && max_text > 0 {
        let raw = &source[br.start..br.end];
        let (t, _) = text_budget::truncate_text(raw, max_text);
        Some(t.to_string())
    } else {
        None
    };
    let pos_range = position_encoding::byte_range_to_range(source, br.start, br.end);
    AstNodeSummary {
        id: None,
        kind: node.kind().to_string(),
        name: crate::context::node_at_range_helpers::extract_name(node, source),
        range: pos_range,
        byte_range: Some((br.start, br.end)),
        text,
        children: None,
    }
}

fn add_block(
    blocks: &mut Vec<ContextBlock>,
    budget: &mut text_budget::TextBudget,
    label: &str,
    node: &tree_sitter::Node,
    source: &str,
    file_path: &str,
) {
    let br = node.byte_range();
    let pos_range = position_encoding::byte_range_to_range(source, br.start, br.end);
    let raw = &source[br.start..br.end];
    let rem = budget.remaining();
    let (text, truncated) = text_budget::truncate_text(raw, rem);
    budget.try_spend(text.len());

    blocks.push(ContextBlock {
        label: label.to_string(),
        kind: node.kind().to_string(),
        file_path: file_path.to_string(),
        range: pos_range,
        text: text.to_string(),
        truncated: truncated || budget.exceeded,
    });
}

fn extension_to_language(path: &str) -> Option<LanguageId> {
    let ext = std::path::Path::new(path).extension().and_then(|s| s.to_str())?;
    let dotted = format!(".{}", ext);
    parser::registry::for_extension(&dotted).map(|d| d.language)
}
