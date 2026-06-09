//! Edit overlap detection.

use std::collections::HashMap;

use crate::shared::errors::AstToolError;
use crate::shared::types_v4::TextEdit;
use crate::text::position_encoding;

/// Group edits by file and detect any overlaps within the same file.
/// Returns `Ok(())` if no overlaps exist, or an error describing the first overlap found.
pub fn detect_overlaps(
    source_by_file: &HashMap<String, &str>,
    edits: &[TextEdit],
) -> Result<(), AstToolError> {
    let mut by_file: HashMap<&str, Vec<&TextEdit>> = HashMap::new();
    for edit in edits {
        by_file.entry(&edit.file_path).or_default().push(edit);
    }

    for (file_path, file_edits) in &by_file {
        let source = source_by_file.get(*file_path).copied().unwrap_or("");

        // Convert ranges to byte offsets
        let mut byte_ranges: Vec<(usize, usize)> = Vec::new();
        for edit in file_edits {
            let (start, end) = position_encoding::range_to_byte_range(source, edit.range)
                .map_err(|e| AstToolError::RewriteInvalidRange(e.to_string()))?;
            byte_ranges.push((start, end));
        }

        // Sort by start byte offset to detect overlaps
        let mut indexed: Vec<(usize, usize, usize)> =
            byte_ranges.iter().enumerate().map(|(i, &(s, e))| (s, e, i)).collect();
        indexed.sort_by_key(|&(s, _, _)| s);

        // Check for overlaps: if edit[i].end > edit[i+1].start, they overlap
        for w in indexed.windows(2) {
            let (s1, e1, _) = w[0];
            let (s2, _, _) = w[1];
            // Adjacent is fine (e1 == s2); overlap is when e1 > s2
            if e1 > s2 {
                return Err(AstToolError::RewriteOverlappingEdits(format!(
                    "overlapping edits in {}: bytes [{},{}] and [{},{}]",
                    file_path, s1, e1, s2, w[1].1
                )));
            }
        }
    }

    Ok(())
}
