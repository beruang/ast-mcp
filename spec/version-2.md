# AST MCP Server — Version 2 Specification

## 1. Purpose

Version 2 extends the Rust-based AST MCP server from core structural extraction into **context selection and pattern search**.

Version 1 focused on the Core Structural MVP:

- parse files
- inspect file structure
- extract top-level nodes
- locate enclosing syntax nodes
- extract imports and exports
- extract functions and classes
- chunk files by syntax structure
- run bounded Tree-sitter queries

Version 2 adds tools for:

- scope-aware context extraction
- node lookup by range
- node text retrieval
- context packs for agent editing tasks
- call expression extraction
- member/property access extraction
- literal and template literal extraction
- bounded workspace-wide AST queries
- structural file metrics

The AST MCP server remains responsible for **syntax and structure**, not semantic meaning.

It must remain independent from the LSP MCP server.

```text
LSP MCP = semantic intelligence
AST MCP = structural intelligence
```

---

## 2. Architectural Boundary

This MCP server is the **AST MCP**.

It exposes only `ast_*` tools.

It must not call:

```text
lsp_* tools
LSP MCP
language servers
TypeScript language service
Pyright
gopls
rust-analyzer
semantic reference engines
```

The AST MCP must remain usable even when LSP MCP is unavailable.

### AST MCP owns

```text
syntax parsing
file structure extraction
AST node inspection
imports/exports extraction
function/class extraction
call/literal/template extraction
syntax-aware context selection
syntax-aware file chunking
Tree-sitter query execution
bounded workspace-wide structural search
structural metrics
```

### AST MCP does not own

```text
type information
symbol references
semantic rename
compiler diagnostics
hover information
implementation lookup
call hierarchy
type hierarchy
language-server lifecycle
```

Those belong to LSP MCP.

### Composite workflows

Higher-level workflows such as:

```text
code_context_pack
code_inspect_symbol
code_prepare_safe_rename
code_analyze_change_impact
code_prepare_diagnostic_fix
```

must live in one of:

```text
Agent Skills
Code Composite MCP
client-side orchestration layer
```

They must not be embedded inside the AST MCP if they require LSP.

---

## 3. Version 2 Goals

V2 adds the following tools:

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

V2 should make AST MCP useful for agent workflows that need focused local context or syntax-pattern search.

Example questions V2 should answer:

```text
What syntactic scope contains this position?
What AST node exactly covers this diagnostic range?
Give me the source text for this node.
Give me imports + enclosing function + enclosing class for this edit.
Find all calls to router.get in this file.
Find string literals containing /api.
Find tagged template literals like sql`...`.
Run this Tree-sitter query across src/**/*.ts with strict limits.
How large/complex is this file structurally?
```

---

## 4. Version 2 Non-Goals

V2 does not include:

```text
direct file mutation
rewrite previews
import insertion previews
route extraction
React component extraction
test extraction
decorator extraction
schema/model extraction
semantic references
compiler diagnostics
LSP-backed type information
persistent indexing database
unbounded workspace scans
```

These belong to later versions or other services.

---

## 5. Required Baseline From Version 1

V2 assumes all V1 tools already exist.

V1 baseline tools:

```text
ast_health_check
ast_list_supported_languages
ast_parse_file
ast_file_outline
ast_top_level_nodes
ast_node_tree
ast_enclosing_node
ast_find_imports
ast_find_exports
ast_find_functions
ast_find_classes
ast_chunk_file
ast_query
```

V2 must preserve backward compatibility with V1 input/output schemas unless a safety correction is required.

---

## 6. Recommended Tech Stack

```text
Language: Rust
Runtime: native binary
Parser: Tree-sitter
Transport: MCP stdio
Serialization: serde / serde_json
Schema generation: schemars
Error handling: thiserror / anyhow
Async runtime: tokio
Filesystem traversal: ignore / walkdir
Glob matching: globset
Parallelism: rayon or tokio task pool
Diff support: not required in V2
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
tree-sitter = "0.22"
ignore = "0.4"
walkdir = "2"
globset = "0.4"
rayon = "1"
uuid = { version = "1", features = ["v4", "serde"] }
```

Tree-sitter grammar crates depend on the chosen language set.

Recommended V2 language support:

```text
Required:
  TypeScript
  JavaScript
  Python

Optional:
  Go
  Rust
```

---

## 7. Workspace Model

The AST MCP server runs against exactly one workspace root in V2.

Workspace root is provided by:

```bash
WORKSPACE_PATH=/absolute/path/to/repo
```

If omitted, the server may use the current working directory.

All file paths accepted by tools must resolve inside `WORKSPACE_PATH`.

Allowed:

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

V2 workspace-wide tools must also enforce:

```text
maxFiles
maxResults
maxBytesPerFile
allowed extensions
ignored directories
```

Default ignored directories:

```text
.git
node_modules
dist
build
coverage
.target
target
.venv
venv
__pycache__
```

---

## 8. Position and Encoding Rules

Shared external API positions should follow the same convention as LSP-compatible contracts:

