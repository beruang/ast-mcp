//! Max nesting depth computation for AST trees.

/// Compute the maximum nesting depth of a tree-sitter tree.
pub fn max_nesting_depth(node: &tree_sitter::Node) -> usize {
    let mut max_depth: usize = 0;
    compute_depth(node, 0, &mut max_depth);
    max_depth
}

fn compute_depth(node: &tree_sitter::Node, current: usize, max: &mut usize) {
    if current > *max {
        *max = current;
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            compute_depth(&child, current + 1, max);
        }
    }
}

/// Count total named nodes in the tree.
pub fn count_nodes(node: &tree_sitter::Node) -> usize {
    let mut count: usize = 0;
    count_nodes_recursive(node, &mut count);
    count
}

fn count_nodes_recursive(node: &tree_sitter::Node, count: &mut usize) {
    if node.is_named() {
        *count += 1;
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            count_nodes_recursive(&child, count);
        }
    }
}

/// Count syntax error nodes.
pub fn count_errors(node: &tree_sitter::Node) -> usize {
    let mut count: usize = 0;
    count_errors_recursive(node, &mut count);
    count
}

fn count_errors_recursive(node: &tree_sitter::Node, count: &mut usize) {
    if node.kind() == "ERROR" || node.has_error() && node.child_count() == 0 {
        *count += 1;
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            count_errors_recursive(&child, count);
        }
    }
}
