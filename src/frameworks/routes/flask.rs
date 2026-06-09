use crate::frameworks::{confidence_high, AstDetector, AstFileContext};
use crate::parser::positions::ts_point_to_position;
use crate::shared::position::Range;
use crate::shared::types_v3::{AstRoute, Evidence};

pub struct FlaskRouteDetector;

impl AstDetector<AstRoute> for FlaskRouteDetector {
    fn detect(&self, ctx: &AstFileContext) -> Vec<AstRoute> {
        let mut routes = Vec::new();
        collect_flask_routes(ctx.tree.root_node(), ctx, &mut routes);
        routes
    }
}

fn collect_flask_routes(node: tree_sitter::Node, ctx: &AstFileContext, routes: &mut Vec<AstRoute>) {
    if node.kind() == "decorator" {
        if let Some(route) = extract_flask_decorator(&node, ctx) {
            routes.push(route);
        }
        return;
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_flask_routes(child, ctx, routes);
        }
    }
}

fn extract_flask_decorator(node: &tree_sitter::Node, ctx: &AstFileContext) -> Option<AstRoute> {
    let text = node.utf8_text(ctx.source.as_bytes()).unwrap_or("");
    let stripped = text.trim_start_matches('@').trim();

    // Detect @app.route('/path', methods=['GET', 'POST']) or @blueprint.route(...)
    if !stripped.contains(".route(") {
        return None;
    }

    let dot_pos = stripped.find(".route(")?;
    let prefix = &stripped[..dot_pos];

    // Extract path (first argument)
    let args_start = dot_pos + ".route(".len();
    let args_str = &stripped[args_start..];
    let path_end = args_str.find([',', ')'])?;
    let path = args_str[..path_end].trim().trim_matches(|c| c == '"' || c == '\'').to_string();

    if path.is_empty() {
        return None;
    }

    // Extract methods if present
    let mut methods: Vec<String> = Vec::new();
    if let Some(methods_pos) = args_str.find("methods=") {
        let after_methods = &args_str[methods_pos + 8..];
        if let Some(bracket) = after_methods.find('[') {
            let bracket_end = after_methods[bracket..].find(']').unwrap_or(after_methods.len());
            let methods_str = &after_methods[bracket + 1..bracket + bracket_end];
            for m in methods_str.split(',') {
                let clean = m.trim().trim_matches(|c| c == '"' || c == '\'');
                if !clean.is_empty() {
                    methods.push(clean.to_string());
                }
            }
        }
    }

    let framework = if prefix.contains("blueprint") {
        "flask_blueprint".to_string()
    } else {
        "flask".to_string()
    };

    // Find target function
    let mut target_name = None;
    let mut target_range = None;
    if let Some(parent) = node.parent() {
        if parent.kind() == "decorated_definition" {
            if let Some(def) = parent.child_by_field_name("definition") {
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

    // For Flask, if methods list is present, return one route per method
    if methods.is_empty() {
        methods.push("GET".to_string());
    }

    let primary_method = methods.first().cloned();
    Some(AstRoute {
        file_path: ctx.relative_path.to_string(),
        language: ctx.language.to_string(),
        framework,
        method: primary_method.map(|m| m.to_lowercase()),
        path: Some(path),
        handler_name: target_name,
        handler_kind: Some("function".to_string()),
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
