use serde::Deserialize;
use serde_json::json;

use crate::config::workspace::Workspace;
use crate::extractors::classes::{find_classes, ClassOptions};
use crate::parser;
use crate::safety;
use crate::shared::errors::AstToolError;
use crate::shared::language::LanguageId;

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct AstFindClassesInput {
    pub file_path: String,
    pub include_methods: Option<bool>,
    pub include_extends: Option<bool>,
    pub include_implements: Option<bool>,
    pub include_decorators: Option<bool>,
    pub max_results: Option<usize>,
}

pub fn handle(workspace: &Workspace, args: serde_json::Value) -> serde_json::Value {
    let input: AstFindClassesInput = match serde_json::from_value(args) {
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

    let opts = ClassOptions {
        max_results: input
            .max_results
            .unwrap_or(crate::safety::limits::MAX_RESULTS),
        include_methods: input.include_methods.unwrap_or(true),
        include_extends: input.include_extends.unwrap_or(true),
        include_implements: input.include_implements.unwrap_or(true),
        include_decorators: input.include_decorators.unwrap_or(true),
    };

    let classes = find_classes(&tree, &source, lang, &opts);

    json!({
        "filePath": resolved.workspace_relative,
        "language": lang.as_str(),
        "classes": classes,
        "count": classes.len(),
    })
}

fn extension_to_language(path: &str) -> Option<LanguageId> {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|s| s.to_str())?;
    let dotted = format!(".{}", ext);
    parser::registry::for_extension(&dotted).map(|d| d.language)
}
