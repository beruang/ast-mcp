use serde::{Deserialize, Serialize};

use crate::shared::position::Range;

/// Confidence level for heuristic detection results.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    Low,
    Medium,
    High,
}

/// Evidence for a framework-aware detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<Range>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_kind: Option<String>,
}

/// Framework detection metadata attached to a result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkDetection {
    pub framework: String,
    pub confidence: Confidence,
    pub evidence: Vec<Evidence>,
}

/// Base fields shared by all V3 extracted items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedItemBase {
    pub file_path: String,
    pub language: String,
    pub range: Range,
    pub confidence: Confidence,
    pub evidence: Vec<Evidence>,
}

// --- ast_find_routes ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstRoute {
    pub file_path: String,
    pub language: String,
    pub framework: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handler_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handler_kind: Option<String>,
    pub range: Range,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_range: Option<Range>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handler_range: Option<Range>,
    pub confidence: Confidence,
    pub evidence: Vec<Evidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstFindRoutesResult {
    pub routes: Vec<AstRoute>,
    pub returned: u32,
    pub truncated: bool,
    pub scanned_files: u32,
}

// --- ast_find_react_components ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstReactComponent {
    pub file_path: String,
    pub name: String,
    pub kind: ComponentKind,
    pub exported: bool,
    pub default_export: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props_type_text: Option<String>,
    pub hooks: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsx_root: Option<String>,
    pub range: Range,
    pub confidence: Confidence,
    pub evidence: Vec<Evidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ComponentKind {
    FunctionComponent,
    ArrowFunctionComponent,
    ClassComponent,
    MemoComponent,
    ForwardRefComponent,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstFindReactComponentsResult {
    pub components: Vec<AstReactComponent>,
    pub returned: u32,
    pub truncated: bool,
    pub scanned_files: u32,
}

// --- ast_find_hooks ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstHook {
    pub file_path: String,
    pub name: String,
    pub kind: HookKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enclosing_component: Option<String>,
    pub range: Range,
    pub confidence: Confidence,
    pub evidence: Vec<Evidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HookKind {
    BuiltinUsage,
    CustomUsage,
    CustomDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstFindHooksResult {
    pub hooks: Vec<AstHook>,
    pub returned: u32,
    pub truncated: bool,
    pub scanned_files: u32,
}

// --- ast_find_tests ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstTestItem {
    pub file_path: String,
    pub language: String,
    pub framework: String,
    pub kind: TestKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub range: Range,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_name: Option<String>,
    pub confidence: Confidence,
    pub evidence: Vec<Evidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TestKind {
    Suite,
    Test,
    Fixture,
    Hook,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstFindTestsResult {
    pub tests: Vec<AstTestItem>,
    pub returned: u32,
    pub truncated: bool,
    pub scanned_files: u32,
}

// --- ast_find_decorators ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstDecorator {
    pub file_path: String,
    pub language: String,
    pub name: String,
    pub arguments_text: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_name: Option<String>,
    pub range: Range,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_range: Option<Range>,
    pub confidence: Confidence,
    pub evidence: Vec<Evidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstFindDecoratorsResult {
    pub decorators: Vec<AstDecorator>,
    pub returned: u32,
    pub truncated: bool,
    pub scanned_files: u32,
}

// --- ast_find_schema_definitions ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstSchemaDefinition {
    pub file_path: String,
    pub language: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub framework: Option<String>,
    pub fields: Vec<AstSchemaField>,
    pub range: Range,
    pub confidence: Confidence,
    pub evidence: Vec<Evidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstSchemaField {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<Range>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstFindSchemaDefinitionsResult {
    pub schemas: Vec<AstSchemaDefinition>,
    pub returned: u32,
    pub truncated: bool,
    pub scanned_files: u32,
}

// --- ast_dependency_edges ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstDependencyEdge {
    pub from_file: String,
    pub to_specifier: String,
    pub kind: EdgeKind,
    pub is_relative: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_type_only: Option<bool>,
    pub range: Range,
    pub confidence: Confidence,
    pub evidence: Vec<Evidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EdgeKind {
    Import,
    Export,
    Require,
    Use,
    Include,
    Mod,
    Package,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstDependencyEdgesResult {
    pub edges: Vec<AstDependencyEdge>,
    pub returned: u32,
    pub truncated: bool,
    pub scanned_files: u32,
}
