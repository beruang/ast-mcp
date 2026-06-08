use serde::{Deserialize, Serialize};
use tree_sitter::Tree;

use crate::parser::positions::ts_point_to_position;
use crate::shared::position::Range;

/// A single top-level node extracted from a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopLevelNode {
    pub kind: String,
    pub name: Option<String>,
    pub range: Range,
}

/// Extract all top-level named nodes from a parse tree.
///
/// Iterates over the root node's named children and builds a `TopLevelNode`
/// for each, extracting the name from the first identifier-like child if
/// present.
pub fn top_level_nodes(tree: &Tree, source: &str) -> Vec<TopLevelNode> {
    let mut nodes: Vec<TopLevelNode> = Vec::new();
    let root = tree.root_node();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor) {
        if nodes.len() >= crate::safety::limits::MAX_NODES {
            break;
        }
        if !child.is_named() {
            continue;
        }
        let kind = child.kind().to_string();
        let name = extract_name(&child, source);
        let start_pos = ts_point_to_position(child.start_position(), source);
        let end_pos = ts_point_to_position(child.end_position(), source);

        nodes.push(TopLevelNode {
            kind,
            name,
            range: Range {
                start: start_pos,
                end: end_pos,
            },
        });
    }

    nodes
}

/// Extract a human-readable name from a node by looking for an identifier-like
/// child.
fn extract_name(node: &tree_sitter::Node, source: &str) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.is_named() {
            match child.kind() {
                "identifier"
                | "property_identifier"
                | "shorthand_property_identifier"
                | "statement_identifier" => {
                    return child
                        .utf8_text(source.as_bytes())
                        .ok()
                        .map(|s| s.to_string());
                }
                _ => {
                    // Recurse one level for unwrapped nodes.
                    if let Some(name) = extract_name(&child, source) {
                        return Some(name);
                    }
                }
            }
        }
    }
    None
}
