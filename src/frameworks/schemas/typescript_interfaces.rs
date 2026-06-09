use crate::frameworks::{confidence_medium, AstDetector, AstFileContext};
use crate::parser::positions::ts_point_to_position;
use crate::shared::position::Range;
use crate::shared::types_v3::{AstSchemaDefinition, AstSchemaField, Evidence};

pub struct TypeScriptInterfaceDetector;

impl AstDetector<AstSchemaDefinition> for TypeScriptInterfaceDetector {
    fn detect(&self, ctx: &AstFileContext) -> Vec<AstSchemaDefinition> {
        let mut schemas = Vec::new();
        collect_interfaces(ctx.tree.root_node(), ctx, &mut schemas);
        schemas
    }
}

fn collect_interfaces(
    node: tree_sitter::Node,
    ctx: &AstFileContext,
    results: &mut Vec<AstSchemaDefinition>,
) {
    match node.kind() {
        "interface_declaration" => {
            if let Some(schema) = extract_interface(&node, ctx) {
                results.push(schema);
            }
            return;
        }
        "type_alias_declaration" => {
            if let Some(schema) = extract_type_alias(&node, ctx) {
                results.push(schema);
            }
            return;
        }
        _ => {}
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_interfaces(child, ctx, results);
        }
    }
}

fn extract_interface(
    node: &tree_sitter::Node,
    ctx: &AstFileContext,
) -> Option<AstSchemaDefinition> {
    let name = node
        .child_by_field_name("name")
        .and_then(|n| n.utf8_text(ctx.source.as_bytes()).ok())
        .map(|s| s.to_string())?;

    let body = node.child_by_field_name("body")?;
    let fields = extract_members(&body, ctx);
    let range = node_range(node, ctx);
    let evidence = make_evidence("interface_declaration", node, ctx);

    Some(AstSchemaDefinition {
        file_path: ctx.relative_path.to_string(),
        language: ctx.language.to_string(),
        kind: "typescript_interface".to_string(),
        name: Some(name),
        framework: Some("typescript".to_string()),
        fields,
        range,
        confidence: confidence_medium(),
        evidence: vec![evidence],
    })
}

fn extract_type_alias(
    node: &tree_sitter::Node,
    ctx: &AstFileContext,
) -> Option<AstSchemaDefinition> {
    let name = node
        .child_by_field_name("name")
        .and_then(|n| n.utf8_text(ctx.source.as_bytes()).ok())
        .map(|s| s.to_string())?;

    // Check if the type is an object type literal
    let value = node.child_by_field_name("value")?;
    if value.kind() != "object_type" {
        return None;
    }

    let fields = extract_members(&value, ctx);
    let range = node_range(node, ctx);
    let evidence = make_evidence("type_alias_declaration", node, ctx);

    Some(AstSchemaDefinition {
        file_path: ctx.relative_path.to_string(),
        language: ctx.language.to_string(),
        kind: "typescript_type".to_string(),
        name: Some(name),
        framework: Some("typescript".to_string()),
        fields,
        range,
        confidence: confidence_medium(),
        evidence: vec![evidence],
    })
}

fn extract_members(body: &tree_sitter::Node, ctx: &AstFileContext) -> Vec<AstSchemaField> {
    let mut fields = Vec::new();
    for i in 0..body.child_count() {
        if let Some(member) = body.child(i) {
            if member.kind() == "property_signature" || member.kind() == "public_field_definition" {
                let name = member
                    .child_by_field_name("name")
                    .and_then(|n| n.utf8_text(ctx.source.as_bytes()).ok())
                    .map(|s| s.to_string())
                    .unwrap_or_default();

                let type_text = member
                    .child_by_field_name("type")
                    .and_then(|t| t.utf8_text(ctx.source.as_bytes()).ok())
                    .map(|s| s.to_string());

                let optional = member.utf8_text(ctx.source.as_bytes()).unwrap_or("").contains('?');
                let range = Some(node_range(&member, ctx));

                fields.push(AstSchemaField { name, type_text, required: Some(!optional), range });
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