```text
line: zero-based
character: zero-based UTF-16 code-unit offset
```

Tree-sitter internally uses byte offsets and row/column points.

V2 must provide conversion helpers:

```text
external Position → byte offset
byte offset → external Position
Range → byte range
byte range → Range
```

Important requirement:

```text
Never treat UTF-8 byte offsets as external character offsets.
```

For files containing non-ASCII characters, conversion must remain correct.

If exact UTF-16 conversion is not fully implemented in early development, the limitation must be documented and tests must be added before production use.

---

## 9. Shared Types

### Position

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}
```

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

### AstNodeSummary

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AstNodeSummary {
    pub id: Option<String>,
    pub kind: String,
    pub name: Option<String>,
    pub range: Range,
    pub byte_range: Option<(usize, usize)>,
    pub text: Option<String>,
    pub children: Option<Vec<AstNodeSummary>>,
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

### ResultLimit

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResultLimit {
    pub returned: usize,
    pub truncated: bool,
    pub total_known: Option<usize>,
}
```

---

## 10. Shared V2 Types

### ScopeKind

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
```

### ScopeSummary

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ScopeSummary {
    pub kind: ScopeKind,
    pub name: Option<String>,
    pub node_kind: String,
    pub range: Range,
    pub selection_range: Option<Range>,
}
```

### ContextBlock

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContextBlock {
    pub label: String,
    pub kind: String,
    pub file_path: String,
    pub range: Range,
    pub text: String,
    pub truncated: bool,
}
```

### CallExpression

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CallExpression {
    pub callee_text: String,
    pub arguments_text: Vec<String>,
    pub range: Range,
    pub enclosing_scope: Option<ScopeSummary>,
}
```

### MemberAccess

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemberAccess {
    pub object_text: String,
    pub property: String,
    pub full_text: String,
    pub range: Range,
    pub enclosing_scope: Option<ScopeSummary>,
}
```

### LiteralMatch

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LiteralMatch {
    pub kind: String,
    pub raw_text: String,
    pub value_text: Option<String>,
    pub range: Range,
    pub enclosing_scope: Option<ScopeSummary>,
}
```

### TemplateLiteralMatch

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TemplateLiteralMatch {
    pub tag: Option<String>,
    pub raw_text: String,
    pub range: Range,
    pub interpolation_count: usize,
    pub enclosing_scope: Option<ScopeSummary>,
}
```

### FileMetrics

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
    pub max_function_lines: Option<usize>,
}
```

---

## 11. Parser Registry Updates

V2 should keep the V1 parser registry and add query packs for call/literal/scope extraction.

Example conceptual registry:

```rust
pub struct LanguageConfig {
    pub language_id: &'static str,
    pub extensions: &'static [&'static str],
    pub tree_sitter_language: fn() -> tree_sitter::Language,
    pub query_packs: QueryPacks,
}

pub struct QueryPacks {
    pub imports: Option<&'static str>,
    pub exports: Option<&'static str>,
    pub functions: Option<&'static str>,
    pub classes: Option<&'static str>,
    pub calls: Option<&'static str>,
    pub member_access: Option<&'static str>,
    pub literals: Option<&'static str>,
    pub template_literals: Option<&'static str>,
    pub scopes: Option<&'static str>,
}
```

V2 does not require perfect extraction across all languages.

Each tool should return clear capability information when a query pack is unavailable for a language.

Example:

```json
{
  "error": {
    "code": "ast_feature_unsupported_for_language",
    "message": "Template literal extraction is not supported for Python."
  }
}
```

---

## 12. V2 Tool Surface

```rust
pub trait AstMcpV2Tools {
    fn ast_enclosing_scope(input: EnclosingScopeInput) -> EnclosingScopeResult;
    fn ast_node_at_range(input: NodeAtRangeInput) -> NodeAtRangeResult;
    fn ast_node_text(input: NodeTextInput) -> NodeTextResult;
    fn ast_context_for_range(input: ContextForRangeInput) -> ContextForRangeResult;
    fn ast_context_pack(input: ContextPackInput) -> ContextPackResult;
    fn ast_find_calls(input: FindCallsInput) -> FindCallsResult;
    fn ast_find_member_access(input: FindMemberAccessInput) -> FindMemberAccessResult;
    fn ast_find_literals(input: FindLiteralsInput) -> FindLiteralsResult;
    fn ast_find_template_literals(input: FindTemplateLiteralsInput) -> FindTemplateLiteralsResult;
    fn ast_query_workspace(input: QueryWorkspaceInput) -> QueryWorkspaceResult;
    fn ast_file_metrics(input: FileMetricsInput) -> FileMetricsResult;
}
```

---

# Tool Specifications

## 13. Tool: ast_enclosing_scope

### Purpose

Return the syntactic scope chain enclosing a source position.

This is different from `ast_enclosing_node`:

- `ast_enclosing_node` can return any AST node.
- `ast_enclosing_scope` returns scope-like containers only.

Examples of scopes:

