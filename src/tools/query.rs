use serde::Deserialize;
use serde_json::json;

use crate::config::workspace::Workspace;
use crate::extractors::queries::{run_query, QueryOptions};
use crate::parser;
use crate::safety;
use crate::shared::errors::AstToolError;
use crate::shared::language::LanguageId;

#[derive(Deserialize)]
#[serde(default)]
pub struct AstQueryInput {
    pub file_path: String,
    pub query: String,
    pub max_results: usize,
    pub include_node_text: bool,
    pub max_text_bytes: usize,
}

impl Default for AstQueryInput {
    fn default() -> Self {
        AstQueryInput {
            file_path: String::new(),
            query: String::new(),
            max_results: crate::safety::limits::MAX_QUERY_MATCHES,
            include_node_text: true,
            max_text_bytes: crate::safety::limits::MAX_TEXT_BYTES,
        }
    }
}

pub fn handle(workspace: &Workspace, args: serde_json::Value) -> serde_json::Value {
    let input: AstQueryInput = match serde_json::from_value(args) {
        Ok(v) => v,
        Err(e) => return AstToolError::InvalidPosition(e.to_string()).payload(),
    };

    if input.query.trim().is_empty() {
        return AstToolError::QueryInvalid("empty query".into(), None).payload();
    }

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

    let query = match parser::queries::compile_query(lang, &input.query) {
        Ok(q) => q,
        Err(e) => return e.payload(),
    };

    let opts = QueryOptions {
        max_results: input.max_results,
        include_node_text: input.include_node_text,
        max_text_bytes: input.max_text_bytes,
    };

    let timeout_ms = crate::safety::limits::QUERY_TIMEOUT_MS;
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let matches = run_query(&query, &tree, &source, opts);
        let _ = tx.send(matches);
    });

    let matches = match rx.recv_timeout(std::time::Duration::from_millis(timeout_ms)) {
        Ok(m) => m,
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            return AstToolError::QueryExecutionFailed(
                format!("query timed out after {}ms", timeout_ms),
                None,
            )
            .payload();
        }
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            return AstToolError::QueryExecutionFailed("query execution panicked".into(), None)
                .payload();
        }
    };

    let returned = matches.len();
    let truncated = returned >= input.max_results;

    json!({
        "filePath": resolved.workspace_relative,
        "language": lang.as_str(),
        "query": input.query,
        "matches": matches,
        "returnedCount": returned,
        "truncated": truncated,
        "parseTimeMs": status.parse_time_ms,
        "hasSyntaxError": status.has_syntax_error,
    })
}

fn extension_to_language(path: &str) -> Option<LanguageId> {
    let ext = std::path::Path::new(path).extension().and_then(|s| s.to_str())?;
    let dotted = format!(".{}", ext);
    parser::registry::for_extension(&dotted).map(|d| d.language)
}
