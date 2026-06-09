use crate::shared::types_v5::{
    ComplexityHotspot, ComplexitySummaryInput, ComplexitySummaryResult, FileComplexitySummary,
};
use serde_json::Value;

pub fn analyze_file(
    file_path: &str,
    source: &str,
    _input: &ComplexitySummaryInput,
) -> Result<(FileComplexitySummary, Vec<ComplexityHotspot>), String> {
    let line_count = source.lines().count();
    let import_count = source.matches("import").count() + source.matches("from ").count();

    // Structural heuristic counts — full implementation would use Tree-sitter queries
    let function_count = source.matches("fn ").count()
        + source.matches("func ").count()
        + source.matches("def ").count()
        + source.matches("function ").count();
    let class_count = source.matches("class ").count()
        + source.matches("struct ").count()
        + source.matches("impl ").count();

    let summary = FileComplexitySummary {
        file_path: file_path.to_string(),
        line_count,
        function_count,
        class_count,
        import_count,
        max_nesting_depth: 0,
        max_function_lines: 0,
        branch_count: source.matches("if ").count()
            + source.matches("else").count()
            + source.matches("match").count()
            + source.matches("switch").count()
            + source.matches("case ").count(),
        loop_count: source.matches("for ").count()
            + source.matches("while ").count()
            + source.matches("loop ").count(),
    };

    Ok((summary, vec![]))
}

pub fn handle_arguments(args: Value) -> ComplexitySummaryInput {
    serde_json::from_value(args).unwrap_or(ComplexitySummaryInput {
        file_path: None,
        glob: None,
        max_files: None,
        include_functions: None,
        include_classes: None,
        max_results: None,
    })
}

pub fn empty_result() -> ComplexitySummaryResult {
    ComplexitySummaryResult {
        total_files_scanned: 0,
        total_nodes_analyzed: 0,
        files: vec![],
        hotspots: vec![],
        truncated: false,
    }
}
