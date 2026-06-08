# AST MCP Server — Version 5 Specification

## 1. Purpose

Version 5 turns the Rust-based AST MCP server into a production-ready structural analysis service.

V1 provided core structural parsing and extraction.

V2 added context selection, node-level inspection, pattern search, and workspace queries.

V3 added framework-aware extraction.

V4 added preview-only structural rewrites and parse-after-rewrite validation.

V5 focuses on operational hardening, repository-scale analysis, performance controls, metrics, and maintainability features.

V5 adds:

```text
ast_complexity_summary
ast_detect_large_nodes
ast_detect_duplicate_shapes
ast_cache_status
ast_clear_caches
ast_request_log
ast_clear_request_log
ast_get_config
ast_update_runtime_config
ast_readiness
ast_liveness
ast_workspace_scan_status
ast_cancel_workspace_scan
ast_parser_status
ast_rebuild_parser_cache
```

The AST MCP remains structural only.

It must not call LSP MCP.

It must not apply edits to files.

---

## 2. Architectural Boundary

The AST MCP owns syntax and structure.

It does not own semantic compiler intelligence.

### AST MCP may do

```text
parse files
walk syntax trees
run Tree-sitter queries
extract imports/exports/functions/classes/calls/routes/tests/decorators
chunk files structurally
preview structural rewrites
validate syntax after in-memory rewrites
compute structural metrics
cache parse trees
scan workspaces with limits
```

### AST MCP must not do

```text
call LSP MCP
resolve semantic references
infer full type information
perform semantic rename
apply patches to disk
execute shell commands
run tests
run typecheck
```

For semantic analysis, use LSP MCP.

For cross-service workflows, use Agent Skills or a future Code Composite MCP.

---

## 3. Version 5 Goals

V5 must make AST MCP reliable for long-running agent sessions and large repositories.

Primary goals:

```text
production observability
cache status and cache clearing
runtime config inspection and safe updates
readiness/liveness checks
request logging
workspace scan tracking
bounded parallel parsing
structural complexity summaries
large-node detection
duplicate structural-shape detection
parser health and parser cache management
```

---

## 4. Version 5 Non-Goals

V5 does not include:

```text
semantic references
semantic rename
compiler diagnostics
LSP calls
direct file mutation
patch application
remote workspace orchestration
distributed indexing
persistent database requirement
full clone-detection engine
security sandboxing beyond workspace/path controls
```

---

## 5. Required Baseline From V1–V4

V5 assumes the following tools already exist.

### V1 baseline

```text
ast_health_check
ast_list_supported_languages
ast_parse_file
ast_file_outline
ast_top_level_nodes
ast_enclosing_node
ast_find_imports
ast_find_exports
ast_find_functions
ast_find_classes
ast_chunk_file
ast_query
```

### V2 baseline

```text
ast_enclosing_scope
ast_node_at_range
ast_node_text
ast_context_for_range
ast_context_pack
ast_find_calls
ast_find_member_access
ast_find_literals
ast_find_template_literals
ast_query_workspace
ast_file_metrics
```

### V3 baseline

```text
ast_find_routes
ast_find_react_components
ast_find_hooks
ast_find_tests
ast_find_decorators
ast_find_schema_definitions
ast_dependency_edges
```

### V4 baseline

```text
ast_rewrite_preview
ast_insert_import_preview
ast_remove_unused_import_preview
ast_rename_local_preview
ast_wrap_node_preview
ast_add_decorator_preview
ast_modify_function_signature_preview
ast_validate_rewrite
ast_parse_after_rewrite
```

V5 must preserve backward compatibility with V1–V4 schemas unless a safety correction is required.

---

## 6. Recommended Rust Stack

```text
Language: Rust
Transport: MCP stdio
Parser: Tree-sitter
Serialization: serde / serde_json
Schema generation: schemars
Errors: thiserror / anyhow
Async runtime: tokio
File walking: ignore / walkdir
Glob matching: globset
Parallelism: rayon or tokio task pool
Diffs: similar
Logging/tracing: tracing / tracing-subscriber
Time: time or chrono
IDs: uuid
```

Recommended crates:

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
schemars = "0.8"
thiserror = "1"
anyhow = "1"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
tracing-subscriber = "0.3"
uuid = { version = "1", features = ["v4", "serde"] }
ignore = "0.4"
globset = "0.4"
walkdir = "2"
rayon = "1"
similar = "2"
tree-sitter = "0.22"
```

Parser crates depend on supported languages:

```toml
tree-sitter-typescript = "0.21"
tree-sitter-javascript = "0.21"
tree-sitter-python = "0.21"
tree-sitter-go = "0.21"
tree-sitter-rust = "0.21"
```

Use exact versions based on the parser crate ecosystem at implementation time.

---

## 7. Shared Contracts

V5 must continue using the shared AST MCP contracts.

### Position

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}
```

