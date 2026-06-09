use crate::frameworks::{confidence_high, AstDetector, AstFileContext};
use crate::parser::positions::ts_point_to_position;
use crate::shared::position::Range;
use crate::shared::types_v3::{AstDependencyEdge, EdgeKind, Evidence};

pub struct GoDependencyDetector;

impl AstDetector<AstDependencyEdge> for GoDependencyDetector {
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
    if node.kind() == "import_declaration" {
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "import_spec" {
                    if let Some(edge) = extract_import_spec(&child, ctx) {
                        edges.push(edge);
                    }
                } else if child.kind() == "import_spec_list" {
                    for j in 0..child.child_count() {
                        if let Some(spec) = child.child(j) {
                            if spec.kind() == "import_spec" {
                                if let Some(edge) = extract_import_spec(&spec, ctx) {
                                    edges.push(edge);
                                }
                            }
                        }
                    }
                }
            }
        }
        return;
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_edges(child, ctx, edges);
        }
    }
}

fn extract_import_spec(
    node: &tree_sitter::Node,
    ctx: &AstFileContext,
) -> Option<AstDependencyEdge> {
    let mut path = String::new();
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "interpreted_string_literal" {
                let raw = child.utf8_text(ctx.source.as_bytes()).unwrap_or("");
                path = raw.trim_matches('"').to_string();
            }
        }
    }

    if path.is_empty() {
        return None;
    }

    let range = node_range(node, ctx);
    let is_relative = path.starts_with('.');
    let evidence = make_edge_evidence("import_spec", node, ctx);

    Some(AstDependencyEdge {
        from_file: ctx.relative_path.to_string(),
        to_specifier: path,
        kind: EdgeKind::Import,
        is_relative,
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
