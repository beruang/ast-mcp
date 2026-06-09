use crate::frameworks::{confidence_high, AstDetector, AstFileContext};
use crate::parser::positions::ts_point_to_position;
use crate::shared::position::Range;
use crate::shared::types_v3::{AstTestItem, Evidence, TestKind};

pub struct GoTestDetector;

impl AstDetector<AstTestItem> for GoTestDetector {
    fn detect(&self, ctx: &AstFileContext) -> Vec<AstTestItem> {
        let mut items = Vec::new();
        collect_tests(ctx.tree.root_node(), ctx, &mut items);
        items
    }
}

fn collect_tests(node: tree_sitter::Node, ctx: &AstFileContext, items: &mut Vec<AstTestItem>) {
    if node.kind() == "function_declaration" {
        if let Some((kind, name)) = detect_go_test(&node, ctx) {
            let range = node_range(&node, ctx);
            let evidence = make_evidence("function_declaration", &node, ctx);
            items.push(AstTestItem {
                file_path: ctx.relative_path.to_string(),
                language: ctx.language.to_string(),
                framework: "go_testing".to_string(),
                kind,
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

fn detect_go_test(node: &tree_sitter::Node, ctx: &AstFileContext) -> Option<(TestKind, String)> {
    let name_node = node.child_by_field_name("name")?;
    let name = name_node.utf8_text(ctx.source.as_bytes()).unwrap_or("").to_string();

    if name.starts_with("Test") || name.starts_with("Benchmark") || name.starts_with("Fuzz") {
        Some((TestKind::Test, name))
    } else {
        None
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
