# AST MCP Server — Version 4 Specification

## 1. Purpose

Version 4 adds a **preview-only structural rewrite engine** to the Rust-based AST MCP server.

V1 established core parsing and structural extraction.

V2 added focused context, node inspection, workspace queries, expression/literal extraction, and file metrics.

V3 added framework-aware extraction for routes, components, hooks, tests, decorators, schemas, and syntax-level dependency edges.

V4 adds safe mechanical edit previews based on AST structure.

The AST MCP server must still remain independent from LSP MCP.

V4 must not write files.

V4 must not apply patches.

V4 must only produce:

```text
edit previews
unified diffs
rewrite validation reports
parse-after-rewrite results
safety violations
```

---

## 2. Architectural Boundary

The AST MCP server owns syntax-level structure and syntax-aware rewrite previews.

It must not call LSP MCP.

It must not rely on semantic type information, symbol references, or diagnostics from language servers.

### AST MCP owns

```text
syntax tree parsing
node/range inspection
imports/exports extraction
functions/classes/types extraction
calls/literals/templates extraction
framework-aware structural extraction
AST query execution
syntax-aware chunking
structural rewrite previews
parse-after-rewrite validation
```

### AST MCP does not own

```text
semantic references
type information
hover information
LSP diagnostics
semantic rename
call hierarchy
type hierarchy
semantic implementation lookup
```

Those belong to LSP MCP.

### Composite workflows

Higher-level workflows that require both AST and LSP must live in:

```text
Agent Skills
Code Composite MCP
client-side orchestration layer
```

Examples:

```text
code_prepare_safe_rename
code_validate_edit_preview
code_prepare_signature_change
code_prepare_import_fix
code_analyze_change_impact
```

---

## 3. Version 4 Goals

V4 adds preview-only rewrite tools:

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

V4 also standardizes rewrite safety infrastructure:

```text
range validation
workspace path validation
edit overlap detection
max changed file limits
max edit limits
unified diff generation
syntax validation after in-memory rewrite
unsupported operation reporting
```

---

## 4. Version 4 Non-Goals

V4 does not include:

```text
direct file writes
patch application
semantic rename
semantic references
type checking
linting
formatting application
running tests
LSP diagnostics
arbitrary shell execution
custom code generation by LLM
multi-repo rewrite operations
```

V4 rewrite tools are structural and syntax-aware only.

---

## 5. Required Baseline From V1–V3

V4 assumes V1, V2, and V3 tools already exist.

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

V4 must preserve backward compatibility with V1–V3 schemas unless a safety correction is required.

---

## 6. Recommended Rust Stack

Core crates:

```toml
[dependencies]
anyhow = "1"
thiserror = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
schemars = "0.8"
tokio = { version = "1", features = ["full"] }
tower-lsp = "0.20"
clap = { version = "4", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = "0.3"
walkdir = "2"
ignore = "0.4"
globset = "0.4"
rayon = "1"
similar = "2"
camino = "1"
uuid = { version = "1", features = ["v4", "serde"] }

# Tree-sitter core
tree-sitter = "0.22"

# Language grammars, exact versions should be pinned by implementation
tree-sitter-typescript = "0.21"
tree-sitter-javascript = "0.21"
tree-sitter-python = "0.21"
tree-sitter-go = "0.21"
tree-sitter-rust = "0.21"
```

Notes:

```text
Use similar for unified diff generation.
Use camino for UTF-8 paths.
Use ignore/globset for bounded workspace scans.
Use schemars to generate JSON schema for shared contracts.
```

---

## 7. Supported Languages in V4

Required V4 support:

```text
TypeScript
TSX
JavaScript
JSX
Python
```

Recommended optional support:

```text
Go
Rust
```

Language registry:

```rust
pub struct LanguageSpec {
    pub language: String,
    pub extensions: Vec<String>,
    pub parser_name: String,
    pub supports_import_rewrite: bool,
    pub supports_decorators: bool,
    pub supports_function_signature_rewrite: bool,
}
```

Initial capability matrix:

| Language | Generic rewrite | Insert import | Local rename | Wrap node | Decorator | Signature preview |
|---|---:|---:|---:|---:|---:|---:|
| TypeScript | yes | yes | yes | yes | yes | yes |
| TSX | yes | yes | yes | yes | yes | partial |
| JavaScript | yes | yes | yes | yes | partial | yes |
| JSX | yes | yes | yes | yes | partial | partial |
| Python | yes | yes | yes | yes | yes | yes |
| Go | yes | partial | yes | partial | no | partial |
| Rust | yes | partial | yes | partial | attributes | partial |

