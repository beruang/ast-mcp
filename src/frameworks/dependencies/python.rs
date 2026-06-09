use crate::frameworks::{confidence_high, AstDetector, AstFileContext};
use crate::parser::positions::ts_point_to_position;
use crate::shared::position::Range;
use crate::shared::types_v3::{AstDependencyEdge, EdgeKind, Evidence};

pub struct PythonDependencyDetector;

impl AstDetector<AstDependencyEdge> for PythonDependencyDetector {
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
        "import_statement" => {
            if let Some(edge) = extract_import_statement(&node, ctx) {
                edges.push(edge);
            }
            return;
        }
        "import_from_statement" => {
            if let Some(edge) = extract_import_from_statement(&node, ctx) {
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

fn extract_import_statement(
    node: &tree_sitter::Node,
    ctx: &AstFileContext,
) -> Option<AstDependencyEdge> {
    let range = node_range(node, ctx);
    let evidence = make_edge_evidence("import_statement", node, ctx);

    // Module is the first dotted_name
    let path = first_dotted_name(node, ctx).unwrap_or_default();
    let is_relative = path.starts_with('.');

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

fn extract_import_from_statement(
    node: &tree_sitter::Node,
    ctx: &AstFileContext,
) -> Option<AstDependencyEdge> {
    let range = node_range(node, ctx);
    let evidence = make_edge_evidence("import_from_statement", node, ctx);

    let path = first_dotted_name(node, ctx).unwrap_or_default();
    let is_relative = path.starts_with('.');

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

fn first_dotted_name(node: &tree_sitter::Node, ctx: &AstFileContext) -> Option<String> {
    for i in 0..node.child_count() {
        let child = node.child(i)?;
        if child.kind() == "dotted_name" {
            return child.utf8_text(ctx.source.as_bytes()).ok().map(|s| s.to_string());
        }
    }
    None
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
