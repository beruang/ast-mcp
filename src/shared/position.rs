use serde::{Deserialize, Serialize};

/// 0-based line/character position. Character is counted in UTF-16 code units
/// (BMP = 1, surrogate pairs = 2), matching the LSP model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

/// Inclusive range between two positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub fn normalize(self) -> Self {
        if self.start.line < self.end.line
            || (self.start.line == self.end.line && self.start.character <= self.end.character)
        {
            self
        } else {
            Range {
                start: self.end,
                end: self.start,
            }
        }
    }
}

/// Ensure `start ≤ end` (line-major, then character).
pub fn normalize_range(r: Range) -> Range {
    r.normalize()
}
