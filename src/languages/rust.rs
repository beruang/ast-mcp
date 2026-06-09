use crate::extractors::outline;
use crate::extractors::OutlineCandidate;

pub fn language() -> tree_sitter::Language {
    tree_sitter_rust::language()
}

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
        match child.kind() {
            "function_item" | "struct_item" | "enum_item" | "trait_item" | "impl_item"
            | "use_declaration" | "mod_item" | "macro_definition" | "const_item"
            | "static_item" => {
                candidates.push(outline::make_candidate(&child, source));
            }
            _ => {}
        }
    }

    // Recurse into struct_item / enum_item for field-level children.
    let mut cursor2 = root.walk();
    for child in root.children(&mut cursor2) {
        if child.kind() == "struct_item" {
            let def_start = child.start_position();
            if let Some(idx) = candidates.iter().position(|c| {
                c.range.start == crate::parser::positions::ts_point_to_position(def_start, source)
            }) {
                let struct_candidate = &mut candidates[idx];
                collect_struct_members(&child, source, struct_candidate);
            }
        }
    }

    candidates
}

fn collect_struct_members(
    struct_item: &tree_sitter::Node,
    source: &str,
    candidate: &mut OutlineCandidate,
) {
    if let Some(body) = struct_item.child_by_field_name("body") {
        let mut cursor = body.walk();
        for child in body.children(&mut cursor) {
            if candidate.children.len() >= crate::safety::limits::MAX_NODES {
                break;
            }
            if !child.is_named() {
                continue;
            }
            if child.kind() == "field_declaration" {
                candidate.children.push(outline::make_candidate(&child, source));
            }
        }
    }
}
