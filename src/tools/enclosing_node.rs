use serde::Deserialize;
use serde_json::json;

use crate::config::workspace::Workspace;
use crate::extractors::enclosing_node::{self, EnclosingOptions, EnclosingResult};
use crate::parser;
use crate::parser::positions;
use crate::safety;
use crate::shared::errors::AstToolError;
use crate::shared::language::LanguageId;
use crate::shared::position::Position;

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct AstEnclosingNodeInput {
    pub file_path: String,
    pub line: u32,
    pub character: u32,
    pub kinds: Option<Vec<String>>,
    pub include_source_text: bool,
}

pub fn handle(workspace: &Workspace, args: serde_json::Value) -> serde_json::Value {
    let input: AstEnclosingNodeInput = match serde_json::from_value(args) {
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

    // Convert line/character position to byte offset.
    let pos = Position {
        line: input.line,
        character: input.character,
    };
    let byte_offset = match positions::position_to_byte_offset(&source, pos) {
        Ok(b) => b,
        Err(e) => return e.payload(),
    };

    let opts = EnclosingOptions {
        kinds: input.kinds,
        include_source_text: input.include_source_text,
        max_ancestors: 64,
    };

    let ancestors = match enclosing_node::enclosing_node(&tree, &source, byte_offset, &opts) {
        Ok(a) => a,
        Err(e) => return e.payload(),
    };

    let truncated = ancestors.len() >= 64;

    let result = EnclosingResult {
        file_path: resolved.workspace_relative.clone(),
        language: lang.as_str().to_string(),
        position: pos,
        ancestors,
        truncated,
    };

    json!({
        "filePath": result.file_path,
        "language": result.language,
        "position": {
            "line": result.position.line,
            "character": result.position.character,
        },
        "ancestors": serde_json::to_value(&result.ancestors).unwrap_or(json!([])),
        "truncated": result.truncated,
    })
}

fn extension_to_language(path: &str) -> Option<LanguageId> {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|s| s.to_str())?;
    let dotted = format!(".{}", ext);
    parser::registry::for_extension(&dotted).map(|d| d.language)
}
