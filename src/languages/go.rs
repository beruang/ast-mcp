use crate::extractors::outline;
use crate::extractors::OutlineCandidate;

pub fn language() -> tree_sitter::Language {
    tree_sitter_go::language()
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
            "function_declaration"
            | "method_declaration"
            | "type_declaration"
            | "import_declaration" => {
                candidates.push(outline::make_candidate(&child, source));
            }
            _ => {}
        }
    }

    // Recurse into type_declaration bodies for method-level children.
    let mut cursor2 = root.walk();
    for child in root.children(&mut cursor2) {
        if child.kind() == "type_declaration" {
            if let Some(type_spec) = child.child_by_field_name("type_spec") {
                let def_start = type_spec.start_position();
                if let Some(idx) = candidates.iter().position(|c| {
                    c.range.start
                        == crate::parser::positions::ts_point_to_position(def_start, source)
                }) {
                    let type_candidate = &mut candidates[idx];
                    collect_struct_members(&type_spec, source, type_candidate);
                }
            }
        }
    }

    candidates
}

fn collect_struct_members(
    type_spec: &tree_sitter::Node,
    source: &str,
    candidate: &mut OutlineCandidate,
) {
    // type_spec -> type_identifier name; field_declaration_list body
    if let Some(body) = type_spec.child_by_field_name("body") {
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