Unsupported operations must return structured errors rather than silently failing.

---

## 8. Shared Types

V4 uses the same shared contracts from previous AST versions.

### Position

External API positions must use line and character.

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

---

## 9. V4 Rewrite Types

### TextEdit

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TextEdit {
    pub file_path: String,
    pub range: Range,
    pub new_text: String,
}
```

### RewriteOperation

```rust
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
        expected_node_kind: Option<String>,
        new_text: String,
    },
    InsertBeforeNode {
        file_path: String,
        range: Range,
        expected_node_kind: Option<String>,
        new_text: String,
    },
    InsertAfterNode {
        file_path: String,
        range: Range,
        expected_node_kind: Option<String>,
        new_text: String,
    },
    DeleteNode {
        file_path: String,
        range: Range,
        expected_node_kind: Option<String>,
    },
}
```

### RewritePreview

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RewritePreview {
    pub safe: bool,
    pub changed_files: Vec<String>,
    pub edit_count: u32,
    pub diff: Option<String>,
    pub edits: Vec<TextEdit>,
    pub parse_after_rewrite: Option<ParseAfterRewriteSummary>,
    pub violations: Vec<SafetyViolation>,
}
```

### ParseAfterRewriteSummary

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ParseAfterRewriteSummary {
    pub ok: bool,
    pub changed_files_checked: u32,
    pub files_with_syntax_errors: Vec<String>,
    pub syntax_errors: Vec<SyntaxErrorSummary>,
}
```

### SyntaxErrorSummary

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SyntaxErrorSummary {
    pub file_path: String,
    pub range: Range,
    pub node_kind: String,
    pub message: String,
}
```

### RewriteValidationResult

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RewriteValidationResult {
    pub safe: bool,
    pub changed_files: Vec<String>,
    pub edit_count: u32,
    pub violations: Vec<SafetyViolation>,
}
```

---

## 10. Workspace Safety Rules

All V4 tools must enforce workspace boundaries.

The server is launched with:

```bash
WORKSPACE_PATH=/absolute/path/to/repo
```

Allowed:

```text
src/user.ts
/absolute/path/to/repo/src/user.ts
```

Rejected:

```text
../outside.ts
/etc/passwd
/another-project/file.ts
```

Rules:

```text
1. Resolve all paths against workspace root.
2. Canonicalize where possible.
3. Reject paths outside workspace.
4. Reject directories where files are required.
5. Reject symlinks that escape workspace unless explicitly allowed.
6. Return workspace-relative paths in all responses.
```

---

## 11. Rewrite Safety Rules

Every rewrite-preview tool must enforce the same central validation.

### Required validation

```text
path is inside workspace
file exists
file language is supported
range is valid
range maps to valid byte offsets
operation is supported for language
changed file count <= maxChangedFiles
edit count <= maxEdits
edits do not overlap in the same file
parse-after-rewrite succeeds if parseCheck is true
```

### Default limits

```text
maxChangedFiles: 20
maxEdits: 200
maxDiffBytes: 500000
maxNewTextBytesPerEdit: 100000
maxParseAfterRewriteFiles: 20
```

### Safety violation types

```text
outside_workspace
file_not_found
unsupported_language
unsupported_operation
invalid_range
range_not_node_aligned
node_kind_mismatch
too_many_files
too_many_edits
new_text_too_large
diff_too_large
overlapping_edits
syntax_error_after_rewrite
unsupported_syntax_shape
ambiguous_rewrite_target
internal_error
```

---

## 12. Position and Offset Handling

Tree-sitter uses byte offsets internally.

The external API uses:

```text
line + character
```

The implementation must define its character offset semantics.

Recommended external contract:

```text
line: zero-based line number
character: zero-based UTF-16 code unit offset
```

Reason:

```text
This aligns with LSP and makes AST MCP easier to combine with LSP MCP in Agent Skills or Code Composite MCP.
```

Internal conversion:

```text
Position → byte offset
byte offset → Position
```

The implementation must handle:

```text
UTF-8 source text
UTF-16 character positions
multi-byte Unicode
CRLF and LF line endings
files with final newline
files without final newline
```

If V4 implementation initially supports only ASCII-safe ranges, that limitation must be explicit and tested.

Preferred: implement robust UTF-16-to-byte conversion from the start.

---

## 13. Diff Generation

V4 must generate unified diffs for preview tools when `includeDiff` is true.

Use Rust crate:

```text
similar
```

Diff output should include workspace-relative file paths.

Example:

```diff
--- src/user.ts
+++ src/user.ts
@@ -1,4 +1,5 @@
 import { User } from "./types";
