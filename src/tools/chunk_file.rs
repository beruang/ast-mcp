use serde::Deserialize;
use serde_json::json;

use crate::config::workspace::Workspace;
use crate::extractors::chunks::{chunk_file, ChunkOptions, ChunkStrategy};
use crate::parser;
use crate::safety;
use crate::shared::errors::AstToolError;
use crate::shared::language::LanguageId;

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct AstChunkFileInput {
    pub file_path: String,
    pub strategy: Option<String>,
    pub max_results: Option<usize>,
    pub max_lines_per_chunk: Option<usize>,
    pub max_bytes_per_chunk: Option<usize>,
}

pub fn handle(workspace: &Workspace, args: serde_json::Value) -> serde_json::Value {
    let input: AstChunkFileInput = match serde_json::from_value(args) {
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

    let strategy = match input.strategy.as_deref() {
        Some("function_class") => ChunkStrategy::FunctionClass,
        Some("semantic_blocks") => ChunkStrategy::SemanticBlocks,
        Some("max_lines") => ChunkStrategy::MaxLinesWithAstBoundaries,
        _ => ChunkStrategy::TopLevel,
    };

    let opts = ChunkOptions {
        strategy,
        max_results: input
            .max_results
            .unwrap_or(crate::safety::limits::MAX_RESULTS),
        max_lines_per_chunk: input
            .max_lines_per_chunk
            .unwrap_or(crate::safety::limits::MAX_CHUNK_LINES),
        max_bytes_per_chunk: input
            .max_bytes_per_chunk
            .unwrap_or(crate::safety::limits::MAX_CHUNK_BYTES),
    };

    let result = match chunk_file(&tree, &source, &resolved.workspace_relative, &opts) {
        Ok(r) => r,
        Err(e) => return e.payload(),
    };

    json!({
        "filePath": resolved.workspace_relative,
        "language": lang.as_str(),
        "chunks": result.chunks,
        "strategy": result.strategy,
        "totalChunks": result.total_chunks,
    })
}

fn extension_to_language(path: &str) -> Option<LanguageId> {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|s| s.to_str())?;
    let dotted = format!(".{}", ext);
    parser::registry::for_extension(&dotted).map(|d| d.language)
}
