//! Unified diff generation via the `similar` crate.

use similar::TextDiff;

use crate::shared::errors::AstToolError;

/// Generate a unified diff between `original` and `modified`.
/// `file_path` is used in the diff header.
/// If the diff exceeds `max_bytes`, returns a `DiffTooLarge` error.
pub fn generate_diff(
    file_path: &str,
    original: &str,
    modified: &str,
    max_bytes: u64,
) -> Result<String, AstToolError> {
    let diff = TextDiff::from_lines(original, modified);
    let unified = diff
        .unified_diff()
        .context_radius(3)
        .header(&format!("--- {}", file_path), &format!("+++ {}", file_path))
        .to_string();

    if unified.len() as u64 > max_bytes {
        return Err(AstToolError::RewriteDiffTooLarge(format!(
            "diff for {} is {} bytes, limit is {}",
            file_path,
            unified.len(),
            max_bytes
        )));
    }

    Ok(unified)
}

/// Generate a combined unified diff across multiple file changes.
pub fn generate_multi_diff(
    changes: &[(String, String, String)], // (file_path, original, modified)
    max_bytes: u64,
) -> Result<String, AstToolError> {
    let mut combined = String::new();
    for (file_path, original, modified) in changes {
        if !combined.is_empty() {
            combined.push('\n');
        }
        let file_diff = generate_diff(file_path, original, modified, max_bytes)?;
        combined.push_str(&file_diff);
    }
    if combined.len() as u64 > max_bytes {
        return Err(AstToolError::RewriteDiffTooLarge(format!(
            "combined diff is {} bytes, limit is {}",
            combined.len(),
            max_bytes
        )));
    }
    Ok(combined)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_single_line_change() -> Result<(), Box<dyn std::error::Error>> {
        let diff = generate_diff("test.ts", "hello\n", "world\n", 1024)?;
        assert!(diff.contains("--- test.ts"));
        assert!(diff.contains("+++ test.ts"));
        assert!(diff.contains("-hello"));
        assert!(diff.contains("+world"));
        Ok(())
    }

    #[test]
    fn diff_too_large() {
        let result = generate_diff("x.ts", "a", "b", 1);
        assert!(result.is_err());
    }
}
