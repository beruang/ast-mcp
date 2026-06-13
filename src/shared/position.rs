use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 0-based line/character position. Character is counted in UTF-16 code units
/// (BMP = 1, surrogate pairs = 2), matching the LSP model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, JsonSchema)]
pub struct Position {
    #[serde(deserialize_with = "crate::shared::lenient::deserialize_lenient_u32")]
    pub line: u32,
    #[serde(deserialize_with = "crate::shared::lenient::deserialize_lenient_u32")]
    pub character: u32,
}

impl<'de> Deserialize<'de> for Position {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Use a helper struct so we can attach field-level deserialize_with.
        #[derive(Deserialize)]
        struct Helper {
            #[serde(deserialize_with = "crate::shared::lenient::deserialize_lenient_u32")]
            line: u32,
            #[serde(deserialize_with = "crate::shared::lenient::deserialize_lenient_u32")]
            character: u32,
        }
        let h = Helper::deserialize(deserializer)?;
        Ok(Position { line: h.line, character: h.character })
    }
}

/// Inclusive range between two positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, JsonSchema)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl<'de> Deserialize<'de> for Range {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            start: Position,
            end: Position,
        }
        let h = Helper::deserialize(deserializer)?;
        Ok(Range { start: h.start, end: h.end })
    }
}

impl Range {
    pub fn normalize(self) -> Self {
        if self.start.line < self.end.line
            || (self.start.line == self.end.line && self.start.character <= self.end.character)
        {
            self
        } else {
            Range { start: self.end, end: self.start }
        }
    }
}

/// Ensure `start ≤ end` (line-major, then character).
pub fn normalize_range(r: Range) -> Range {
    r.normalize()
}
