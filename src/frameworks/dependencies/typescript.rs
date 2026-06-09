use crate::frameworks::{confidence_high, AstDetector, AstFileContext};
use crate::parser::positions::ts_point_to_position;
use crate::shared::position::Range;
use crate::shared::types_v3::{AstDependencyEdge, EdgeKind, Evidence};

pub struct TypeScriptDependencyDetector;

impl AstDetector<AstDependencyEdge> for TypeScriptDependencyDetector {
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
        "export_statement" => {
            if let Some(edge) = extract_export_statement(&node, ctx) {
                edges.push(edge);
            }
            // Don't return — may contain re-exports with source
        }
        "call_expression" => {
            if let Some(edge) = extract_call_import(&node, ctx) {
                edges.push(edge);
            }
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
    let mut module_path = String::new();
    let mut is_type_only = false;
    let mut has_source = false;

    for i in 0..node.child_count() {
        let child = node.child(i)?;
        match child.kind() {
            "type" => is_type_only = true,
            "string" => {
                module_path = strip_quotes(&child, ctx.source)?;
                has_source = true;
            }
            "from_clause" => {
                for j in 0..child.child_count() {
                    if let Some(gc) = child.child(j) {
                        if gc.kind() == "string" {
                            module_path = strip_quotes(&gc, ctx.source)?;
                            has_source = true;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if !has_source {
        return None;
    }

    let range = node_range(node, ctx);
    let is_relative = module_path.starts_with('.');
    let evidence = make_edge_evidence("import_statement", node, ctx);

    Some(AstDependencyEdge {
        from_file: ctx.relative_path.to_string(),
        to_specifier: module_path,
        kind: EdgeKind::Import,
        is_relative,
        is_type_only: Some(is_type_only),
        range,
        confidence: confidence_high(),
        evidence: vec![evidence],
    })
}

fn extract_export_statement(
    node: &tree_sitter::Node,
    ctx: &AstFileContext,
) -> Option<AstDependencyEdge> {
    let mut module_path: Option<String> = None;

    for i in 0..node.child_count() {
        let child = node.child(i)?;
        if child.kind() == "from_clause" {
            for j in 0..child.child_count() {
                if let Some(gc) = child.child(j) {
                    if gc.kind() == "string" {
                        module_path = strip_quotes(&gc, ctx.source);
                    }
                }
            }
        }
    }

    let path = module_path?;
    let range = node_range(node, ctx);
    let is_relative = path.starts_with('.');
    let evidence = make_edge_evidence("export_statement", node, ctx);

    Some(AstDependencyEdge {
        from_file: ctx.relative_path.to_string(),
        to_specifier: path,
        kind: EdgeKind::Export,
        is_relative,
        is_type_only: None,
        range,
        confidence: confidence_high(),
        evidence: vec![evidence],
    })
}

fn extract_call_import(
    node: &tree_sitter::Node,
    ctx: &AstFileContext,
) -> Option<AstDependencyEdge> {
    let func = node.child_by_field_name("function")?;
    let kind = match func.kind() {
        "identifier" => {
            let name = func.utf8_text(ctx.source.as_bytes()).unwrap_or("");
            if name == "require" {
                EdgeKind::Require
            } else {
                return None;
            }
        }
        "import" => EdgeKind::Import, // dynamic import()
        _ => return None,
    };

    let args = node.child_by_field_name("arguments")?;
    let mut module_path = None;
    for i in 0..args.child_count() {
        if let Some(arg) = args.child(i) {
            if arg.kind() == "string" {
                module_path = strip_quotes(&arg, ctx.source);
                break;
            }
        }
    }

    let path = module_path?;
    let range = node_range(node, ctx);
    let is_relative = path.starts_with('.');
    let evidence = make_edge_evidence("call_expression", node, ctx);

    Some(AstDependencyEdge {
        from_file: ctx.relative_path.to_string(),
        to_specifier: path,
        kind,
        is_relative,
        is_type_only: None,
        range,
        confidence: confidence_high(),
        evidence: vec![evidence],
    })
}

fn strip_quotes(node: &tree_sitter::Node, source: &str) -> Option<String> {
    let raw = node.utf8_text(source.as_bytes()).ok()?;
    let trimmed = raw.trim();
    if trimmed.len() >= 2 {
        let first = trimmed.chars().next()?;
        let last = trimmed.chars().last()?;
        if (first == '"' && last == '"')
            || (first == '\'' && last == '\'')
            || (first == '`' && last == '`')
        {
            Some(trimmed[1..trimmed.len() - 1].to_string())
        } else {
            Some(trimmed.to_string())
        }
    } else {
        Some(trimmed.to_string())
    }
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
