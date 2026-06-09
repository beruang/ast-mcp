use crate::frameworks::{confidence_high, AstDetector, AstFileContext};
use crate::parser::positions::ts_point_to_position;
use crate::shared::position::Range;
use crate::shared::types_v3::{AstRoute, Evidence};

const HTTP_METHODS: &[&str] = &["get", "post", "put", "patch", "delete", "head", "options"];

pub struct FastApiRouteDetector;

impl AstDetector<AstRoute> for FastApiRouteDetector {
    fn detect(&self, ctx: &AstFileContext) -> Vec<AstRoute> {
        let mut routes = Vec::new();
        collect_decorators(ctx.tree.root_node(), ctx, &mut routes);
        routes
    }
}

fn collect_decorators(node: tree_sitter::Node, ctx: &AstFileContext, routes: &mut Vec<AstRoute>) {
    if node.kind() == "decorator" {
        if let Some(route) = extract_fastapi_decorator(&node, ctx) {
            routes.push(route);
        }
        return;
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_decorators(child, ctx, routes);
        }
    }
}

fn extract_fastapi_decorator(node: &tree_sitter::Node, ctx: &AstFileContext) -> Option<AstRoute> {
    let text = node.utf8_text(ctx.source.as_bytes()).unwrap_or("");
    let stripped = text.trim_start_matches('@');

    // Parse @app.get('/path') or @router.post('/path')
    let dot_pos = stripped.find('.')?;
    let object = &stripped[..dot_pos]; // "app" or "router"
    let rest = &stripped[dot_pos + 1..]; // "get('/path')" or "get"

    let paren_pos = rest.find('(');
    let method = paren_pos.map_or(rest, |p| &rest[..p]).to_lowercase();

    if !HTTP_METHODS.contains(&method.as_str()) {
        return None;
    }

    let path = paren_pos.and_then(|p| {
        let inner = &rest[p + 1..];
        let end = inner.rfind(')').unwrap_or(inner.len());
        let arg = inner[..end].trim();
        if arg.len() >= 2 {
            let trimmed = arg.trim_matches(|c| c == '"' || c == '\'');
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        } else {
            None
        }
    });

    let framework = if object.contains("router") { "fastapi_router" } else { "fastapi" };

    // Find target function
    let mut target_kind = None;
    let mut target_name = None;
    let mut target_range = None;

    if let Some(parent) = node.parent() {
        if parent.kind() == "decorated_definition" {
            if let Some(def) = parent.child_by_field_name("definition") {
                target_kind = Some(def.kind().to_string());
                target_name = def
                    .child_by_field_name("name")
                    .and_then(|n| n.utf8_text(ctx.source.as_bytes()).ok())
                    .map(|s| s.to_string());
                target_range = Some(node_range(&def, ctx));
            }
        }
    }

    let range = node_range(node, ctx);
    let evidence = make_evidence("decorator", node, ctx);

    Some(AstRoute {
        file_path: ctx.relative_path.to_string(),
        language: ctx.language.to_string(),
        framework: framework.to_string(),
        method: Some(method),
        path,
        handler_name: target_name,
        handler_kind: target_kind,
        range,
        path_range: None,
        handler_range: target_range,
        confidence: confidence_high(),
        evidence: vec![evidence],
    })
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
