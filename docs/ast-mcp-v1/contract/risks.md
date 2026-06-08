# Risks

Each risk is rated by severity (critical, high, medium, low, info) and likelihood (near-certain, likely, possible, unlikely, rare).

## R1. UTF-16 position conversion is subtly wrong for non-BMP characters

- **Severity:** medium
- **Likelihood:** possible
- **Impact:** `ast_enclosing_node`, `ast_top_level_nodes`, `ast_query`, and any tool that returns `Range` may report an off-by-one column for files containing emoji or other astral-plane characters.
- **Mitigation:** Add explicit tests for non-BMP strings (e.g., `"😀"` and surrogate-pair identifiers). Document any known limitation in the README. If full correctness is required, adopt a UTF-16 ↔ byte-offset table built from the source bytes.
- **Owner:** Phase 5 (position conversion).

## R2. Tree-sitter version drift breaks extraction

- **Severity:** medium
- **Likelihood:** possible
- **Impact:** The `tree-sitter-typescript = "0.20"` and friends may release breaking changes; node-kind names can change.
- **Mitigation:** Pin exact versions in `Cargo.toml`. Run the full test suite on a clean `cargo update -p <crate>`. Record any pinned version in `decisions.md`.
- **Owner:** Phase 3 (parser registry).

## R3. Bounded output drops important context

- **Severity:** medium
- **Likelihood:** likely
- **Impact:** A `maxResults: 200` cap will silently hide matching nodes from an agent. If the agent does not see the `truncated: true` flag, it will assume the result is complete.
- **Mitigation:** Every truncated response carries `truncated: true` and `returned: <n>`. Add an integration test that asserts the flag is set when limits are hit. Document the behavior in tool descriptions.
- **Owner:** Phases 4–11 (all tool phases).

## R4. "Never write files" is broken by a future refactor

- **Severity:** high
- **Likelihood:** unlikely
- **Impact:** A refactor that introduces a write — even for caching or temp files — violates the contract.
- **Mitigation:** Architectural lint test (`tests/architecture/no_write.rs`) scans the source tree for `fs::write`, `tokio::fs::write`, `OpenOptions::write`, and `rename`. The test fails CI if any of these appear.
- **Owner:** Phase 2 (workspace safety) — the lint test ships here so the invariant is protected from day one.

## R5. "Never call LSP" is broken by a future refactor

- **Severity:** critical
- **Likelihood:** unlikely
- **Impact:** A dependency on an LSP crate, or a subprocess of a language server, would silently cross the architectural boundary.
- **Mitigation:** Architectural lint test (`tests/architecture/no_lsp.rs`) scans `Cargo.toml` for `lsp` deps and source for `use lsp`. The test fails CI if any are found.
- **Owner:** Phase 2 (workspace safety) — same rationale as R4.

## R6. Workspace path traversal via symbolic links

- **Severity:** high
- **Likelihood:** possible
- **Impact:** A symlink inside the workspace that points outside it could allow `ast_parse_file` to read `/etc/passwd`.
- **Mitigation:** Resolve symlinks during path validation (`std::fs::canonicalize`) and re-check containment against the workspace root. Add a safety test that creates a symlink to `/etc/passwd` and asserts the request is rejected.
- **Owner:** Phase 2 (workspace safety).

## R7. Query timeout is a soft limit, not a hard cancel

- **Severity:** medium
- **Likelihood:** possible
- **Impact:** `queryTimeoutMs = 5000` is a budget, not a hard interrupt. A pathological query could exceed it before the budget is checked, freezing the request thread.
- **Mitigation:** Run the query on a Tokio task and wrap the result with `tokio::time::timeout`. If the timeout fires, return `query_execution_failed` with `details.timeout_ms`. Document the soft-vs-hard nature in the tool description.
- **Owner:** Phase 11 (Tree-sitter query).

## R8. Tree-sitter crashes on malformed input

- **Severity:** low
- **Likelihood:** rare
- **Impact:** A fuzzed or pathological input could panic in the parser. V1 must not crash the server.
- **Mitigation:** Wrap parser invocation in `catch_unwind` (or equivalent panic-safe boundary) and convert panics to `parse_failed` errors. Add a fuzz test corpus in Phase 12.
- **Owner:** Phase 4 (basic parsing) and Phase 12 (acceptance tests).

## R9. Chunk IDs are not stable across versions

- **Severity:** low
- **Likelihood:** possible
- **Impact:** A change in chunking strategy may change the `id` format, breaking agent-side caching by `id`.
- **Mitigation:** Document the ID format in `docs/ast-mcp-v1/chunk-id.md` and commit to backward compatibility for the `id` shape. If the format must change, version it (e.g., `id = "v2:..."`).
- **Owner:** Phase 10 (chunking).

## R10. Multi-version spec drift

- **Severity:** low
- **Likelihood:** possible
- **Impact:** `spec/version-2.md` … `spec/version-5.md` already exist alongside V1. Implementation agents may pick up tools from V2+ that are not in V1.
- **Mitigation:** The V1 contract is explicit about scope and V2+ are deferred. The V1 spec files (in `docs/ast-mcp-v1/spec/`) reference the V1 tool surface only. The `specs.index.ndjson` is namespaced to V1.
- **Owner:** This contract.

## R11. JSON payload size for full parse trees

- **Severity:** low
- **Likelihood:** likely
- **Impact:** `ast_parse_file` with `includeTree: true` can produce a multi-MB JSON payload, blowing past the `maxTextBytes` default for many tools.
- **Mitigation:** `includeTree` is opt-in and bounded by `maxNodes` (500) and `maxDepth` (3). The default of `includeTree: false` is documented in tool descriptions and the spec.
- **Owner:** Phase 4 (basic parsing).
