use crate::frameworks::{confidence_medium, AstDetector, AstFileContext};
use crate::parser::positions::ts_point_to_position;
use crate::shared::position::Range;
use crate::shared::types_v3::{AstRoute, Evidence};

pub struct DjangoRouteDetector;

impl AstDetector<AstRoute> for DjangoRouteDetector {
    fn detect(&self, ctx: &AstFileContext) -> Vec<AstRoute> {
        let mut routes = Vec::new();
        collect_django_routes(ctx.tree.root_node(), ctx, &mut routes);
        routes
    }
}

const URL_FUNCTIONS: &[&str] = &["path", "re_path", "url"];

fn collect_django_routes(
    node: tree_sitter::Node,
    ctx: &AstFileContext,
    routes: &mut Vec<AstRoute>,
) {
    if node.kind() == "call" {
        if let Some(route) = extract_django_call(&node, ctx) {
            routes.push(route);
        }
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_django_routes(child, ctx, routes);
        }
    }
}

fn extract_django_call(node: &tree_sitter::Node, ctx: &AstFileContext) -> Option<AstRoute> {
    let func = node.child_by_field_name("function")?;
    let func_name = func.utf8_text(ctx.source.as_bytes()).unwrap_or("");

    if !URL_FUNCTIONS.contains(&func_name) {
        return None;
    }

    let args = node.child_by_field_name("arguments")?;
    // First argument is the path pattern
    let path = first_string_arg(&args, ctx)?;

    let range = node_range(node, ctx);
    let evidence = make_evidence("call", node, ctx);

    Some(AstRoute {
        file_path: ctx.relative_path.to_string(),
        language: ctx.language.to_string(),
        framework: "django".to_string(),
        method: None,
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
            if arg.kind() == "string" {
                let raw = arg.utf8_text(ctx.source.as_bytes()).unwrap_or("");
                return Some(raw.trim_matches(|c| c == '"' || c == '\'').to_string());
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