External API positions should remain compatible with LSP-style zero-based line and UTF-16 character offsets when practical.

Tree-sitter internally uses byte offsets and row/column points.

The AST MCP must explicitly convert between external positions and internal byte offsets.

### Range

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}
```

### FileRange

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileRange {
    pub file_path: String, // workspace-relative
    pub range: Range,
}
```

### ToolError

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolError {
    pub error: ToolErrorBody,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolErrorBody {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}
```

### SafetyViolation

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SafetyViolation {
    pub violation_type: String,
    pub message: String,
    pub file_path: Option<String>,
    pub details: Option<serde_json::Value>,
}
```

---

## 8. Workspace Safety

Every tool must enforce workspace boundaries.

Workspace root is configured by:

```bash
WORKSPACE_PATH=/absolute/path/to/repo
```

Accepted:

```text
src/user.ts
/repo/src/user.ts
```

Rejected:

```text
../outside.ts
/etc/passwd
/another-project/file.ts
```

All outputs should use workspace-relative paths.

V5 must preserve the same safety model from V1–V4.

---

## 9. Runtime Configuration

V5 introduces runtime configuration.

### RuntimeConfig

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RuntimeConfig {
    pub workspace_path: String,
    pub limits: RuntimeLimits,
    pub timeouts_ms: RuntimeTimeouts,
    pub caches: RuntimeCaches,
    pub scans: RuntimeScans,
    pub debug: RuntimeDebug,
}
```

### RuntimeLimits

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RuntimeLimits {
    pub max_file_bytes: usize,
    pub max_parse_tree_nodes: usize,
    pub max_query_results: usize,
    pub max_workspace_files: usize,
    pub max_workspace_results: usize,
    pub max_context_characters: usize,
    pub max_chunk_lines: usize,
    pub max_changed_files: usize,
    pub max_edits: usize,
    pub max_duplicate_candidates: usize,
}
```

Recommended defaults:

```json
{
  "max_file_bytes": 1048576,
  "max_parse_tree_nodes": 200000,
  "max_query_results": 1000,
  "max_workspace_files": 500,
  "max_workspace_results": 5000,
  "max_context_characters": 20000,
  "max_chunk_lines": 160,
  "max_changed_files": 100,
  "max_edits": 1000,
  "max_duplicate_candidates": 200
}
```

### RuntimeTimeouts

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RuntimeTimeouts {
    pub parse_file: u64,
    pub query_file: u64,
    pub query_workspace: u64,
    pub chunk_file: u64,
    pub framework_extraction: u64,
    pub rewrite_preview: u64,
    pub complexity_summary: u64,
    pub duplicate_detection: u64,
}
```

Recommended defaults in milliseconds:

```json
{
  "parse_file": 5000,
  "query_file": 5000,
  "query_workspace": 30000,
  "chunk_file": 10000,
  "framework_extraction": 30000,
  "rewrite_preview": 10000,
  "complexity_summary": 30000,
  "duplicate_detection": 60000
}
```

### RuntimeCaches

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RuntimeCaches {
    pub parse_tree_ttl_ms: u64,
    pub query_result_ttl_ms: u64,
    pub framework_result_ttl_ms: u64,
    pub request_log_max_entries: usize,
    pub max_cached_files: usize,
}
```

Recommended defaults:

```json
{
  "parse_tree_ttl_ms": 300000,
  "query_result_ttl_ms": 120000,
  "framework_result_ttl_ms": 120000,
  "request_log_max_entries": 500,
  "max_cached_files": 1000
}
```

### RuntimeScans

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RuntimeScans {
    pub max_parallelism: usize,
    pub respect_gitignore: bool,
    pub include_hidden: bool,
    pub default_exclude_globs: Vec<String>,
}
```

Recommended defaults:

```json
{
  "max_parallelism": 8,
  "respect_gitignore": true,
  "include_hidden": false,
  "default_exclude_globs": [
    "**/node_modules/**",
    "**/.git/**",
    "**/dist/**",
    "**/build/**",
    "**/target/**",
    "**/.venv/**",
    "**/__pycache__/**"
  ]
}
```

### RuntimeDebug

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RuntimeDebug {
    pub verbose_logging: bool,
    pub include_node_text_in_logs: bool,
    pub include_raw_tree_debug: bool,
}
```

Defaults:

```json
{
  "verbose_logging": false,
  "include_node_text_in_logs": false,
  "include_raw_tree_debug": false
}
```

---

## 10. Environment Variables

V5 should support environment configuration.

Recommended environment variables:

```text
WORKSPACE_PATH=/repo

AST_MAX_FILE_BYTES=1048576
AST_MAX_WORKSPACE_FILES=500
AST_MAX_WORKSPACE_RESULTS=5000
AST_MAX_CONTEXT_CHARACTERS=20000
AST_MAX_PARALLELISM=8
AST_RESPECT_GITIGNORE=true
AST_INCLUDE_HIDDEN=false

AST_PARSE_TREE_TTL_MS=300000
AST_REQUEST_LOG_MAX_ENTRIES=500
AST_MAX_CACHED_FILES=1000

AST_VERBOSE_LOGGING=false
AST_INCLUDE_NODE_TEXT_IN_LOGS=false
```

Runtime config updates may override some values in memory.

Runtime config updates must not change:

```text
workspace_path
parser registry
language grammar paths
workspace boundary policy
```

Changing those requires restarting the AST MCP server.

---

## 11. Tool: ast_complexity_summary

### Purpose

Return structural complexity information for one file or a bounded workspace scan.

This is not a full compiler-grade complexity analysis.

It is a structural heuristic used for agent planning and maintainability detection.

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ComplexitySummaryInput {
    pub file_path: Option<String>,
    pub glob: Option<String>,
    pub max_files: Option<usize>,
    pub include_functions: Option<bool>,
    pub include_classes: Option<bool>,
    pub max_results: Option<usize>,
}
```

Defaults:

```json
{
  "max_files": 200,
  "include_functions": true,
  "include_classes": true,
  "max_results": 500
}
```

### Output Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ComplexitySummaryResult {
    pub total_files_scanned: usize,
    pub total_nodes_analyzed: usize,
    pub files: Vec<FileComplexitySummary>,
    pub hotspots: Vec<ComplexityHotspot>,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileComplexitySummary {
    pub file_path: String,
    pub line_count: usize,
    pub function_count: usize,
    pub class_count: usize,
    pub import_count: usize,
    pub max_nesting_depth: usize,
    pub max_function_lines: usize,
    pub branch_count: usize,
    pub loop_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ComplexityHotspot {
    pub file_path: String,
    pub name: Option<String>,
    pub kind: String,
    pub range: Range,
    pub line_count: usize,
    pub branch_count: usize,
    pub loop_count: usize,
    pub nesting_depth: usize,
    pub risk: String, // low | medium | high
    pub reasons: Vec<String>,
}
```

### Behavior

1. Validate workspace boundaries.
2. If `file_path` is provided, analyze one file.
3. If `glob` is provided, scan bounded workspace files.
4. Respect `.gitignore` by default.
5. Exclude configured default globs.
6. Parse files with language-specific Tree-sitter parsers.
7. Count structural metrics:
   - lines
   - imports
   - top-level declarations
   - functions/methods
   - classes/structs/interfaces where supported
   - branch nodes
   - loop nodes
   - nesting depth
8. Return hotspots sorted by risk, line count, and nesting depth.
9. Apply `max_results`.

### Risk Heuristics

High risk if:

```text
function or method > 100 lines
nesting depth >= 6
branch count >= 20
loop count >= 5
class/module > 500 lines
```

Medium risk if:

```text
function or method > 50 lines
nesting depth >= 4
branch count >= 10
```

Low risk otherwise.

---

## 12. Tool: ast_detect_large_nodes

### Purpose

Find large syntax nodes such as huge functions, classes, components, test suites, or modules.

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DetectLargeNodesInput {
    pub file_path: Option<String>,
    pub glob: Option<String>,
    pub max_files: Option<usize>,
    pub min_lines: Option<usize>,
    pub node_kinds: Option<Vec<String>>,
    pub max_results: Option<usize>,
}
```

Defaults:

```json
{
  "max_files": 200,
  "min_lines": 80,
  "max_results": 200
}
```

### Output Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DetectLargeNodesResult {
    pub nodes: Vec<LargeNode>,
    pub scanned_files: usize,
    pub returned: usize,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LargeNode {
    pub file_path: String,
    pub kind: String,
    pub name: Option<String>,
    pub range: Range,
    pub line_count: usize,
    pub child_count: usize,
    pub nesting_depth: usize,
}
```

### Behavior

- Use AST node ranges to compute line counts.
- Support language-specific mapping of function/class/module/test/component node kinds.
- Sort by line count descending.
- Apply `min_lines` and `max_results`.

---

## 13. Tool: ast_detect_duplicate_shapes

### Purpose

Detect structurally similar code shapes.

This is a heuristic clone-detection feature, not a complete duplicate-code engine.

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DetectDuplicateShapesInput {
    pub glob: String,
    pub max_files: Option<usize>,
    pub min_node_lines: Option<usize>,
    pub node_kinds: Option<Vec<String>>,
    pub normalize_identifiers: Option<bool>,
    pub normalize_literals: Option<bool>,
    pub max_candidates: Option<usize>,
}
```

Defaults:

```json
{
  "max_files": 200,
  "min_node_lines": 10,
  "normalize_identifiers": true,
  "normalize_literals": true,
  "max_candidates": 200
}
```

### Output Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DetectDuplicateShapesResult {
    pub groups: Vec<DuplicateShapeGroup>,
    pub scanned_files: usize,
    pub candidate_count: usize,
    pub returned: usize,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuplicateShapeGroup {
    pub fingerprint: String,
    pub similarity_kind: String, // exact_shape | normalized_shape
    pub occurrences: Vec<DuplicateShapeOccurrence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuplicateShapeOccurrence {
    pub file_path: String,
    pub kind: String,
    pub name: Option<String>,
    pub range: Range,
    pub line_count: usize,
}
```

### Behavior

1. Walk files matching glob with limits.
2. Extract candidate nodes:
   - functions
   - methods
   - classes
   - test blocks
   - route handlers
   - configurable node kinds
3. Convert each candidate to a structural fingerprint.
4. If `normalize_identifiers`, replace identifiers with placeholders.
5. If `normalize_literals`, replace literals with placeholders.
6. Group nodes with same fingerprint.
7. Return groups with at least two occurrences.
8. Sort by occurrence count and line count.

### Safety and Performance

- Enforce `max_files`.
- Enforce `max_candidates`.
- Enforce duplicate detection timeout.
- Do not return full node text by default.

---

## 14. Tool: ast_cache_status

### Purpose

Return cache sizes, TTLs, and cache health.

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CacheStatusInput {}
```

### Output Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CacheStatusResult {
    pub caches: AstCacheStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AstCacheStatus {
    pub parse_trees: CacheSectionStatus,
    pub query_results: CacheSectionStatus,
    pub framework_results: CacheSectionStatus,
    pub request_log: RequestLogCacheStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CacheSectionStatus {
    pub entries: usize,
    pub max_entries: Option<usize>,
    pub ttl_ms: u64,
    pub estimated_bytes: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RequestLogCacheStatus {
    pub entries: usize,
    pub max_entries: usize,
}
```

### Behavior

Report:

```text
parse tree cache entries
query result cache entries
framework extraction cache entries
request log entries
TTL values
estimated memory usage if available
```

---

## 15. Tool: ast_clear_caches

### Purpose

Clear selected AST caches.

### Input Schema

```rust
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
```

### Output Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ClearCachesResult {
    pub cleared: std::collections::BTreeMap<String, usize>,
}
```

### Behavior

- `all` clears all caches.
- Clearing parse trees should also clear dependent query/framework result caches.
- Return number of entries cleared per cache.

---

## 16. Tool: ast_request_log

### Purpose

Return recent AST MCP request history.

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RequestLogInput {
    pub tool: Option<String>,
    pub status: Option<RequestStatus>,
    pub file_path: Option<String>,
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
```

