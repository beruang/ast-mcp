use crate::shared::types_v5::{DetectLargeNodesInput, DetectLargeNodesResult, LargeNode};
use serde_json::Value;

pub fn detect_in_file(
    file_path: &str,
    source: &str,
    input: &DetectLargeNodesInput,
) -> Vec<LargeNode> {
    let min_lines = input.min_lines.unwrap_or(80);
    let lines: Vec<&str> = source.lines().collect();
    let total_lines = lines.len();

    // Heuristic: if file is large, report it
    if total_lines >= min_lines {
        let func_lines = source.matches("fn ").count()
            + source.matches("func ").count()
            + source.matches("def ").count()
            + source.matches("function ").count();

        return vec![LargeNode {
            file_path: file_path.to_string(),
            kind: "file".to_string(),
            name: None,
            range: crate::shared::position::Range {
                start: crate::shared::position::Position { line: 0, character: 0 },
                end: crate::shared::position::Position { line: total_lines as u32, character: 0 },
            },
            line_count: total_lines,
            child_count: func_lines,
            nesting_depth: 0,
        }];
    }

    vec![]
}

pub fn handle_arguments(args: Value) -> DetectLargeNodesInput {
    serde_json::from_value(args).unwrap_or(DetectLargeNodesInput {
        file_path: None,
        glob: None,
        max_files: None,
        min_lines: None,
        node_kinds: None,
        max_results: None,
    })
}

pub fn empty_result() -> DetectLargeNodesResult {
    DetectLargeNodesResult { nodes: vec![], scanned_files: 0, returned: 0, truncated: false }
}
