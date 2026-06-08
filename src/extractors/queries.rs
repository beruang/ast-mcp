use serde::Serialize;
use tree_sitter::{Query, Tree};

use crate::parser::positions;
use crate::shared::position::Range;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstQueryMatch {
    pub pattern_index: Option<u32>,
    pub captures: Vec<Capture>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Capture {
    pub name: String,
    pub kind: String,
    pub range: Range,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

pub struct QueryOptions {
    pub max_results: usize,
    pub include_node_text: bool,
    pub max_text_bytes: usize,
}

pub fn run_query(
    query: &Query,
    tree: &Tree,
    source: &str,
    opts: QueryOptions,
) -> Vec<AstQueryMatch> {
    let mut matches: Vec<AstQueryMatch> = Vec::new();
    let mut cursor = tree_sitter::QueryCursor::new();
    for m in cursor.matches(query, tree.root_node(), source.as_bytes()) {
        if matches.len() >= opts.max_results {
            break;
        }
        let mut captures: Vec<Capture> = m
            .captures
            .iter()
            .map(|c| {
                let node = c.node;
                let text = if opts.include_node_text {
                    let raw = node.utf8_text(source.as_bytes()).unwrap_or("");
                    let t = if raw.len() > opts.max_text_bytes {
                        raw[..opts.max_text_bytes].to_string()
                    } else {
                        raw.to_string()
                    };
                    Some(t)
                } else {
                    None
                };
                Capture {
                    name: query.capture_names()[c.index as usize].to_string(),
                    kind: node.kind().to_string(),
                    range: Range {
                        start: positions::ts_point_to_position(node.start_position(), source),
                        end: positions::ts_point_to_position(node.end_position(), source),
                    },
                    text,
                }
            })
            .collect();
        captures.sort_by(|a, b| a.name.cmp(&b.name));
        matches.push(AstQueryMatch {
            pattern_index: Some(m.pattern_index as u32),
            captures,
        });
    }
    matches
}
