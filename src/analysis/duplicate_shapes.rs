use crate::shared::types_v5::{DetectDuplicateShapesInput, DetectDuplicateShapesResult};
use serde_json::Value;

pub fn handle_arguments(args: Value) -> DetectDuplicateShapesInput {
    serde_json::from_value(args).unwrap_or(DetectDuplicateShapesInput {
        glob: String::new(),
        max_files: None,
        min_node_lines: None,
        node_kinds: None,
        normalize_identifiers: None,
        normalize_literals: None,
        max_candidates: None,
    })
}

pub fn empty_result(scanned: usize) -> DetectDuplicateShapesResult {
    DetectDuplicateShapesResult {
        groups: vec![],
        scanned_files: scanned,
        candidate_count: 0,
        returned: 0,
        truncated: false,
    }
}
