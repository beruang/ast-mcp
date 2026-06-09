use std::path::Path;

use crate::shared::position::Range;
use crate::shared::types_v3::{Confidence, Evidence};

pub mod decorators;
pub mod dependencies;
pub mod react;
pub mod routes;
pub mod schemas;
pub mod tests;

/// Context passed to every detector for a single file.
pub struct AstFileContext<'a> {
    pub workspace_path: &'a Path,
    pub file_path: &'a Path,
    pub relative_path: &'a str,
    pub language: &'a str,
    pub source: &'a str,
    pub tree: &'a tree_sitter::Tree,
}

/// Common trait implemented by every framework detector.
pub trait AstDetector<T> {
    fn detect(&self, ctx: &AstFileContext) -> Vec<T>;
}

// --- Confidence helpers ---

pub fn confidence_high() -> Confidence {
    Confidence::High
}

pub fn confidence_medium() -> Confidence {
    Confidence::Medium
}

pub fn confidence_low() -> Confidence {
    Confidence::Low
}

// --- Evidence helpers ---

pub const MAX_EVIDENCE_TEXT: usize = 500;

pub fn make_evidence(
    kind: &str,
    text: Option<&str>,
    range: Option<Range>,
    node_kind: Option<&str>,
) -> Evidence {
    Evidence {
        kind: kind.to_string(),
        text: text.map(truncate_evidence_text),
        range,
        node_kind: node_kind.map(|s| s.to_string()),
    }
}

pub fn truncate_evidence_text(text: &str) -> String {
    if text.len() <= MAX_EVIDENCE_TEXT {
        text.to_string()
    } else {
        let mut end = MAX_EVIDENCE_TEXT;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        text[..end].to_string()
    }
}

// --- Naming heuristics ---

pub fn is_pascal_case(name: &str) -> bool {
    name.starts_with(|c: char| c.is_uppercase()) && !name.contains('_') && !name.starts_with('_')
}

pub fn is_camel_case(name: &str) -> bool {
    name.starts_with(|c: char| c.is_lowercase()) && !name.contains('_')
}