+import { UserInput } from "./inputs";
```

If diff exceeds `maxDiffBytes`, return:

```json
{
  "safe": false,
  "violations": [
    {
      "violation_type": "diff_too_large",
      "message": "Generated diff exceeded maxDiffBytes."
    }
  ]
}
```

---

## 14. Applying Edits In Memory

V4 must apply text edits in memory only.

It must never write changed text to disk.

Edits for the same file must be applied from latest byte offset to earliest byte offset.

Pseudo-process:

```text
1. Convert Range to byte offsets.
2. Sort edits descending by start byte offset.
3. Reject overlapping edits.
4. Apply edits to in-memory string.
5. Parse modified string with Tree-sitter.
6. Generate diff between original and modified string.
7. Return preview.
```

---

## 15. Tool: ast_rewrite_preview

### Purpose

Generic structural rewrite preview tool.

This is the lowest-level V4 rewrite tool.

It accepts one or more rewrite operations and returns a safe preview.

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AstRewritePreviewInput {
    pub operations: Vec<RewriteOperation>,
    pub include_diff: Option<bool>,
    pub parse_check: Option<bool>,
    pub max_changed_files: Option<u32>,
    pub max_edits: Option<u32>,
}
```

Defaults:

```json
{
  "include_diff": true,
  "parse_check": true,
  "max_changed_files": 20,
  "max_edits": 200
}
```

### Output Schema

```rust
pub type AstRewritePreviewResult = RewritePreview;
```

### Behavior

1. Validate every operation.
2. Validate all paths are inside workspace.
3. Validate files exist and are supported.
4. Convert ranges to byte offsets.
5. For node operations:
   - find node at range
   - verify node alignment if required
   - verify expected node kind if provided
6. Build text edits.
7. Reject overlapping edits.
8. Apply edits in memory.
9. Re-parse changed files if `parse_check` is true.
10. Generate diff if `include_diff` is true.
11. Return preview.

### Safety

This tool must not write files.

---

## 16. Tool: ast_insert_import_preview

### Purpose

Preview adding or merging an import statement structurally.

This tool is syntax-level and does not know whether the imported symbol exists.

