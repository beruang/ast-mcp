use serde::Deserialize;
use serde_json::json;

use crate::config::workspace::Workspace;
use crate::extractors::functions::{find_functions, FunctionOptions};
use crate::parser;
use crate::safety;
use crate::shared::errors::AstToolError;
use crate::shared::language::LanguageId;

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct AstFindFunctionsInput {
    pub file_path: String,
    pub include_anonymous: Option<bool>,
    pub include_parameters: Option<bool>,
    pub include_return_type: Option<bool>,
    pub include_signature: Option<bool>,
    pub max_results: Option<usize>,
}

pub fn handle(workspace: &Workspace, args: serde_json::Value) -> serde_json::Value {
    let input: AstFindFunctionsInput = match serde_json::from_value(args) {
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

    let opts = FunctionOptions {
        max_results: input.max_results.unwrap_or(crate::safety::limits::MAX_RESULTS),
        include_anonymous: input.include_anonymous.unwrap_or(true),
        include_parameters: input.include_parameters.unwrap_or(true),
        include_return_type: input.include_return_type.unwrap_or(true),
        include_signature: input.include_signature.unwrap_or(true),
    };

    let functions = find_functions(&tree, &source, lang, &opts);

    json!({
        "filePath": resolved.workspace_relative,
        "language": lang.as_str(),
        "functions": functions,
        "count": functions.len(),
    })
}

fn extension_to_language(path: &str) -> Option<LanguageId> {
    let ext = std::path::Path::new(path).extension().and_then(|s| s.to_str())?;
    let dotted = format!(".{}", ext);
    parser::registry::for_extension(&dotted).map(|d| d.language)
}
