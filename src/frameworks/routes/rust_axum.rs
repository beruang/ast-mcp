use crate::frameworks::{confidence_medium, AstDetector, AstFileContext};
use crate::parser::positions::ts_point_to_position;
use crate::shared::position::Range;
use crate::shared::types_v3::{AstRoute, Evidence};

pub struct RustAxumRouteDetector;

impl AstDetector<AstRoute> for RustAxumRouteDetector {
    fn detect(&self, ctx: &AstFileContext) -> Vec<AstRoute> {
        let mut routes = Vec::new();
        collect_axum_routes(ctx.tree.root_node(), ctx, &mut routes);
        routes
    }
}

fn collect_axum_routes(node: tree_sitter::Node, ctx: &AstFileContext, routes: &mut Vec<AstRoute>) {
    if node.kind() == "call_expression" {
        if let Some(route) = extract_axum_call(&node, ctx) {
            routes.push(route);
        }
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_axum_routes(child, ctx, routes);
        }
    }
}

fn extract_axum_call(node: &tree_sitter::Node, ctx: &AstFileContext) -> Option<AstRoute> {
    let func = node.child_by_field_name("function")?;

    // Check for axum::routing::get / axum::routing::post etc.
    // or Router::new().route("/path", get(handler))
    let func_text = func.utf8_text(ctx.source.as_bytes()).unwrap_or("");

    // Pattern: axum::routing::get(handler) - function call
    if let Some(method) = detect_axum_method(func_text) {
        let args = node.child_by_field_name("arguments")?;
        let handler_name = args
            .child(0)
            .and_then(|a| a.utf8_text(ctx.source.as_bytes()).ok())
            .map(|s| s.to_string());

        let range = node_range(node, ctx);
        let evidence = make_evidence("call_expression", node, ctx);

        return Some(AstRoute {
            file_path: ctx.relative_path.to_string(),
            language: ctx.language.to_string(),
            framework: "axum".to_string(),
            method: Some(method),
            path: None, // path is determined by Router::route() calls, not the method itself
            handler_name,
            handler_kind: None,
            range,
            path_range: None,
            handler_range: None,
            confidence: confidence_medium(),
            evidence: vec![evidence],
        });
    }

    // Pattern: .route("/path", get(handler)) — method call on Router
    if func.kind() == "field_expression" {
        let field = func.child_by_field_name("field")?;
        let field_name = field.utf8_text(ctx.source.as_bytes()).unwrap_or("");
        if field_name == "route" {
            let args = node.child_by_field_name("arguments")?;
            let path = first_string_arg(&args, ctx);
            let mut method = None;

            // Second argument is get(handler) — extract method from the call
            for i in 0..args.child_count() {
                if let Some(arg) = args.child(i) {
                    if arg.kind() == "call_expression" {
                        if let Some(arg_func) = arg.child_by_field_name("function") {
                            let m = arg_func.utf8_text(ctx.source.as_bytes()).unwrap_or("");
                            method = detect_axum_method(m);
                        }
                    }
                }
            }

            let range = node_range(node, ctx);
            let evidence = make_evidence("method_call", node, ctx);

            return Some(AstRoute {
                file_path: ctx.relative_path.to_string(),
                language: ctx.language.to_string(),
                framework: "axum".to_string(),
                method,
                path,
                handler_name: None,
                handler_kind: None,
                range,
                path_range: None,
                handler_range: None,
                confidence: confidence_medium(),
                evidence: vec![evidence],
            });
        }
    }

    None
}

fn detect_axum_method(func_text: &str) -> Option<String> {
    let methods = ["get", "post", "put", "patch", "delete", "head", "options", "trace", "connect"];
    for m in &methods {
        if func_text.ends_with(&format!("::{}", m)) || func_text == *m {
            return Some(m.to_string());
        }
    }
    None
}

fn first_string_arg(args: &tree_sitter::Node, ctx: &AstFileContext) -> Option<String> {
    for i in 0..args.child_count() {
        if let Some(arg) = args.child(i) {
            if arg.kind() == "string_literal" || arg.kind() == "raw_string_literal" {
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
