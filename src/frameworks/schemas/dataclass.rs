use crate::frameworks::{confidence_high, AstDetector, AstFileContext};
use crate::parser::positions::ts_point_to_position;
use crate::shared::position::Range;
use crate::shared::types_v3::{AstSchemaDefinition, AstSchemaField, Evidence};

pub struct DataclassSchemaDetector;

impl AstDetector<AstSchemaDefinition> for DataclassSchemaDetector {
    fn detect(&self, ctx: &AstFileContext) -> Vec<AstSchemaDefinition> {
        let mut schemas = Vec::new();
        collect_dataclasses(ctx.tree.root_node(), ctx, &mut schemas);
        schemas
    }
}

fn collect_dataclasses(
    node: tree_sitter::Node,
    ctx: &AstFileContext,
    results: &mut Vec<AstSchemaDefinition>,
) {
    if node.kind() == "decorated_definition" {
        if let Some(def) = node.child_by_field_name("definition") {
            if def.kind() == "class_definition" && has_dataclass_decorator(&node, ctx) {
                let name = def
                    .child_by_field_name("name")
                    .and_then(|n| n.utf8_text(ctx.source.as_bytes()).ok())
                    .map(|s| s.to_string())
                    .unwrap_or_default();

                let fields = extract_pydantic_fields(&def, ctx);
                let range = node_range(&node, ctx);
                let evidence = make_evidence("decorated_definition", &node, ctx);

                results.push(AstSchemaDefinition {
                    file_path: ctx.relative_path.to_string(),
                    language: ctx.language.to_string(),
                    kind: "dataclass".to_string(),
                    name: Some(name),
                    framework: Some("dataclass".to_string()),
                    fields,
                    range,
                    confidence: confidence_high(),
                    evidence: vec![evidence],
                });
                return;
            }
        }
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_dataclasses(child, ctx, results);
        }
    }
}

fn has_dataclass_decorator(node: &tree_sitter::Node, ctx: &AstFileContext) -> bool {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "decorator" {
                let text = child.utf8_text(ctx.source.as_bytes()).unwrap_or("");
                if text.contains("dataclass") || text.contains("dataclasses.dataclass") {
                    return true;
                }
            }
        }
    }
    false
}

fn extract_pydantic_fields(node: &tree_sitter::Node, ctx: &AstFileContext) -> Vec<AstSchemaField> {
    let body = match node.child_by_field_name("body") {
        Some(b) => b,
        None => return vec![],
    };
    let mut fields = Vec::new();
    for i in 0..body.child_count() {
        if let Some(child) = body.child(i) {
            if child.kind() == "expression_statement" {
                if let Some(assign) = child.child(0) {
                    if assign.kind() == "assignment" {
                        if let Some(left) = assign.child_by_field_name("left") {
                            if let Some(name_node) = left.child_by_field_name("name") {
                                let name = name_node
                                    .utf8_text(ctx.source.as_bytes())
                                    .unwrap_or("")
                                    .to_string();
                                let type_text = left
                                    .child_by_field_name("type")
                                    .and_then(|t| t.utf8_text(ctx.source.as_bytes()).ok())
                                    .map(|s| s.to_string());
                                let range = Some(node_range(&assign, ctx));
                                fields.push(AstSchemaField {
                                    name,
                                    type_text,
                                    required: Some(true),
                                    range,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    fields
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
