//! ast_node_text — return exact source text for a range.
use serde::Deserialize;
use serde_json::json;

use crate::config::defaults::MAX_TEXT_BYTES;
use crate::config::workspace::Workspace;
use crate::safety;
use crate::shared::errors::AstToolError;
use crate::shared::position::Range;
use crate::text::{position_encoding, text_budget};

#[derive(Deserialize)]
#[serde(default)]
pub struct AstNodeTextInput {
    pub file_path: String,
    pub range: Option<Range>,
    pub max_bytes: usize,
}

impl Default for AstNodeTextInput {
    fn default() -> Self {
        Self { file_path: String::new(), range: None, max_bytes: MAX_TEXT_BYTES }
    }
}

pub fn handle(workspace: &Workspace, args: serde_json::Value) -> serde_json::Value {
    let input: AstNodeTextInput = match serde_json::from_value(args) {
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

    let source = match std::fs::read_to_string(&resolved.absolute) {
        Ok(s) => s,
        Err(e) => return AstToolError::FileNotFound(e.to_string()).payload(),
    };

    let (start_byte, end_byte) = match position_encoding::validate_range_in_bounds(&source, range) {
        Ok(v) => v,
        Err(e) => return e.payload(),
    };

    let raw = &source[start_byte..end_byte.min(source.len())];
    let byte_count = raw.len();
    let (text, truncated) = text_budget::truncate_text(raw, input.max_bytes);

    let actual_end_byte = start_byte + text.len();
    let actual_range = position_encoding::byte_range_to_range(&source, start_byte, actual_end_byte);

    json!({
        "filePath": resolved.workspace_relative,
        "range": {
            "start": { "line": actual_range.start.line, "character": actual_range.start.character },
            "end": { "line": actual_range.end.line, "character": actual_range.end.character }
        },
        "text": text,
        "truncated": truncated,
        "byteCount": byte_count,
    })
}