Defaults:

```json
{
  "limit": 50
}
```

### Output Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RequestLogResult {
    pub entries: Vec<RequestLogEntry>,
    pub returned: usize,
    pub total_stored: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RequestLogEntry {
    pub id: String,
    pub tool: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub duration_ms: Option<u64>,
    pub status: RequestStatus,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub file_path: Option<String>,
    pub result_count: Option<usize>,
}
```

### Privacy Rules

Do not log by default:

```text
full file contents
node text
raw query result payloads
rewrite content
```

Allowed by default:

```text
tool name
duration
status
workspace-relative file path
result count
error code
error message
```

---

## 17. Tool: ast_clear_request_log

### Purpose

Clear request log entries.

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ClearRequestLogInput {
    pub tool: Option<String>,
}
```

### Output Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ClearRequestLogResult {
    pub cleared: usize,
}
```

---

## 18. Tool: ast_get_config

### Purpose

Return effective runtime configuration.

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetConfigInput {
    pub include_defaults: Option<bool>,
}
```

Defaults:

```json
{
  "include_defaults": false
}
```

### Output Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetConfigResult {
    pub config: RuntimeConfig,
    pub sources: Option<ConfigSources>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConfigSources {
    pub defaults: serde_json::Value,
    pub environment: serde_json::Value,
    pub runtime_overrides: serde_json::Value,
}
```

---

## 19. Tool: ast_update_runtime_config

### Purpose

Update safe runtime configuration values in memory.

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateRuntimeConfigInput {
    pub limits: Option<PartialRuntimeLimits>,
    pub timeouts_ms: Option<PartialRuntimeTimeouts>,
    pub caches: Option<PartialRuntimeCaches>,
    pub scans: Option<PartialRuntimeScans>,
    pub debug: Option<PartialRuntimeDebug>,
}
```

