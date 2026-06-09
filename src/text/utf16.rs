//! UTF-16 code unit helpers for position encoding.

/// Return the UTF-16 code unit length of a `char`.
/// BMP characters count as 1; supplementary-plane characters (surrogate pairs) count as 2.
pub fn utf16_len(ch: char) -> usize {
    if (ch as u32) > 0xFFFF {
        2
    } else {
        1
    }
}

/// Count the total number of UTF-16 code units in `s`.
pub fn count_utf16_code_units(s: &str) -> usize {
    s.chars().map(utf16_len).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf16_bmp_is_one() {
        assert_eq!(utf16_len('a'), 1);
        assert_eq!(utf16_len('Z'), 1);
        assert_eq!(utf16_len('0'), 1);
    }

    #[test]
    fn utf16_surrogate_pair_is_two() {
        assert_eq!(utf16_len('\u{1F600}'), 2); // 😀
        assert_eq!(utf16_len('\u{1F4A9}'), 2); // 💩
    }

    #[test]
    fn count_ascii() {
        assert_eq!(count_utf16_code_units("hello"), 5);
    }

    #[test]
    fn count_mixed() {
        // "a😀b" = 1 + 2 + 1 = 4 UTF-16 code units
        assert_eq!(count_utf16_code_units("a😀b"), 4);
    }
}