Semantic validation belongs to LSP MCP or composite workflows.

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AstInsertImportPreviewInput {
    pub file_path: String,
    pub import: ImportRequest,
    pub include_diff: Option<bool>,
    pub parse_check: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ImportRequest {
    pub source: String,
    pub default_import: Option<String>,
    pub named_imports: Vec<String>,
    pub namespace_import: Option<String>,
    pub is_type_only: Option<bool>,
}
```

Defaults:

```json
{
  "include_diff": true,
  "parse_check": true
}
```

### Output Schema

```rust
pub type AstInsertImportPreviewResult = RewritePreview;
```

### Supported Languages

V4 required:

```text
TypeScript
TSX
JavaScript
JSX
Python
```

Optional:

```text
Go
Rust
```

### Behavior for TypeScript/JavaScript

1. Parse existing import declarations.
2. If import source already exists:
   - merge named imports
   - preserve default import if compatible
   - preserve namespace import if compatible
   - avoid duplicates
3. If source does not exist:
   - insert into import block
   - preserve rough grouping
4. Generate edit preview.
5. Parse after rewrite.

Example:

Before:

```ts
import { User } from "./types";
```

Request:

```json
{
  "source": "./types",
  "named_imports": ["UserInput"],
  "is_type_only": true
}
```

After:

```ts
import type { User, UserInput } from "./types";
```

If type-only merging is ambiguous, return a safety violation instead of making an unsafe edit.

### Behavior for Python

Support:

```python
import module
from module import name
```

Example request:

```json
{
  "source": "app.types",
  "named_imports": ["UserInput"]
}
```

Preview:

```python
from app.types import UserInput
```

### Important Limitation

This tool is structural only.

It must not guarantee the import source or symbol exists.

---

## 17. Tool: ast_remove_unused_import_preview

### Purpose

Preview removal of imports that are syntactically unused in a single file.

This is a syntax-level approximation.

It is not a semantic unused-import checker.

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AstRemoveUnusedImportPreviewInput {
    pub file_path: String,
    pub import_names: Option<Vec<String>>,
    pub include_diff: Option<bool>,
    pub parse_check: Option<bool>,
}
```

Defaults:

```json
{
  "include_diff": true,
  "parse_check": true
}
```

### Output Schema

```rust
pub type AstRemoveUnusedImportPreviewResult = RewritePreview;
```

### Behavior

1. Parse import declarations.
2. Build syntax-level identifier usage map.
3. If `import_names` provided, only consider those names.
4. Remove unused specifiers.
5. Remove entire import statement if empty.
6. Generate preview.
7. Parse after rewrite.

### Safety Warning

This tool may be unsafe with:

```text
dynamic usage
global side-effect imports
type-only imports in complex TS configs
macro-like patterns
string-based references
```

If an import is side-effect only, never remove it.

Side-effect import example:

```ts
import "reflect-metadata";
```

Must remain untouched.

---

## 18. Tool: ast_rename_local_preview

### Purpose

Preview local structural rename inside a known syntax scope.

This is not a semantic rename.

For cross-file or semantic rename, use LSP MCP.

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AstRenameLocalPreviewInput {
    pub file_path: String,
    pub position: Position,
    pub new_name: String,
    pub scope_range: Option<Range>,
    pub include_diff: Option<bool>,
    pub parse_check: Option<bool>,
}
```

Defaults:

```json
{
  "include_diff": true,
  "parse_check": true
}
```

### Output Schema

```rust
pub type AstRenameLocalPreviewResult = RewritePreview;
```

### Behavior

1. Find identifier node at `position`.
2. Determine local scope:
   - use `scope_range` if provided
   - otherwise use enclosing function/block/class scope where supported
3. Find identifier occurrences with same text inside that scope.
4. Exclude property keys/member names where unsafe.
5. Exclude string literals and comments.
6. Generate edits.
7. Parse after rewrite.

### Safety

Return unsafe if:

```text
scope cannot be determined
identifier appears in ambiguous destructuring pattern
identifier appears in dynamic property access
identifier is imported/exported
identifier is top-level public symbol
```

This tool is best for:

```text
local variables
function parameters
local helper names inside a function
```

It must not replace LSP rename.

---

## 19. Tool: ast_wrap_node_preview

### Purpose

Preview wrapping a syntax node with provided prefix/suffix or a known wrapper template.

Use cases:

```text
wrap expression with await
wrap expression with logger()
wrap statement block with try/catch
wrap JSX element with provider
wrap handler body with middleware call
```

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AstWrapNodePreviewInput {
    pub file_path: String,
    pub range: Range,
    pub expected_node_kind: Option<String>,
    pub wrapper: WrapRequest,
    pub include_diff: Option<bool>,
    pub parse_check: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WrapRequest {
    PrefixSuffix {
        prefix: String,
        suffix: String,
    },
    TryCatch {
        catch_binding: Option<String>,
        catch_body: String,
    },
    CallExpression {
        callee: String,
    },
}
```

### Output Schema

```rust
pub type AstWrapNodePreviewResult = RewritePreview;
```

### Behavior

1. Validate selected node.
2. Verify expected node kind if provided.
3. Generate wrapper text.
4. Preserve original node text.
5. Generate preview.
6. Parse after rewrite.

### Example: call expression wrapper

Before:

```ts
getUser(id)
```

Request:

```json
{
  "kind": "call_expression",
  "callee": "trace"
}
```

After:

```ts
trace(getUser(id))
```

### Example: try/catch wrapper

Before:

```ts
const user = await getUser(id);
return user;
```

After:

```ts
try {
  const user = await getUser(id);
  return user;
} catch (error) {
  throw error;
}
```

Formatting does not need to be perfect in AST MCP.

Formatting can be handled by a formatter tool or LSP format preview later.

---

## 20. Tool: ast_add_decorator_preview

### Purpose

Preview adding a decorator or attribute to a class, method, function, or field.

Supported examples:

```text
TypeScript/NestJS decorators
Angular decorators
Python decorators
Rust attributes, optional
Java annotations, optional future
```

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AstAddDecoratorPreviewInput {
    pub file_path: String,
    pub target_range: Range,
    pub decorator_text: String,
    pub expected_target_kind: Option<String>,
    pub include_diff: Option<bool>,
    pub parse_check: Option<bool>,
}
```

### Output Schema

```rust
pub type AstAddDecoratorPreviewResult = RewritePreview;
```

### Behavior

1. Validate target node.
2. Verify target supports decorators/attributes.
3. Check if same decorator already exists where possible.
4. Insert decorator before target node.
5. Preserve indentation.
6. Generate preview.
7. Parse after rewrite.

### Safety

Return unsafe if:

```text
target does not support decorators
decorator text is syntactically invalid after insertion
language does not support decorators/attributes
same decorator already exists and duplicate behavior is unclear
```

---

## 21. Tool: ast_modify_function_signature_preview

### Purpose

Preview structural modification to a function or method signature.

This is syntax-level only.

It does not update call sites.

Call-site planning belongs to LSP MCP or Code Composite MCP.

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AstModifyFunctionSignaturePreviewInput {
    pub file_path: String,
    pub function_range: Range,
    pub operation: FunctionSignatureOperation,
    pub include_diff: Option<bool>,
    pub parse_check: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FunctionSignatureOperation {
    ReplaceSignature {
        new_signature_text: String,
    },
    AddParameter {
        parameter_text: String,
        position: Option<u32>,
    },
    RemoveParameter {
        parameter_name: String,
    },
    RenameParameter {
        old_name: String,
        new_name: String,
        rename_body_occurrences: bool,
    },
}
```

### Output Schema

```rust
pub type AstModifyFunctionSignaturePreviewResult = RewritePreview;
```

### Behavior

1. Validate function/method node.
2. Extract signature range.
3. Apply operation structurally.
4. If `RenameParameter.rename_body_occurrences` is true:
   - only rename occurrences inside function body
   - use local structural rename rules
5. Generate preview.
6. Parse after rewrite.

### Safety

Return unsafe if:

```text
function node cannot be identified
parameter cannot be found
operation would create duplicate parameters
rename target collides with local binding
signature text fails parse-after-rewrite
```

This tool must not update external call sites.

---

## 22. Tool: ast_validate_rewrite

### Purpose

Validate rewrite operations without generating a diff.

Useful for cheap safety checks before expensive preview generation.

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AstValidateRewriteInput {
    pub operations: Vec<RewriteOperation>,
    pub max_changed_files: Option<u32>,
    pub max_edits: Option<u32>,
}
```

### Output Schema

```rust
pub type AstValidateRewriteResult = RewriteValidationResult;
```

### Behavior

1. Validate all paths.
2. Validate ranges.
3. Validate node alignment where applicable.
4. Validate expected node kinds.
5. Check limits.
6. Check overlap.
7. Return validation result.

This tool does not read all file text unless required for range validation.

It does not generate diffs.

It does not parse modified output.

---

## 23. Tool: ast_parse_after_rewrite

### Purpose

Apply a set of edits in memory and verify that changed files still parse.

This tool is useful for validating diffs generated outside AST MCP, as long as edits are provided as structured ranges.

### Input Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AstParseAfterRewriteInput {
    pub edits: Vec<TextEdit>,
    pub max_changed_files: Option<u32>,
    pub max_edits: Option<u32>,
}
```

### Output Schema

```rust
pub type AstParseAfterRewriteResult = ParseAfterRewriteSummary;
```

### Behavior

1. Validate paths and ranges.
2. Reject overlapping edits.
3. Apply edits in memory.
4. Parse modified files.
5. Return syntax status and error summaries.

### Output Example

```json
{
  "ok": false,
  "changed_files_checked": 1,
  "files_with_syntax_errors": ["src/user.ts"],
  "syntax_errors": [
    {
      "file_path": "src/user.ts",
      "range": {
        "start": { "line": 12, "character": 0 },
        "end": { "line": 12, "character": 10 }
      },
      "node_kind": "ERROR",
      "message": "Tree-sitter reported syntax error after rewrite."
    }
  ]
}
```

---

## 24. Tool Surface Summary

```rust
pub trait AstMcpV4Tools {
    fn ast_rewrite_preview(input: AstRewritePreviewInput) -> AstRewritePreviewResult;
    fn ast_insert_import_preview(input: AstInsertImportPreviewInput) -> AstInsertImportPreviewResult;
    fn ast_remove_unused_import_preview(input: AstRemoveUnusedImportPreviewInput) -> AstRemoveUnusedImportPreviewResult;
    fn ast_rename_local_preview(input: AstRenameLocalPreviewInput) -> AstRenameLocalPreviewResult;
    fn ast_wrap_node_preview(input: AstWrapNodePreviewInput) -> AstWrapNodePreviewResult;
    fn ast_add_decorator_preview(input: AstAddDecoratorPreviewInput) -> AstAddDecoratorPreviewResult;
    fn ast_modify_function_signature_preview(input: AstModifyFunctionSignaturePreviewInput) -> AstModifyFunctionSignaturePreviewResult;
    fn ast_validate_rewrite(input: AstValidateRewriteInput) -> AstValidateRewriteResult;
    fn ast_parse_after_rewrite(input: AstParseAfterRewriteInput) -> AstParseAfterRewriteResult;
}
```

---

## 25. Internal Architecture Additions

V4 adds modules:

```text
src/
  rewrite/
    mod.rs
    operations.rs
    preview.rs
    validate.rs
    apply_edits.rs
    parse_after.rs
    diff.rs
    overlap.rs

  rewrite_tools/
    insert_import.rs
    remove_unused_import.rs
    rename_local.rs
    wrap_node.rs
    add_decorator.rs
    modify_signature.rs

  safety/
    paths.rs
    ranges.rs
    limits.rs
    violations.rs

  text/
    position.rs
    utf16.rs
    line_index.rs
    indentation.rs
```

Existing modules from V1–V3 remain:

```text
src/
  parser/
  registry/
  queries/
  extraction/
  framework/
  workspace/
  mcp/
```

---

## 26. Rewrite Engine Design

The rewrite engine should expose internal functions:

```rust
pub fn validate_rewrite_operations(
    workspace: &Workspace,
    operations: &[RewriteOperation],
    limits: RewriteLimits,
) -> RewriteValidationResult;

pub fn build_text_edits(
    workspace: &Workspace,
    operations: &[RewriteOperation],
) -> Result<Vec<TextEdit>, RewriteError>;

pub fn preview_edits(
    workspace: &Workspace,
    edits: &[TextEdit],
    options: PreviewOptions,
) -> Result<RewritePreview, RewriteError>;

pub fn parse_after_edits(
    workspace: &Workspace,
    edits: &[TextEdit],
) -> Result<ParseAfterRewriteSummary, RewriteError>;
```

High-level tools should reuse these functions rather than implementing their own edit logic.

---

## 27. Import Rewrite Requirements

### TypeScript/JavaScript import support

Support:

```ts
import x from "mod";
import { a, b } from "mod";
import * as ns from "mod";
import type { T } from "mod";
import "side-effect";
const x = require("mod"); // extraction yes, rewrite optional
```

V4 insert import should support ES import declarations.

CommonJS rewrite may be deferred.

### Python import support

Support:

```python
import module
import module as alias
from module import name
from module import name as alias
```

### Side-effect imports

Do not remove or merge side-effect imports unless explicitly supported.

Examples:

```ts
import "reflect-metadata";
```

```python
import sitecustomize
```

---

## 28. Local Rename Requirements

Local rename must be conservative.

Allowed targets:

```text
local variable
function parameter
catch binding
for-loop binding
local helper function inside enclosing scope
```

Unsafe targets:

```text
imported binding
exported binding
top-level function/class/type
object property name
class method name
public field name
string literal
comment
JSX tag name, unless explicitly supported
```

If unsure, return unsafe.

This is intentional because semantic rename belongs to LSP MCP.

---

## 29. Parse-After-Rewrite Requirements

Every preview tool defaults to `parse_check: true`.

A rewrite preview is safe only if:

```text
all validation checks pass
all changed files parse after rewrite
no ERROR nodes are found in changed syntax regions
```

If a file already contains syntax errors before the rewrite, behavior should be explicit.

Recommended behavior:

```text
If file had pre-existing syntax errors:
  parse_after_rewrite may still run,
  but result should include preExistingSyntaxErrors: true where implemented.
```

For V4, simpler acceptable behavior:

```text
If original file has syntax errors, mark rewrite as unsafe unless allowPreExistingSyntaxErrors is added in future.
```

---

## 30. Error Codes Added in V4

```text
rewrite_invalid_range
rewrite_node_kind_mismatch
rewrite_range_not_node_aligned
rewrite_unsupported_operation
rewrite_unsupported_language
rewrite_too_many_files
rewrite_too_many_edits
rewrite_overlapping_edits
rewrite_new_text_too_large
rewrite_diff_too_large
rewrite_syntax_error_after
rewrite_ambiguous_target
rewrite_import_conflict
rewrite_parameter_not_found
rewrite_duplicate_parameter
rewrite_scope_unavailable
rewrite_identifier_not_found
rewrite_unsafe_local_rename
```

Generic AST errors remain valid:

```text
workspace_not_found
path_outside_workspace
file_not_found
unsupported_language
parse_failed
internal_error
```

---

## 31. Example Tool Calls

### Generic rewrite preview

```json
{
  "tool": "ast_rewrite_preview",
  "arguments": {
    "operations": [
      {
        "kind": "replace_node",
        "file_path": "src/user.ts",
        "range": {
          "start": { "line": 10, "character": 0 },
          "end": { "line": 12, "character": 1 }
        },
        "expected_node_kind": "function_declaration",
        "new_text": "function getUser() {\n  return null;\n}"
      }
    ],
    "include_diff": true,
    "parse_check": true
  }
}
```

### Insert import preview

```json
{
  "tool": "ast_insert_import_preview",
  "arguments": {
    "file_path": "src/user.ts",
    "import": {
      "source": "./types",
      "default_import": null,
      "named_imports": ["UserInput"],
      "namespace_import": null,
      "is_type_only": true
    },
    "include_diff": true,
    "parse_check": true
  }
}
```

### Local rename preview

```json
{
  "tool": "ast_rename_local_preview",
  "arguments": {
    "file_path": "src/user.ts",
    "position": { "line": 15, "character": 10 },
    "new_name": "accountId",
    "include_diff": true,
    "parse_check": true
  }
}
```

### Wrap node preview

```json
{
  "tool": "ast_wrap_node_preview",
  "arguments": {
    "file_path": "src/user.ts",
    "range": {
      "start": { "line": 22, "character": 9 },
      "end": { "line": 22, "character": 20 }
    },
    "expected_node_kind": "call_expression",
    "wrapper": {
      "kind": "call_expression",
      "callee": "trace"
    },
    "include_diff": true,
    "parse_check": true
  }
}
```

### Add decorator preview

```json
{
  "tool": "ast_add_decorator_preview",
  "arguments": {
    "file_path": "src/controller.ts",
    "target_range": {
      "start": { "line": 12, "character": 2 },
      "end": { "line": 18, "character": 3 }
    },
    "decorator_text": "@Get(\"/users/:id\")",
    "expected_target_kind": "method_definition",
    "include_diff": true,
    "parse_check": true
  }
}
```

### Modify function signature preview

```json
{
  "tool": "ast_modify_function_signature_preview",
  "arguments": {
    "file_path": "src/user.ts",
    "function_range": {
      "start": { "line": 10, "character": 0 },
      "end": { "line": 20, "character": 1 }
    },
    "operation": {
      "kind": "add_parameter",
      "parameter_text": "includeProfile?: boolean",
      "position": 1
    },
    "include_diff": true,
    "parse_check": true
  }
}
```

### Validate rewrite

```json
{
  "tool": "ast_validate_rewrite",
  "arguments": {
    "operations": [
      {
        "kind": "delete_node",
        "file_path": "src/user.ts",
        "range": {
          "start": { "line": 10, "character": 0 },
          "end": { "line": 12, "character": 1 }
        },
        "expected_node_kind": "function_declaration"
      }
    ]
  }
}
```

### Parse after rewrite

```json
{
  "tool": "ast_parse_after_rewrite",
  "arguments": {
    "edits": [
      {
        "file_path": "src/user.ts",
        "range": {
          "start": { "line": 10, "character": 0 },
          "end": { "line": 10, "character": 3 }
        },
        "new_text": "let"
      }
    ]
  }
}
```

---

## 32. Development Milestones

### Milestone 1: Rewrite Core Types

Implement:

```text
TextEdit
RewriteOperation
RewritePreview
RewriteValidationResult
ParseAfterRewriteSummary
SafetyViolation
```

### Milestone 2: Range and Offset Conversion

Implement:

```text
UTF-16 position to byte offset
byte offset to Position
line index
range validation
CRLF/LF handling
```

### Milestone 3: Rewrite Validator

Implement:

```text
workspace path validation
file existence validation
language support validation
node alignment validation
node kind validation
edit count limits
changed file limits
overlap detection
```

### Milestone 4: In-Memory Edit Application

Implement:

```text
apply edits by descending byte offset
reject overlapping edits
preserve original file on disk
produce modified text in memory
```

### Milestone 5: Diff Generation

Implement:

```text
unified diff generation
max diff size enforcement
workspace-relative file path labels
```

### Milestone 6: Parse After Rewrite

Implement:

```text
parse modified text
collect ERROR nodes
return syntax summaries
```

### Milestone 7: Generic Rewrite Preview

Implement:

```text
ast_validate_rewrite
ast_parse_after_rewrite
ast_rewrite_preview
```

### Milestone 8: Import Preview

Implement:

```text
ast_insert_import_preview
ast_remove_unused_import_preview
TypeScript/JavaScript import handling
Python import handling
```

### Milestone 9: Local Rename Preview

Implement:

```text
identifier target detection
scope inference
local usage detection
unsafe target rejection
```

### Milestone 10: Wrapper and Decorator Preview

Implement:

```text
ast_wrap_node_preview
ast_add_decorator_preview
indentation preservation
parse-after validation
```

### Milestone 11: Function Signature Preview

Implement:

```text
signature range extraction
add/remove/rename parameter
replace signature
parse-after validation
```

### Milestone 12: V4 Acceptance Tests

Run all V1, V2, V3, and V4 acceptance tests.

---

## 33. Acceptance Criteria

V4 is acceptable when all V1–V3 acceptance criteria still pass and the following are true.

### Generic rewrite preview

- `ast_rewrite_preview` previews replace/insert/delete operations.
- It returns a unified diff when requested.
- It does not write files.
- It validates node kind when provided.
- It rejects invalid ranges.

### Insert import preview

- `ast_insert_import_preview` can insert TypeScript/JavaScript imports.
- It can merge named imports with an existing import where safe.
- It can insert Python `from ... import ...` statements.
- It avoids duplicate imports where possible.
- It does not guarantee semantic existence of imported symbols.

### Remove unused import preview

- `ast_remove_unused_import_preview` removes syntactically unused import specifiers.
- It does not remove side-effect imports.
- It marks ambiguous cases unsafe.

### Local rename preview

- `ast_rename_local_preview` can rename local variables/parameters inside a local scope.
- It rejects imported/exported/top-level symbols.
- It rejects ambiguous scope cases.
- It does not replace semantic rename.

### Wrap node preview

- `ast_wrap_node_preview` can wrap a selected node with prefix/suffix or simple wrapper templates.
- It validates target node kind.
- It parses successfully after rewrite or marks unsafe.

### Add decorator preview

- `ast_add_decorator_preview` inserts decorators/attributes where supported.
- It preserves indentation.
- It rejects unsupported languages or target nodes.

### Modify function signature preview

- `ast_modify_function_signature_preview` can add/remove/rename parameters structurally where supported.
- It detects missing/duplicate parameters.
- It does not update external call sites.

### Validate rewrite

- `ast_validate_rewrite` validates operations without generating diff.
- It detects overlapping edits.
- It detects outside-workspace paths.
- It enforces file/edit limits.

### Parse after rewrite

- `ast_parse_after_rewrite` applies edits in memory.
- It reports syntax errors without writing files.
- It rejects overlapping edits.

### Safety

- No V4 tool writes files.
- No V4 tool applies patches.
- No V4 tool calls LSP MCP.
- All tools enforce workspace boundaries.
- All tools enforce configured limits.

---

## 34. Testing Requirements

### Unit Tests

Required unit tests:

```text
UTF-16 position to byte offset conversion
byte offset to Position conversion
range validation
node alignment validation
node kind validation
workspace path validation
overlapping edit detection
edit application order
diff generation
syntax error collection
import merge logic
side-effect import preservation
local scope detection
unsafe local rename rejection
function parameter parsing
signature modification
```

### Integration Tests

Required integration tests:

```text
TypeScript insert import preview
TypeScript merge import preview
TypeScript remove unused import preview
TypeScript local rename preview
TypeScript wrap call expression preview
TypeScript add decorator preview
TypeScript modify function signature preview
TSX parse-after rewrite
JavaScript insert import preview
Python insert import preview
Python add decorator preview
Python local rename preview
```

### Safety Tests

Required safety tests:

```text
reject path outside workspace
reject invalid range
reject unsupported language
reject too many files
reject too many edits
reject overlapping edits
reject node kind mismatch
reject ambiguous local rename
reject semantic/top-level rename attempt
reject syntax error after rewrite
confirm no file writes occur
confirm LSP MCP is not called
```

---

## 35. Agent Skill Guidance for V4

Agent Skills should use AST V4 for structural rewrite previews only.

### Good uses

```text
Add an import.
Add a decorator.
Wrap a local expression.
Rename a local variable.
Modify a function declaration.
Validate parse after a proposed edit.
Preview a mechanical node replacement.
```

### Bad uses

```text
Rename an exported symbol across files.
Update all semantic references.
Decide whether a type exists.
Decide whether an import resolves.
Validate compiler correctness.
```

For semantic validation, use LSP MCP separately.

Recommended hybrid workflow outside AST MCP:

```text
1. Use AST MCP to create structural rewrite preview.
2. Apply patch through trusted external file-edit tool.
3. Use AST MCP parse-after-rewrite if needed.
4. Use LSP MCP diagnostics/typecheck validation.
5. Run tests externally if needed.
```

---

## 36. Final V4 Design Principle

AST V4 turns the AST MCP into a safe structural rewrite preview engine.

It may:

```text
create text edits
preview structural changes
generate diffs
validate syntax after rewrite
reject unsafe operations
```

It must not:

```text
write files
apply patches
perform semantic rename
call LSP MCP
execute shell commands
claim semantic correctness
```

The boundary remains:

```text
Need structure? Use AST MCP.
Need meaning? Use LSP MCP.
Need both? Use Agent Skills or Code Composite MCP.
```
