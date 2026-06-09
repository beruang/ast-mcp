use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Generate a structural fingerprint from a sequence of node kinds.
/// Optionally normalizes identifier and literal node kinds.
pub fn fingerprint(
    node_kinds: &[&str],
    normalize_identifiers: bool,
    normalize_literals: bool,
) -> String {
    let mut hasher = DefaultHasher::new();

    for kind in node_kinds {
        let normalized = if normalize_identifiers && is_identifier_kind(kind) {
            "IDENT"
        } else if normalize_literals && is_literal_kind(kind) {
            "LITERAL"
        } else {
            kind
        };
        normalized.hash(&mut hasher);
    }

    format!("{:x}", hasher.finish())
}

fn is_identifier_kind(kind: &str) -> bool {
    matches!(
        kind,
        "identifier" | "property_identifier" | "shorthand_property_identifier" | "type_identifier"
    )
}

fn is_literal_kind(kind: &str) -> bool {
    matches!(
        kind,
        "string"
            | "number"
            | "template_string"
            | "true"
            | "false"
            | "null"
            | "string_literal"
            | "integer"
            | "float"
            | "boolean"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_fingerprints() {
        let a = &["function_declaration", "identifier", "parameters", "block"];
        let b = &["function_declaration", "identifier", "parameters", "block"];
        assert_eq!(fingerprint(a, false, false), fingerprint(b, false, false));
    }

    #[test]
    fn different_fingerprints() {
        let a = &["function_declaration", "block"];
        let b = &["class_declaration", "block"];
        assert_ne!(fingerprint(a, false, false), fingerprint(b, false, false));
    }

    #[test]
    fn normalize_identifiers_collapses() {
        let a = &["function_declaration", "identifier", "block"];
        let b = &["function_declaration", "property_identifier", "block"];
        assert_eq!(fingerprint(a, true, false), fingerprint(b, true, false));
    }

    #[test]
    fn normalize_literals_collapses() {
        let a = &["expression_statement", "string"];
        let b = &["expression_statement", "number"];
        assert_eq!(fingerprint(a, false, true), fingerprint(b, false, true));
    }
}
