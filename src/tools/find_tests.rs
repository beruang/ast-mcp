use serde::Deserialize;
use serde_json::json;

use crate::config::defaults::{
    MAX_BYTES_PER_WORKSPACE_FILE, MAX_WORKSPACE_QUERY_FILES, MAX_WORKSPACE_QUERY_RESULTS,
};
use crate::config::workspace::Workspace;
use crate::frameworks::tests::{go, javascript, python, rust};
use crate::frameworks::AstDetector;
use crate::parser;
use crate::safety::paths;
use crate::shared::errors::AstToolError;
use crate::shared::language::LanguageId;
use crate::workspace::file_scanner::{self, ScanOptions};

#[derive(Deserialize)]
#[serde(default)]
pub struct AstFindTestsInput {
    pub file_path: Option<String>,
    pub glob: Option<String>,
    pub frameworks: Option<Vec<String>>,
    pub max_files: usize,
    pub max_results: usize,
}

impl Default for AstFindTestsInput {
    fn default() -> Self {
        Self {
            file_path: None,
            glob: None,
            frameworks: None,
            max_files: MAX_WORKSPACE_QUERY_FILES,
            max_results: MAX_WORKSPACE_QUERY_RESULTS,
        }
    }
}

pub fn handle(workspace: &Workspace, args: serde_json::Value) -> serde_json::Value {
    let input: AstFindTestsInput = match serde_json::from_value(args) {
        Ok(v) => v,
        Err(e) => return AstToolError::InternalError(format!("invalid input: {}", e)).payload(),
    };

    match (&input.file_path, &input.glob) {
        (Some(fp), _) => handle_single_file(workspace, fp, &input),
        (None, Some(g)) => handle_workspace(workspace, g, &input),
        (None, None) => AstToolError::InvalidGlob("file_path or glob required".into()).payload(),
    }
}

fn handle_single_file(
    workspace: &Workspace,
    file_path: &str,
    input: &AstFindTestsInput,
) -> serde_json::Value {
    let resolved = match paths::resolve_file(workspace, file_path) {
        Ok(r) => r,
        Err(e) => return e.payload(),
    };

    let ext = resolved
        .absolute
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e))
        .unwrap_or_default();

    let lang = match parser::registry::for_extension(&ext) {
        Some(d) => d.language,
        None => return AstToolError::UnsupportedLanguage(ext).payload(),
    };

    let source = match std::fs::read_to_string(&resolved.absolute) {
        Ok(s) => s,
        Err(e) => return AstToolError::FileNotFound(format!("{}: {}", file_path, e)).payload(),
    };

    let (tree, _status) = match parser::parse::parse_source(&source, lang) {
        Ok(t) => t,
        Err(e) => return e.payload(),
    };

    let ctx = crate::frameworks::AstFileContext {
        workspace_path: workspace.root(),
        file_path: &resolved.absolute,
        relative_path: file_path,
        language: lang.as_str(),
        source: &source,
        tree: &tree,
    };

    let tests = detect_tests(&ctx, lang);
    let filtered = apply_framework_filter(tests, &input.frameworks);

    let returned = filtered.len() as u32;
    json!({
        "tests": filtered,
        "returned": returned,
        "truncated": false,
        "scannedFiles": 1,
    })
}

fn handle_workspace(
    workspace: &Workspace,
    glob: &str,
    input: &AstFindTestsInput,
) -> serde_json::Value {
    let scan_opts = ScanOptions {
        root: workspace.root().to_path_buf(),
        glob: Some(glob.to_string()),
        max_files: input.max_files,
        max_bytes_per_file: MAX_BYTES_PER_WORKSPACE_FILE,
    };
    let files = file_scanner::scan_files(&scan_opts);

    let mut all_tests = Vec::new();
    let mut scanned: u32 = 0;

    for (path, file_lang) in &files {
        if all_tests.len() >= input.max_results {
            break;
        }

        let lang = match file_lang {
            Some(l) => *l,
            None => continue,
        };

        let source = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let (tree, _status) = match parser::parse::parse_source(&source, lang) {
            Ok(t) => t,
            Err(_) => continue,
        };

        let rel_path = path
            .strip_prefix(workspace.root())
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");

        let ctx = crate::frameworks::AstFileContext {
            workspace_path: workspace.root(),
            file_path: path,
            relative_path: &rel_path,
            language: lang.as_str(),
            source: &source,
            tree: &tree,
        };

        let mut tests = detect_tests(&ctx, lang);
        let remaining = input.max_results.saturating_sub(all_tests.len());
        if tests.len() > remaining {
            tests.truncate(remaining);
        }
        all_tests.extend(tests);
        scanned += 1;
    }

    let filtered = apply_framework_filter(all_tests, &input.frameworks);
    let returned = filtered.len() as u32;
    let truncated = returned as usize >= input.max_results || scanned as usize >= input.max_files;

    json!({
        "tests": filtered,
        "returned": returned,
        "truncated": truncated,
        "scannedFiles": scanned,
    })
}

fn detect_tests(
    ctx: &crate::frameworks::AstFileContext,
    lang: LanguageId,
) -> Vec<crate::shared::types_v3::AstTestItem> {
    match lang {
        LanguageId::TypeScript
        | LanguageId::TypeScriptReact
        | LanguageId::JavaScript
        | LanguageId::JavaScriptReact => javascript::JavaScriptTestDetector.detect(ctx),
        LanguageId::Python => python::PythonTestDetector.detect(ctx),
        LanguageId::Go => go::GoTestDetector.detect(ctx),
        LanguageId::Rust => rust::RustTestDetector.detect(ctx),
    }
}

fn apply_framework_filter(
    mut items: Vec<crate::shared::types_v3::AstTestItem>,
    frameworks: &Option<Vec<String>>,
) -> Vec<crate::shared::types_v3::AstTestItem> {
    if let Some(fw) = frameworks {
        items.retain(|i| fw.iter().any(|f| i.framework.contains(f.as_str())));
    }
    items
}
