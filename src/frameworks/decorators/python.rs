use crate::frameworks::{confidence_high, AstDetector, AstFileContext};
use crate::parser::positions::ts_point_to_position;
use crate::shared::position::Range;
use crate::shared::types_v3::AstDecorator;

pub struct PythonDecoratorDetector;

impl AstDetector<AstDecorator> for PythonDecoratorDetector {
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
    let text = node.utf8_text(ctx.source.as_bytes()).unwrap_or("");
    // Strip leading @
    let stripped = text.trim_start_matches('@');

    // Extract name (before first '(' or '.' if dotted)
    let name = if let Some(dot) = stripped.find('.') {
        // @pytest.fixture -> "pytest.fixture"
        if let Some(paren) = stripped.find('(') {
            let name_with_args = &stripped[..paren];
            if dot < paren {
                name_with_args.to_string()
            } else {
                stripped[..paren].to_string()
            }
        } else {
            stripped.to_string()
        }
    } else if let Some(paren) = stripped.find('(') {
        stripped[..paren].to_string()
    } else {
        stripped.to_string()
    };

    // Extract arguments
    let args_text = if let Some(paren) = stripped.find('(') {
        let end = stripped.rfind(')').unwrap_or(stripped.len());
        if paren < end {
            vec![stripped[paren + 1..end].to_string()]
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    // Try to find target declaration
    let mut target_kind = None;
    let mut target_name = None;
    let mut target_range = None;

    if let Some(parent) = node.parent() {
        // parent is decorated_definition, the target is the "definition" field child
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
    let evidence_text = text.to_string();
    let evidence = crate::frameworks::make_evidence(
        "decorator",
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
