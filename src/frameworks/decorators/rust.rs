use crate::frameworks::{confidence_high, AstDetector, AstFileContext};
use crate::parser::positions::ts_point_to_position;
use crate::shared::position::Range;
use crate::shared::types_v3::AstDecorator;

pub struct RustDecoratorDetector;

impl AstDetector<AstDecorator> for RustDecoratorDetector {
    fn detect(&self, ctx: &AstFileContext) -> Vec<AstDecorator> {
        let mut decorators = Vec::new();
        collect_attributes(ctx.tree.root_node(), ctx, &mut decorators);
        decorators
    }
}

fn collect_attributes(
    node: tree_sitter::Node,
    ctx: &AstFileContext,
    results: &mut Vec<AstDecorator>,
) {
    if node.kind() == "attribute_item" {
        if let Some(dec) = extract_attribute(&node, ctx) {
            results.push(dec);
        }
        return;
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_attributes(child, ctx, results);
        }
    }
}

fn extract_attribute(node: &tree_sitter::Node, ctx: &AstFileContext) -> Option<AstDecorator> {
    let text = node.utf8_text(ctx.source.as_bytes()).unwrap_or("");
    // Strip #[ ... ] wrapper
    let inner = text.trim_start_matches("#[").trim_end_matches(']').trim();

    // Extract name (before first '(' if call-like, or the whole thing)
    let name = if let Some(paren) = inner.find('(') {
        inner[..paren].trim().to_string()
    } else {
        inner.to_string()
    };

    // Extract arguments
    let args_text = if let Some(paren) = inner.find('(') {
        let end = inner.rfind(')').unwrap_or(inner.len());
        if paren < end {
            vec![inner[paren + 1..end].trim().to_string()]
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    // Find target (next named sibling or parent)
    let mut target_kind = None;
    let mut target_name = None;
    let mut target_range = None;

    if let Some(parent) = node.parent() {
        for i in 0..parent.child_count() {
            if let Some(sibling) = parent.child(i) {
                if sibling.id() != node.id()
                    && sibling.is_named()
                    && sibling.kind() != "attribute_item"
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

    let range = node_range(node, ctx);
    let evidence_text = text.to_string();
    let evidence = crate::frameworks::make_evidence(
        "attribute_item",
        Some(&evidence_text),
        Some(range),
        Some(node.kind()),
    );

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

fn node_range(node: &tree_sitter::Node, ctx: &AstFileContext) -> Range {
    let start = ts_point_to_position(node.start_position(), ctx.source);
    let end = ts_point_to_position(node.end_position(), ctx.source);
    Range { start, end }
}
