use crate::frameworks::{confidence_high, AstDetector, AstFileContext};
use crate::parser::positions::ts_point_to_position;
use crate::shared::position::Range;
use crate::shared::types_v3::{AstSchemaDefinition, AstSchemaField, Evidence};

pub struct RustStructDetector;

impl AstDetector<AstSchemaDefinition> for RustStructDetector {
    fn detect(&self, ctx: &AstFileContext) -> Vec<AstSchemaDefinition> {
        let mut schemas = Vec::new();
        collect_rust_structs(ctx.tree.root_node(), ctx, &mut schemas);
        schemas
    }
}

fn collect_rust_structs(
    node: tree_sitter::Node,
    ctx: &AstFileContext,
    results: &mut Vec<AstSchemaDefinition>,
) {
    match node.kind() {
        "struct_item" | "enum_item" => {
            let name = node
                .child_by_field_name("name")
                .and_then(|n| n.utf8_text(ctx.source.as_bytes()).ok())
                .map(|s| s.to_string())
                .unwrap_or_default();

            let fields = if node.kind() == "struct_item" {
                extract_struct_fields(&node, ctx)
            } else {
                extract_enum_variants(&node, ctx)
            };

            let kind = if node.kind() == "struct_item" { "rust_struct" } else { "rust_enum" };
            let range = node_range(&node, ctx);
            let evidence = make_evidence(node.kind(), &node, ctx);

            results.push(AstSchemaDefinition {
                file_path: ctx.relative_path.to_string(),
                language: ctx.language.to_string(),
                kind: kind.to_string(),
                name: Some(name),
                framework: Some("rust".to_string()),
                fields,
                range,
                confidence: confidence_high(),
                evidence: vec![evidence],
            });
            return;
        }
        _ => {}
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_rust_structs(child, ctx, results);
        }
    }
}

fn extract_struct_fields(node: &tree_sitter::Node, ctx: &AstFileContext) -> Vec<AstSchemaField> {
    let body = match node.child_by_field_name("body") {
        Some(b) => b,
        None => return vec![],
    };

    let mut fields = Vec::new();
    for i in 0..body.child_count() {
        if let Some(child) = body.child(i) {
            if child.kind() == "field_declaration" {
                let mut name = String::new();
                let mut type_text = String::new();

                // First named child is the name, second is the type
                for j in 0..child.child_count() {
                    if let Some(fc) = child.child(j) {
                        if fc.is_named() && fc.kind() != "visibility_modifier" {
                            if name.is_empty() {
                                name =
                                    fc.utf8_text(ctx.source.as_bytes()).unwrap_or("").to_string();
                            } else if type_text.is_empty() {
                                let full = fc.utf8_text(ctx.source.as_bytes()).unwrap_or("");
                                type_text = if full.len() > 200 {
                                    full[..200].to_string()
                                } else {
                                    full.to_string()
                                };
                                break;
                            }
                        }
                    }
                }

                let range = Some(node_range(&child, ctx));
                fields.push(AstSchemaField {
                    name,
                    type_text: if type_text.is_empty() { None } else { Some(type_text) },
                    required: Some(true),
                    range,
                });
            }
        }
    }
    fields
}

fn extract_enum_variants(node: &tree_sitter::Node, ctx: &AstFileContext) -> Vec<AstSchemaField> {
    let body = match node.child_by_field_name("body") {
        Some(b) => b,
        None => return vec![],
    };
    let mut fields = Vec::new();
    for i in 0..body.child_count() {
        if let Some(child) = body.child(i) {
            if child.kind() == "enum_variant" {
                let name = child
                    .child_by_field_name("name")
                    .and_then(|n| n.utf8_text(ctx.source.as_bytes()).ok())
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                let range = Some(node_range(&child, ctx));
                fields.push(AstSchemaField { name, type_text: None, required: Some(true), range });
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
