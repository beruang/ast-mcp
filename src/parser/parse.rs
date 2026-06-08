use std::time::Instant;

use tree_sitter::{Parser, Tree};

use crate::parser::registry;
use crate::shared::errors::AstToolError;
use crate::shared::language::LanguageId;

#[derive(Debug)]
pub struct ParseStatus {
    pub has_syntax_error: bool,
    pub root_kind: String,
    pub node_count: usize,
    pub parse_time_ms: u64,
}

pub fn parse_source(source: &str, lang: LanguageId) -> Result<(Tree, ParseStatus), AstToolError> {
    let def = registry::for_language(lang).ok_or_else(|| {
        AstToolError::ParserUnavailable(format!("no parser registered for {}", lang.as_str()))
    })?;

    let mut parser = Parser::new();
    parser
        .set_language((def.tree_sitter_language)())
        .map_err(|e| AstToolError::ParserUnavailable(e.to_string()))?;

    let start = Instant::now();
    let tree = parser
        .parse(source, None)
        .ok_or_else(|| AstToolError::ParseFailed("tree-sitter returned None".into()))?;
    let parse_time_ms = start.elapsed().as_millis() as u64;

    let root = tree.root_node();
    let has_syntax_error = root.has_error();
    let root_kind = root.kind().to_string();
    let node_count = count_nodes(&root);

    Ok((tree, ParseStatus { has_syntax_error, root_kind, node_count, parse_time_ms }))
}

fn count_nodes(n: &tree_sitter::Node) -> usize {
    let mut c = 1;
    let mut cursor = n.walk();
    for child in n.children(&mut cursor) {
        c += count_nodes(&child);
    }
    c
}
