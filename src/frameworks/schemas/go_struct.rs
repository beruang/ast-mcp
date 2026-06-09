use crate::frameworks::{confidence_high, AstDetector, AstFileContext};
use crate::parser::positions::ts_point_to_position;
use crate::shared::position::Range;
use crate::shared::types_v3::{AstSchemaDefinition, AstSchemaField, Evidence};

pub struct GoStructDetector;

impl AstDetector<AstSchemaDefinition> for GoStructDetector {
    fn detect(&self, ctx: &AstFileContext) -> Vec<AstSchemaDefinition> {
        let mut schemas = Vec::new();
        collect_go_structs(ctx.tree.root_node(), ctx, &mut schemas);
        schemas
    }
}

fn collect_go_structs(
    node: tree_sitter::Node,
    ctx: &AstFileContext,
    results: &mut Vec<AstSchemaDefinition>,
) {
    if node.kind() == "type_declaration" {
        if let Some(type_spec) = node.child_by_field_name("type_spec") {
            if type_spec.kind() == "type_spec" {
                let name = type_spec
                    .child_by_field_name("name")
                    .and_then(|n| n.utf8_text(ctx.source.as_bytes()).ok())
                    .map(|s| s.to_string())
                    .unwrap_or_default();

                if let Some(body_node) = type_spec.child_by_field_name("body") {
                    if body_node.kind() == "field_declaration_list" {
                        let fields = extract_struct_fields(&body_node, ctx);
                        let range = node_range(&node, ctx);
                        let evidence = make_evidence("type_declaration", &node, ctx);

                        results.push(AstSchemaDefinition {
                            file_path: ctx.relative_path.to_string(),
                            language: ctx.language.to_string(),
                            kind: "go_struct".to_string(),
                            name: Some(name),
                            framework: Some("go".to_string()),
                            fields,
                            range,
                            confidence: confidence_high(),
                            evidence: vec![evidence],
                        });
                    }
                }
            }
        }
        return;
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_go_structs(child, ctx, results);
        }
    }
}

fn extract_struct_fields(body: &tree_sitter::Node, ctx: &AstFileContext) -> Vec<AstSchemaField> {
    let mut fields = Vec::new();
    for i in 0..body.child_count() {
        if let Some(child) = body.child(i) {
            if child.kind() == "field_declaration" {
                let mut field_name = String::new();
                let mut type_text = String::new();
                let mut got_name = false;

                for j in 0..child.child_count() {
                    if let Some(fc) = child.child(j) {
                        if fc.is_named() {
                            if !got_name {
                                field_name =
                                    fc.utf8_text(ctx.source.as_bytes()).unwrap_or("").to_string();
                                got_name = true;
                            } else {
                                type_text =
                                    fc.utf8_text(ctx.source.as_bytes()).unwrap_or("").to_string();
                                break;
                            }
                        }
                    }
                }

                let range = Some(node_range(&child, ctx));
                fields.push(AstSchemaField {
                    name: field_name,
                    type_text: if type_text.is_empty() { None } else { Some(type_text) },
                    required: Some(true),
                    range,
                });
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
