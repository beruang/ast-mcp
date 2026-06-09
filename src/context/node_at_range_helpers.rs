//! Re-exported helpers from node_at_range, used by context_for_range and context_pack.
use crate::text::position_encoding;

/// Find the smallest node whose byte range fully contains [ts, te).
pub fn find_smallest_containing<'t>(
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

/// Extract a human-readable name from a tree-sitter node.
pub fn extract_name(node: &tree_sitter::Node, source: &str) -> Option<String> {
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

/// Build a summary for a node.
pub fn make_summary(
    node: &tree_sitter::Node,
    source: &str,
    include_text: bool,
    max_text: usize,
) -> crate::shared::ast_node::AstNodeSummary {
    let br = node.byte_range();
    let text = if include_text && max_text > 0 {
        let raw = &source[br.start..br.end];
        let (t, _) = crate::text::text_budget::truncate_text(raw, max_text);
        Some(t.to_string())
    } else {
        None
    };
    let pos_range = position_encoding::byte_range_to_range(source, br.start, br.end);
    crate::shared::ast_node::AstNodeSummary {
        id: None,
        kind: node.kind().to_string(),
        name: extract_name(node, source),
        range: pos_range,
        byte_range: Some((br.start, br.end)),
        text,
        children: None,
    }
}