```text
module
class
function
method
constructor
arrow function
lambda
block
```

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnclosingScopeInput {
    pub file_path: String,
    pub position: Position,
    pub include_block_scopes: Option<bool>,
}
```

Defaults:

```json
{
  "includeBlockScopes": false
}
```

### Output Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnclosingScopeResult {
    pub file_path: String,
    pub position: Position,
    pub scopes: Vec<ScopeSummary>, // outermost to innermost
}
```

### Behavior

1. Validate file path.
2. Parse file.
3. Convert input position to byte offset.
4. Find leaf node at position.
5. Walk ancestors.
6. Keep only scope-like nodes.
7. Normalize scope kind/name/range.
8. Sort from outermost to innermost.

### Example Output

```json
{
  "filePath": "src/user.ts",
  "position": { "line": 20, "character": 12 },
  "scopes": [
    {
      "kind": "module",
      "name": null,
      "nodeKind": "program",
      "range": {
        "start": { "line": 0, "character": 0 },
        "end": { "line": 120, "character": 0 }
      }
    },
    {
      "kind": "class",
      "name": "UserService",
      "nodeKind": "class_declaration",
      "range": {}
    },
    {
      "kind": "method",
      "name": "getUser",
      "nodeKind": "method_definition",
      "range": {}
    }
  ]
}
```

---

## 14. Tool: ast_node_at_range

### Purpose

Return the smallest AST node that exactly matches or contains a source range.

Useful for:

