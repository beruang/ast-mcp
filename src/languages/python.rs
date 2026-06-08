use crate::extractors::outline;
use crate::extractors::OutlineCandidate;

pub fn language() -> tree_sitter::Language {
    tree_sitter_python::language()
}

/// Return outline-significant nodes from the root of a Python file.
///
/// Matches: import_statement, import_from_statement, class_definition
/// (unwrapping decorated_definition), and function_definition (detecting
/// async functions via the "async" child).
///
/// Class bodies are recursed into for function_definition children (methods).
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
            "import_statement" | "import_from_statement" => {
                candidates.push(outline::make_candidate(&child, source));
            }
            "class_definition" => {
                candidates.push(outline::make_candidate(&child, source));
            }
            "function_definition" => {
                let mut candidate = outline::make_candidate(&child, source);
                // If the function has an "async" child, prefix the kind.
                if has_async_child(&child) {
                    candidate.kind = "async_function_definition".to_string();
                }
                candidates.push(candidate);
            }
            "decorated_definition" => {
                // Unwrap: the actual definition is the "definition" field child.
                if let Some(def) = child.child_by_field_name("definition") {
                    match def.kind() {
                        "class_definition" => {
                            candidates.push(outline::make_candidate(&def, source));
                        }
                        "function_definition" => {
                            let mut candidate = outline::make_candidate(&def, source);
                            if has_async_child(&def) {
                                candidate.kind = "async_function_definition".to_string();
                            }
                            candidates.push(candidate);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    // Recurse into class bodies for method-level outline nodes.
    let mut cursor2 = root.walk();
    for child in root.children(&mut cursor2) {
        let class_node: Option<tree_sitter::Node> = if child.kind() == "class_definition" {
            Some(child)
        } else if child.kind() == "decorated_definition" {
            child.child_by_field_name("definition")
        } else {
            None
        };

        if let Some(class_def) = class_node {
            if class_def.kind() == "class_definition" {
                // Find matching candidate
                let def_start = class_def.start_position();
                if let Some(idx) = candidates.iter().position(|c| {
                    c.kind == "class_definition" || c.kind == "async_function_definition"
                }) {
                    let class_candidate = &mut candidates[idx];
                    collect_class_members_py(&class_def, source, class_candidate);
                }
                let _ = def_start;
            }
        }
    }

    candidates
}

/// Check whether a function_definition node has an "async" child.
fn has_async_child(node: &tree_sitter::Node) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "async" {
            return true;
        }
    }
    false
}

/// Walk a class body and add function_definition children (methods) to the
/// candidate.
fn collect_class_members_py(
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
                "function_definition" => {
                    let mut method = outline::make_candidate(&child, source);
                    if has_async_child(&child) {
                        method.kind = "async_function_definition".to_string();
                    }
                    candidate.children.push(method);
                }
                "class_definition" => {
                    // Nested class.
                    candidate
                        .children
                        .push(outline::make_candidate(&child, source));
                }
                _ => {}
            }
        }
    }
}
