# Spec Phase 11: Tree-sitter Query

## Phase Goal

Ship `ast_query` with query compilation, capture normalization, and a hard timeout.

## Dependencies

- Requires: phase-10.
- Produces: 12 of 12 V1 tools.

## Existing Code References

- Pattern to follow: `src/tools/find_classes::handle` (phase 9).
- Related modules: `parser::parse::parse_source` (phase 3), `parser::positions` (phase 5).
- Test pattern: `tests/integration/chunking.rs` (phase 10) — extend with query tests.

## Technical Approach

A `compile_query` helper that maps `tree_sitter::QueryError` to a structured `query_invalid` error. A `run_query` helper that wraps the match loop in `tokio::time::timeout`. The tool handler stitches them together.

## File Changes

### New Files

| File | Purpose |
|---|---|
| `src/parser/queries.rs` | `compile_query` helper. |
| `src/extractors/queries.rs` | `run_query` extractor. |
| `src/tools/query.rs` | `ast_query` handler. |
| `tests/fixtures/query/functions.ts` | A TS file with several functions. |
| `tests/fixtures/query/functions.py` | A Python file with several functions. |
| `tests/integration/query.rs` | End-to-end tests. |

### Modified Files

| File | Change |
|---|---|
| `src/mcp/register_tools.rs` | Register the new tool. |

## Implementation Steps

1. `src/parser/queries.rs`:
   ```rust
   pub fn compile_query(language: LanguageId, source: &str) -> Result<tree_sitter::Query, AstToolError> {
       let lang_fn = parser::registry::for_language(language)
           .ok_or_else(|| AstToolError::ParserUnavailable(language.as_str().into()))?
           .tree_sitter_language;
       tree_sitter::Query::new((lang_fn)(), source).map_err(|e| {
           let details = json!({
               "language": language.as_str(),
               "row": e.row,
               "column": e.column,
               "offset": e.offset,
               "kind": format!("{:?}", e.kind),
           });
           AstToolError::QueryInvalid(format!("row {} column {}: {}", e.row, e.column, e.message)).with_details(details)
       })
   }
   ```
   `AstToolError::QueryInvalid` is extended with a `details: serde_json::Value` field. Add a `with_details` constructor.
2. `src/extractors/queries.rs`:
   ```rust
   pub struct AstQueryMatch { pub pattern_index: Option<u32>, pub captures: Vec<Capture> }
   pub struct Capture { pub name: String, pub kind: String, pub range: Range, pub text: Option<String> }
   pub fn run_query(query: &tree_sitter::Query, tree: &Tree, source: &str, opts: QueryOptions) -> Vec<AstQueryMatch>;
   ```
   - For each `query.matches(...)`:
     - For each capture in the match, push a `Capture` with the capture name, the node kind, the range (via `ts_point_to_position`), and optional text.
     - Sort captures by name to make the output deterministic.
   - Cap the result count at `MAX_QUERY_MATCHES` (200) with `truncated: true` if hit.
3. `src/tools/query.rs`:
   - `AstQueryInput { file_path, query, max_results?, include_node_text?, max_text_bytes? }`.
   - Defaults: `max_results: 200, include_node_text: true, max_text_bytes: 20,000`.
   - Pipeline:
     ```rust
     let result = tokio::time::timeout(Duration::from_millis(QUERY_TIMEOUT_MS), async {
         let resolved = safety::paths::resolve_file(workspace, &input.file_path)?;
         let source = std::fs::read_to_string(&resolved.absolute)?;
         safety::paths::ensure_under_size(source.len() as u64)?;
         let lang = extension_to_language(&resolved)?;
         let (tree, _) = parser::parse::parse_source(&source, lang)?;
         let query = parser::queries::compile_query(lang, &input.query)?;
         let matches = extractors::queries::run_query(&query, &tree, &source, opts);
         Ok::<_, AstToolError>(matches)
     }).await;
     match result {
         Ok(Ok(matches)) => json!({ "filePath": ..., "language": ..., "matches": matches, "returned": matches.len(), "truncated": matches.len() >= MAX_QUERY_MATCHES }),
         Ok(Err(e)) => e.payload(),
         Err(_) => AstToolError::QueryExecutionFailed(format!("exceeded {} ms", QUERY_TIMEOUT_MS)).with_details(json!({ "timeoutMs": QUERY_TIMEOUT_MS })).payload(),
     }
     ```
