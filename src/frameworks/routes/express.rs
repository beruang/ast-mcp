use crate::frameworks::{confidence_high, AstDetector, AstFileContext};
use crate::parser::positions::ts_point_to_position;
use crate::shared::position::Range;
use crate::shared::types_v3::{AstRoute, Evidence};

pub struct ExpressRouteDetector;

impl AstDetector<AstRoute> for ExpressRouteDetector {
    fn detect(&self, ctx: &AstFileContext) -> Vec<AstRoute> {
        let mut routes = Vec::new();
        collect_express_routes(ctx.tree.root_node(), ctx, &mut routes);
        routes
    }
}

fn collect_express_routes(
    node: tree_sitter::Node,
    ctx: &AstFileContext,
    routes: &mut Vec<AstRoute>,
) {
    if node.kind() == "call_expression" {
        if let Some(route) = extract_express_call(&node, ctx) {
            routes.push(route);
        }
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_express_routes(child, ctx, routes);
        }
    }
}

fn extract_express_call(node: &tree_sitter::Node, ctx: &AstFileContext) -> Option<AstRoute> {
    // Pattern: app.get('/path', handler) or router.post('/path', handler)
    // This is a member_expression call: app.get(...)
    let func = node.child_by_field_name("function")?;

    if func.kind() != "member_expression" {
        return None;
    }

    // Get the property (method) - e.g., "get", "post", "put", etc.
    let property = func.child_by_field_name("property")?;
    let method = property.utf8_text(ctx.source.as_bytes()).unwrap_or("");
    let method_lower = method.to_lowercase();

    let is_http_method = matches!(
        method_lower.as_str(),
        "get" | "post" | "put" | "patch" | "delete" | "head" | "options" | "all" | "use"
    );

    // Also check for server.route({ method: 'GET', url: '/path', handler })
    if !is_http_method {
        // Check if it's server.route(...) or app.route(...)
        if method_lower == "route" {
            return extract_route_object_call(node, ctx);
        }
        return None;
    }

    // Extract path from first string argument
    let args = node.child_by_field_name("arguments")?;
    let path = extract_first_string(&args, ctx)?;

    let handler_name = extract_handler(&args, ctx, 1);
    let range = node_range(node, ctx);
    let path_range = find_path_range_in_args(&args, ctx);
    let handler_range = find_handler_range(&args, ctx);

    let framework = detect_express_framework(&func, ctx);
    let evidence = make_evidence("member_expression_call", node, ctx);

    Some(AstRoute {
        file_path: ctx.relative_path.to_string(),
        language: ctx.language.to_string(),
        framework,
        method: Some(method_lower),
        path: Some(path),
        handler_name,
        handler_kind: None,
        range,
        path_range,
        handler_range,
        confidence: confidence_high(),
        evidence: vec![evidence],
    })
}

fn extract_route_object_call(node: &tree_sitter::Node, ctx: &AstFileContext) -> Option<AstRoute> {
    // server.route({ method: 'GET', url: '/path', handler: fn })
    let args = node.child_by_field_name("arguments")?;
    // Find the object argument
    for i in 0..args.child_count() {
        if let Some(arg) = args.child(i) {
            if arg.kind() == "object" {
                let mut method = None;
                let mut path = None;
                let mut handler_name = None;

                // Walk object pairs
                for j in 0..arg.child_count() {
                    if let Some(pair) = arg.child(j) {
                        if pair.kind() == "pair" {
                            let key = pair.child_by_field_name("key");
                            let value = pair.child_by_field_name("value");
                            if let (Some(k), Some(v)) = (key, value) {
                                let key_str = k.utf8_text(ctx.source.as_bytes()).unwrap_or("");
                                match key_str.trim_matches(|c| c == '"' || c == '\'' || c == '`') {
                                    "method" => {
                                        method = v.utf8_text(ctx.source.as_bytes()).ok().map(|s| {
                                            s.trim_matches(|c| c == '"' || c == '\'' || c == '`')
                                                .to_lowercase()
                                        });
                                    }
                                    "url" | "path" => {
                                        path = v.utf8_text(ctx.source.as_bytes()).ok().map(|s| {
                                            s.trim_matches(|c| c == '"' || c == '\'' || c == '`')
                                                .to_string()
                                        });
                                    }
                                    "handler" => {
                                        handler_name = v
                                            .utf8_text(ctx.source.as_bytes())
                                            .ok()
                                            .map(|s| s.to_string());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }

                if let (Some(m), Some(p)) = (method, path) {
                    let range = node_range(node, ctx);
                    let evidence = make_evidence("route_object", node, ctx);
                    return Some(AstRoute {
                        file_path: ctx.relative_path.to_string(),
                        language: ctx.language.to_string(),
                        framework: "express".to_string(),
                        method: Some(m),
                        path: Some(p),
                        handler_name,
                        handler_kind: None,
                        range,
                        path_range: None,
                        handler_range: None,
                        confidence: confidence_high(),
                        evidence: vec![evidence],
                    });
                }
            }
        }
    }
    None
}

fn detect_express_framework(func: &tree_sitter::Node, ctx: &AstFileContext) -> String {
    let obj = func.child_by_field_name("object");
    if let Some(o) = obj {
        let text = o.utf8_text(ctx.source.as_bytes()).unwrap_or("");
        if text.contains("fastify") {
            return "fastify".to_string();
        }
        if text.contains("hono") || text == "c" {
            return "hono".to_string();
        }
        if text.contains("router") {
            return "express".to_string();
        }
        if text.contains("app") {
            return "express".to_string();
        }
    }
    "express".to_string()
}

fn extract_first_string(args: &tree_sitter::Node, ctx: &AstFileContext) -> Option<String> {
    for i in 0..args.child_count() {
        if let Some(arg) = args.child(i) {
            if arg.kind() == "string" || arg.kind() == "template_string" {
                let raw = arg.utf8_text(ctx.source.as_bytes()).unwrap_or("");
                let trimmed = raw.trim_matches(|c| c == '"' || c == '\'' || c == '`');
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn extract_handler(args: &tree_sitter::Node, ctx: &AstFileContext, skip: usize) -> Option<String> {
    let mut count = 0;
    for i in 0..args.child_count() {
        if let Some(arg) = args.child(i) {
            if arg.is_named() {
                if count == skip {
                    return Some(arg.utf8_text(ctx.source.as_bytes()).unwrap_or("").to_string());
                }
                count += 1;
            }
        }
    }
    None
}

fn find_path_range_in_args(args: &tree_sitter::Node, ctx: &AstFileContext) -> Option<Range> {
    for i in 0..args.child_count() {
        if let Some(arg) = args.child(i) {
            if arg.kind() == "string" || arg.kind() == "template_string" {
                return Some(node_range(&arg, ctx));
            }
        }
    }
    None
}

fn find_handler_range(args: &tree_sitter::Node, ctx: &AstFileContext) -> Option<Range> {
    let mut named = 0;
    for i in 0..args.child_count() {
        if let Some(arg) = args.child(i) {
            if arg.is_named() {
                if named == 1 {
                    return Some(node_range(&arg, ctx));
                }
                named += 1;
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
