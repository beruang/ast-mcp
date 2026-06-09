//! UTF-16 position ↔ byte offset conversion helpers.
//!
//! The external API uses zero-based line/character positions where character
//! is a UTF-16 code-unit offset (matching the LSP model).  Tree-sitter works
//! in byte offsets.  This module provides the bridge.
//!
//! It reuses the `LineIndex` from `parser::positions` which is the canonical
//! implementation shared by V1 and V2.

use crate::parser::positions::LineIndex;
use crate::shared::errors::AstToolError;
use crate::shared::position::{Position, Range};

/// Convert a `Position` to a byte offset into `source`.
///
/// Returns `PositionEncodingError` when the line or character is out of
/// bounds for the given source text.
pub fn position_to_byte(source: &str, pos: Position) -> Result<usize, AstToolError> {
    LineIndex::new(source).position_to_byte(source, pos).map_err(|e| {
        // Map the legacy InvalidPosition to the V2 code
        AstToolError::PositionEncodingError(e.to_string())
    })
}

/// Convert a byte offset to a `Position` (line, UTF-16 character).
pub fn byte_to_position(source: &str, byte: usize) -> Position {
    LineIndex::new(source).byte_to_position(source, byte)
}

/// Convert a `Range` (external positions) to a byte-offset pair `(start, end)`.
pub fn range_to_byte_range(source: &str, range: Range) -> Result<(usize, usize), AstToolError> {
    let start = position_to_byte(source, range.start)?;
    let end = position_to_byte(source, range.end)?;
    if start > end {
        return Err(AstToolError::PositionEncodingError(
            "range start byte > end byte after conversion".into(),
        ));
    }
    Ok((start, end))
}

/// Convert a byte-offset pair `(start, end)` to an external `Range`.
pub fn byte_range_to_range(source: &str, start: usize, end: usize) -> Range {
    let index = LineIndex::new(source);
    Range { start: index.byte_to_position(source, start), end: index.byte_to_position(source, end) }
}

/// Validate that a `Range` is well-formed: start ≤ end after normalization.
pub fn validate_range(range: Range) -> Result<Range, AstToolError> {
    let normalized = range.normalize();
    if normalized.start == normalized.end && range.start != range.end {
        // Zero-width range is fine, but if start > end we already normalized
    }
    Ok(normalized)
}

/// Validate a `Range` is well-formed and its positions are within source bounds.
pub fn validate_range_in_bounds(
    source: &str,
    range: Range,
) -> Result<(usize, usize), AstToolError> {
    let normalized = validate_range(range)?;
    let (start_byte, end_byte) = range_to_byte_range(source, normalized)?;
    if start_byte > source.len() || end_byte > source.len() {
        return Err(AstToolError::RangeOutOfBounds);
    }
    Ok((start_byte, end_byte))
}

/// Check whether `pos` is contained within `range` (inclusive start, exclusive end).
pub fn range_contains_position(range: Range, pos: Position) -> bool {
    let normalized = range.normalize();
    if pos.line < normalized.start.line || pos.line > normalized.end.line {
        return false;
    }
    if pos.line == normalized.start.line && pos.character < normalized.start.character {
        return false;
    }
    if pos.line == normalized.end.line && pos.character >= normalized.end.character {
        return false;
    }
    true
}

/// Check whether two ranges intersect.
pub fn ranges_intersect(a: Range, b: Range) -> bool {
    let a = a.normalize();
    let b = b.normalize();
    // a is entirely before b
    if a.end.line < b.start.line {
        return false;
    }
    if a.end.line == b.start.line && a.end.character <= b.start.character {
        return false;
    }
    // b is entirely before a
    if b.end.line < a.start.line {
        return false;
    }
    if b.end.line == a.start.line && b.end.character <= a.start.character {
        return false;
    }
    true
}

/// Check if a node's byte range matches the given external `Range` within
/// a small tolerance (for practical position→byte→position round-trip drift).
pub fn node_range_matches(
    source: &str,
    range: Range,
    node_start_byte: usize,
    node_end_byte: usize,
) -> Result<bool, AstToolError> {
    let (byte_start, byte_end) = validate_range_in_bounds(source, range)?;
    // Allow 1-byte tolerance for position encoding edge cases
    let start_ok = byte_start.abs_diff(node_start_byte) <= 1;
    let end_ok = byte_end.abs_diff(node_end_byte) <= 1;
    Ok(start_ok && end_ok)
}
