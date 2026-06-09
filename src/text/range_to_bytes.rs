//! Range validation helpers.
//!
//! Every V2 tool that accepts a `Range` must validate it before use.
//! This module provides the canonical validation entry-points.

use crate::shared::errors::AstToolError;
use crate::shared::position::Range;

/// Reject ranges where start > end (even after normalization, an explicitly
/// reversed range is still usable — we just normalize and proceed).
///
/// This is the light-weight check for tools that only need a valid ordering.
pub fn ensure_valid_range(range: Range) -> Result<Range, AstToolError> {
    let n = range.normalize();
    Ok(n)
}

/// Ensure the byte-extent of `range` within `source` does not exceed `max_bytes`.
///
/// Returns `ContextBudgetExceeded` when the range covers more bytes than
/// allowed.
pub fn ensure_range_within_budget(
    source: &str,
    range: Range,
    max_bytes: usize,
) -> Result<(usize, usize), AstToolError> {
    let (start, end) = crate::text::position_encoding::range_to_byte_range(source, range)?;
    if end - start > max_bytes {
        return Err(AstToolError::ContextBudgetExceeded);
    }
    Ok((start, end))
}

/// Slice `source` by `range` and return the substring, truncated to `max_bytes`.
pub fn slice_range(source: &str, start_byte: usize, end_byte: usize) -> &str {
    let start = start_byte.min(source.len());
    let end = end_byte.min(source.len());
    if start >= end {
        return "";
    }
    // Walk to a valid char boundary if needed.
    let mut s = start;
    while s > 0 && !source.is_char_boundary(s) {
        s -= 1;
    }
    let mut e = end;
    while e < source.len() && !source.is_char_boundary(e) {
        e += 1;
    }
    &source[s..e.min(source.len())]
}