Partial structs contain optional versions of corresponding config fields.

### Output Schema

```rust
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
```

### Behavior

Allowed runtime updates:

```text
limits
timeouts
cache TTLs
request log max entries
scan parallelism
verbosity flags
```

Rejected runtime updates:

```text
workspace_path
parser registry
language grammar paths
workspace isolation policy
```

Validation rules:

```text
numbers must be positive
max_parallelism must be between 1 and configured maximum
max_file_bytes must not exceed configured hard safety ceiling
request log max entries must be bounded
```

---

## 20. Tool: ast_readiness

### Purpose

Report whether the AST MCP server is ready to serve structural analysis requests.

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReadinessInput {
    pub require_languages: Option<Vec<String>>,
}
```

### Output Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReadinessResult {
    pub ready: bool,
    pub workspace_path: String,
    pub checks: Vec<ReadinessCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReadinessCheck {
    pub name: String,
    pub ok: bool,
    pub message: Option<String>,
}
```

### Required Checks

```text
workspace exists
workspace is directory
workspace is readable
required parsers are registered
required languages are available
cache stores are initialized
runtime config is valid
```

---

## 21. Tool: ast_liveness

### Purpose

Report whether the AST MCP server process is alive and internally responsive.

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LivenessInput {}
```

### Output Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LivenessResult {
    pub alive: bool,
    pub uptime_ms: u64,
    pub started_at: String,
    pub memory: Option<MemoryUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemoryUsage {
    pub rss_bytes: Option<u64>,
    pub heap_bytes: Option<u64>,
}
```

