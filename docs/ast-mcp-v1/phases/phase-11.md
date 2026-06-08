# Phase 11: Tree-sitter Query

> Spec milestone: `spec/version-1.md` § 33 Milestone 11 + § 24

## Goal

Ship `ast_query`. This is the power-tool for advanced agent workflows and internal development.

## Dependencies

- Requires: phase-10.
- Produces: 12 of 12 V1 tools.

## Risk

- **medium** — query compilation errors must be reported cleanly, and the timeout must be a hard cancel (not a soft budget).

## Value

Unlocks structural search across the file. Combined with chunking, it gives agents a way to do "find all calls to `getUser`" or "find all classes implementing `IService`" — syntactically, not semantically.

## Implementation Notes

- `src/parser/queries.rs`:
  - `pub fn compile_query(language: LanguageId, source: &str) -> Result<Query, AstToolError>`.
  - On `tree_sitter::QueryError`, return `query_invalid` with `details: { language, line, column, offset, kind: "..." }`.
  - Map error kinds: `NodeType`, `Field`, `Capture`, `Structure`, `Language`, `Symbol`, `Syntax`, `Other`.
- `src/extractors/queries.rs` (or in `tools/query.rs` directly):
  - Parse the file.
  - Compile the query.
  - Run on a Tokio task wrapped in `tokio::time::timeout(Duration::from_millis(QUERY_TIMEOUT_MS), ...)`.
  - For each match, collect captures (named first, then anonymous), normalize to `{ name, kind, range, text? }`.
  - Cap results at `MAX_QUERY_MATCHES` (200) with `truncated: true` if hit.
- `src/tools/query.rs`:
  - Input: `AstQueryInput { filePath, query, maxResults?, includeNodeText?, maxTextBytes? }`.
  - Defaults: `maxResults: 200, includeNodeText: true, maxTextBytes: 20,000`.
  - On timeout, return `query_execution_failed` with `details.timeout_ms: QUERY_TIMEOUT_MS`.

## Validation

```bash
cargo test --test integration ast_query
cargo test --lib parser::queries
```

Unit tests: invalid query string returns `query_invalid`; empty query returns `query_invalid`; valid query compiles and runs.

Integration tests:
- TS fixture: `(function_declaration name: (identifier) @function.name)` — returns one match per function with the captured name.
- Python fixture: `(function_definition name: (identifier) @function.name)` — same shape.
- A pathological query that exceeds `QUERY_TIMEOUT_MS` returns `query_execution_failed` with `details.timeout_ms`.

## Acceptance

- [ ] `ast_query` on a valid TS query returns matches with normalized captures.
- [ ] `ast_query` on a valid Python query returns matches.
- [ ] `ast_query` on an invalid query string returns `query_invalid` with `details` describing the parse error.
- [ ] `ast_query` on a query that exceeds the timeout returns `query_execution_failed` with `details.timeout_ms`.
- [ ] Results are capped at `maxResults` (default 200); `truncated: true` is set if hit.
- [ ] Capture text is included only if `includeNodeText: true` and bounded by `maxTextBytes`.
