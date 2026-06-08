# Spec Phase 4: Basic Parsing Tools

## Phase Goal

Ship `ast_health_check`, `ast_list_supported_languages`, `ast_parse_file`. These are the first 3 of 12 V1 tools and validate the full pipeline.

## Dependencies

- Requires: phase-3 (parsers, language enum).
- Produces: 3 of 12 V1 tools.

## Existing Code References

- Pattern to follow: `mcp::register_tools::ToolSpec` (phase 1) — every tool follows this shape.
- Related module: `safety::paths::resolve_file` (phase 2), `parser::parse::parse_source` (phase 3), `shared::errors::AstToolError` (phase 2).
- Test pattern: `tests/integration/transport.rs` (phase 1) — extend with a tempdir + tool-call test.

## Technical Approach

Three tool handlers, one `AstNodeSummary` type, and one `AstParseFileResult` type. Tool registration is hand-written (no `schemars` in V1) — each tool's JSON Schema is a `serde_json::Value` constant.

## File Changes

### New Files

| File | Purpose |
|---|---|
| `src/shared/ast_node.rs` | `AstNodeSummary` type with placeholder ranges. |
| `src/tools/mod.rs` | `register_all(workspace: &Workspace) -> Vec<ToolSpec>`. |
| `src/tools/health_check.rs` | `ast_health_check` handler. |
| `src/tools/list_supported_languages.rs` | `ast_list_supported_languages` handler. |
| `src/tools/parse_file.rs` | `ast_parse_file` handler. |
| `tests/integration/parse_file.rs` | End-to-end test using a tempdir. |
| `tests/fixtures/sample.ts` | A small TS file. |
| `tests/fixtures/sample.py` | A small Python file. |

### Modified Files

| File | Change |
|---|---|
| `src/mcp/register_tools.rs` | Replace the dummy `ast_health_check` with the real handler. |
| `src/main.rs` | Construct `Workspace` and pass it to `register_all`. |

## Implementation Steps

1. `src/shared/ast_node.rs`:
   ```rust
   #[derive(Debug, Clone, serde::Serialize)]
   pub struct AstNodeSummary {
       pub kind: String,
       pub name: Option<String>,
       pub range: Range,                  // Range from phase-5; placeholder for now
       pub start_byte: Option<usize>,
       pub end_byte: Option<usize>,
       pub text: Option<String>,
       pub children: Option<Vec<AstNodeSummary>>,
   }
   ```
   Use `serde(tag = "kind")` if the type is exposed in error details; otherwise default to `kind: String` is fine.
2. `src/tools/health_check.rs`:
   ```rust
   pub fn handle(workspace: &Workspace, _args: serde_json::Value) -> serde_json::Value {
       let parsers: Vec<_> = parser::registry::registry().iter().map(|d| json!({
           "language": d.language.as_str(),
           "extensions": d.extensions,
           "available": true,
           "parser": format!("tree-sitter-{}", d.language.as_str()),
       })).collect();
       json!({
           "workspacePath": workspace.root().display().to_string(),
           "ok": true,
           "parsers": parsers,
           "limits": {
               "maxFileBytes": config::defaults::MAX_FILE_BYTES,
               "maxNodes": config::defaults::MAX_NODES,
               "maxResults": config::defaults::MAX_RESULTS,
           }
       })
   }
   ```
3. `src/tools/list_supported_languages.rs`:
   ```rust
   pub fn handle(_args: serde_json::Value) -> serde_json::Value {
       let languages: Vec<_> = parser::registry::registry().iter().map(|d| json!({
           "language": d.language.as_str(),
           "extensions": d.extensions,
           "parser": format!("tree-sitter-{}", d.language.as_str()),
           "available": true,
       })).collect();
       json!({ "languages": languages })
   }
   ```
4. `src/tools/parse_file.rs`:
   ```rust
   pub fn handle(workspace: &Workspace, args: serde_json::Value) -> serde_json::Value {
       let input: AstParseFileInput = match serde_json::from_value(args) {
           Ok(v) => v,
           Err(e) => return AstToolError::InvalidPosition(e.to_string()).payload(),
       };
       let resolved = match safety::paths::resolve_file(workspace, &input.file_path) {
           Ok(r) => r,
           Err(e) => return e.payload(),
       };
       let meta = match std::fs::metadata(&resolved.absolute) {
           Ok(m) => m,
           Err(e) => return AstToolError::FileNotFound(e.to_string()).payload(),
       };
       if let Err(e) = safety::paths::ensure_under_size(meta.len()) { return e.payload(); }
       let source = match std::fs::read_to_string(&resolved.absolute) {
           Ok(s) => s,
           Err(e) => return AstToolError::FileNotFound(e.to_string()).payload(),
       };
       let lang = match extension_to_language(&resolved) {
           Some(l) => l,
           None => return AstToolError::UnsupportedLanguage(resolved.absolute.extension().and_then(|s| s.to_str()).unwrap_or("").to_string()).payload(),
       };
       let (tree, status) = match parser::parse::parse_source(&source, lang) {
           Ok(t) => t,
           Err(e) => return e.payload(),
       };
       let mut result = json!({
           "filePath": resolved.workspace_relative,
           "language": lang.as_str(),
           "parsed": true,
           "hasSyntaxError": status.has_syntax_error,
           "rootKind": status.root_kind,
           "nodeCount": status.node_count,
           "parseTimeMs": status.parse_time_ms,
       });
       if input.include_tree {
           let mut count = 0usize;
           let tree_json = walk_tree(&tree.root_node(), &source, input.max_depth.unwrap_or(3), input.include_node_text.unwrap_or(false), safety::limits::MAX_TEXT_BYTES, &mut count, safety::limits::MAX_NODES);
           result["tree"] = tree_json;
           result["truncated"] = json!(count >= safety::limits::MAX_NODES);
       }
       result
   }
   ```