### Behavior

- Must be fast.
- Must not scan files.
- Must not parse files.
- Must not initialize expensive caches.

---

## 22. Tool: ast_workspace_scan_status

### Purpose

Return status of currently running workspace-wide scans.

Workspace-wide tools include:

```text
ast_query_workspace
ast_complexity_summary with glob
ast_detect_large_nodes with glob
ast_detect_duplicate_shapes
framework-wide extractors
```

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceScanStatusInput {
    pub scan_id: Option<String>,
}
```

### Output Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceScanStatusResult {
    pub scans: Vec<WorkspaceScanInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceScanInfo {
    pub scan_id: String,
    pub tool: String,
    pub status: String, // running | completed | failed | cancelled
    pub started_at: String,
    pub completed_at: Option<String>,
    pub files_discovered: usize,
    pub files_processed: usize,
    pub results_found: usize,
    pub error: Option<String>,
}
```

### Behavior

- Track active and recent scans.
- Keep completed scan metadata for a short TTL.
- Do not store full result payloads unless required by a separate result cache.

---

## 23. Tool: ast_cancel_workspace_scan

### Purpose

Cancel a running workspace-wide scan.

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CancelWorkspaceScanInput {
    pub scan_id: String,
}
```

### Output Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CancelWorkspaceScanResult {
    pub scan_id: String,
    pub cancelled: bool,
    pub previous_status: String,
    pub new_status: String,
}
```

### Behavior

- Support cooperative cancellation.
- Stop scheduling new files.
- Allow already-running parse tasks to finish or be dropped safely.
- Mark scan as cancelled.

---

## 24. Tool: ast_parser_status

### Purpose

Return parser registry and parser health.

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ParserStatusInput {
    pub language: Option<String>,
}
```

### Output Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ParserStatusResult {
    pub parsers: Vec<ParserStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ParserStatus {
    pub language: String,
    pub extensions: Vec<String>,
    pub available: bool,
    pub parser_name: String,
    pub version: Option<String>,
    pub query_count: usize,
    pub last_error: Option<String>,
}
```

### Behavior

- Do not parse files.
- Report parser registry readiness.
- Report language extension mapping.

---

## 25. Tool: ast_rebuild_parser_cache

### Purpose

Clear and rebuild parser-related caches.

