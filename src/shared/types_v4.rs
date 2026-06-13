//! V4 types — structural rewrite preview types per spec sections 8–9.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::shared::lenient;
use crate::shared::position::Range;

// ── SafetyViolation ──

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SafetyViolation {
    pub violation_type: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(skip)]
    pub details: Option<serde_json::Value>,
}

// ── TextEdit ──

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TextEdit {
    pub file_path: String,
    pub range: Range,
    pub new_text: String,
}

// ── RewriteOperation ──

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RewriteOperation {
    ReplaceRange {
        file_path: String,
        range: Range,
        new_text: String,
    },
    ReplaceNode {
        file_path: String,
        range: Range,
        #[serde(skip_serializing_if = "Option::is_none")]
        expected_node_kind: Option<String>,
        new_text: String,
    },
    InsertBeforeNode {
        file_path: String,
        range: Range,
        #[serde(skip_serializing_if = "Option::is_none")]
        expected_node_kind: Option<String>,
        new_text: String,
    },
    InsertAfterNode {
        file_path: String,
        range: Range,
        #[serde(skip_serializing_if = "Option::is_none")]
        expected_node_kind: Option<String>,
        new_text: String,
    },
    DeleteNode {
        file_path: String,
        range: Range,
        #[serde(skip_serializing_if = "Option::is_none")]
        expected_node_kind: Option<String>,
    },
}

// ── RewritePreview ──

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RewritePreview {
    pub safe: bool,
    pub changed_files: Vec<String>,
    pub edit_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff: Option<String>,
    pub edits: Vec<TextEdit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_after_rewrite: Option<ParseAfterRewriteSummary>,
    pub violations: Vec<SafetyViolation>,
}

// ── ParseAfterRewriteSummary ──

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ParseAfterRewriteSummary {
    pub ok: bool,
    pub changed_files_checked: u32,
    pub files_with_syntax_errors: Vec<String>,
    pub syntax_errors: Vec<SyntaxErrorSummary>,
}

// ── SyntaxErrorSummary ──

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SyntaxErrorSummary {
    pub file_path: String,
    pub range: Range,
    pub node_kind: String,
    pub message: String,
}

// ── RewriteValidationResult ──

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RewriteValidationResult {
    pub safe: bool,
    pub changed_files: Vec<String>,
    pub edit_count: u32,
    pub violations: Vec<SafetyViolation>,
}

// ── ImportRequest ──

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ImportRequest {
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_import: Option<String>,
    #[serde(default)]
    pub named_imports: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace_import: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_bool")]
    pub is_type_only: Option<bool>,
}

// ── WrapRequest ──

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WrapRequest {
    PrefixSuffix {
        prefix: String,
        suffix: String,
    },
    TryCatch {
        #[serde(skip_serializing_if = "Option::is_none")]
        catch_binding: Option<String>,
        catch_body: String,
    },
    CallExpression {
        callee: String,
    },
}

// ── FunctionSignatureOperation ──

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FunctionSignatureOperation {
    ReplaceSignature {
        new_signature_text: String,
    },
    AddParameter {
        parameter_text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(deserialize_with = "lenient::deserialize_lenient_opt_u32")]
        position: Option<u32>,
    },
    RemoveParameter {
        parameter_name: String,
    },
    RenameParameter {
        old_name: String,
        new_name: String,
        #[serde(default)]
        #[serde(deserialize_with = "lenient::deserialize_lenient_bool")]
        rename_body_occurrences: bool,
    },
}