4. Tests:
   - TS: `(function_declaration name: (identifier) @function.name)` → one match per function with the captured name.
   - Python: `(function_definition name: (identifier) @function.name)` → same shape.
   - Invalid query: `(((` → `query_invalid` with `details.row > 0` and `details.column > 0`.
   - Timeout: a query that loops for too long (e.g., a deliberately pathological pattern) returns `query_execution_failed` with `details.timeoutMs`.

## Data / API / Interface Contract

```rust
// parser::queries
pub fn compile_query(language: LanguageId, source: &str) -> Result<tree_sitter::Query, AstToolError>;
// extractors::queries
pub struct AstQueryMatch { pub pattern_index: Option<u32>, pub captures: Vec<Capture> }
pub struct Capture { pub name: String, pub kind: String, pub range: Range, pub text: Option<String> }
pub struct QueryOptions { pub max_results: usize, pub include_node_text: bool, pub max_text_bytes: usize }
pub fn run_query(query: &tree_sitter::Query, tree: &Tree, source: &str, opts: QueryOptions) -> Vec<AstQueryMatch>;
```

Tool response shape (matches spec § 24):

```jsonc
{
  "filePath": "src/user.ts",
  "language": "typescript",
  "matches": [
    { "patternIndex": 0, "captures": [{ "name": "function.name", "kind": "identifier", "range": { /* ... */ }, "text": "getUser" }] }
  ],
  "returned": 4,
  "truncated": false
}
```

## Error Handling

- `query_invalid` — query string failed to compile. `details: { language, row, column, offset, kind }`.
- `query_execution_failed` — query exceeded `QUERY_TIMEOUT_MS`. `details: { timeoutMs }`.
- `file_not_found`, `file_too_large`, `unsupported_language` — propagated.

## Observability

- Logs: `tracing::debug!` with `file_path`, `language`, query size, match count, elapsed time. Stderr.

## Testing Requirements

### Unit Tests

- `compile_query` returns `QueryInvalid` with `details` for malformed queries.
- `compile_query` returns `ParserUnavailable` if a `LanguageId` is given without a registry entry (defensive).
- `run_query` caps at `max_results` and sets `truncated: true`.
- `run_query` returns captures in deterministic order (sorted by name).

### Integration Tests

- `tests/integration/query.rs`:
  - Valid TS query → expected matches.
  - Valid Python query → expected matches.
  - Invalid query string → `query_invalid` with `details`.
  - Pathological query → `query_execution_failed` with `details.timeoutMs`.

## Validation Commands

```bash
cargo test --lib parser::queries
cargo test --test integration query
```

## Acceptance Criteria

- [ ] `ast_query` on a valid TS query returns matches with normalized captures.
- [ ] `ast_query` on a valid Python query returns matches.
- [ ] `ast_query` on an invalid query string returns `query_invalid` with `details`.
- [ ] `ast_query` on a query that exceeds the timeout returns `query_execution_failed` with `details.timeoutMs`.
- [ ] Results are capped at `maxResults` (default 200); `truncated: true` is set if hit.
- [ ] Capture text is included only if `includeNodeText: true` and bounded by `maxTextBytes`.
- [ ] `cargo clippy --all-targets -- -D warnings` passes.

## Risks

| Risk | Severity | Mitigation |
|---|---|---|
| `tokio::time::timeout` is a soft cancel: the inner task keeps running until next yield | medium | The query is CPU-bound; the timeout fires but the task continues. We accept this in V1 — the response returns within 5 s even if the inner task takes longer. The next request that touches the same parser is unaffected because each request is its own task. |
| `tree_sitter::Query` is not `Send` | low | The query lives in a single async task; we do not share it across tasks. |
| `details` for `QueryInvalid` is not part of the original error type | low | Extend `AstToolError::QueryInvalid` to carry `details: Option<serde_json::Value>`. |
