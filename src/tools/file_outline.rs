use serde::Deserialize;
use serde_json::json;

use crate::config::workspace::Workspace;
use crate::extractors::outline::{self, AstFileOutlineResult, OutlineOptions};
use crate::parser;
use crate::safety;
use crate::shared::errors::AstToolError;
use crate::shared::language::LanguageId;

#[derive(Deserialize)]
#[serde(default)]
pub struct AstFileOutlineInput {
    pub file_path: String,
    pub max_depth: usize,
    pub include_ranges: bool,
    pub include_imports: bool,
    pub include_exports: bool,
}

impl Default for AstFileOutlineInput {
    fn default() -> Self {
        AstFileOutlineInput {
            file_path: String::new(),
            max_depth: 4,
            include_ranges: true,
            include_imports: false,
            include_exports: false,
        }
    }
}

pub fn handle(workspace: &Workspace, args: serde_json::Value) -> serde_json::Value {
    let input: AstFileOutlineInput = match serde_json::from_value(args) {
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

    let opts = OutlineOptions {
        max_depth: input.max_depth,
        include_ranges: input.include_ranges,
        include_imports: input.include_imports,
        include_exports: input.include_exports,
    };

    let result: AstFileOutlineResult =
        outline::file_outline(&tree, &source, &opts, lang, &resolved.workspace_relative);

    json!({
        "filePath": result.file_path,
        "language": result.language,
        "outlineText": result.outline_text,
        "nodes": serde_json::to_value(&result.nodes).unwrap_or(json!([])),
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
