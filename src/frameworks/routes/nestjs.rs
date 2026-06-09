use crate::frameworks::{confidence_high, AstDetector, AstFileContext};
use crate::parser::positions::ts_point_to_position;
use crate::shared::position::Range;
use crate::shared::types_v3::{AstRoute, Evidence};

pub struct NestJsRouteDetector;

impl AstDetector<AstRoute> for NestJsRouteDetector {
    fn detect(&self, ctx: &AstFileContext) -> Vec<AstRoute> {
        let mut routes = Vec::new();
        let mut controller_prefix = String::new();
        collect_nestjs_routes(ctx.tree.root_node(), ctx, &mut routes, &mut controller_prefix);
        routes
    }
}

const HTTP_METHODS: &[(&str, &str)] = &[
    ("Get", "get"),
    ("Post", "post"),
    ("Put", "put"),
    ("Patch", "patch"),
    ("Delete", "delete"),
    ("Head", "head"),
    ("Options", "options"),
    ("All", "all"),
];

fn collect_nestjs_routes(
    node: tree_sitter::Node,
    ctx: &AstFileContext,
    routes: &mut Vec<AstRoute>,
    controller_prefix: &mut String,
) {
    if node.kind() == "decorator" {
        let decorator_name = get_decorator_name(&node, ctx).unwrap_or_default();

        if decorator_name == "Controller" {
            // Extract controller prefix from argument
            if let Some(prefix) = extract_decorator_arg(&node, ctx) {
                *controller_prefix = prefix;
            }
        } else if HTTP_METHODS.iter().any(|(n, _)| *n == decorator_name) {
            let method =
                HTTP_METHODS.iter().find(|(n, _)| *n == decorator_name).map(|(_, m)| m.to_string());
            let path = extract_decorator_arg(&node, ctx);
            let full_path = join_paths(controller_prefix, path.as_deref());

            // Find target method
            let mut target_kind = None;
            let mut target_name = None;
            let mut target_range = None;

            if let Some(parent) = node.parent() {
                for i in 0..parent.child_count() {
                    if let Some(sibling) = parent.child(i) {
                        if sibling.id() != node.id()
                            && sibling.is_named()
                            && sibling.kind() != "decorator"
                        {
                            target_kind = Some(sibling.kind().to_string());
                            target_name = sibling
                                .child_by_field_name("name")
                                .and_then(|n| n.utf8_text(ctx.source.as_bytes()).ok())
                                .map(|s| s.to_string());
                            target_range = Some(node_range(&sibling, ctx));
                            break;
                        }
                    }
                }
            }

            let range = node_range(&node, ctx);
            let evidence = make_evidence("decorator", &node, ctx);

            routes.push(AstRoute {
                file_path: ctx.relative_path.to_string(),
                language: ctx.language.to_string(),
                framework: "nestjs".to_string(),
                method,
                path: full_path,
                handler_name: target_name,
                handler_kind: target_kind.map(|k| format!("decorated_{}", k)),
                range,
                path_range: None,
                handler_range: target_range,
                confidence: confidence_high(),
                evidence: vec![evidence],
            });
        }
        return;
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_nestjs_routes(child, ctx, routes, controller_prefix);
        }
    }
}

fn get_decorator_name(node: &tree_sitter::Node, ctx: &AstFileContext) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "identifier" {
                return child.utf8_text(ctx.source.as_bytes()).ok().map(|s| s.to_string());
            }
            if child.kind() == "call_expression" {
                if let Some(func) = child.child_by_field_name("function") {
                    return func.utf8_text(ctx.source.as_bytes()).ok().map(|s| s.to_string());
                }
            }
        }
    }
    None
}

fn extract_decorator_arg(node: &tree_sitter::Node, ctx: &AstFileContext) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "call_expression" {
                if let Some(args) = child.child_by_field_name("arguments") {
                    for j in 0..args.child_count() {
                        if let Some(arg) = args.child(j) {
                            if arg.kind() == "string" {
                                let raw = arg.utf8_text(ctx.source.as_bytes()).unwrap_or("");
                                return Some(
                                    raw.trim_matches(|c| c == '"' || c == '\'' || c == '`')
                                        .to_string(),
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn join_paths(prefix: &str, suffix: Option<&str>) -> Option<String> {
    match (prefix.is_empty(), suffix) {
        (true, None) => None,
        (true, Some(s)) => Some(s.to_string()),
        (false, None) => Some(prefix.to_string()),
        (false, Some("")) => Some(prefix.to_string()),
        (false, Some(s)) => {
            let p = prefix.trim_end_matches('/');
            let s = s.trim_start_matches('/');
            Some(format!("{}/{}", p, s))
        }
    }
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
