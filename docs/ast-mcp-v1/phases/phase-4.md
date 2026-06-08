# Phase 4: Basic Parsing Tools

> Spec milestone: `spec/version-1.md` § 33 Milestone 4 + § 13 / § 14 / § 15

## Goal

Ship the first three user-visible tools: `ast_health_check`, `ast_list_supported_languages`, `ast_parse_file`. These prove the full path from JSON-RPC request to JSON-RPC response through the safety and parser layers.

## Dependencies

- Requires: phase-3 (parsers).
- Produces: 3 of 12 V1 tools.

## Risk

- **medium** — well-specified; the risk is wiring the tool registration correctly.

## Value

The first 3 tools a user will exercise in the first session. They validate the full pipeline.

## Implementation Notes

- `src/tools/health_check.rs`:
  - Input: `AstHealthCheckInput { workspacePath?: Option<String> }`.
  - Output: `AstHealthCheckResult { workspacePath, ok, parsers: Vec<ParserStatus>, limits: Limits }`.
  - Does **not** parse files; only checks the registry and the workspace.
- `src/tools/list_supported_languages.rs`:
  - Input: `{}` (empty).
  - Output: `AstListSupportedLanguagesResult { languages: Vec<LanguageDescriptor> }`.
  - Walks the registry and emits one entry per parser.
- `src/tools/parse_file.rs`:
  - Input: `AstParseFileInput { filePath, includeTree?: bool, maxDepth?: usize, includeNodeText?: bool }`.
  - Defaults: `includeTree: false, maxDepth: 3, includeNodeText: false`.
  - Pipeline: resolve path via `safety::paths` → check `MAX_FILE_BYTES` → read → call `parser::parse::parse_source` → return `AstParseFileResult`.
  - If `includeTree`, walk the tree to `maxDepth` (default 3) and emit `AstNodeSummary` nodes, capped at `MAX_NODES` (500) with `truncated: true` if hit. Text is included only if `includeNodeText` is true and capped at `MAX_TEXT_BYTES`.
- `src/mcp/register_tools.rs` registers all 3 tools with their JSON schema. The schema is hand-written for V1 (per spec § 31); `schemars` may be added later.
- `src/shared/errors.rs` defines the `AstToolError` enum and the `error_payload()` helper used by all tools.
- `src/shared/ast_node.rs` defines `AstNodeSummary { kind, name?, range, startByte?, endByte?, text?, children? }` — `Range` is filled with placeholder `{0,0}–{0,0}` in this phase; phase 5 fills in real positions.

## Validation

```bash
cargo build
cargo test --test integration
# Manual: point at a tiny TS workspace and exercise the 3 tools.
```

The integration test must build a tempdir with one `.ts` and one `.py` file, call the 3 tools, and assert the response shapes match the spec.

## Acceptance

- [ ] `ast_health_check` returns the workspace path, `ok: true` for a valid dir, and a `parsers` array of length 5 with all `available: true`.
- [ ] `ast_list_supported_languages` returns the 5 V1 languages with extension lists and parser crate names.
- [ ] `ast_parse_file` on a valid `.ts` file returns `parsed: true, hasSyntaxError: false, rootKind: "program"`, plus a non-zero `nodeCount` and a `parseTimeMs`.
- [ ] `ast_parse_file` on a file with a syntax error returns `parsed: true, hasSyntaxError: true`.
- [ ] `ast_parse_file` on a missing file returns the `file_not_found` error.
- [ ] `ast_parse_file` on a too-large file returns `file_too_large`.
- [ ] `ast_parse_file` with `includeTree: true` returns a tree capped at `MAX_NODES` and `maxDepth`.
