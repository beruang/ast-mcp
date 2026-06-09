//! ast_file_metrics — return structural metrics for a file.
use serde::Deserialize;
use serde_json::json;

use crate::config::workspace::Workspace;
use crate::metrics::{function_metrics, nesting};
use crate::parser;
use crate::safety;
use crate::shared::errors::AstToolError;
use crate::shared::language::LanguageId;
use crate::shared::types_v2::FileMetrics;

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct AstFileMetricsInput {
    pub file_path: String,
    #[serde(default)]
    pub include_function_metrics: bool,
}

pub fn handle(workspace: &Workspace, args: serde_json::Value) -> serde_json::Value {
    let input: AstFileMetricsInput = match serde_json::from_value(args) {
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

    let root = tree.root_node();
    let line_count = source.lines().count();
    let byte_count = source.len();
    let node_count = nesting::count_nodes(&root);
    let syntax_error_count = nesting::count_errors(&root);
    let max_nesting_depth = nesting::max_nesting_depth(&root);

    // Count named declarations
    let import_count = count_named(&root, &["import_statement", "import_from_statement"]);
    let export_count = count_named(&root, &["export_statement", "export_default"]);
    let function_count = count_named(
        &root,
        &["function_declaration", "function_definition", "method_definition", "arrow_function"],
    );
    let class_count = count_named(&root, &["class_declaration", "class_definition"]);

    // Max function lines
    let max_function_lines = max_function_line_count(&root, &source);

    let metrics = FileMetrics {
        file_path: resolved.workspace_relative.clone(),
        language: lang.as_str().to_string(),
        line_count,
        byte_count,
        node_count,
        syntax_error_count,
        import_count,
        export_count,
        function_count,
        class_count,
        max_nesting_depth,
        max_function_lines,
    };

    let functions = if input.include_function_metrics {
        Some(function_metrics::extract_function_metrics(&root, &source))
    } else {
        None
    };

    json!({
        "metrics": metrics,
        "functions": functions,
    })
}

fn count_named(node: &tree_sitter::Node, kinds: &[&str]) -> usize {
    let mut count: usize = 0;
    if kinds.contains(&node.kind()) {
        count += 1;
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            count += count_named(&child, kinds);
        }
    }
    count
}

fn max_function_line_count(node: &tree_sitter::Node, source: &str) -> Option<usize> {
    let mut max_lines: Option<usize> = None;
    find_max_func_lines(node, source, &mut max_lines);
    max_lines
}

fn find_max_func_lines(node: &tree_sitter::Node, source: &str, max_lines: &mut Option<usize>) {
    if matches!(
        node.kind(),
        "function_declaration"
            | "function_definition"
            | "method_definition"
            | "arrow_function"
            | "lambda"
    ) {
        let br = node.byte_range();
        let text = &source[br.start..br.end];
        let lines = text.lines().count();
        if max_lines.is_none_or(|m| lines > m) {
            *max_lines = Some(lines);
        }
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            find_max_func_lines(&child, source, max_lines);
        }
    }
}

fn extension_to_language(path: &str) -> Option<LanguageId> {
    let ext = std::path::Path::new(path).extension().and_then(|s| s.to_str())?;
    let dotted = format!(".{}", ext);
    parser::registry::for_extension(&dotted).map(|d| d.language)
}