```text
diagnostic ranges
selected text
edit target validation
context extraction
```

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NodeAtRangeInput {
    pub file_path: String,
    pub range: Range,
    pub mode: Option<NodeAtRangeMode>,
    pub include_text: Option<bool>,
    pub max_text_bytes: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NodeAtRangeMode {
    Exact,
    SmallestContaining,
    LargestContained,
}
```

Defaults:

```json
{
  "mode": "smallest_containing",
  "includeText": true,
  "maxTextBytes": 12000
}
```

### Output Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NodeAtRangeResult {
    pub file_path: String,
    pub range: Range,
    pub node: Option<AstNodeSummary>,
    pub ancestors: Vec<AstNodeSummary>,
    pub matched_mode: String,
}
```

### Behavior

1. Validate range.
2. Parse file.
3. Convert range to byte range.
4. Find node according to mode:
   - `exact`: node byte range must equal input byte range.
   - `smallest_containing`: smallest node containing the full range.
   - `largest_contained`: largest node fully inside the range.
5. Return node and ancestor chain.
6. Include text only if requested and under byte limit.

### Error Cases

```text
invalid_range
range_out_of_bounds
file_not_found
unsupported_language
```

---

## 15. Tool: ast_node_text

### Purpose

Return exact source text for a node or range without returning the whole file.

This tool can be used after:

```text
ast_node_at_range
ast_enclosing_node
ast_enclosing_scope
ast_query
```

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NodeTextInput {
    pub file_path: String,
    pub range: Range,
    pub max_bytes: Option<usize>,
}
```

Defaults:

```json
{
  "maxBytes": 20000
}
```

### Output Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NodeTextResult {
    pub file_path: String,
    pub range: Range,
    pub text: String,
    pub truncated: bool,
    pub byte_count: usize,
}
```

### Behavior

1. Validate path.
2. Read file.
3. Validate range bounds.
4. Convert range to byte offsets.
5. Slice text.
6. Truncate if larger than `maxBytes`.

### Safety

This tool only reads source text.

It must not mutate files.

---

## 16. Tool: ast_context_for_range

### Purpose

Return minimal useful syntax context around a source range.

This is useful for:

```text
diagnostics
agent edit planning
reviewing selected text
understanding local code around a range
```

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContextForRangeInput {
    pub file_path: String,
    pub range: Range,
    pub include_parents: Option<bool>,
    pub include_siblings: Option<bool>,
    pub max_parent_depth: Option<usize>,
    pub max_context_bytes: Option<usize>,
}
```

Defaults:

```json
{
  "includeParents": true,
  "includeSiblings": false,
  "maxParentDepth": 4,
  "maxContextBytes": 20000
}
```

### Output Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContextForRangeResult {
    pub file_path: String,
    pub target: Option<AstNodeSummary>,
    pub parents: Vec<AstNodeSummary>,
    pub siblings: Vec<AstNodeSummary>,
    pub context_blocks: Vec<ContextBlock>,
    pub truncated: bool,
}
```

### Behavior

1. Validate range.
2. Call the same node selection logic as `ast_node_at_range` with `smallest_containing`.
3. Include target node text.
4. Optionally include parent nodes up to `maxParentDepth`.
5. Optionally include immediate sibling summaries.
6. Enforce `maxContextBytes` globally.

### Context Block Labels

Recommended labels:

```text
target_node
parent_1
parent_2
sibling_before
sibling_after
```

---

## 17. Tool: ast_context_pack

### Purpose

Return a compact, agent-ready structural context pack for a file position or range.

This is one of the highest-value AST V2 tools.

It should provide the minimum structural context needed before editing or reasoning about a code region.

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContextPackInput {
    pub file_path: String,
    pub position: Option<Position>,
    pub range: Option<Range>,
    pub include: Option<Vec<ContextPackPart>>,
    pub max_bytes: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
```

Defaults:

```json
{
  "include": [
    "imports",
    "exports",
    "enclosing_scope",
    "enclosing_node",
    "top_level_outline"
  ],
  "maxBytes": 30000
}
```

Exactly one of `position` or `range` should be provided.

If both are provided, `range` takes precedence.

### Output Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContextPackResult {
    pub file_path: String,
    pub language: String,
    pub blocks: Vec<ContextBlock>,
    pub summaries: Vec<serde_json::Value>,
    pub truncated: bool,
}
```

### Behavior

Allowed internal calls:

```text
ast_find_imports
ast_find_exports
ast_enclosing_scope
ast_enclosing_node
ast_file_outline
ast_find_functions
ast_find_classes
```

Forbidden internal calls:

```text
lsp_hover
lsp_definition
lsp_references
lsp_diagnostics
```

### Output Block Examples

```text
imports
exports
enclosing_scope
enclosing_node
top_level_outline
nearby_functions
nearby_classes
```

### Safety

The total response must respect `maxBytes`.

If the context is too large, preserve:

```text
imports
enclosing scope
target node summary
```

and truncate lower-priority blocks first.

---

## 18. Tool: ast_find_calls

### Purpose

Find call expressions in a file.

This is syntax-level only.

It can answer:

```text
Where are calls named getUser?
Where are calls matching router.get?
What arguments are passed to this call expression?
```

It cannot answer:

```text
Which symbol does this call resolve to?
Is this getUser the imported function or a local shadow?
```

Those are semantic questions for LSP MCP.

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FindCallsInput {
    pub file_path: String,
    pub callee: Option<String>,
    pub callee_contains: Option<String>,
    pub include_arguments: Option<bool>,
    pub include_enclosing_scope: Option<bool>,
    pub max_results: Option<usize>,
}
```

Defaults:

```json
{
  "includeArguments": true,
  "includeEnclosingScope": true,
  "maxResults": 200
}
```

### Output Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FindCallsResult {
    pub file_path: String,
    pub calls: Vec<CallExpression>,
    pub returned: usize,
    pub truncated: bool,
}
```

### Behavior

1. Validate path.
2. Parse file.
3. Use language-specific call query pack.
4. Extract callee text.
5. Filter by `callee` exact match if provided.
6. Filter by `callee_contains` if provided.
7. Extract argument text if requested.
8. Extract enclosing scope if requested.
9. Apply `maxResults`.

### Examples

Input:

```json
{
  "filePath": "src/routes.ts",
  "callee": "router.get",
  "maxResults": 100
}
```

Output:

```json
{
  "filePath": "src/routes.ts",
  "calls": [
    {
      "calleeText": "router.get",
      "argumentsText": ["\"/users/:id\"", "getUserHandler"],
      "range": {},
      "enclosingScope": {
        "kind": "function",
        "name": "registerRoutes"
      }
    }
  ],
  "returned": 1,
  "truncated": false
}
```

---

## 19. Tool: ast_find_member_access

### Purpose

Find member/property access expressions.

Examples:

```text
user.profile.email
router.get
ctx.request.body
self.user.name
```

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FindMemberAccessInput {
    pub file_path: String,
    pub property: Option<String>,
    pub object_contains: Option<String>,
    pub full_text_contains: Option<String>,
    pub include_enclosing_scope: Option<bool>,
    pub max_results: Option<usize>,
}
```

Defaults:

```json
{
  "includeEnclosingScope": true,
  "maxResults": 200
}
```

### Output Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FindMemberAccessResult {
    pub file_path: String,
    pub members: Vec<MemberAccess>,
    pub returned: usize,
    pub truncated: bool,
}
```

### Behavior

1. Parse file.
2. Use language-specific member access query.
3. Extract:
   - object text
   - property name
   - full text
   - range
4. Apply filters.
5. Include enclosing scope if requested.
6. Apply result limit.

---

## 20. Tool: ast_find_literals

### Purpose

Find literals in a file.

Useful for:

```text
route paths
feature flags
config keys
SQL fragments
GraphQL strings
magic numbers
```

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FindLiteralsInput {
    pub file_path: String,
    pub literal_kind: Option<LiteralKind>,
    pub contains: Option<String>,
    pub exact: Option<String>,
    pub include_enclosing_scope: Option<bool>,
    pub max_results: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LiteralKind {
    String,
    Number,
    Boolean,
    Null,
    Regex,
    Unknown,
}
```

Defaults:

```json
{
  "includeEnclosingScope": true,
  "maxResults": 200
}
```

### Output Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FindLiteralsResult {
    pub file_path: String,
    pub literals: Vec<LiteralMatch>,
    pub returned: usize,
    pub truncated: bool,
}
```

### Behavior

1. Parse file.
2. Use literal query pack.
3. Normalize literal kind.
4. Extract raw text.
5. Best-effort extract value text without quotes for simple strings.
6. Apply `contains` and `exact` filters against value text if available, otherwise raw text.
7. Apply result limit.

### Notes

This is syntax-level.

It should not evaluate expressions.

For example:

```ts
const path = "/api" + "/users";
```

V2 may return two string literals but should not compute `/api/users`.

---

## 21. Tool: ast_find_template_literals

### Purpose

Find template literals and tagged template literals.

Important for:

```text
SQL templates
GraphQL templates
CSS-in-JS
HTML templates
route builders
logging templates
```

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FindTemplateLiteralsInput {
    pub file_path: String,
    pub tag: Option<String>,
    pub contains: Option<String>,
    pub include_untagged: Option<bool>,
    pub include_enclosing_scope: Option<bool>,
    pub max_results: Option<usize>,
}
```

Defaults:

```json
{
  "includeUntagged": true,
  "includeEnclosingScope": true,
  "maxResults": 100
}
```

### Output Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FindTemplateLiteralsResult {
    pub file_path: String,
    pub templates: Vec<TemplateLiteralMatch>,
    pub returned: usize,
    pub truncated: bool,
}
```

### Behavior

1. Validate language support.
2. Parse file.
3. Find template literal nodes.
4. Detect optional tag expression.
5. Filter by tag if provided.
6. Filter by `contains` against raw text.
7. Count interpolations.
8. Include enclosing scope if requested.

### Unsupported Languages

For languages without template literals, return structured error or empty results depending on language semantics.

Recommended:

```json
{
  "filePath": "src/app.py",
  "templates": [],
  "returned": 0,
  "truncated": false
}
```

---

## 22. Tool: ast_query_workspace

### Purpose

Run a bounded Tree-sitter query across workspace files.

This is a powerful advanced tool and must enforce strict limits.

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QueryWorkspaceInput {
    pub query: String,
    pub language: Option<String>,
    pub glob: Option<String>,
    pub max_files: Option<usize>,
    pub max_results: Option<usize>,
    pub max_bytes_per_file: Option<usize>,
    pub include_text: Option<bool>,
}
```

Defaults:

```json
{
  "maxFiles": 200,
  "maxResults": 1000,
  "maxBytesPerFile": 1000000,
  "includeText": true
}
```

### Output Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QueryWorkspaceResult {
    pub matches: Vec<WorkspaceQueryMatch>,
    pub files_scanned: usize,
    pub files_skipped: usize,
    pub returned: usize,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceQueryMatch {
    pub file_path: String,
    pub captures: Vec<QueryCapture>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QueryCapture {
    pub name: String,
    pub kind: String,
    pub text: Option<String>,
    pub range: Range,
}
```

### Behavior

1. Validate query string.
2. Determine target language:
   - if `language` is provided, use that parser only.
   - if omitted, infer from file extension per file.
3. Build file list using `glob` and ignore rules.
4. Enforce `maxFiles`.
5. Skip files larger than `maxBytesPerFile`.
6. Parse files.
7. Run query.
8. Normalize captures.
9. Stop when `maxResults` is reached.
10. Return scan stats.

### Safety Requirements

```text
Must not scan unbounded workspace.
Must not follow symlinks outside workspace.
Must not include ignored directories.
Must enforce maxFiles and maxResults.
Must enforce maxBytesPerFile.
```

### Example Input

```json
{
  "language": "typescript",
  "glob": "src/**/*.ts",
  "query": "(call_expression function: (identifier) @callee)",
  "maxFiles": 100,
  "maxResults": 500
}
```

---

## 23. Tool: ast_file_metrics

### Purpose

Return basic structural metrics for a file.

Useful for:

```text
agent planning
large file detection
complexity estimation
chunking decisions
review prioritization
```

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileMetricsInput {
    pub file_path: String,
    pub include_function_metrics: Option<bool>,
};
```

Defaults:

```json
{
  "includeFunctionMetrics": false
}
```

### Output Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileMetricsResult {
    pub metrics: FileMetrics,
    pub functions: Option<Vec<FunctionMetric>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FunctionMetric {
    pub name: Option<String>,
    pub kind: String,
    pub range: Range,
    pub line_count: usize,
    pub branch_count: usize,
    pub loop_count: usize,
    pub nesting_depth: usize,
}
```

### Behavior

1. Validate path.
2. Parse file.
3. Count:
   - lines
   - bytes
   - total AST nodes
   - syntax errors
   - imports
   - exports
   - functions
   - classes
   - max nesting depth
   - max function lines
4. If `includeFunctionMetrics`, compute per-function metrics.

### Notes

V2 metrics are approximate and structural.

They are not a replacement for language-specific static analyzers.

---

## 24. Runtime Limits

Recommended V2 defaults:

```text
maxTextBytes: 20,000
maxContextBytes: 30,000
maxResultsPerFile: 200
maxWorkspaceQueryFiles: 200
maxWorkspaceQueryResults: 1,000
maxBytesPerFileForWorkspaceQuery: 1,000,000
parseTimeoutMs: 5,000
workspaceQueryTimeoutMs: 20,000
```

All limits should be configurable through environment variables or runtime config in later versions.

Suggested environment variables:

```text
AST_MAX_TEXT_BYTES=20000
AST_MAX_CONTEXT_BYTES=30000
AST_MAX_RESULTS_PER_FILE=200
AST_MAX_WORKSPACE_QUERY_FILES=200
AST_MAX_WORKSPACE_QUERY_RESULTS=1000
AST_MAX_BYTES_PER_FILE=1000000
AST_PARSE_TIMEOUT_MS=5000
AST_WORKSPACE_QUERY_TIMEOUT_MS=20000
```

---

## 25. Error Codes Added in V2

```text
invalid_range
range_out_of_bounds
node_not_found
scope_not_found
ast_feature_unsupported_for_language
query_workspace_limit_exceeded
workspace_query_timeout
file_too_large
invalid_glob
invalid_query
position_encoding_error
context_budget_exceeded
```

Existing V1 error codes remain valid:

```text
workspace_not_found
path_outside_workspace
file_not_found
unsupported_language
parse_failed
syntax_error
internal_error
```

---

## 26. Tool Response Format

All MCP tools must return a single text content block containing pretty JSON.

Success:

```json
{
  "content": [
    {
      "type": "text",
      "text": "{ ...pretty JSON... }"
    }
  ]
}
```

Error:

```json
{
  "content": [
    {
      "type": "text",
      "text": "{ \"error\": { ... } }"
    }
  ]
}
```

Tool outputs should not include prose.

They should be machine-readable.

---

## 27. Safety Rules

V2 must enforce all V1 safety rules plus additional workspace-query limits.

```text
1. Never allow paths outside workspace.
2. Never write files.
3. Never call LSP MCP.
4. Never depend on LSP services.
5. Never execute source code.
6. Never evaluate expressions.
7. Never scan workspace without strict maxFiles and maxResults.
8. Never follow symlinks outside workspace.
9. Never return unbounded source text.
10. Always enforce max text/context budgets.
11. Always enforce parse/query timeouts.
12. Return structured errors for unsupported language features.
```

---

## 28. Internal Architecture Additions

V2 adds these modules:

```text
src/
  context/
    enclosing_scope.rs
    node_at_range.rs
    node_text.rs
    context_for_range.rs
    context_pack.rs

  extraction/
    calls.rs
    member_access.rs
    literals.rs
    template_literals.rs

  workspace/
    query_workspace.rs
    file_scanner.rs
    ignore_rules.rs

  metrics/
    file_metrics.rs
    nesting.rs
    function_metrics.rs

  text/
    position_encoding.rs
    range_to_bytes.rs
    text_budget.rs
```

Updated conceptual structure:

```text
ast-mcp/
  Cargo.toml
  src/
    main.rs

    mcp/
      server.rs
      register_tools.rs
      schemas.rs
      tool_errors.rs

    config/
      language_registry.rs
      runtime_config.rs
      defaults.rs

    parser/
      parser_manager.rs
      parse_file.rs
      tree_cache.rs
      language.rs

    safety/
      paths.rs
      limits.rs
      workspace.rs

    normalize/
      node.rs
      range.rs
      point.rs
      names.rs

    outline/
      file_outline.rs
      top_level_nodes.rs

    node/
      node_tree.rs
      enclosing_node.rs

    context/
      enclosing_scope.rs
      node_at_range.rs
      node_text.rs
      context_for_range.rs
      context_pack.rs

    extraction/
      imports.rs
      exports.rs
      functions.rs
      classes.rs
      calls.rs
      member_access.rs
      literals.rs
      template_literals.rs

    query/
      ast_query.rs
      query_workspace.rs
      query_compiler.rs

    workspace/
      file_scanner.rs
      ignore_rules.rs

    metrics/
      file_metrics.rs
      nesting.rs
      function_metrics.rs

    text/
      position_encoding.rs
      range_to_bytes.rs
      text_budget.rs

    utils/
      ids.rs
      time.rs
      json.rs
```

No LSP modules should be added.

Do not add:

```text
src/lsp/
src/language_server/
src/semantic/
```

---

## 29. Query Pack Requirements

V2 should add query packs for each supported language where possible.

### TypeScript / JavaScript

Required query support:

```text
calls
member access
string/number/boolean/null literals
template literals
tagged template literals
scope nodes
```

### Python

Required query support:

```text
calls
attribute access
string/number/boolean/none literals
f-strings where possible
scope nodes
```

### Go, optional

Recommended query support:

```text
calls
selector expressions
basic literals
function/method scopes
```

### Rust, optional

Recommended query support:

```text
call expressions
field expressions
string/number/bool literals
function/impl scopes
macros best-effort
```

---

## 30. Workspace Query Rules

`ast_query_workspace` must use bounded scanning.

File discovery should:

```text
respect workspace root
respect .gitignore where feasible
respect default ignored directories
match glob safely
filter by supported extensions
skip files above maxBytesPerFile
stop at maxFiles
```

Query execution should:

```text
compile query once per language
reuse parser where safe
stop at maxResults
record filesScanned and filesSkipped
return partial results when truncated
```

Parallelism should be bounded.

Recommended default worker count:

```text
min(num_cpus, 8)
```

---

## 31. Performance Requirements

V2 should target:

```text
single-file parse under 50ms for typical files
single-file extraction under 100ms for typical files
workspace query over 200 medium files under 20s
bounded memory use under large files
```

These are targets, not strict guarantees.

For large files:

```text
skip or truncate according to config
return file_too_large or skipped stats
```

---

## 32. Caching Guidance

V2 may reuse the V1 parse-tree cache if implemented.

Cache key should include:

```text
absolute file path
file mtime
file size
language
```

If the file changes, invalidate parse tree.

Workspace queries may use cached parse results but must respect memory limits.

V2 does not require persistent disk cache.

---

## 33. Development Milestones

### Milestone 1: Position and Range Robustness

Implement:

```text
UTF-16 position to byte offset conversion
byte offset to UTF-16 position conversion
range validation
range slicing
```

Add Unicode tests.

### Milestone 2: Node Lookup Tools

Implement:

```text
ast_node_at_range
ast_node_text
```

### Milestone 3: Scope Detection

Implement:

```text
ast_enclosing_scope
scope query packs
scope normalization
```

### Milestone 4: Context Tools

Implement:

```text
ast_context_for_range
ast_context_pack
context byte budgeting
```

### Milestone 5: Call and Member Extraction

Implement:

```text
ast_find_calls
ast_find_member_access
```

Start with TypeScript/JavaScript and Python.

### Milestone 6: Literal Extraction

Implement:

```text
ast_find_literals
ast_find_template_literals
```

### Milestone 7: Workspace Query

Implement:

```text
file scanner
ignore rules
glob matching
bounded parallel query execution
ast_query_workspace
```

### Milestone 8: Metrics

Implement:

```text
ast_file_metrics
function metrics optional
nesting depth calculation
```

### Milestone 9: Integration and Safety Tests

Run all V1 and V2 acceptance tests.

---

## 34. Acceptance Criteria

V2 is acceptable when all V1 acceptance criteria still pass and the following are true.

### ast_enclosing_scope

- Returns outer-to-inner scope chain.
- Works for TypeScript/JavaScript functions/classes/methods.
- Works for Python modules/classes/functions.
- Does not return arbitrary non-scope nodes unless `includeBlockScopes` allows block scopes.

### ast_node_at_range

- Finds exact node where possible.
- Finds smallest containing node.
- Rejects invalid ranges.
- Handles Unicode positions correctly.

### ast_node_text

- Returns exact source text for a range.
- Enforces max byte limit.
- Does not return entire huge files by accident.

### ast_context_for_range

- Returns target node context.
- Includes parent context when requested.
- Enforces global context budget.

### ast_context_pack

- Returns imports, exports, enclosing scope, enclosing node, and outline according to requested parts.
- Enforces max byte budget.
- Does not call LSP tools.

### ast_find_calls

- Finds call expressions in TypeScript/JavaScript.
- Finds call expressions in Python.
- Can filter by exact callee text or substring.
- Returns arguments text when requested.

### ast_find_member_access

- Finds member/property access expressions where supported.
- Can filter by property or full-text substring.

### ast_find_literals

- Finds string and numeric literals.
- Supports `contains` and `exact` filters.
- Does not evaluate expressions.

### ast_find_template_literals

- Finds TypeScript/JavaScript template literals.
- Finds tagged templates.
- Counts interpolations.
- Returns empty results or feature-unsupported error for unsupported languages.

### ast_query_workspace

- Runs bounded Tree-sitter queries across workspace files.
- Enforces `maxFiles`, `maxResults`, and `maxBytesPerFile`.
- Does not scan ignored directories.
- Does not follow symlinks outside workspace.
- Returns partial/truncated results when limits are reached.

### ast_file_metrics

- Returns structural metrics.
- Reports syntax error count.
- Reports node count, import count, function count, class count, and nesting depth.

### Decoupling

- AST MCP has no dependency on LSP MCP.
- AST MCP has no language-server dependency.
- AST MCP does not call `lsp_*` tools.

---

## 35. Testing Requirements

### Unit Tests

Required unit tests:

```text
workspace path validation
UTF-16 position conversion
range to byte offset conversion
invalid range rejection
node at range selection modes
node text truncation
scope detection
context budget enforcement
call extraction
member access extraction
literal extraction
template literal extraction
query workspace limit enforcement
file metrics counting
```

### Integration Tests

Required integration tests:

```text
TypeScript ast_enclosing_scope
TypeScript ast_find_calls
TypeScript ast_find_member_access
TypeScript ast_find_literals
TypeScript ast_find_template_literals
TypeScript ast_context_pack
TypeScript ast_query_workspace
Python ast_enclosing_scope
Python ast_find_calls
Python ast_find_member_access
Python ast_find_literals
Python ast_context_pack
```

### Safety Tests

Required safety tests:

```text
reject path outside workspace
reject invalid glob
reject invalid Tree-sitter query
skip files larger than maxBytesPerFile
stop at maxFiles
stop at maxResults
do not follow symlink outside workspace
do not return source beyond maxBytes
do not call LSP tools
```

### Unicode Tests

Required Unicode tests:

```text
position after emoji
range containing emoji
node text extraction with non-ASCII characters
line/character to byte offset correctness
byte offset to line/character correctness
```

---

## 36. Example Tool Calls

### Enclosing Scope

```json
{
  "tool": "ast_enclosing_scope",
  "arguments": {
    "filePath": "src/user.ts",
    "position": {
      "line": 20,
      "character": 12
    },
    "includeBlockScopes": false
  }
}
```

### Node at Range

```json
{
  "tool": "ast_node_at_range",
  "arguments": {
    "filePath": "src/user.ts",
    "range": {
      "start": { "line": 20, "character": 4 },
      "end": { "line": 20, "character": 20 }
    },
    "mode": "smallest_containing",
    "includeText": true
  }
}
```

### Node Text

```json
{
  "tool": "ast_node_text",
  "arguments": {
    "filePath": "src/user.ts",
    "range": {
      "start": { "line": 14, "character": 2 },
      "end": { "line": 28, "character": 3 }
    },
    "maxBytes": 12000
  }
}
```

### Context for Range

```json
{
  "tool": "ast_context_for_range",
  "arguments": {
    "filePath": "src/user.ts",
    "range": {
      "start": { "line": 20, "character": 4 },
      "end": { "line": 20, "character": 20 }
    },
    "includeParents": true,
    "includeSiblings": false,
    "maxParentDepth": 4,
    "maxContextBytes": 20000
  }
}
```

### Context Pack

```json
{
  "tool": "ast_context_pack",
  "arguments": {
    "filePath": "src/user.ts",
    "position": {
      "line": 20,
      "character": 12
    },
    "include": [
      "imports",
      "exports",
      "enclosing_scope",
      "enclosing_node",
      "top_level_outline"
    ],
    "maxBytes": 30000
  }
}
```

### Find Calls

```json
{
  "tool": "ast_find_calls",
  "arguments": {
    "filePath": "src/routes.ts",
    "callee": "router.get",
    "includeArguments": true,
    "includeEnclosingScope": true,
    "maxResults": 100
  }
}
```

### Find Member Access

```json
{
  "tool": "ast_find_member_access",
  "arguments": {
    "filePath": "src/user.ts",
    "property": "email",
    "includeEnclosingScope": true,
    "maxResults": 100
  }
}
```

### Find Literals

```json
{
  "tool": "ast_find_literals",
  "arguments": {
    "filePath": "src/routes.ts",
    "literalKind": "string",
    "contains": "/api",
    "includeEnclosingScope": true,
    "maxResults": 100
  }
}
```

### Find Template Literals

```json
{
  "tool": "ast_find_template_literals",
  "arguments": {
    "filePath": "src/db.ts",
    "tag": "sql",
    "includeUntagged": false,
    "includeEnclosingScope": true,
    "maxResults": 50
  }
}
```

### Query Workspace

```json
{
  "tool": "ast_query_workspace",
  "arguments": {
    "language": "typescript",
    "glob": "src/**/*.ts",
    "query": "(call_expression function: (identifier) @callee)",
    "maxFiles": 100,
    "maxResults": 500,
    "includeText": true
  }
}
```

### File Metrics

```json
{
  "tool": "ast_file_metrics",
  "arguments": {
    "filePath": "src/user.ts",
    "includeFunctionMetrics": true
  }
}
```

---

## 37. Agent Skill Guidance for AST V2

Agent Skills should use AST V2 tools for structural context and pattern search.

### For focused edit context

Use:

```text
ast_context_pack
ast_context_for_range
ast_enclosing_scope
ast_node_text
```

### For diagnostic-local context

Use:

```text
ast_node_at_range
ast_context_for_range
ast_enclosing_scope
```

Then optionally use LSP MCP outside AST MCP for semantic diagnostics or type information.

### For syntax pattern search

Use:

```text
ast_find_calls
ast_find_member_access
ast_find_literals
ast_find_template_literals
ast_query_workspace
```

### For file planning

Use:

```text
ast_file_metrics
ast_file_outline
ast_chunk_file
```

---

## 38. Final V2 Design Principle

Version 2 makes the AST MCP a practical structural context engine.

```text
V1 answers: What is in this file?
V2 answers: What exact syntax context or pattern should the agent inspect?
```

V2 may:

```text
parse files
inspect scopes
return node text
build context packs
find calls/member access/literals/templates
run bounded workspace queries
compute structural metrics
```

V2 must not:

```text
write files
rewrite code
call LSP MCP
evaluate code
infer semantic identity
scan workspace without strict limits
```

The clean boundary remains:

```text
Need syntax or structure? Use AST MCP.
Need semantic meaning? Use LSP MCP.
Need both? Use Agent Skills or Code Composite MCP.
```
