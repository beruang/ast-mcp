use serde::Deserialize;
use serde_json::json;

use crate::config::workspace::Workspace;
use crate::parser;
use crate::safety;
use crate::shared::errors::AstToolError;
use crate::shared::language::LanguageId;

#[derive(Deserialize)]
#[serde(default)]
pub struct AstParseFileInput {
    pub file_path: String,
    pub include_tree: bool,
    pub max_depth: Option<usize>,
    pub include_node_text: bool,
}

impl Default for AstParseFileInput {
    fn default() -> Self {
        AstParseFileInput {
            file_path: String::new(),
            include_tree: false,
            max_depth: Some(3),
            include_node_text: false,
        }
    }
}

pub fn handle(workspace: &Workspace, args: serde_json::Value) -> serde_json::Value {
    let input: AstParseFileInput = match serde_json::from_value(args) {
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

    let (tree, status) = match parser::parse::parse_source(&source, lang) {
        Ok(t) => t,
        Err(e) => return e.payload(),
    };

    let max_depth = input.max_depth.unwrap_or(3);
    let mut result = json!({
        "filePath": resolved.workspace_relative,
        "language": lang.as_str(),
        "parsed": true,
        "hasSyntaxError": status.has_syntax_error,
        "rootKind": status.root_kind,
        "nodeCount": status.node_count,
        "parseTimeMs": status.parse_time_ms,
    });

    if input.include_tree {
        let mut count: usize = 0;
        let tree_json =
            walk_tree(&tree.root_node(), &source, max_depth, input.include_node_text, &mut count);
        result["tree"] = tree_json;
        result["truncated"] = json!(count >= crate::safety::limits::MAX_NODES);
    }

    result
}

fn extension_to_language(path: &str) -> Option<LanguageId> {
    let ext = std::path::Path::new(path).extension().and_then(|s| s.to_str())?;
    let dotted = format!(".{}", ext);
    parser::registry::for_extension(&dotted).map(|d| d.language)
}

fn walk_tree(
    node: &tree_sitter::Node,
    source: &str,
    max_depth: usize,
    include_text: bool,
    count: &mut usize,
) -> serde_json::Value {
    *count += 1;

    let text = if include_text {
        let raw = node.utf8_text(source.as_bytes()).unwrap_or("");
        if raw.len() > crate::safety::limits::MAX_TEXT_BYTES {
            let mut end = crate::safety::limits::MAX_TEXT_BYTES;
            while end > 0 && !source.is_char_boundary(end) {
                end -= 1;
            }
            Some(source[node.start_byte()..node.start_byte() + end].to_string())
        } else {
            Some(raw.to_string())
        }
    } else {
        None
    };

    let children = if max_depth > 0 && *count < crate::safety::limits::MAX_NODES {
        let mut cursor = node.walk();
        let kids: Vec<_> = node
            .children(&mut cursor)
            .filter_map(|child| {
                if *count >= crate::safety::limits::MAX_NODES {
                    None
                } else {
                    Some(walk_tree(&child, source, max_depth - 1, include_text, count))
                }
            })
            .collect();
        if kids.is_empty() {
            None
        } else {
            Some(kids)
        }
    } else {
        None
    };

    let start_pos = crate::parser::positions::ts_point_to_position(node.start_position(), source);
    let end_pos = crate::parser::positions::ts_point_to_position(node.end_position(), source);

    json!({
        "kind": node.kind(),
        "startByte": node.start_byte(),
        "endByte": node.end_byte(),
        "range": {
            "start": { "line": start_pos.line, "character": start_pos.character },
            "end": { "line": end_pos.line, "character": end_pos.character },
        },
        "text": text,
        "children": children,
    })
}
