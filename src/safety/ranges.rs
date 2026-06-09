//! Range validation helpers for rewrite operations.

use tree_sitter::Node;

use crate::shared::errors::AstToolError;
use crate::shared::position::Range;
use crate::text::position_encoding;

/// Validate a range is well-formed, within source bounds, and resolve to byte offsets.
pub fn validate_range(source: &str, range: Range) -> Result<(usize, usize), AstToolError> {
    position_encoding::validate_range_in_bounds(source, range)
}

/// Check that `node` is aligned to the given range (i.e. the node's byte range
/// matches the external range within tolerance).
pub fn check_node_alignment(source: &str, range: Range, node: &Node) -> Result<(), AstToolError> {
    let matches =
        position_encoding::node_range_matches(source, range, node.start_byte(), node.end_byte())?;
    if !matches {
        return Err(AstToolError::RewriteRangeNotNodeAligned(format!(
            "node range [{}, {}] does not match target range",
            node.start_byte(),
            node.end_byte()
        )));
    }
    Ok(())
}

/// Check that `node`'s kind matches the expected kind (if provided).
pub fn check_node_kind(node: &Node, expected_kind: Option<&str>) -> Result<(), AstToolError> {
    if let Some(expected) = expected_kind {
        if node.kind() != expected {
            return Err(AstToolError::RewriteNodeKindMismatch(format!(
                "expected '{}', got '{}'",
                expected,
                node.kind()
            )));
        }
    }
    Ok(())
}