This does not recompile parsers; it refreshes runtime parser/query cache objects.

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RebuildParserCacheInput {
    pub languages: Option<Vec<String>>,
}
```

### Output Schema

```rust
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
```

### Behavior

- Clear parser/query cache objects for selected languages.
- Reinitialize parsers and precompiled queries if used.
- Does not scan workspace files.

---

## 26. Request Tracking

Every tool call should be tracked.

Required fields:

```text
id
tool
started_at
completed_at
duration_ms
status
error_code
error_message
file_path if applicable
result_count if cheap to compute
```

Request tracking must not log full source text by default.

---

## 27. Workspace Scan Tracking

Workspace-wide tools should create a scan record.

A scan record tracks:

```text
scan_id
tool
status
files_discovered
files_processed
results_found
start/completion timestamps
cancellation state
error state
```

Workspace scans must be bounded by:

```text
max_files
max_results
timeout
parallelism
exclude globs
workspace boundary
```

---

## 28. Cache Strategy

Recommended caches:

```text
parse tree cache
query result cache
framework extraction result cache
request log ring buffer
recent workspace scan metadata
```

### Parse Tree Cache Key

```text
workspace_path + file_path + file_mtime + file_size + parser_language
```

If file mtime or size changes, invalidate parse tree.

### Query Result Cache Key

```text
file_path + query_string + file_mtime + file_size + parser_language
```

### Framework Result Cache Key

```text
file_path + extractor_name + extractor_version + file_mtime + file_size
```

---

## 29. Structural Metric Definitions

V5 metrics are heuristic.

### Branch nodes

Examples:

```text
if / else
switch / match
conditional expression
case/default arms
try/catch where applicable
```

### Loop nodes

Examples:

```text
for
for...of
for...in
while
do while
loop
foreach
```

### Nesting depth

Maximum depth of nested control-flow or block-like nodes inside a function/method.

### Function lines

Range line count from function/method start to end.

### Class/module lines

Range line count from class/module/struct declaration start to end.

---

## 30. Duplicate Shape Fingerprinting

V5 duplicate detection uses structural fingerprints.

Recommended fingerprint pipeline:

```text
1. Select candidate node.
2. Traverse subtree.
3. Emit node kind sequence.
4. Optionally normalize identifiers.
5. Optionally normalize literals.
6. Ignore comments/whitespace.
7. Hash normalized structural sequence.
```

Example normalized shape:

```text
function_declaration(identifier)(parameters(identifier))(block(return_statement(call_expression(identifier))))
```

This is intentionally approximate.

Do not claim semantic equivalence.

---

## 31. Performance Requirements

V5 must enforce:

```text
max file size
max workspace files
max workspace results
max parse tree nodes
max parallelism
request timeouts
scan cancellation
cache TTLs
```

Workspace-wide scans should process files in parallel but bounded.

Recommended max parallelism default:

```text
8 workers
```

The implementation should avoid loading all file contents into memory at once for large workspace scans.

---

## 32. Error Codes Added in V5

```text
complexity_analysis_failed
large_node_detection_failed
duplicate_shape_detection_failed
cache_unavailable
cache_clear_failed
request_log_unavailable
invalid_runtime_config
config_update_rejected
readiness_check_failed
workspace_scan_not_found
workspace_scan_cancel_failed
workspace_scan_already_completed
parser_status_unavailable
parser_rebuild_failed
scan_limit_exceeded
scan_timeout
```

Existing error codes from earlier versions remain valid:

```text
workspace_not_found
path_outside_workspace
file_not_found
unsupported_language
parse_failed
query_failed
invalid_range
internal_error
```

---

## 33. Updated Tool Surface

```rust
pub trait AstMcpV5Tools {
    // V5 metrics and analysis
    fn ast_complexity_summary(input: ComplexitySummaryInput) -> ComplexitySummaryResult;
    fn ast_detect_large_nodes(input: DetectLargeNodesInput) -> DetectLargeNodesResult;
    fn ast_detect_duplicate_shapes(input: DetectDuplicateShapesInput) -> DetectDuplicateShapesResult;

    // V5 cache management
    fn ast_cache_status(input: CacheStatusInput) -> CacheStatusResult;
    fn ast_clear_caches(input: ClearCachesInput) -> ClearCachesResult;

    // V5 observability
    fn ast_request_log(input: RequestLogInput) -> RequestLogResult;
    fn ast_clear_request_log(input: ClearRequestLogInput) -> ClearRequestLogResult;

    // V5 config
    fn ast_get_config(input: GetConfigInput) -> GetConfigResult;
    fn ast_update_runtime_config(input: UpdateRuntimeConfigInput) -> UpdateRuntimeConfigResult;

    // V5 health
    fn ast_readiness(input: ReadinessInput) -> ReadinessResult;
    fn ast_liveness(input: LivenessInput) -> LivenessResult;

    // V5 workspace scan management
    fn ast_workspace_scan_status(input: WorkspaceScanStatusInput) -> WorkspaceScanStatusResult;
    fn ast_cancel_workspace_scan(input: CancelWorkspaceScanInput) -> CancelWorkspaceScanResult;

