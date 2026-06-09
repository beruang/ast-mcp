use crate::frameworks::{confidence_high, AstDetector, AstFileContext};
use crate::parser::positions::ts_point_to_position;
use crate::shared::position::Range;
use crate::shared::types_v3::{AstTestItem, Evidence, TestKind};

pub struct RustTestDetector;

impl AstDetector<AstTestItem> for RustTestDetector {
    fn detect(&self, ctx: &AstFileContext) -> Vec<AstTestItem> {
        let mut items = Vec::new();
        collect_tests(ctx.tree.root_node(), ctx, &mut items);
        items
    }
}

fn collect_tests(node: tree_sitter::Node, ctx: &AstFileContext, items: &mut Vec<AstTestItem>) {
    if node.kind() == "function_item" {
        if has_test_attribute(&node, ctx) {
            let name = get_fn_name(&node, ctx).unwrap_or_default();
            let range = node_range(&node, ctx);
            let evidence = make_evidence("function_item", &node, ctx);
            items.push(AstTestItem {
                file_path: ctx.relative_path.to_string(),
                language: ctx.language.to_string(),
                framework: "rust_test".to_string(),
                kind: TestKind::Test,
                name: Some(name),
                range,
                parent_name: None,
                confidence: confidence_high(),
                evidence: vec![evidence],
            });
        }
        return;
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_tests(child, ctx, items);
        }
    }
}

fn has_test_attribute(node: &tree_sitter::Node, ctx: &AstFileContext) -> bool {
    // In tree-sitter-rust, attributes are on the parent (function_item)
    // or may be previous siblings. Check for attribute_item children.
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "attribute_item" {
                let text = child.utf8_text(ctx.source.as_bytes()).unwrap_or("");
                if text.contains("test") {
                    return true;
                }
            }
        }
    }
    false
}

fn get_fn_name(node: &tree_sitter::Node, ctx: &AstFileContext) -> Option<String> {
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
