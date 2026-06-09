//! In-memory edit application — sorts edits by descending byte offset and applies them.

use std::collections::HashMap;

use crate::shared::errors::AstToolError;
use crate::shared::types_v4::TextEdit;
use crate::text::position_encoding;

/// Apply `edits` to `source` in descending byte-offset order.
/// Edits must not overlap (validated upstream).
pub fn apply_edits(source: &str, edits: &[TextEdit]) -> Result<String, AstToolError> {
    if edits.is_empty() {
        return Ok(source.to_string());
    }

    // Convert to byte offsets and sort descending
    let mut byte_edits: Vec<(usize, usize, &str)> = edits
        .iter()
        .map(|e| {
            let (start, end) = position_encoding::range_to_byte_range(source, e.range)?;
            Ok((start, end, e.new_text.as_str()))
        })
        .collect::<Result<Vec<_>, AstToolError>>()?;

    // Sort descending by start byte (apply from end of file toward beginning)
    byte_edits.sort_by_key(|b| std::cmp::Reverse(b.0));

    // Defense-in-depth: re-check overlaps after sort
    for w in byte_edits.windows(2) {
        let (s1, e1, _) = w[0];
        let (s2, e2, _) = w[1];
        // After descending sort: s1 >= s2. If s2 < s1 and e2 > s1, overlap.
        if s2 < s1 && e2 > s1 {
            return Err(AstToolError::RewriteOverlappingEdits(format!(
                "overlap: [{},{}] and [{},{}]",
                s1, e1, s2, e2
            )));
        }
    }

    // Apply edits
    let mut result = String::with_capacity(source.len());
    let mut cursor = 0usize;

    // Collect edits in ascending order for efficient splicing
    let mut ascending: Vec<(usize, usize, &str)> = byte_edits;
    ascending.sort_by_key(|a| a.0);

    for (start, end, new_text) in &ascending {
        if *start < cursor {
            // Overlap or edit within already-modified region
            return Err(AstToolError::RewriteOverlappingEdits(
                "edit overlaps with preceding edit".into(),
            ));
        }
        result.push_str(&source[cursor..*start]);
        result.push_str(new_text);
        cursor = *end;
    }
    result.push_str(&source[cursor..]);

    Ok(result)
}

/// Apply edits to multiple files, returning modified content for each changed file.
pub fn apply_edits_multi(
    sources: &HashMap<String, &str>,
    edits: &[TextEdit],
) -> Result<HashMap<String, String>, AstToolError> {
    let mut by_file: HashMap<&str, Vec<&TextEdit>> = HashMap::new();
    for edit in edits {
        by_file.entry(&edit.file_path).or_default().push(edit);
    }

    let mut results = HashMap::new();
    for (file_path, file_edits) in &by_file {
        let source = sources
            .get(*file_path)
            .copied()
            .ok_or_else(|| AstToolError::FileNotFound((*file_path).to_string()))?;
        let owned: Vec<TextEdit> = file_edits.iter().map(|&e| e.clone()).collect();
        let modified = apply_edits(source, &owned)?;
        results.insert(file_path.to_string(), modified);
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::position::{Position, Range};

    fn pos(line: u32, character: u32) -> Position {
        Position { line, character }
    }

    #[test]
    fn apply_single_replace() -> Result<(), Box<dyn std::error::Error>> {
        let src = "hello world";
        let edits = vec![TextEdit {
            file_path: "test.txt".into(),
            range: Range { start: pos(0, 6), end: pos(0, 11) },
            new_text: "earth".into(),
        }];
        let result = apply_edits(src, &edits)?;
        assert_eq!(result, "hello earth");
        Ok(())
    }

    #[test]
    fn apply_descending_order() -> Result<(), Box<dyn std::error::Error>> {
        let src = "ABCDEFGHIJ";
        let edits = vec![
            TextEdit {
                file_path: "t".into(),
                range: Range { start: pos(0, 0), end: pos(0, 3) },
                new_text: "123".into(),
            },
            TextEdit {
                file_path: "t".into(),
                range: Range { start: pos(0, 7), end: pos(0, 10) },
                new_text: "89".into(),
            },
        ];
        let result = apply_edits(src, &edits)?;
        assert_eq!(result, "123DEFG89");
        Ok(())
    }

    #[test]
    fn apply_delete() -> Result<(), Box<dyn std::error::Error>> {
        let src = "hello world";
        let edits = vec![TextEdit {
            file_path: "t".into(),
            range: Range { start: pos(0, 5), end: pos(0, 11) },
            new_text: String::new(),
        }];
        let result = apply_edits(src, &edits)?;
        assert_eq!(result, "hello");
        Ok(())
    }
}
