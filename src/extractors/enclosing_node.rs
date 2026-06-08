use serde::{Deserialize, Serialize};
use tree_sitter::Tree;

use crate::parser::positions::ts_point_to_position;
use crate::shared::errors::AstToolError;
use crate::shared::position::{Position, Range};

/// Options for the enclosing-node search.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EnclosingOptions {
    /// Optional whitelist of node kinds. When non-empty, only ancestors
    /// whose `kind()` matches one of these entries are included.
    pub kinds: Option<Vec<String>>,
    /// When true, the source text of each ancestor is included.
    pub include_source_text: bool,
    /// Maximum number of ancestors to return (default 64, capped at 64).
    pub max_ancestors: usize,
}

impl Default for EnclosingOptions {
    fn default() -> Self {
        EnclosingOptions {
            kinds: None,
            include_source_text: false,
            max_ancestors: 64,
        }
    }
}

/// A summarised ancestor node in the enclosing-node chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeSummary {
    pub kind: String,
    pub name: Option<String>,
    pub start_byte: usize,
    pub end_byte: usize,
    pub range: Range,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_text: Option<String>,
}

/// Full result of an enclosing-node query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnclosingResult {
    pub file_path: String,
    pub language: String,
    pub position: Position,
    pub ancestors: Vec<NodeSummary>,
    pub truncated: bool,
}

/// Find the enclosing node chain at a given byte offset.
///
/// Uses `descendant_for_byte_range` to locate the deepest node covering the
/// position, then walks the parent chain upward. The resulting `ancestors`
/// list is ordered outermost-first (root nearest the front).
pub fn enclosing_node(
    tree: &Tree,
    source: &str,
    byte_offset: usize,
    opts: &EnclosingOptions,
) -> Result<Vec<NodeSummary>, AstToolError> {
    let root = tree.root_node();
    let byte_offset = byte_offset.min(source.len());

    // Find the deepest node that contains the byte offset.
    let deepest = root
        .descendant_for_byte_range(byte_offset, byte_offset)
        .unwrap_or(root);

    let max_ancestors = opts.max_ancestors.min(64);

    // Walk parent chain upward collecting summaries.
    let mut summaries: Vec<NodeSummary> = Vec::new();
    let mut current = deepest;
    loop {
        if summaries.len() >= max_ancestors {
            break;
        }

        // Apply kind filter if specified.
        let include = if let Some(ref kinds) = opts.kinds {
            if kinds.is_empty() {
                true
            } else {
                kinds.iter().any(|k| k == current.kind())
            }
        } else {
            true
        };

        if include {
            summaries.push(node_to_summary(&current, source, opts.include_source_text));
        }

        // Move to parent.
        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            break;
        }
    }

    // Reverse so ancestors are outermost-first.
    summaries.reverse();

    Ok(summaries)
}

/// Convert a tree-sitter node into a `NodeSummary`.
fn node_to_summary(node: &tree_sitter::Node, source: &str, include_text: bool) -> NodeSummary {
    let name = extract_name(node, source);

    let start_pos = ts_point_to_position(node.start_position(), source);
    let end_pos = ts_point_to_position(node.end_position(), source);

    let source_text = if include_text {
        let raw = node.utf8_text(source.as_bytes()).unwrap_or("");
        let capped = if raw.len() > crate::safety::limits::MAX_TEXT_BYTES {
            let mut bound = crate::safety::limits::MAX_TEXT_BYTES;
            while bound > 0 && !raw.is_char_boundary(bound) {
                bound -= 1;
            }
            raw[..bound].to_string()
        } else {
            raw.to_string()
        };
        Some(capped)
    } else {
        None
    };

    NodeSummary {
        kind: node.kind().to_string(),
        name,
        start_byte: node.start_byte(),
        end_byte: node.end_byte(),
        range: Range {
            start: start_pos,
            end: end_pos,
        },
        source_text,
    }
}

/// Extract a human-readable name from a node by looking for an
/// identifier-like child.
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