    // V5 parser operations
    fn ast_parser_status(input: ParserStatusInput) -> ParserStatusResult;
    fn ast_rebuild_parser_cache(input: RebuildParserCacheInput) -> RebuildParserCacheResult;
}
```

Actual MCP handlers may be async.

---

## 34. Example Tool Calls

### Complexity summary for one file

```json
{
  "tool": "ast_complexity_summary",
  "arguments": {
    "file_path": "src/user.ts",
    "include_functions": true,
    "include_classes": true
  }
}
```

### Complexity summary for workspace

```json
{
  "tool": "ast_complexity_summary",
  "arguments": {
    "glob": "src/**/*.{ts,tsx}",
    "max_files": 200,
    "max_results": 100
  }
}
```

### Detect large nodes

```json
{
  "tool": "ast_detect_large_nodes",
  "arguments": {
    "glob": "src/**/*.ts",
    "min_lines": 80,
    "max_files": 200,
    "max_results": 100
  }
}
```

### Detect duplicate shapes

```json
{
  "tool": "ast_detect_duplicate_shapes",
  "arguments": {
    "glob": "src/**/*.ts",
    "min_node_lines": 12,
    "normalize_identifiers": true,
    "normalize_literals": true,
    "max_files": 200
  }
}
```

### Cache status

```json
{
  "tool": "ast_cache_status",
  "arguments": {}
}
```

### Clear caches

```json
{
  "tool": "ast_clear_caches",
  "arguments": {
    "caches": ["parse_trees", "query_results"]
  }
}
```

### Request log

```json
{
  "tool": "ast_request_log",
  "arguments": {
    "status": "timeout",
    "limit": 20
  }
}
```

### Get config

```json
{
  "tool": "ast_get_config",
  "arguments": {
    "include_defaults": true
  }
}
```

### Update runtime config

```json
{
  "tool": "ast_update_runtime_config",
  "arguments": {
    "limits": {
      "max_workspace_files": 1000
    },
    "scans": {
      "max_parallelism": 12
    }
  }
}
```

### Readiness

```json
{
  "tool": "ast_readiness",
  "arguments": {
    "require_languages": ["typescript", "python"]
  }
}
```

### Liveness

```json
{
  "tool": "ast_liveness",
  "arguments": {}
}
```

### Workspace scan status

```json
{
  "tool": "ast_workspace_scan_status",
  "arguments": {}
}
```

### Cancel workspace scan

```json
{
  "tool": "ast_cancel_workspace_scan",
  "arguments": {
    "scan_id": "7ef46902-9a08-47ef-b6cc-4f61e4f63a0f"
  }
}
```

### Parser status

```json
{
  "tool": "ast_parser_status",
  "arguments": {}
}
```

### Rebuild parser cache

```json
{
  "tool": "ast_rebuild_parser_cache",
  "arguments": {
    "languages": ["typescript", "python"]
  }
}
```

---

## 35. Internal Architecture Additions

V5 adds these modules:

```text
src/
  analysis/
    complexity.rs
    large_nodes.rs
    duplicate_shapes.rs
    fingerprints.rs

  cache/
    parse_tree_cache.rs
    query_result_cache.rs
    framework_result_cache.rs
    cache_status.rs
    clear_caches.rs

  config/
    defaults.rs
    env_config.rs
    runtime_config.rs
    validate_config.rs

  observability/
    request_log.rs
    request_tracker.rs
    scan_tracker.rs
    metrics.rs

  ops/
    readiness.rs
    liveness.rs
    parser_status.rs
    rebuild_parser_cache.rs

  scan/
    workspace_scan.rs
    cancel.rs
    file_discovery.rs
    parallel.rs
