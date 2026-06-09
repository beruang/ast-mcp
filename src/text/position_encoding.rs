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
