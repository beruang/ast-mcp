use crate::frameworks::{confidence_high, AstDetector, AstFileContext};
use crate::parser::positions::ts_point_to_position;
use crate::shared::position::Range;
use crate::shared::types_v3::{AstDecorator, Evidence};

pub struct TypeScriptDecoratorDetector;

impl AstDetector<AstDecorator> for TypeScriptDecoratorDetector {
    fn detect(&self, ctx: &AstFileContext) -> Vec<AstDecorator> {
        let mut decorators = Vec::new();
        collect_decorators(ctx.tree.root_node(), ctx, &mut decorators);
        decorators
    }
}

fn collect_decorators(
    node: tree_sitter::Node,
    ctx: &AstFileContext,
    results: &mut Vec<AstDecorator>,
) {
    if node.kind() == "decorator" {
        if let Some(dec) = extract_decorator(&node, ctx) {
            results.push(dec);
        }
        return;
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_decorators(child, ctx, results);
        }
    }
}

fn extract_decorator(node: &tree_sitter::Node, ctx: &AstFileContext) -> Option<AstDecorator> {
    // Decorator: @Name(...) or @Name
    let mut name = String::new();
    let mut args_text: Vec<String> = Vec::new();
    let mut target_kind = None;
    let mut target_name = None;
    let mut target_range = None;

    for i in 0..node.child_count() {
        let child = node.child(i)?;
        match child.kind() {
            "identifier" => {
                name = child.utf8_text(ctx.source.as_bytes()).unwrap_or("").to_string();
            }
            "call_expression" => {
                // @Get('/path') — name is in the function, args are in arguments
                let func = child.child_by_field_name("function");
                if let Some(f) = func {
                    name = f.utf8_text(ctx.source.as_bytes()).unwrap_or("").to_string();
                }
                let cargs = child.child_by_field_name("arguments");
                if let Some(args) = cargs {
                    let text = args.utf8_text(ctx.source.as_bytes()).unwrap_or("");
                    args_text.push(text.trim_matches(|c| c == '(' || c == ')').to_string());
                }
            }
            "arguments" => {
                // Direct arguments on decorator
                let text = child.utf8_text(ctx.source.as_bytes()).unwrap_or("");
                args_text.push(text.trim_matches(|c| c == '(' || c == ')').to_string());
            }
            _ => {}
        }
    }

    if name.is_empty() {
        return None;
    }

    // Find the target declaration (the node right after the decorator)
    if let Some(parent) = node.parent() {
        for i in 0..parent.child_count() {
            if let Some(sibling) = parent.child(i) {
                if sibling.id() != node.id() && sibling.is_named() && sibling.kind() != "decorator"
                {
                    target_kind = Some(sibling.kind().to_string());
                    target_name = node_name(&sibling, ctx);
                    target_range = Some(node_range(&sibling, ctx));
                    break;
                }
            }
        }
    }

    let range = node_range(node, ctx);
    let evidence = make_evidence("decorator", node, ctx);

    Some(AstDecorator {
        file_path: ctx.relative_path.to_string(),
        language: ctx.language.to_string(),
        name,
        arguments_text: args_text,
        target_kind,
        target_name,
        range,
        target_range,
        confidence: confidence_high(),
        evidence: vec![evidence],
    })
}

fn node_name(node: &tree_sitter::Node, ctx: &AstFileContext) -> Option<String> {
    node.child_by_field_name("name")
        .and_then(|n| n.utf8_text(ctx.source.as_bytes()).ok())
        .map(|s| s.to_string())
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
