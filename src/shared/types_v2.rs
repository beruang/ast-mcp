//! V2 shared types — scope, context, extraction result types.
use serde::{Deserialize, Serialize};

use crate::shared::position::Range;

/// Scope kind taxonomy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopeKind {
    Module,
    Class,
    Interface,
    Function,
    Method,
    Constructor,
    ArrowFunction,
    Lambda,
    Block,
    Unknown,
}

/// Summary of a single syntactic scope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeSummary {
    pub kind: ScopeKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub node_kind: String,
    pub range: Range,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selection_range: Option<Range>,
}

/// A labelled block of source context (used by context_for_range / context_pack).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBlock {
    pub label: String,
    pub kind: String,
    pub file_path: String,
    pub range: Range,
    pub text: String,
    pub truncated: bool,
}

/// Parts requestable in a context pack.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextPackPart {
    Imports,
    Exports,
    EnclosingScope,
    EnclosingNode,
    TopLevelOutline,
    NearbyFunctions,
    NearbyClasses,
}

/// Call expression match (used by ast_find_calls).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallExpression {
    pub callee_text: String,
    pub arguments_text: Vec<String>,
    pub range: Range,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enclosing_scope: Option<ScopeSummary>,
}

/// Member access match (used by ast_find_member_access).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberAccess {
    pub object_text: String,
    pub property: String,
    pub full_text: String,
    pub range: Range,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enclosing_scope: Option<ScopeSummary>,
}

/// Literal match (used by ast_find_literals).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteralMatch {
    pub kind: String,
    pub raw_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_text: Option<String>,
    pub range: Range,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enclosing_scope: Option<ScopeSummary>,
}

/// Template literal match (used by ast_find_template_literals).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateLiteralMatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    pub raw_text: String,
    pub range: Range,
    pub interpolation_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enclosing_scope: Option<ScopeSummary>,
}

/// File metrics (used by ast_file_metrics).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetrics {
    pub file_path: String,
    pub language: String,
    pub line_count: usize,
    pub byte_count: usize,
    pub node_count: usize,
    pub syntax_error_count: usize,
    pub import_count: usize,
    pub export_count: usize,
    pub function_count: usize,
    pub class_count: usize,
    pub max_nesting_depth: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_function_lines: Option<usize>,
}

/// Per-function metric (used by ast_file_metrics when include_function_metrics=true).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMetric {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub kind: String,
    pub range: Range,
    pub line_count: usize,
    pub branch_count: usize,
    pub loop_count: usize,
    pub nesting_depth: usize,
}

/// Workspace query match.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceQueryMatch {
    pub file_path: String,
    pub captures: Vec<QueryCapture>,
}

/// A single capture within a workspace query match.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryCapture {
    pub name: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    pub range: Range,
}

/// Result limit info returned by listing tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultLimit {
    pub returned: usize,
    pub truncated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_known: Option<usize>,
}