// Custom Deserialize for FunctionSignatureOperation so the field-level
// deserialize_with annotations take effect (serde derive for tagged enums
// uses an intermediate representation that bypasses field-level attrs).
impl<'de> Deserialize<'de> for FunctionSignatureOperation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Accept any map and deserialize kind + fields leniently.
        let value = serde_json::Value::deserialize(deserializer)?;
        let kind = value.get("kind").and_then(|v| v.as_str()).unwrap_or("");
        match kind {
            "replace_signature" => {
                let text = value.get("new_signature_text").and_then(|v| v.as_str()).unwrap_or("");
                Ok(FunctionSignatureOperation::ReplaceSignature {
                    new_signature_text: text.to_string(),
                })
            }
            "add_parameter" => {
                let parameter_text =
                    value.get("parameter_text").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let position = parse_opt_u32(&value, "position");
                Ok(FunctionSignatureOperation::AddParameter { parameter_text, position })
            }
            "remove_parameter" => {
                let parameter_name =
                    value.get("parameter_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                Ok(FunctionSignatureOperation::RemoveParameter { parameter_name })
            }
            "rename_parameter" => {
                let old_name =
                    value.get("old_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let new_name =
                    value.get("new_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let rename_body_occurrences = value
                    .get("rename_body_occurrences")
                    .map(|v| match v {
                        serde_json::Value::Bool(b) => *b,
                        serde_json::Value::String(s) => {
                            matches!(s.to_lowercase().as_str(), "true" | "1" | "yes")
                        }
                        _ => false,
                    })
                    .unwrap_or(false);
                Ok(FunctionSignatureOperation::RenameParameter {
                    old_name,
                    new_name,
                    rename_body_occurrences,
                })
            }
            _ => Err(serde::de::Error::custom(format!(
                "unknown kind for FunctionSignatureOperation: {}",
                kind
            ))),
        }
    }
}

fn parse_opt_u32(value: &serde_json::Value, key: &str) -> Option<u32> {
    value.get(key).and_then(|v| match v {
        serde_json::Value::Number(n) => n.as_u64().map(|x| x as u32),
        serde_json::Value::String(s) => s.parse::<u32>().ok(),
        _ => None,
    })
}

// ── Tool-specific input types ──

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AstRewritePreviewInput {
    pub operations: Vec<RewriteOperation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_bool")]
    pub include_diff: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_bool")]
    pub parse_check: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_u32")]
    pub max_changed_files: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_u32")]
    pub max_edits: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AstInsertImportPreviewInput {
    pub file_path: String,
    pub import: ImportRequest,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_bool")]
    pub include_diff: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_bool")]
    pub parse_check: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AstRemoveUnusedImportPreviewInput {
    pub file_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_bool")]
    pub include_diff: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_bool")]
    pub parse_check: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AstRenameLocalPreviewInput {
    pub file_path: String,
    pub position: crate::shared::position::Position,
    pub new_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope_range: Option<Range>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_bool")]
    pub include_diff: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_bool")]
    pub parse_check: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AstWrapNodePreviewInput {
    pub file_path: String,
    pub range: Range,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_node_kind: Option<String>,
    pub wrapper: WrapRequest,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_bool")]
    pub include_diff: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_bool")]
    pub parse_check: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AstAddDecoratorPreviewInput {
    pub file_path: String,
    pub target_range: Range,
    pub decorator_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_target_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_bool")]
    pub include_diff: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_bool")]
    pub parse_check: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AstModifyFunctionSignaturePreviewInput {
    pub file_path: String,
    pub function_range: Range,
    pub operation: FunctionSignatureOperation,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_bool")]
    pub include_diff: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_bool")]
    pub parse_check: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AstValidateRewriteInput {
    pub operations: Vec<RewriteOperation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_u32")]
    pub max_changed_files: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_u32")]
    pub max_edits: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AstParseAfterRewriteInput {
    pub edits: Vec<TextEdit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_u32")]
    pub max_changed_files: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_u32")]
    pub max_edits: Option<u32>,
}

// ── Preview options (internal, not an API type) ──

#[derive(Debug, Clone)]
pub struct PreviewOptions {
    pub include_diff: bool,
    pub parse_check: bool,
    pub max_diff_bytes: u64,
}

impl Default for PreviewOptions {
    fn default() -> Self {
        Self { include_diff: true, parse_check: true, max_diff_bytes: 500_000 }
    }
}
