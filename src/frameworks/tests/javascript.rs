use crate::frameworks::{confidence_high, AstDetector, AstFileContext};
use crate::parser::positions::ts_point_to_position;
use crate::shared::position::Range;
use crate::shared::types_v3::{AstTestItem, Evidence, TestKind};

pub struct JavaScriptTestDetector;

impl AstDetector<AstTestItem> for JavaScriptTestDetector {
    fn detect(&self, ctx: &AstFileContext) -> Vec<AstTestItem> {
        let mut items = Vec::new();
        let mut suite_stack: Vec<String> = Vec::new();
        collect_tests(ctx.tree.root_node(), ctx, &mut items, &mut suite_stack);
        items
    }
}

fn collect_tests(
    node: tree_sitter::Node,
    ctx: &AstFileContext,
    items: &mut Vec<AstTestItem>,
    suite_stack: &mut Vec<String>,
) {
    if node.kind() == "call_expression" {
        if let Some((kind, name)) = detect_test_call(&node, ctx) {
            let parent_name = suite_stack.last().cloned();
            let range = node_range(&node, ctx);
            let evidence = make_evidence("call_expression", &node, ctx);
            let is_suite = kind == TestKind::Suite;
            items.push(AstTestItem {
                file_path: ctx.relative_path.to_string(),
                language: ctx.language.to_string(),
                framework: "jest".to_string(),
                kind,
                name,
                range,
                parent_name,
                confidence: confidence_high(),
                evidence: vec![evidence],
            });

            // Track suite nesting
            if is_suite {
                if let Some(ref n) = items.last().and_then(|i| i.name.clone()) {
                    suite_stack.push(n.clone());
                }
            }
        }
        // Recurse into children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                collect_tests(child, ctx, items, suite_stack);
            }
        }
        // Pop suite if we pushed one
        if kind_matches_suite(&node, ctx) {
            suite_stack.pop();
        }
        return;
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_tests(child, ctx, items, suite_stack);
        }
    }
}

fn detect_test_call(
    node: &tree_sitter::Node,
    ctx: &AstFileContext,
) -> Option<(TestKind, Option<String>)> {
    let func = node.child_by_field_name("function")?;
    let callee = func.utf8_text(ctx.source.as_bytes()).unwrap_or("");

    let kind = match callee {
        "describe" => TestKind::Suite,
        "it" | "test" => TestKind::Test,
        "beforeEach" | "afterEach" | "beforeAll" | "afterAll" => TestKind::Hook,
        _ => return None,
    };

    // Extract test name from first string argument
    let name = extract_first_string_arg(node, ctx);

    Some((kind, name))
}

fn kind_matches_suite(node: &tree_sitter::Node, ctx: &AstFileContext) -> bool {
    if let Some(func) = node.child_by_field_name("function") {
        func.utf8_text(ctx.source.as_bytes()).unwrap_or("") == "describe"
    } else {
        false
    }
}

fn extract_first_string_arg(node: &tree_sitter::Node, ctx: &AstFileContext) -> Option<String> {
    let args = node.child_by_field_name("arguments")?;
    for i in 0..args.child_count() {
        if let Some(arg) = args.child(i) {
            if arg.kind() == "string" {
                let raw = arg.utf8_text(ctx.source.as_bytes()).unwrap_or("");
                return strip_quotes(raw);
            }
            if arg.kind() == "template_string" {
                let raw = arg.utf8_text(ctx.source.as_bytes()).unwrap_or("");
                return Some(raw.trim_matches('`').to_string());
            }
        }
    }
    None
}

fn strip_quotes(s: &str) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.len() >= 2 {
        let first = trimmed.chars().next()?;
        let last = trimmed.chars().last()?;
        if (first == '"' && last == '"') || (first == '\'' && last == '\'') {
            return Some(trimmed[1..trimmed.len() - 1].to_string());
        }
    }
    Some(trimmed.to_string())
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
