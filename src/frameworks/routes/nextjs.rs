use crate::frameworks::{confidence_medium, AstDetector, AstFileContext};
use crate::parser::positions::ts_point_to_position;
use crate::shared::position::Range;
use crate::shared::types_v3::{AstRoute, Evidence};

const HTTP_METHODS: &[&str] = &["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

pub struct NextJsRouteDetector;

impl AstDetector<AstRoute> for NextJsRouteDetector {
    fn detect(&self, ctx: &AstFileContext) -> Vec<AstRoute> {
        let mut routes = Vec::new();
        collect_exports(ctx.tree.root_node(), ctx, &mut routes);
        routes
    }
}

fn collect_exports(node: tree_sitter::Node, ctx: &AstFileContext, routes: &mut Vec<AstRoute>) {
    match node.kind() {
        "export_statement" => {
            // Look for exported function declarations or lexical declarations
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    if child.kind() == "function_declaration" {
                        if let Some(route) = extract_fn_decl(&child, ctx, true) {
                            routes.push(route);
                        }
                    } else if child.kind() == "lexical_declaration" {
                        for j in 0..child.child_count() {
                            if let Some(inner) = child.child(j) {
                                if inner.kind() == "variable_declarator" {
                                    if let Some(route) = extract_var_decl(&inner, ctx, true) {
                                        routes.push(route);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            return;
        }
        "function_declaration" => {
            if let Some(route) = extract_fn_decl(&node, ctx, false) {
                routes.push(route);
            }
            return;
        }
        "lexical_declaration" => {
            for j in 0..node.child_count() {
                if let Some(inner) = node.child(j) {
                    if inner.kind() == "variable_declarator" {
                        if let Some(route) = extract_var_decl(&inner, ctx, false) {
                            routes.push(route);
                        }
                    }
                }
            }
            return;
        }
        _ => {}
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_exports(child, ctx, routes);
        }
    }
}

fn extract_fn_decl(
    node: &tree_sitter::Node,
    ctx: &AstFileContext,
    exported: bool,
) -> Option<AstRoute> {
    let name_node = node.child_by_field_name("name")?;
    let name = name_node.utf8_text(ctx.source.as_bytes()).unwrap_or("");

    if !HTTP_METHODS.contains(&name) {
        return None;
    }

    let range = node_range(node, ctx);
    let evidence = make_evidence("function_declaration", node, ctx);

    // Infer path from file path convention
    let (path, confidence) = infer_path_from_file(ctx);

    Some(AstRoute {
        file_path: ctx.relative_path.to_string(),
        language: ctx.language.to_string(),
        framework: "nextjs".to_string(),
        method: Some(name.to_lowercase()),
        path,
        handler_name: Some(name.to_string()),
        handler_kind: Some(if exported {
            "exported_function".to_string()
        } else {
            "function".to_string()
        }),
        range,
        path_range: None,
        handler_range: Some(range),
        confidence,
        evidence: vec![evidence],
    })
}

fn extract_var_decl(
    node: &tree_sitter::Node,
    ctx: &AstFileContext,
    _exported: bool,
) -> Option<AstRoute> {
    // const GET = async (req) => { ... }
    let name_node = node.child_by_field_name("name")?;
    let name = name_node.utf8_text(ctx.source.as_bytes()).unwrap_or("");

    if !HTTP_METHODS.contains(&name) {
        return None;
    }

    let range = node_range(node, ctx);
    let evidence = make_evidence("variable_declarator", node, ctx);
    let (path, confidence) = infer_path_from_file(ctx);

    Some(AstRoute {
        file_path: ctx.relative_path.to_string(),
        language: ctx.language.to_string(),
        framework: "nextjs".to_string(),
        method: Some(name.to_lowercase()),
        path,
        handler_name: Some(name.to_string()),
        handler_kind: Some("arrow_function".to_string()),
        range,
        path_range: None,
        handler_range: Some(range),
        confidence,
        evidence: vec![evidence],
    })
}

fn infer_path_from_file(
    ctx: &AstFileContext,
) -> (Option<String>, crate::shared::types_v3::Confidence) {
    // app/**/route.ts -> path inferred from directory structure
    let rel = ctx.relative_path;

    if rel.contains("app/") && rel.ends_with("route.ts")
        || rel.ends_with("route.tsx")
        || rel.ends_with("route.js")
        || rel.ends_with("route.jsx")
    {
        // app/users/[id]/route.ts -> /users/:id
        let path = rel
            .replace("app/", "")
            .replace("/route.ts", "")
            .replace("/route.tsx", "")
            .replace("/route.js", "")
            .replace("/route.jsx", "")
            .replace("[", ":")
            .replace("]", "");
        (Some(format!("/{}", path)), confidence_medium())
    } else if rel.contains("pages/api/") {
        let path = rel
            .split("pages/api/")
            .nth(1)
            .unwrap_or("")
            .replace(".ts", "")
            .replace(".tsx", "")
            .replace(".js", "")
            .replace(".jsx", "")
            .replace("[", ":")
            .replace("]", "")
            .replace("/index", "");
        (Some(format!("/api/{}", path)), confidence_medium())
    } else {
        (None, confidence_medium())
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
