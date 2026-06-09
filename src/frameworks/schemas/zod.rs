use crate::frameworks::{confidence_high, AstDetector, AstFileContext};
use crate::parser::positions::ts_point_to_position;
use crate::shared::position::Range;
use crate::shared::types_v3::{AstSchemaDefinition, AstSchemaField, Evidence};

pub struct ZodSchemaDetector;

impl AstDetector<AstSchemaDefinition> for ZodSchemaDetector {
    fn detect(&self, ctx: &AstFileContext) -> Vec<AstSchemaDefinition> {
        let mut schemas = Vec::new();
        collect_zod_schemas(ctx.tree.root_node(), ctx, &mut schemas);
        schemas
    }
}

fn collect_zod_schemas(
    node: tree_sitter::Node,
    ctx: &AstFileContext,
    results: &mut Vec<AstSchemaDefinition>,
) {
    if node.kind() == "variable_declarator" {
        if let Some(schema) = extract_zod_schema(&node, ctx) {
            results.push(schema);
            return;
        }
    }
    if node.kind() == "call_expression" {
        // Handle direct z.object({...}) calls
        if let Some(schema) = extract_zod_call(&node, ctx) {
            results.push(schema);
            return;
        }
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_zod_schemas(child, ctx, results);
        }
    }
}

fn extract_zod_schema(
    node: &tree_sitter::Node,
    ctx: &AstFileContext,
) -> Option<AstSchemaDefinition> {
    let name_node = node.child_by_field_name("name")?;
    let name = name_node.utf8_text(ctx.source.as_bytes()).unwrap_or("").to_string();
    let value = node.child_by_field_name("value")?;

    // Look for z.object({...}) in the value
    if !contains_z_object(&value, ctx) {
        return None;
    }

    let fields = extract_zod_fields(&value, ctx);
    let range = node_range(node, ctx);
    let evidence = make_evidence("variable_declarator", node, ctx);

    Some(AstSchemaDefinition {
        file_path: ctx.relative_path.to_string(),
        language: ctx.language.to_string(),
        kind: "zod_object".to_string(),
        name: Some(name),
        framework: Some("zod".to_string()),
        fields,
        range,
        confidence: confidence_high(),
        evidence: vec![evidence],
    })
}

fn extract_zod_call(node: &tree_sitter::Node, ctx: &AstFileContext) -> Option<AstSchemaDefinition> {
    let func = node.child_by_field_name("function")?;
    // Check if it's z.object(...)
    let func_text = func.utf8_text(ctx.source.as_bytes()).unwrap_or("");
    if !func_text.ends_with(".object") && func_text != "z.object" {
        return None;
    }
    let args = node.child_by_field_name("arguments")?;
    let fields = extract_zod_fields_from_object(&args, ctx);
    if fields.is_empty() {
        return None;
    }
    let range = node_range(node, ctx);
    let evidence = make_evidence("call_expression", node, ctx);
    Some(AstSchemaDefinition {
        file_path: ctx.relative_path.to_string(),
        language: ctx.language.to_string(),
        kind: "zod_object".to_string(),
        name: None,
        framework: Some("zod".to_string()),
        fields,
        range,
        confidence: confidence_high(),
        evidence: vec![evidence],
    })
}

fn contains_z_object(node: &tree_sitter::Node, ctx: &AstFileContext) -> bool {
    if node.kind() == "call_expression" {
        if let Some(func) = node.child_by_field_name("function") {
            let text = func.utf8_text(ctx.source.as_bytes()).unwrap_or("");
            if text.contains("z.object") || text.contains(".object(") {
                return true;
            }
        }
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if contains_z_object(&child, ctx) {
                return true;
            }
        }
    }
    false
}

fn extract_zod_fields(node: &tree_sitter::Node, ctx: &AstFileContext) -> Vec<AstSchemaField> {
    let mut fields = Vec::new();
    find_z_object_and_extract(node, ctx, &mut fields);
    fields
}

fn find_z_object_and_extract(
    node: &tree_sitter::Node,
    ctx: &AstFileContext,
    fields: &mut Vec<AstSchemaField>,
) {
    if node.kind() == "call_expression" {
        if let Some(func) = node.child_by_field_name("function") {
            let text = func.utf8_text(ctx.source.as_bytes()).unwrap_or("");
            if text.contains("z.object") || text.contains(".object(") {
                if let Some(args) = node.child_by_field_name("arguments") {
                    extract_zod_fields_from_object(&args, ctx)
                        .into_iter()
                        .for_each(|f| fields.push(f));
                }
                return;
            }
        }
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            find_z_object_and_extract(&child, ctx, fields);
        }
    }
}

fn extract_zod_fields_from_object(
    args: &tree_sitter::Node,
    ctx: &AstFileContext,
) -> Vec<AstSchemaField> {
    let mut fields = Vec::new();
    // Find the object literal
    for i in 0..args.child_count() {
        if let Some(arg) = args.child(i) {
            if arg.kind() == "object" {
                for j in 0..arg.child_count() {
                    if let Some(pair) = arg.child(j) {
                        if pair.kind() == "pair" {
                            if let Some(key) = pair.child_by_field_name("key") {
                                let field_name = key
                                    .utf8_text(ctx.source.as_bytes())
                                    .unwrap_or("")
                                    .trim_matches(|c| c == '"' || c == '\'' || c == '`')
                                    .to_string();
                                let value = pair.child_by_field_name("value");
                                let type_text = value.map(|v| {
                                    let text = v.utf8_text(ctx.source.as_bytes()).unwrap_or("");
                                    if text.len() > 200 {
                                        text[..200].to_string()
                                    } else {
                                        text.to_string()
                                    }
                                });
                                let range = value.map(|v| node_range(&v, ctx));
                                let required = !type_text.as_deref().is_some_and(|t| {
                                    t.contains(".optional()") || t.contains(".nullable()")
                                });

                                fields.push(AstSchemaField {
                                    name: field_name,
                                    type_text,
                                    required: Some(required),
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
