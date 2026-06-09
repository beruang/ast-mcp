//! Per-function metric computation.
use crate::shared::types_v2::FunctionMetric;
use crate::text::position_encoding;

/// Extract function metrics for all function-like nodes in a tree.
pub fn extract_function_metrics(node: &tree_sitter::Node, source: &str) -> Vec<FunctionMetric> {
    let mut metrics = Vec::new();
    collect_functions(node, source, &mut metrics);
    metrics
}

fn collect_functions(node: &tree_sitter::Node, source: &str, metrics: &mut Vec<FunctionMetric>) {
    if is_function(node.kind()) {
        if let Some(fm) = build_function_metric(node, source) {
            metrics.push(fm);
        }
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_functions(&child, source, metrics);
        }
    }
}

fn is_function(kind: &str) -> bool {
    matches!(
        kind,
        "function_declaration"
            | "function_definition"
            | "method_definition"
            | "arrow_function"
            | "lambda"
    )
}

fn build_function_metric(node: &tree_sitter::Node, source: &str) -> Option<FunctionMetric> {
    let br = node.byte_range();
    let range = position_encoding::byte_range_to_range(source, br.start, br.end);

    let name = crate::context::node_at_range_helpers::extract_name(node, source);

    // Line count
    let node_text = &source[br.start..br.end];
    let line_count = node_text.lines().count();

    // Branch count (if/else/switch/match)
    let branch_count =
        count_kind_recursive(node, &["if_statement", "else_clause", "switch_case", "match_arm"]);

    // Loop count
    let loop_count = count_kind_recursive(
        node,
        &["for_statement", "while_statement", "do_statement", "for_in_statement"],
    );

    // Nesting depth
    let nesting_depth = crate::metrics::nesting::max_nesting_depth(node);

    Some(FunctionMetric {
        name,
        kind: node.kind().to_string(),
        range,
        line_count: line_count.max(1),
        branch_count,
        loop_count,
        nesting_depth,
    })
}

fn count_kind_recursive(node: &tree_sitter::Node, kinds: &[&str]) -> usize {
    let mut count: usize = 0;
    if kinds.contains(&node.kind()) {
        count += 1;
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            count += count_kind_recursive(&child, kinds);
        }
    }
    count
}
