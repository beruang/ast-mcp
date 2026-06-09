use crate::frameworks::{confidence_medium, AstDetector, AstFileContext};
use crate::parser::positions::ts_point_to_position;
use crate::shared::position::Range;
use crate::shared::types_v3::{AstRoute, Evidence};

pub struct GoHttpRouteDetector;

impl AstDetector<AstRoute> for GoHttpRouteDetector {
    fn detect(&self, ctx: &AstFileContext) -> Vec<AstRoute> {
        let mut routes = Vec::new();
        collect_go_routes(ctx.tree.root_node(), ctx, &mut routes);
        routes
    }
}

fn collect_go_routes(node: tree_sitter::Node, ctx: &AstFileContext, routes: &mut Vec<AstRoute>) {
    if node.kind() == "call_expression" {
        if let Some(route) = extract_go_handle_func(&node, ctx) {
            routes.push(route);
        }
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_go_routes(child, ctx, routes);
        }
    }
}

fn extract_go_handle_func(node: &tree_sitter::Node, ctx: &AstFileContext) -> Option<AstRoute> {
    let func = node.child_by_field_name("function")?;

    // Check for selector_expression: http.HandleFunc or mux.HandleFunc / mux.Handle
    if func.kind() != "selector_expression" {
        return None;
    }

    let field = func.child_by_field_name("field")?;
    let method = field.utf8_text(ctx.source.as_bytes()).unwrap_or("");

    if method != "HandleFunc" && method != "Handle" {
        return None;
    }

    let args = node.child_by_field_name("arguments")?;
    // First argument should be the path
    let path = first_string_arg(&args, ctx)?;

    let range = node_range(node, ctx);
    let evidence = make_evidence("call_expression", node, ctx);

    Some(AstRoute {
        file_path: ctx.relative_path.to_string(),
        language: ctx.language.to_string(),
        framework: "go_http".to_string(),
        method: Some(method.to_lowercase()),
        path: Some(path),
        handler_name: None,
        handler_kind: None,
        range,
        path_range: None,
        handler_range: None,
        confidence: confidence_medium(),
        evidence: vec![evidence],
    })
}

fn first_string_arg(args: &tree_sitter::Node, ctx: &AstFileContext) -> Option<String> {
    for i in 0..args.child_count() {
        if let Some(arg) = args.child(i) {
            if arg.kind() == "interpreted_string_literal" || arg.kind() == "raw_string_literal" {
                let raw = arg.utf8_text(ctx.source.as_bytes()).unwrap_or("");
                return Some(raw.trim_matches('"').to_string());
            }
        }
    }
    None
}

fn node_range(node: &tree_sitter::Node, ctx: &AstFileContext) -> Range {
    let start = ts_point_to_position(node.start_position(), ctx.source);
    let end = ts_point_to_position(node.end_position(), ctx.source);
    Range { start, end }
}

fn make_evidence(kind: &str, node: &tree_sitter::Node, ctx: &AstFileContext) -> Evidence {
    let text = node.utf8_text(ctx.source.as_bytes()).ok().map(|t| t.to_string());
    let range = Some(node_range(node, ctx));
    crate::frameworks::make_evidence(kind, text.as_deref(), range, Some(node.kind()))
}
