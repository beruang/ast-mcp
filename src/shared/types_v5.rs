//! V5 types — runtime configuration, cache status, request logging, scan tracking,
//! health probes, parser operations, complexity, large nodes, and duplicate shapes.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::shared::lenient;

// ── RuntimeConfig (spec section 9) ──

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RuntimeConfig {
    pub workspace_path: String,
    pub limits: RuntimeLimits,
    #[serde(rename = "timeoutsMs")]
    pub timeouts_ms: RuntimeTimeouts,
    pub caches: RuntimeCaches,
    pub scans: RuntimeScans,
    pub debug: RuntimeDebug,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RuntimeLimits {
    #[serde(rename = "maxFileBytes")]
    pub max_file_bytes: usize,
    #[serde(rename = "maxParseTreeNodes")]
    pub max_parse_tree_nodes: usize,
    #[serde(rename = "maxQueryResults")]
    pub max_query_results: usize,
    #[serde(rename = "maxWorkspaceFiles")]
    pub max_workspace_files: usize,
    #[serde(rename = "maxWorkspaceResults")]
    pub max_workspace_results: usize,
    #[serde(rename = "maxContextCharacters")]
    pub max_context_characters: usize,
    #[serde(rename = "maxChunkLines")]
    pub max_chunk_lines: usize,
    #[serde(rename = "maxChangedFiles")]
    pub max_changed_files: usize,
    #[serde(rename = "maxEdits")]
    pub max_edits: usize,
    #[serde(rename = "maxDuplicateCandidates")]
    pub max_duplicate_candidates: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RuntimeTimeouts {
    #[serde(rename = "parseFile")]
    pub parse_file: u64,
    #[serde(rename = "queryFile")]
    pub query_file: u64,
    #[serde(rename = "queryWorkspace")]
    pub query_workspace: u64,
    #[serde(rename = "chunkFile")]
    pub chunk_file: u64,
    #[serde(rename = "frameworkExtraction")]
    pub framework_extraction: u64,
    #[serde(rename = "rewritePreview")]
    pub rewrite_preview: u64,
    #[serde(rename = "complexitySummary")]
    pub complexity_summary: u64,
    #[serde(rename = "duplicateDetection")]
    pub duplicate_detection: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RuntimeCaches {
    #[serde(rename = "parseTreeTtlMs")]
    pub parse_tree_ttl_ms: u64,
    #[serde(rename = "queryResultTtlMs")]
    pub query_result_ttl_ms: u64,
    #[serde(rename = "frameworkResultTtlMs")]
    pub framework_result_ttl_ms: u64,
    #[serde(rename = "requestLogMaxEntries")]
    pub request_log_max_entries: usize,
    #[serde(rename = "maxCachedFiles")]
    pub max_cached_files: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RuntimeScans {
    #[serde(rename = "maxParallelism")]
    pub max_parallelism: usize,
    #[serde(rename = "respectGitignore")]
    pub respect_gitignore: bool,
    #[serde(rename = "includeHidden")]
    pub include_hidden: bool,
    #[serde(rename = "defaultExcludeGlobs")]
    pub default_exclude_globs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RuntimeDebug {
    #[serde(rename = "verboseLogging")]
    pub verbose_logging: bool,
    #[serde(rename = "includeNodeTextInLogs")]
    pub include_node_text_in_logs: bool,
    #[serde(rename = "includeRawTreeDebug")]
    pub include_raw_tree_debug: bool,
}

// ── Partial configs for runtime updates (spec section 19) ──

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PartialRuntimeLimits {
    #[serde(rename = "maxFileBytes", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_usize")]
    pub max_file_bytes: Option<usize>,
    #[serde(rename = "maxParseTreeNodes", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_usize")]
    pub max_parse_tree_nodes: Option<usize>,
    #[serde(rename = "maxQueryResults", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_usize")]
    pub max_query_results: Option<usize>,
    #[serde(rename = "maxWorkspaceFiles", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_usize")]
    pub max_workspace_files: Option<usize>,
    #[serde(rename = "maxWorkspaceResults", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_usize")]
    pub max_workspace_results: Option<usize>,
    #[serde(rename = "maxContextCharacters", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_usize")]
    pub max_context_characters: Option<usize>,
    #[serde(rename = "maxChunkLines", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_usize")]
    pub max_chunk_lines: Option<usize>,
    #[serde(rename = "maxChangedFiles", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_usize")]
    pub max_changed_files: Option<usize>,
    #[serde(rename = "maxEdits", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_usize")]
    pub max_edits: Option<usize>,
    #[serde(rename = "maxDuplicateCandidates", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_usize")]
    pub max_duplicate_candidates: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PartialRuntimeTimeouts {
    #[serde(rename = "parseFile", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_u64")]
    pub parse_file: Option<u64>,
    #[serde(rename = "queryFile", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_u64")]
    pub query_file: Option<u64>,
    #[serde(rename = "queryWorkspace", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_u64")]
    pub query_workspace: Option<u64>,
    #[serde(rename = "chunkFile", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_u64")]
    pub chunk_file: Option<u64>,
    #[serde(rename = "frameworkExtraction", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_u64")]
    pub framework_extraction: Option<u64>,
    #[serde(rename = "rewritePreview", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_u64")]
    pub rewrite_preview: Option<u64>,
    #[serde(rename = "complexitySummary", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_u64")]
    pub complexity_summary: Option<u64>,
    #[serde(rename = "duplicateDetection", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_u64")]
    pub duplicate_detection: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PartialRuntimeCaches {
    #[serde(rename = "parseTreeTtlMs", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_u64")]
    pub parse_tree_ttl_ms: Option<u64>,
    #[serde(rename = "queryResultTtlMs", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_u64")]
    pub query_result_ttl_ms: Option<u64>,
    #[serde(rename = "frameworkResultTtlMs", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_u64")]
    pub framework_result_ttl_ms: Option<u64>,
    #[serde(rename = "requestLogMaxEntries", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_usize")]
    pub request_log_max_entries: Option<usize>,
    #[serde(rename = "maxCachedFiles", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_usize")]
    pub max_cached_files: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PartialRuntimeScans {
    #[serde(rename = "maxParallelism", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_usize")]
    pub max_parallelism: Option<usize>,
    #[serde(rename = "respectGitignore", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_bool")]
    pub respect_gitignore: Option<bool>,
    #[serde(rename = "includeHidden", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_bool")]
    pub include_hidden: Option<bool>,
    #[serde(rename = "defaultExcludeGlobs", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub default_exclude_globs: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PartialRuntimeDebug {
    #[serde(rename = "verboseLogging", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_bool")]
    pub verbose_logging: Option<bool>,
    #[serde(rename = "includeNodeTextInLogs", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_bool")]
    pub include_node_text_in_logs: Option<bool>,
    #[serde(rename = "includeRawTreeDebug", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_bool")]
    pub include_raw_tree_debug: Option<bool>,
}

// ── Config tools I/O (spec sections 18–19) ──

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetConfigInput {
    #[serde(rename = "includeDefaults", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(deserialize_with = "lenient::deserialize_lenient_opt_bool")]
    pub include_defaults: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetConfigResult {
    pub config: RuntimeConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<ConfigSources>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConfigSources {
    pub defaults: serde_json::Value,
    pub environment: serde_json::Value,
    #[serde(rename = "runtimeOverrides")]
    pub runtime_overrides: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateRuntimeConfigInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<PartialRuntimeLimits>,
    #[serde(rename = "timeoutsMs", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub timeouts_ms: Option<PartialRuntimeTimeouts>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caches: Option<PartialRuntimeCaches>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scans: Option<PartialRuntimeScans>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug: Option<PartialRuntimeDebug>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateRuntimeConfigResult {
    pub updated: bool,
    pub config: RuntimeConfig,
    pub rejected: Vec<RejectedConfigUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RejectedConfigUpdate {
    pub path: String,
    pub reason: String,
}

// ── Cache types (spec sections 14–15) ──

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CacheStatusInput {}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CacheStatusResult {
    pub caches: AstCacheStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AstCacheStatus {
    #[serde(rename = "parseTrees")]
    pub parse_trees: CacheSectionStatus,
    #[serde(rename = "queryResults")]
    pub query_results: CacheSectionStatus,
    #[serde(rename = "frameworkResults")]
    pub framework_results: CacheSectionStatus,
    #[serde(rename = "requestLog")]
    pub request_log: RequestLogCacheStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CacheSectionStatus {
    pub entries: usize,
    #[serde(rename = "maxEntries", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_entries: Option<usize>,
    #[serde(rename = "ttlMs")]
    pub ttl_ms: u64,
    #[serde(rename = "estimatedBytes", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub estimated_bytes: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RequestLogCacheStatus {
    pub entries: usize,
    #[serde(rename = "maxEntries")]
    pub max_entries: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ClearCachesInput {
    pub caches: Vec<AstCacheName>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AstCacheName {
    ParseTrees,
    QueryResults,
    FrameworkResults,
    RequestLog,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ClearCachesResult {
    pub cleared: BTreeMap<String, usize>,
}

// ── Request log types (spec sections 16–17) ──

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RequestLogInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<RequestStatus>,
    #[serde(rename = "filePath", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RequestStatus {
    Ok,
    Error,
    Timeout,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RequestLogResult {
    pub entries: Vec<RequestLogEntry>,
    pub returned: usize,
    #[serde(rename = "totalStored")]
    pub total_stored: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RequestLogEntry {
    pub id: String,
    pub tool: String,
    #[serde(rename = "startedAt")]
    pub started_at: String,
    #[serde(rename = "completedAt", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub completed_at: Option<String>,
    #[serde(rename = "durationMs", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub duration_ms: Option<u64>,
    pub status: RequestStatus,
    #[serde(rename = "errorCode", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub error_code: Option<String>,
    #[serde(rename = "errorMessage", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub error_message: Option<String>,
    #[serde(rename = "filePath", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub file_path: Option<String>,
    #[serde(rename = "resultCount", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub result_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ClearRequestLogInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ClearRequestLogResult {
    pub cleared: usize,
}

// ── Health types (spec sections 20–21) ──

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReadinessInput {
    #[serde(rename = "requireLanguages", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub require_languages: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReadinessResult {
    pub ready: bool,
    #[serde(rename = "workspacePath")]
    pub workspace_path: String,
    pub checks: Vec<ReadinessCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReadinessCheck {
    pub name: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LivenessInput {}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LivenessResult {
    pub alive: bool,
    #[serde(rename = "uptimeMs")]
    pub uptime_ms: u64,
    #[serde(rename = "startedAt")]
    pub started_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<MemoryUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemoryUsage {
    #[serde(rename = "rssBytes", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub rss_bytes: Option<u64>,
    #[serde(rename = "heapBytes", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub heap_bytes: Option<u64>,
}

// ── Workspace scan types (spec sections 22–23) ──

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceScanStatusInput {
    #[serde(rename = "scanId", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub scan_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceScanStatusResult {
    pub scans: Vec<WorkspaceScanInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceScanInfo {
    #[serde(rename = "scanId")]
    pub scan_id: String,
    pub tool: String,
    pub status: String,
    #[serde(rename = "startedAt")]
    pub started_at: String,
    #[serde(rename = "completedAt", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub completed_at: Option<String>,
    #[serde(rename = "filesDiscovered")]
    pub files_discovered: usize,
    #[serde(rename = "filesProcessed")]
    pub files_processed: usize,
    #[serde(rename = "resultsFound")]
    pub results_found: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CancelWorkspaceScanInput {
    #[serde(rename = "scanId")]
    pub scan_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CancelWorkspaceScanResult {
    #[serde(rename = "scanId")]
    pub scan_id: String,
    pub cancelled: bool,
    #[serde(rename = "previousStatus")]
    pub previous_status: String,
    #[serde(rename = "newStatus")]
    pub new_status: String,
}

// ── Parser ops types (spec sections 24–25) ──

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ParserStatusInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ParserStatusResult {
    pub parsers: Vec<ParserStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ParserStatus {
    pub language: String,
    pub extensions: Vec<String>,
    pub available: bool,
    #[serde(rename = "parserName")]
    pub parser_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(rename = "queryCount")]
    pub query_count: usize,
    #[serde(rename = "lastError", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RebuildParserCacheInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub languages: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RebuildParserCacheResult {
    pub rebuilt: Vec<String>,
    pub failed: Vec<ParserRebuildFailure>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ParserRebuildFailure {
    pub language: String,
    pub error: String,
}

// ── Complexity types (spec section 11) ──

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ComplexitySummaryInput {
    #[serde(rename = "filePath", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glob: Option<String>,
    #[serde(rename = "maxFiles", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_files: Option<usize>,
    #[serde(rename = "includeFunctions", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub include_functions: Option<bool>,
    #[serde(rename = "includeClasses", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub include_classes: Option<bool>,
    #[serde(rename = "maxResults", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_results: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ComplexitySummaryResult {
    #[serde(rename = "totalFilesScanned")]
    pub total_files_scanned: usize,
    #[serde(rename = "totalNodesAnalyzed")]
    pub total_nodes_analyzed: usize,
    pub files: Vec<FileComplexitySummary>,
    pub hotspots: Vec<ComplexityHotspot>,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileComplexitySummary {
    #[serde(rename = "filePath")]
    pub file_path: String,
    #[serde(rename = "lineCount")]
    pub line_count: usize,
    #[serde(rename = "functionCount")]
    pub function_count: usize,
    #[serde(rename = "classCount")]
    pub class_count: usize,
    #[serde(rename = "importCount")]
    pub import_count: usize,
    #[serde(rename = "maxNestingDepth")]
    pub max_nesting_depth: usize,
    #[serde(rename = "maxFunctionLines")]
    pub max_function_lines: usize,
    #[serde(rename = "branchCount")]
    pub branch_count: usize,
    #[serde(rename = "loopCount")]
    pub loop_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ComplexityHotspot {
    #[serde(rename = "filePath")]
    pub file_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub kind: String,
    pub range: crate::shared::position::Range,
    #[serde(rename = "lineCount")]
    pub line_count: usize,
    #[serde(rename = "branchCount")]
    pub branch_count: usize,
    #[serde(rename = "loopCount")]
    pub loop_count: usize,
    #[serde(rename = "nestingDepth")]
    pub nesting_depth: usize,
    pub risk: String,
    pub reasons: Vec<String>,
}

// ── Large node types (spec section 12) ──

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DetectLargeNodesInput {
    #[serde(rename = "filePath", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glob: Option<String>,
    #[serde(rename = "maxFiles", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_files: Option<usize>,
    #[serde(rename = "minLines", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub min_lines: Option<usize>,
    #[serde(rename = "nodeKinds", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub node_kinds: Option<Vec<String>>,
    #[serde(rename = "maxResults", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_results: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DetectLargeNodesResult {
    pub nodes: Vec<LargeNode>,
    #[serde(rename = "scannedFiles")]
    pub scanned_files: usize,
    pub returned: usize,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LargeNode {
    #[serde(rename = "filePath")]
    pub file_path: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub range: crate::shared::position::Range,
    #[serde(rename = "lineCount")]
    pub line_count: usize,
    #[serde(rename = "childCount")]
    pub child_count: usize,
    #[serde(rename = "nestingDepth")]
    pub nesting_depth: usize,
}

// ── Duplicate shape types (spec section 13) ──

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DetectDuplicateShapesInput {
    pub glob: String,
    #[serde(rename = "maxFiles", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_files: Option<usize>,
    #[serde(rename = "minNodeLines", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub min_node_lines: Option<usize>,
    #[serde(rename = "nodeKinds", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub node_kinds: Option<Vec<String>>,
    #[serde(rename = "normalizeIdentifiers", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub normalize_identifiers: Option<bool>,
    #[serde(rename = "normalizeLiterals", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub normalize_literals: Option<bool>,
    #[serde(rename = "maxCandidates", skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_candidates: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DetectDuplicateShapesResult {
    pub groups: Vec<DuplicateShapeGroup>,
    #[serde(rename = "scannedFiles")]
    pub scanned_files: usize,
    #[serde(rename = "candidateCount")]
    pub candidate_count: usize,
    pub returned: usize,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuplicateShapeGroup {
    pub fingerprint: String,
    #[serde(rename = "similarityKind")]
    pub similarity_kind: String,
    pub occurrences: Vec<DuplicateShapeOccurrence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuplicateShapeOccurrence {
    #[serde(rename = "filePath")]
    pub file_path: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub range: crate::shared::position::Range,
    #[serde(rename = "lineCount")]
    pub line_count: usize,
}