5. `AstParseFileInput`:
   ```rust
   #[derive(serde::Deserialize)]
   #[serde(default)]
   pub struct AstParseFileInput {
       pub file_path: String,
       pub include_tree: bool,         // default false
       pub max_depth: Option<usize>,   // default 3
       pub include_node_text: bool,    // default false
   }
   impl Default for AstParseFileInput { /* include_tree: false, max_depth: Some(3), include_node_text: false */ }
   ```
6. `walk_tree` — recursive walker that returns an `AstNodeSummary` and increments a counter. When the counter reaches `MAX_NODES`, the recursion stops and the parent is marked truncated. Text is included only if `include_node_text` is true and the text fits in `max_text_bytes` (truncated at byte boundary if needed).
7. `tests/integration/parse_file.rs`:
   - Build a tempdir with `sample.ts` (5 lines) and `sample.py` (5 lines).
   - Call `ast_health_check` → assert `ok: true`, `parsers.len() == 5`.
   - Call `ast_list_supported_languages` → assert 5 entries.
   - Call `ast_parse_file` on `sample.ts` → assert `parsed: true`, `hasSyntaxError: false`, `rootKind: "program"`.
   - Call `ast_parse_file` on `sample.py` → same shape.
   - Call `ast_parse_file` on a non-existent file → assert error code `file_not_found`.
   - Call `ast_parse_file` on a 2 MiB file → assert error code `file_too_large`.

## Data / API / Interface Contract

```rust
// tools::parse_file
pub struct AstParseFileInput { file_path: String, include_tree: bool, max_depth: Option<usize>, include_node_text: bool }

// shared::ast_node
pub struct AstNodeSummary { kind: String, name: Option<String>, range: Range, start_byte: Option<usize>, end_byte: Option<usize>, text: Option<String>, children: Option<Vec<AstNodeSummary>> }
```

Tool response shape (matches spec § 15):

```jsonc
{
  "filePath": "src/user.ts",
  "language": "typescript",
  "parsed": true,
  "hasSyntaxError": false,
  "rootKind": "program",
  "nodeCount": 142,
  "parseTimeMs": 3,
  "tree": { /* present only if includeTree: true */ }
}
```

## Error Handling

- `file_not_found` — file does not exist.
- `file_too_large` — file size > `MAX_FILE_BYTES`.
- `unsupported_language` — extension not in the registry.
- `parse_failed` — `Parser::parse` returned `None` (rare; usually a parser bug).
- `internal_error` — anything else.

## Observability

- Logs: `tracing::debug!` with `file_path`, `language`, `parse_time_ms`. Stderr.
- No metrics. No traces.

## Testing Requirements

### Unit Tests

- `walk_tree` clamps at `max_depth`.
- `walk_tree` clamps at `MAX_NODES` and sets `truncated: true` at the parent.
- Text truncation in `walk_tree` is byte-safe (cuts at a char boundary, not mid-codepoint).

### Integration Tests

- `tests/integration/parse_file.rs` — the cases in step 7.

## Validation Commands

```bash
cargo build
cargo test
cargo test --test integration parse_file
```

## Acceptance Criteria

- [ ] `ast_health_check` returns `ok: true` and 5 parsers, all `available: true`.
- [ ] `ast_list_supported_languages` returns 5 entries with extension lists and parser crate names.
- [ ] `ast_parse_file` on a valid `.ts` file returns `parsed: true, hasSyntaxError: false, rootKind: "program"`.
- [ ] `ast_parse_file` on a file with a syntax error returns `hasSyntaxError: true`.
- [ ] `ast_parse_file` on a missing file returns `file_not_found`.
- [ ] `ast_parse_file` on a 2 MiB file returns `file_too_large`.
- [ ] `ast_parse_file` on an unsupported extension returns `unsupported_language`.
- [ ] `ast_parse_file` with `includeTree: true` returns a tree capped at `max_depth` (3) and `MAX_NODES` (500).
- [ ] `cargo clippy --all-targets -- -D warnings` passes.

## Risks

| Risk | Severity | Mitigation |
|---|---|---|
| `walk_tree` recursion blows the stack on a very deep tree | low | Cap depth at 64. The default `max_depth: 3` is far below. |
| `walk_tree` text truncation splits a multi-byte character | medium | Walk back to a `char_boundary` if the truncation would split a codepoint. |
| `parse_source` panic on malformed input | low | Wrap the parser call in `std::panic::catch_unwind` and return `parse_failed`. (Phase 12 adds a fuzz harness.) |
