use serde::Deserialize;
use serde_json::json;

use crate::config::defaults::{MAX_BYTES_PER_WORKSPACE_FILE, MAX_WORKSPACE_QUERY_FILES};
use crate::config::workspace::Workspace;
use crate::frameworks::react::components::ReactComponentDetector;
use crate::frameworks::AstDetector;
use crate::parser;
use crate::safety::paths;
use crate::shared::errors::AstToolError;
use crate::shared::language::LanguageId;
use crate::workspace::file_scanner::{self, ScanOptions};

#[derive(Deserialize)]
#[serde(default)]
pub struct AstFindReactComponentsInput {
    pub file_path: Option<String>,
    pub glob: Option<String>,
    pub max_files: usize,
    pub max_results: usize,
    pub include_hooks: bool,
    pub include_jsx_summary: bool,
}

impl Default for AstFindReactComponentsInput {
    fn default() -> Self {
        Self {
            file_path: None,
            glob: None,
            max_files: MAX_WORKSPACE_QUERY_FILES,
            max_results: 500,
            include_hooks: true,
            include_jsx_summary: false,
        }
    }
}

pub fn handle(workspace: &Workspace, args: serde_json::Value) -> serde_json::Value {
    let input: AstFindReactComponentsInput = match serde_json::from_value(args) {
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
    input: &AstFindReactComponentsInput,
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

    let mut components = if is_js_ts(lang) { ReactComponentDetector.detect(&ctx) } else { vec![] };
    if input.include_hooks {
        let hooks = crate::frameworks::react::hooks::ReactHookDetector.detect(&ctx);
        for comp in &mut components {
            comp.hooks = hooks
                .iter()
                .filter(|h| {
                    h.range.start.line >= comp.range.start.line
                        && h.range.end.line <= comp.range.end.line
                })
                .map(|h| h.name.clone())
                .collect();
        }
    }
    if !input.include_jsx_summary {
        for comp in &mut components {
            comp.jsx_root = None;
        }
    }
    let returned = components.len() as u32;
    json!({ "components": components, "returned": returned, "truncated": false, "scannedFiles": 1 })
}

fn handle_workspace(
    workspace: &Workspace,
    glob: &str,
    input: &AstFindReactComponentsInput,
) -> serde_json::Value {
    let scan_opts = ScanOptions {
        root: workspace.root().to_path_buf(),
        glob: Some(glob.to_string()),
        max_files: input.max_files,
        max_bytes_per_file: MAX_BYTES_PER_WORKSPACE_FILE,
    };
    let files = file_scanner::scan_files(&scan_opts);
    let mut all = Vec::new();
    let mut scanned: u32 = 0;

    for (path, file_lang) in &files {
        if all.len() >= input.max_results {
            break;
        }
        let lang = match file_lang {
            Some(l) => *l,
            None => continue,
        };
        if !is_js_ts(lang) {
            continue;
        }
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
        let mut comps = ReactComponentDetector.detect(&ctx);
        let remaining = input.max_results.saturating_sub(all.len());
        if comps.len() > remaining {
            comps.truncate(remaining);
        }
        all.extend(comps);
        scanned += 1;
    }

    let returned = all.len() as u32;
    let truncated = returned as usize >= input.max_results || scanned as usize >= input.max_files;
    json!({ "components": all, "returned": returned, "truncated": truncated, "scannedFiles": scanned })
}

fn is_js_ts(lang: LanguageId) -> bool {
    matches!(
        lang,
        LanguageId::TypeScript
            | LanguageId::TypeScriptReact
            | LanguageId::JavaScript
            | LanguageId::JavaScriptReact
    )
}
