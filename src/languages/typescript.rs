use crate::extractors::outline;
use crate::extractors::OutlineCandidate;

pub fn language() -> tree_sitter::Language {
    tree_sitter_typescript::language_typescript()
}

pub fn language_tsx() -> tree_sitter::Language {
    tree_sitter_typescript::language_tsx()
}

/// Return outline-significant nodes from the root of a TypeScript/TSX file.
///
/// Matches: import_statement, export_statement, function_declaration,
/// generator_function_declaration, class_declaration, interface_declaration,
/// type_alias_declaration, enum_declaration, and lexical_declaration.
///
/// `export_statement` is unwrapped so that exported declarations are
/// reported by their inner kind (e.g. `class_declaration`).
///
/// Class bodies are recursed into for method-level children.
pub fn outline_candidates(root: tree_sitter::Node, source: &str) -> Vec<OutlineCandidate> {
    let mut candidates: Vec<OutlineCandidate> = Vec::new();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor) {
        if candidates.len() >= crate::safety::limits::MAX_NODES {
            break;
        }
        if !child.is_named() {
            continue;
        }
        let kind = child.kind();
        match kind {
            "import_statement" => {
                candidates.push(outline::make_candidate(&child, source));
            }
            "export_statement" => {
                // Unwrap: report the inner declaration instead.
                if let Some(inner) = extract_inner_declaration(&child) {
                    candidates.push(outline::make_candidate(&inner, source));
                } else {
                    // Standalone re-export or default export — reported as-is.
                    candidates.push(outline::make_candidate(&child, source));
                }
            }
            "function_declaration"
            | "generator_function_declaration"
            | "class_declaration"
            | "interface_declaration"
            | "type_alias_declaration"
            | "enum_declaration" => {
                candidates.push(outline::make_candidate(&child, source));
            }
            "lexical_declaration" => {
                // Include all top-level lexical declarations (const/let/var).
                candidates.push(outline::make_candidate(&child, source));
            }
            _ => {}
        }
    }

    // Recurse into class bodies for method-level outline nodes.
    // Look at both direct class_declaration and class inside export_statement.
    let mut cursor2 = root.walk();
    for child in root.children(&mut cursor2) {
        let class_node = if child.kind() == "class_declaration" {
            Some(child)
        } else if child.kind() == "export_statement" {
            extract_inner_declaration(&child)
        } else {
            None
        };

        if let Some(class_def) = class_node {
            if class_def.kind() == "class_declaration" {
                // Find matching candidate by start position
                let def_start = class_def.start_position();
                if let Some(idx) = candidates.iter().position(|c| {
                    c.range.start
                        == crate::parser::positions::ts_point_to_position(def_start, source)
                }) {
                    let class_candidate = &mut candidates[idx];
                    collect_class_members_ts(&class_def, source, class_candidate);
                }
            }
        }
    }

    candidates
}

/// Given an `export_statement` node, return the inner declaration
/// (`class_declaration`, `function_declaration`, etc.) if present.
fn extract_inner_declaration<'a>(
    export_node: &tree_sitter::Node<'a>,
) -> Option<tree_sitter::Node<'a>> {
    for i in 0..export_node.child_count() {
        if let Some(child) = export_node.child(i) {
            if child.is_named() {
                match child.kind() {
                    "class_declaration"
                    | "function_declaration"
                    | "generator_function_declaration"
                    | "interface_declaration"
                    | "type_alias_declaration"
                    | "enum_declaration"
                    | "lexical_declaration" => {
                        return Some(child);
                    }
                    _ => {}
                }
            }
        }
    }
    None
}

/// Walk a class body and add method_definition / public_field_definition
/// children to the candidate.
fn collect_class_members_ts(
    class_node: &tree_sitter::Node,
    source: &str,
    candidate: &mut OutlineCandidate,
) {
    let body = class_node.child_by_field_name("body");
    if let Some(body) = body {
        let mut cursor = body.walk();
        for child in body.children(&mut cursor) {
            if candidate.children.len() >= crate::safety::limits::MAX_NODES {
                break;
            }
            if !child.is_named() {
                continue;
            }
            match child.kind() {
                "method_definition"
                | "public_field_definition"
                | "method_signature"
                | "property_signature"
                | "index_signature"
                | "constructor_signature" => {
                    candidate.children.push(outline::make_candidate(&child, source));
                }
                _ => {}
            }
        }
    }
}
