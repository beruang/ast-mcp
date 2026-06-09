use crate::frameworks::{confidence_high, AstDetector, AstFileContext};
use crate::parser::positions::ts_point_to_position;
use crate::shared::position::Range;
use crate::shared::types_v3::{AstDependencyEdge, EdgeKind, Evidence};

pub struct RustDependencyDetector;

impl AstDetector<AstDependencyEdge> for RustDependencyDetector {
    fn detect(&self, ctx: &AstFileContext) -> Vec<AstDependencyEdge> {
        let mut edges = Vec::new();
        collect_edges(ctx.tree.root_node(), ctx, &mut edges);
        edges
    }
}

fn collect_edges(
    node: tree_sitter::Node,
    ctx: &AstFileContext,
    edges: &mut Vec<AstDependencyEdge>,
) {
    match node.kind() {
        "use_declaration" => {
            if let Some(edge) = extract_use_declaration(&node, ctx) {
                edges.push(edge);
            }
            return;
        }
        "mod_item" => {
            if let Some(edge) = extract_mod_item(&node, ctx) {
                edges.push(edge);
            }
            return;
        }
        "extern_crate_declaration" => {
            if let Some(edge) = extract_extern_crate(&node, ctx) {
                edges.push(edge);
            }
            return;
        }
        _ => {}
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_edges(child, ctx, edges);
        }
    }
}

fn extract_use_declaration(
    node: &tree_sitter::Node,
    ctx: &AstFileContext,
) -> Option<AstDependencyEdge> {
    // Collect the full use path
    let path = collect_use_path(node, ctx);
    if path.is_empty() {
        return None;
    }

    let range = node_range(node, ctx);
    // crate:: / super:: / self:: are relative; bare identifiers are external
    let is_relative =
        path.starts_with("crate::") || path.starts_with("super::") || path.starts_with("self::");
    let evidence = make_edge_evidence("use_declaration", node, ctx);

    Some(AstDependencyEdge {
        from_file: ctx.relative_path.to_string(),
        to_specifier: path,
        kind: EdgeKind::Use,
        is_relative,
        is_type_only: None,
        range,
        confidence: confidence_high(),
        evidence: vec![evidence],
    })
}

fn collect_use_path(node: &tree_sitter::Node, ctx: &AstFileContext) -> String {
    let mut parts: Vec<String> = Vec::new();
    collect_use_parts(node, ctx, &mut parts);
    parts.join("::")
}

fn collect_use_parts(node: &tree_sitter::Node, ctx: &AstFileContext, parts: &mut Vec<String>) {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            match child.kind() {
                "identifier" | "scoped_identifier" => {
                    if let Ok(text) = child.utf8_text(ctx.source.as_bytes()) {
                        parts.push(text.to_string());
                    }
                }
                "scoped_use_list" | "use_list" => {
                    // Don't recurse into lists — they don't add to the prefix
                }
                _ => {
                    collect_use_parts(&child, ctx, parts);
                }
            }
        }
    }
}

fn extract_mod_item(node: &tree_sitter::Node, ctx: &AstFileContext) -> Option<AstDependencyEdge> {
    let mut name = String::new();
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "identifier" {
                name = child.utf8_text(ctx.source.as_bytes()).unwrap_or("").to_string();
                break;
            }
        }
    }

    if name.is_empty() {
        return None;
    }

    let range = node_range(node, ctx);
    let evidence = make_edge_evidence("mod_item", node, ctx);

    Some(AstDependencyEdge {
        from_file: ctx.relative_path.to_string(),
        to_specifier: name,
        kind: EdgeKind::Mod,
        is_relative: true,
        is_type_only: None,
        range,
        confidence: confidence_high(),
        evidence: vec![evidence],
    })
}

fn extract_extern_crate(
    node: &tree_sitter::Node,
    ctx: &AstFileContext,
) -> Option<AstDependencyEdge> {
    let mut name = String::new();
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "identifier" {
                name = child.utf8_text(ctx.source.as_bytes()).unwrap_or("").to_string();
                break;
            }
        }
    }

    if name.is_empty() {
        return None;
    }

    let range = node_range(node, ctx);
    let evidence = make_edge_evidence("extern_crate_declaration", node, ctx);

    Some(AstDependencyEdge {
        from_file: ctx.relative_path.to_string(),
        to_specifier: name,
        kind: EdgeKind::Package,
        is_relative: false,
        is_type_only: None,
        range,
        confidence: confidence_high(),
        evidence: vec![evidence],
    })
}

fn node_range(node: &tree_sitter::Node, ctx: &AstFileContext) -> Range {
    let start = ts_point_to_position(node.start_position(), ctx.source);
    let end = ts_point_to_position(node.end_position(), ctx.source);
    Range { start, end }
}

fn make_edge_evidence(kind: &str, node: &tree_sitter::Node, ctx: &AstFileContext) -> Evidence {
    let text = node.utf8_text(ctx.source.as_bytes()).ok().map(|t| t.to_string());
    let range = Some(node_range(node, ctx));
    crate::frameworks::make_evidence(kind, text.as_deref(), range, Some(node.kind()))
}