```

Existing modules from V1–V4 should remain separate:

```text
parser/
queries/
extractors/
rewrites/
safety/
workspace/
types/
```

---

## 36. Development Milestones

### Milestone 1: Runtime Config

Implement:

```text
default config
environment config
runtime override config
config validation
ast_get_config
ast_update_runtime_config
```

### Milestone 2: Request Logging

Implement:

```text
request tracker
request log ring buffer
request log filtering
ast_request_log
ast_clear_request_log
```

### Milestone 3: Cache Operations

Implement:

```text
parse tree cache status
query result cache status
framework result cache status
ast_cache_status
ast_clear_caches
```

### Milestone 4: Health and Parser Ops

Implement:

```text
ast_readiness
ast_liveness
ast_parser_status
ast_rebuild_parser_cache
```

### Milestone 5: Workspace Scan Tracking

Implement:

```text
scan IDs
scan status tracking
cooperative cancellation
ast_workspace_scan_status
ast_cancel_workspace_scan
```

### Milestone 6: Complexity Summary

Implement:

```text
branch counting
loop counting
nesting depth
function/class line counts
hotspot ranking
ast_complexity_summary
```

### Milestone 7: Large Node Detection

Implement:

```text
node line counting
configurable node kinds
workspace bounded scan
ast_detect_large_nodes
```

### Milestone 8: Duplicate Shape Detection

Implement:

```text
candidate extraction
structural fingerprinting
identifier/literal normalization
grouping and ranking
ast_detect_duplicate_shapes
```

### Milestone 9: Performance and Safety Tests

Test limits, timeouts, cache invalidation, cancellation, and large repo behavior.

---

## 37. Acceptance Criteria

V5 is acceptable when all V1–V4 acceptance criteria still pass and the following are true.

### Complexity Summary

- `ast_complexity_summary` works for one file.
- `ast_complexity_summary` works for bounded workspace globs.
- Hotspots are sorted by risk.
- Limits and timeouts are enforced.

### Large Node Detection

- `ast_detect_large_nodes` finds large functions/classes/modules.
- It supports `min_lines`.
- It respects glob and max file limits.

### Duplicate Shape Detection

- `ast_detect_duplicate_shapes` finds repeated structural shapes.
- It supports identifier/literal normalization.
- It does not return full node text by default.
- It enforces candidate and workspace limits.

### Cache Management

- `ast_cache_status` reports parse, query, framework, and request-log caches.
- `ast_clear_caches` clears selected caches.
- Clearing parse trees clears dependent query/framework caches.

### Request Logging

- Tool calls are logged with duration and status.
- Timeouts are recorded.
- Logs do not include full file contents by default.
- Logs can be filtered by tool/status/file.

### Runtime Config

- `ast_get_config` returns effective config.
- `ast_update_runtime_config` updates allowed fields.
- Invalid config values are rejected.
- Workspace path and parser registry cannot be changed at runtime.

### Health

- `ast_readiness` checks workspace, parser registry, and cache initialization.
- `ast_liveness` is fast and does not parse or scan files.

### Workspace Scans

- Workspace-wide tools create scan records.
- `ast_workspace_scan_status` reports active/recent scans.
- `ast_cancel_workspace_scan` cooperatively cancels running scans.

### Parser Ops

- `ast_parser_status` returns parser registry health.
- `ast_rebuild_parser_cache` refreshes selected parser/query caches.

### Safety

- No V5 tool writes source files.
- No V5 tool calls LSP MCP.
- All paths remain workspace-bound.
- Workspace scans are bounded.
- Cache/log tools do not leak full source text by default.

---

## 38. Testing Requirements

### Unit Tests

Required unit tests:

```text
runtime config validation
environment config parsing
request log ring buffer
request log filtering
cache status counting
cache clearing dependency behavior
workspace scan tracker
scan cancellation state transitions
parser status registry
branch counting
loop counting
nesting depth calculation
large node detection
duplicate fingerprint generation
identifier normalization
literal normalization
```

### Integration Tests

Required integration tests:

```text
complexity summary on TypeScript file
complexity summary on Python file
large node detection on TypeScript workspace
duplicate shape detection on sample duplicate functions
cache status after parse/query calls
clear parse tree cache
request log after successful tool call
request log after failed tool call
readiness with required languages
liveness fast response
workspace scan status during query_workspace
cancel workspace scan
parser status for supported languages
rebuild parser cache for TypeScript/Python
```

### Safety Tests

Required safety tests:

```text
reject paths outside workspace
workspace scans respect max_files
workspace scans respect default excludes
duplicate detection respects max_candidates
request logs omit file text
runtime config rejects negative values
runtime config rejects workspace_path update
liveness does not parse files
readiness does not scan workspace
cancel unknown scan returns structured error
```

---

## 39. Agent Skill Guidance for V5

Agent Skills should use V5 tools for planning and operations.

### Before large refactors

```text
1. ast_complexity_summary
2. ast_detect_large_nodes
3. ast_find_tests or framework extraction from V3
4. LSP MCP diagnostics separately if needed
```

### Before workspace-wide structural search

```text
1. ast_readiness
2. ast_get_config
3. ast_query_workspace or framework extractor
4. ast_workspace_scan_status if long-running
```

### Long-running agent sessions

```text
1. ast_cache_status periodically
2. ast_request_log for timeouts/errors
3. ast_clear_caches if cache grows too large
4. ast_parser_status if parser behavior seems wrong
```

### Performance troubleshooting

```text
1. ast_request_log with status=timeout
2. ast_cache_status
3. ast_get_config
4. adjust limits or parallelism via ast_update_runtime_config
```

---

## 40. Final V5 Design Principle

V5 makes AST MCP production-grade.

It should be:

```text
fast
bounded
observable
cache-aware
configurable
safe for long-running sessions
useful for repository-scale structural planning
```

It must preserve the core AST boundary:

```text
AST MCP understands structure.
AST MCP does not own semantic compiler truth.
AST MCP does not call LSP MCP.
AST MCP does not write files.
```

