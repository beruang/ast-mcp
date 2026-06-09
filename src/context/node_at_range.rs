//! ast_node_at_range — find the AST node at or containing a source range.
use serde::Deserialize;
use serde_json::json;

use crate::config::workspace::Workspace;
use crate::parser;
use crate::safety;
use crate::shared::ast_node::{AstNodeSummary, NodeAtRangeMode};
use crate::shared::errors::AstToolError;
use crate::shared::language::LanguageId;
use crate::shared::position::Range;
use crate::text::position_encoding;

#[derive(Deserialize)]
#[serde(default)]
pub struct AstNodeAtRangeInput {
    pub file_path: String,
    pub range: Option<Range>,
    pub mode: NodeAtRangeMode,
    pub include_text: bool,
    pub max_text_bytes: usize,
}

impl Default for AstNodeAtRangeInput {
    fn default() -> Self {
        Self {
            file_path: String::new(),
            range: None,
            mode: NodeAtRangeMode::SmallestContaining,
            include_text: true,
            max_text_bytes: 12000,
        }
    }
}

pub fn handle(workspace: &Workspace, args: serde_json::Value) -> serde_json::Value {
    let input: AstNodeAtRangeInput = match serde_json::from_value(args) {
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
    let target = find_node(root, start_byte, end_byte, &input.mode);

    let node = target.map(|n| node_summary(&n, &source, input.include_text, input.max_text_bytes));

    let ancestors = if node.is_some() {
        build_ancestors(root, range, &source, input.include_text, input.max_text_bytes)
    } else {
        vec![]
    };

    let mode_str = match input.mode {
        NodeAtRangeMode::Exact => "exact",
        NodeAtRangeMode::SmallestContaining => "smallest_containing",
        NodeAtRangeMode::LargestContained => "largest_contained",
    };

    json!({
        "filePath": resolved.workspace_relative,
        "range": {
            "start": { "line": range.start.line, "character": range.start.character },
            "end": { "line": range.end.line, "character": range.end.character }
        },
        "node": node,
        "ancestors": ancestors,
        "matchedMode": mode_str,
    })
}

// ── Node search ──

fn find_node<'t>(
    start: tree_sitter::Node<'t>,
    target_start: usize,
    target_end: usize,
    mode: &NodeAtRangeMode,
) -> Option<tree_sitter::Node<'t>> {
    match mode {
        NodeAtRangeMode::Exact => find_exact(start, target_start, target_end),
        NodeAtRangeMode::SmallestContaining => {
            find_smallest_containing(start, target_start, target_end)
        }
        NodeAtRangeMode::LargestContained => {
            find_largest_contained(start, target_start, target_end)
        }
    }
}

fn find_exact<'t>(
    start: tree_sitter::Node<'t>,
    target_start: usize,
    target_end: usize,
) -> Option<tree_sitter::Node<'t>> {
    let mut best: Option<tree_sitter::Node<'t>> = None;
    walk_for_exact(start, target_start, target_end, &mut best);
    best
}

fn walk_for_exact<'t>(
    node: tree_sitter::Node<'t>,
    ts: usize,
    te: usize,
    best: &mut Option<tree_sitter::Node<'t>>,
) {
    if node.byte_range().start == ts && node.byte_range().end == te {
        *best = Some(node);
        return;
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.byte_range().start <= ts && child.byte_range().end >= te {
                walk_for_exact(child, ts, te, best);
                if best.is_some() {
                    return;
                }
            }
        }
    }
}

fn find_smallest_containing<'t>(
    start: tree_sitter::Node<'t>,
    target_start: usize,
    target_end: usize,
) -> Option<tree_sitter::Node<'t>> {
    let mut cursor = start;
    loop {
        let mut found = false;
        for i in 0..cursor.child_count() {
            if let Some(child) = cursor.child(i) {
                let br = child.byte_range();
                if br.start <= target_start && br.end >= target_end {
                    cursor = child;
                    found = true;
                    break;
                }
            }
        }
        if !found {
            return Some(cursor);
        }
    }
}

fn find_largest_contained<'t>(
    start: tree_sitter::Node<'t>,
    target_start: usize,
    target_end: usize,
) -> Option<tree_sitter::Node<'t>> {
    let mut best: Option<tree_sitter::Node<'t>> = None;
    let mut best_size: usize = 0;
    walk_largest(start, target_start, target_end, &mut best, &mut best_size);
    best
}

fn walk_largest<'t>(
    node: tree_sitter::Node<'t>,
    ts: usize,
    te: usize,
    best: &mut Option<tree_sitter::Node<'t>>,
    best_size: &mut usize,
) {
    let br = node.byte_range();
    if br.start >= ts && br.end <= te {
        let size = br.end - br.start;
        if size > *best_size {
            *best = Some(node);
            *best_size = size;
        }
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            walk_largest(child, ts, te, best, best_size);
        }
    }
}

// ── Helpers ──

fn node_summary(
    node: &tree_sitter::Node,
    source: &str,
    include_text: bool,
    max_text_bytes: usize,
) -> AstNodeSummary {
    let br = node.byte_range();
    let text = if include_text {
        let raw = &source[br.start..br.end];
        if raw.len() <= max_text_bytes {
            Some(raw.to_string())
        } else {
            let (trunc, _) = crate::text::text_budget::truncate_text(raw, max_text_bytes);
            Some(trunc.to_string())
        }
    } else {
        None
    };

    let pos_range = position_encoding::byte_range_to_range(source, br.start, br.end);

    AstNodeSummary {
        id: None,
        kind: node.kind().to_string(),
        name: extract_name(node, source),
        range: pos_range,
        byte_range: Some((br.start, br.end)),
        text,
        children: None,
    }
}

fn extract_name(node: &tree_sitter::Node, source: &str) -> Option<String> {
    if let Some(name_node) = node.child_by_field_name("name") {
        let br = name_node.byte_range();
        return Some(source[br.start..br.end].to_string());
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "identifier" || child.kind() == "property_identifier" {
                let br = child.byte_range();
                return Some(source[br.start..br.end].to_string());
            }
        }
    }
    None
}

fn build_ancestors(
    root: tree_sitter::Node,
    target_range: Range,
    source: &str,
    include_text: bool,
    max_text_bytes: usize,
) -> Vec<AstNodeSummary> {
    let mut ancestors = Vec::new();
    let mut cursor = root;
    let start_byte = position_encoding::position_to_byte(source, target_range.start).unwrap_or(0);
    let end_byte = position_encoding::position_to_byte(source, target_range.end).unwrap_or(0);

    loop {
        ancestors.push(node_summary(&cursor, source, include_text, max_text_bytes));
        let mut next: Option<tree_sitter::Node> = None;
        for i in 0..cursor.child_count() {
            if let Some(child) = cursor.child(i) {
                let br = child.byte_range();
                if br.start <= start_byte && br.end >= end_byte {
                    next = Some(child);
                    break;
                }
            }
        }
        match next {
            Some(n) => cursor = n,
            None => break,
        }
    }
    ancestors
}

fn extension_to_language(path: &str) -> Option<LanguageId> {
    let ext = std::path::Path::new(path).extension().and_then(|s| s.to_str())?;
    let dotted = format!(".{}", ext);
    parser::registry::for_extension(&dotted).map(|d| d.language)
}
