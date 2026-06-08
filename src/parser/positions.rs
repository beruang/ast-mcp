use crate::shared::errors::AstToolError;
use crate::shared::position::{Position, Range};

/// Cache of byte offsets where each line begins.
pub struct LineIndex {
    starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(source: &str) -> Self {
        let mut starts = vec![0usize];
        for (i, b) in source.bytes().enumerate() {
            if b == b'\n' {
                starts.push(i + 1);
            }
        }
        Self { starts }
    }

    /// Convert a byte offset into a 0-based line/character position.
    /// Character is counted in UTF-16 code units.
    /// Non-char-boundary offsets are clamped backward to the nearest boundary.
    pub fn byte_to_position(&self, source: &str, byte: usize) -> Position {
        let mut byte = byte.min(source.len());
        while byte > 0 && !source.is_char_boundary(byte) {
            byte -= 1;
        }
        let line = match self.starts.binary_search(&byte) {
            Ok(line) => line,
            Err(line) => line.saturating_sub(1),
        };
        let line_start = self.starts[line];
        let character = utf16_len(&source[line_start..byte]);
        Position {
            line: line as u32,
            character: character as u32,
        }
    }

    /// Convert a Position to a byte offset. Returns an error if the position
    /// is out of bounds.
    pub fn position_to_byte(&self, source: &str, pos: Position) -> Result<usize, AstToolError> {
        let line = pos.line as usize;
        if line >= self.starts.len() {
            return Err(AstToolError::InvalidPosition(format!(
                "line {} out of bounds (max {})",
                line,
                self.starts.len().saturating_sub(1)
            )));
        }
        let line_start = self.starts[line];
        let line_end = if line + 1 < self.starts.len() {
            self.starts[line + 1]
        } else {
            source.len()
        };
        let line_slice = &source[line_start..line_end];
        let target_utf16 = pos.character as usize;
        let mut utf16_count: usize = 0;
        let mut byte_offset: usize = 0;
        for ch in line_slice.chars() {
            if utf16_count >= target_utf16 {
                break;
            }
            utf16_count += ch.len_utf16();
            byte_offset += ch.len_utf8();
        }
        if utf16_count < target_utf16 {
            return Err(AstToolError::InvalidPosition(format!(
                "character {} out of bounds on line {}",
                pos.character, line
            )));
        }
        Ok(line_start + byte_offset)
    }
}

/// Count UTF-16 code units in a string slice.
fn utf16_len(s: &str) -> usize {
    s.chars().map(|c| c.len_utf16()).sum()
}

/// Build a fresh LineIndex and convert a byte offset to a Position.
pub fn byte_offset_to_position(source: &str, byte: usize) -> Position {
    let index = LineIndex::new(source);
    index.byte_to_position(source, byte)
}

/// Build a fresh LineIndex and convert a Position to a byte offset.
pub fn position_to_byte_offset(source: &str, pos: Position) -> Result<usize, AstToolError> {
    let index = LineIndex::new(source);
    index.position_to_byte(source, pos)
}

/// Convert a Tree-sitter Point (row + byte-column) to a UTF-16 Position.
pub fn ts_point_to_position(p: tree_sitter::Point, source: &str) -> Position {
    let index = LineIndex::new(source);
    let line_start = if p.row < index.starts.len() {
        index.starts[p.row]
    } else {
        source.len()
    };
    let byte_offset = (line_start + p.column).min(source.len());
    index.byte_to_position(source, byte_offset)
}

/// Convert a Range (UTF-16 positions) to byte offsets.
pub fn range_to_byte_range(source: &str, r: Range) -> Result<(usize, usize), AstToolError> {
    let index = LineIndex::new(source);
    let start = index.position_to_byte(source, r.start)?;
    let end = index.position_to_byte(source, r.end)?;
    Ok((start, end))
}
