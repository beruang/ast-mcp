use crate::frameworks::{confidence_high, AstDetector, AstFileContext};
use crate::parser::positions::ts_point_to_position;
use crate::shared::position::Range;
use crate::shared::types_v3::{AstTestItem, Evidence, TestKind};

pub struct PythonTestDetector;

impl AstDetector<AstTestItem> for PythonTestDetector {
    fn detect(&self, ctx: &AstFileContext) -> Vec<AstTestItem> {
        let mut items = Vec::new();
        let mut class_stack: Vec<String> = Vec::new();
        collect_tests(ctx.tree.root_node(), ctx, &mut items, &mut class_stack);
        items
    }
}

fn collect_tests(
    node: tree_sitter::Node,
    ctx: &AstFileContext,
    items: &mut Vec<AstTestItem>,
    class_stack: &mut Vec<String>,
) {
    match node.kind() {
        "class_definition" => {
            let class_name = get_class_name(&node, ctx);
            let is_test_class = class_name.as_deref().is_some_and(|n| n.starts_with("Test"));

            if is_test_class {
                if let Some(ref n) = class_name {
                    class_stack.push(n.clone());
                }
                // Check if unittest TestCase — look at parent/arguments for TestCase
                let framework =
                    if is_unittest_testcase(&node, ctx) { "unittest" } else { "pytest" };
                let range = node_range(&node, ctx);
                let evidence = make_evidence("class_definition", &node, ctx);
                items.push(AstTestItem {
                    file_path: ctx.relative_path.to_string(),
                    language: ctx.language.to_string(),
                    framework: framework.to_string(),
                    kind: TestKind::Suite,
                    name: class_name.clone(),
                    range,
                    parent_name: class_stack.iter().nth_back(1).cloned(),
                    confidence: confidence_high(),
                    evidence: vec![evidence],
                });
            }

            // Recurse into class body
            if let Some(body) = node.child_by_field_name("body") {
                for i in 0..body.child_count() {
                    if let Some(child) = body.child(i) {
                        collect_tests(child, ctx, items, class_stack);
                    }
                }
            }

            if is_test_class {
                class_stack.pop();
            }
            return;
        }
        "function_definition" => {
            let name = get_function_name(&node, ctx);
            if let Some(ref n) = name {
                if n.starts_with("test_") {
                    let parent_name = class_stack.last().cloned();
                    let range = node_range(&node, ctx);
                    let evidence = make_evidence("function_definition", &node, ctx);
                    items.push(AstTestItem {
                        file_path: ctx.relative_path.to_string(),
                        language: ctx.language.to_string(),
                        framework: "pytest".to_string(),
                        kind: TestKind::Test,
                        name: Some(n.clone()),
                        range,
                        parent_name,
                        confidence: confidence_high(),
                        evidence: vec![evidence],
                    });
                }
            }
            return;
        }
        "decorated_definition" => {
            // Check for @pytest.fixture, @pytest.mark.*
            if let Some(def) = node.child_by_field_name("definition") {
                let is_fixture = has_decorator(&node, ctx, "fixture");
                let is_mark = has_decorator_starting_with(&node, ctx, "mark");

                let (kind, framework) = if is_fixture {
                    (TestKind::Fixture, "pytest")
                } else if is_mark {
                    (TestKind::Test, "pytest")
                } else {
                    // Not a test decorator — still recurse
                    for i in 0..node.child_count() {
                        if let Some(child) = node.child(i) {
                            collect_tests(child, ctx, items, class_stack);
                        }
                    }
                    return;
                };

                let name = get_function_name(&def, ctx);
                let parent_name = class_stack.last().cloned();
                let range = node_range(&node, ctx);
                let evidence = make_evidence("decorated_definition", &node, ctx);
                items.push(AstTestItem {
                    file_path: ctx.relative_path.to_string(),
                    language: ctx.language.to_string(),
                    framework: framework.to_string(),
                    kind,
                    name,
                    range,
                    parent_name,
                    confidence: confidence_high(),
                    evidence: vec![evidence],
                });
            }
            return;
        }
        _ => {}
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_tests(child, ctx, items, class_stack);
        }
    }
}

fn get_class_name(node: &tree_sitter::Node, ctx: &AstFileContext) -> Option<String> {
    node.child_by_field_name("name")
        .and_then(|n| n.utf8_text(ctx.source.as_bytes()).ok())
        .map(|s| s.to_string())
}

fn get_function_name(node: &tree_sitter::Node, ctx: &AstFileContext) -> Option<String> {
    node.child_by_field_name("name")
        .and_then(|n| n.utf8_text(ctx.source.as_bytes()).ok())
        .map(|s| s.to_string())
}

fn is_unittest_testcase(node: &tree_sitter::Node, ctx: &AstFileContext) -> bool {
    // Check argument list for TestCase
    if let Some(args) = node.child_by_field_name("superclasses") {
        let text = args.utf8_text(ctx.source.as_bytes()).unwrap_or("");
        return text.contains("TestCase");
    }
    false
}

fn has_decorator(node: &tree_sitter::Node, ctx: &AstFileContext, name: &str) -> bool {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "decorator" {
                let text = child.utf8_text(ctx.source.as_bytes()).unwrap_or("");
                // Check for @pytest.fixture or @fixture
                if text.contains(&format!("pytest.{}", name)) || text.contains(name) {
                    return true;
                }
            }
        }
    }
    false
}

fn has_decorator_starting_with(
    node: &tree_sitter::Node,
    ctx: &AstFileContext,
    prefix: &str,
) -> bool {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "decorator" {
                let text = child.utf8_text(ctx.source.as_bytes()).unwrap_or("");
                if text.contains(&format!("pytest.{}", prefix)) {
                    return true;
                }
            }
        }
    }
    false
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
