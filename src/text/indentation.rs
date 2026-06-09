//! Indentation extraction for decorator/wrapper insertion.

/// Extract the leading whitespace from the line containing `line_start_byte`.
/// Returns the indentation string (spaces, tabs, or empty).
pub fn extract_indentation(source: &str, line_start_byte: usize) -> &str {
    let rest = &source[line_start_byte..];
    let end = rest
        .char_indices()
        .take_while(|(_, ch)| ch.is_whitespace() && *ch != '\n' && *ch != '\r')
        .last()
        .map(|(i, ch)| i + ch.len_utf8())
        .unwrap_or(0);
    &rest[..end]
}

/// Return the indentation of the line containing `position_byte` as a String.
pub fn indentation_string(source: &str, position_byte: usize) -> String {
    let line_start = find_line_start(source, position_byte);
    extract_indentation(source, line_start).to_string()
}

/// Find the byte offset of the start of the line containing `byte`.
pub fn find_line_start(source: &str, byte: usize) -> usize {
    source[..byte].rfind('\n').map(|p| p + 1).unwrap_or(0)
}

/// Indent every line in `text` by `indent` spaces.
pub fn indent_text(text: &str, indent: &str) -> String {
    text.lines()
        .map(|line| if line.is_empty() { String::new() } else { format!("{}{}", indent, line) })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_spaces() {
        let src = "    fn foo() {\n        bar();\n    }";
        let indent = extract_indentation(src, 0);
        assert_eq!(indent, "    ");
    }

    #[test]
    fn extract_tabs() {
        let src = "\t\tfn foo() {";
        let indent = extract_indentation(src, 0);
        assert_eq!(indent, "\t\t");
    }

    #[test]
    fn extract_empty() {
        let src = "fn foo() {";
        let indent = extract_indentation(src, 0);
        assert_eq!(indent, "");
    }

    #[test]
    fn find_line_start_mid_line() -> Result<(), Box<dyn std::error::Error>> {
        let src = "line1\nline2\nline3";
        let pos = src.find("line3").ok_or("not found")?;
        assert_eq!(find_line_start(src, pos), 12); // after second \n
        Ok(())
    }

    #[test]
    fn indent_multiline() {
        let text = "hello\nworld";
        let result = indent_text(text, "  ");
        assert_eq!(result, "  hello\n  world");
    }
}
