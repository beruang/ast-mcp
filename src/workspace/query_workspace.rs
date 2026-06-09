//! ast_query_workspace — run a bounded Tree-sitter query across workspace files.
use serde::Deserialize;
use serde_json::json;

use crate::config::defaults::{
    MAX_BYTES_PER_WORKSPACE_FILE, MAX_WORKSPACE_QUERY_FILES, MAX_WORKSPACE_QUERY_RESULTS,
};
use crate::config::workspace::Workspace;
use crate::parser;
use crate::shared::errors::AstToolError;
use crate::shared::language::LanguageId;
use crate::shared::types_v2::{QueryCapture, WorkspaceQueryMatch};
use crate::text::position_encoding;
use crate::workspace::file_scanner::{self, ScanOptions};

#[derive(Deserialize)]
#[serde(default)]
pub struct AstQueryWorkspaceInput {
    pub query: String,
    pub language: Option<String>,
    pub glob: Option<String>,
    pub max_files: usize,
    pub max_results: usize,
    pub max_bytes_per_file: usize,
    pub include_text: bool,
}

impl Default for AstQueryWorkspaceInput {
    fn default() -> Self {
        Self {
            query: String::new(),
            language: None,
            glob: None,
            max_files: MAX_WORKSPACE_QUERY_FILES,
            max_results: MAX_WORKSPACE_QUERY_RESULTS,
            max_bytes_per_file: MAX_BYTES_PER_WORKSPACE_FILE,
            include_text: true,
        }
    }
}

pub fn handle(workspace: &Workspace, args: serde_json::Value) -> serde_json::Value {
    let input: AstQueryWorkspaceInput = match serde_json::from_value(args) {
        Ok(v) => v,
        Err(e) => return AstToolError::QueryInvalid(e.to_string(), None).payload(),
    };

    if input.query.trim().is_empty() {
        return AstToolError::QueryInvalid("query is empty".into(), None).payload();
    }

    // Resolve target language
    let target_lang: Option<LanguageId> = input.language.as_deref().and_then(lang_from_str);

    // Scan files
    let scan_opts = ScanOptions {
        root: workspace.root().to_path_buf(),
        glob: input.glob.clone(),
        max_files: input.max_files,
        max_bytes_per_file: input.max_bytes_per_file,
    };
    let files = file_scanner::scan_files(&scan_opts);

    let mut matches: Vec<WorkspaceQueryMatch> = Vec::new();
    let mut files_scanned: usize = 0;
    let mut files_skipped: usize = 0;

    for (path, file_lang) in &files {
        if matches.len() >= input.max_results {
            break;
        }

        // Language filter
        let lang = match (target_lang, file_lang) {
            (Some(tl), Some(fl)) if tl == *fl => *fl,
            (None, Some(fl)) => *fl,
            _ => {
                files_skipped += 1;
                continue;
            }
        };

        let source = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => {
                files_skipped += 1;
                continue;
            }
        };

        let (tree, _status) = match parser::parse::parse_source(&source, lang) {
            Ok(t) => t,
            Err(_) => {
                files_skipped += 1;
                continue;
            }
        };

        // Compile and run query
        let ts_lang = match parser::registry::for_language(lang) {
            Some(def) => (def.tree_sitter_language)(),
            None => {
                files_skipped += 1;
                continue;
            }
        };
        let query = match tree_sitter::Query::new(&ts_lang, &input.query) {
            Ok(q) => q,
            Err(e) => {
                return AstToolError::QueryInvalid(
                    format!("query compilation failed: {}", e),
                    None,
                )
                .payload();
            }
        };

        let mut qc = tree_sitter::QueryCursor::new();
        let mut file_matches: Vec<QueryCapture> = Vec::new();
        let root = tree.root_node();

        for m in qc.matches(&query, root, source.as_bytes()) {
            if matches.len() + file_matches.len() >= input.max_results {
                break;
            }
            for capture in m.captures {
                let node = capture.node;
                let br = node.byte_range();
                let range = position_encoding::byte_range_to_range(&source, br.start, br.end);
                let text = if input.include_text {
                    Some(source[br.start..br.end].to_string())
                } else {
                    None
                };
                let cap_name = query.capture_names()[capture.index as usize].to_string();
                file_matches.push(QueryCapture {
                    name: cap_name,
                    kind: node.kind().to_string(),
                    text,
                    range,
                });
            }
        }

        if !file_matches.is_empty() {
            let rel_path = path
                .strip_prefix(workspace.root())
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            matches.push(WorkspaceQueryMatch { file_path: rel_path, captures: file_matches });
        }

        files_scanned += 1;
    }

    let returned = matches.len();
    let truncated = returned >= input.max_results || files_scanned >= input.max_files;

    json!({
        "matches": matches,
        "filesScanned": files_scanned,
        "filesSkipped": files_skipped,
        "returned": returned,
        "truncated": truncated,
    })
}

fn lang_from_str(s: &str) -> Option<LanguageId> {
    match s.to_lowercase().as_str() {
        "typescript" | "ts" => Some(LanguageId::TypeScript),
        "typescriptreact" | "tsx" => Some(LanguageId::TypeScriptReact),
        "javascript" | "js" => Some(LanguageId::JavaScript),
        "javascriptreact" | "jsx" => Some(LanguageId::JavaScriptReact),
        "python" | "py" => Some(LanguageId::Python),
        "go" => Some(LanguageId::Go),
        "rust" | "rs" => Some(LanguageId::Rust),
        _ => None,
    }
}
