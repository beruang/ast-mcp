# Spec Phase 12: V1 Acceptance Tests

## Phase Goal

Run the full test suite, verify every acceptance criterion in `contract/success-criteria.md`, and produce the V1 acceptance report.

## Dependencies

- Requires: phase-11.
- Produces: `docs/ast-mcp-v1/v1-acceptance.md`.

## Existing Code References

- Pattern to follow: `tests/integration/query.rs` (phase 11) — the integration sweep follows the same shape.
- Related modules: all of the V1 codebase.

## Technical Approach

Add the safety tests that earlier phases deferred, a minimal fuzz harness, a final integration sweep, and the acceptance report. No new tool logic lands in this phase.

## File Changes

### New Files

| File | Purpose |
|---|---|
| `tests/safety/rejections.rs` | All "reject" cases. |
| `tests/safety/truncation.rs` | Truncation behavior. |
| `tests/fuzz/parser_fuzz.rs` | Random-input fuzz for each parser. |
| `tests/integration/sweep.rs` | End-to-end sweep of all 12 tools against a fixture corpus. |
| `docs/ast-mcp-v1/v1-acceptance.md` | Acceptance report. |
| `docs/ast-mcp-v1/position-encoding.md` (extend) | Add the "Verified in V1.0" line. |

### Modified Files

| File | Change |
|---|---|
| `Cargo.toml` | Add `[dev-dependencies]` for `tempfile` and `rand` if not already present. |

## Implementation Steps

1. `tests/safety/rejections.rs` — for each case, build a tempdir with the right shape, call the tool, assert the error code:
   - `../outside.ts` → `path_outside_workspace`.
   - `/etc/passwd` → `path_outside_workspace`.
   - `Cargo.toml` (which is a directory in the test setup) → `file_not_found` (canonical path is a directory).
   - `missing.ts` → `file_not_found`.
   - A `.rb` file → `unsupported_language`.
   - A 2 MiB file → `file_too_large`.
2. `tests/safety/truncation.rs`:
   - `ast_parse_file` with `includeTree: true` on a 1,000-node file → `truncated: true` (if the file has > 500 nodes).
   - `ast_query` on a query that would return > 200 matches → `truncated: true`.
3. `tests/fuzz/parser_fuzz.rs`:
   - For each of the 5 languages, generate 100 random byte sequences of length 1–4,096. Feed each to the parser. Assert no panic; assert a structured response (a tree or a structured error).
4. `tests/integration/sweep.rs`:
   - Build a tempdir with 5 files: `sample.ts`, `sample.tsx`, `sample.js`, `sample.jsx`, `sample.py`. Each file is a small program with imports, exports, classes, functions, and types.
   - For each of the 12 tools, call it with the appropriate file and assert:
     - Response is valid JSON.
     - Response shape matches the spec.
     - All paths in the response are workspace-relative.
     - `truncated` field is present and is a boolean.
5. `docs/ast-mcp-v1/v1-acceptance.md`:
   - Header: AST MCP Server V1 Acceptance Report, date, sign-off line.
   - One row per acceptance criterion in `contract/success-criteria.md`, with a green check and the test that proves it.
   - "Known limitations" section that calls out:
     - Position-encoding caveat (V1 is exact for ASCII, Latin-1, BMP, and surrogate pairs; full non-BMP correctness is asserted by the test suite).
     - The `parseTimeoutMs` and `queryTimeoutMs` are soft budgets enforced via `tokio::time::timeout`, not interrupts.
   - "Performance notes" section: real parse times recorded during the sweep, against the 1 MiB and 5 s limits.
6. `position-encoding.md` — add a line at the top: "Verified in V1.0 — see `tests/integration/position_utf16.rs`."

## Data / API / Interface Contract

This phase introduces no new public API. The acceptance report is a Markdown file, not a typed artifact.

## Error Handling

No new error codes.

## Observability

- Logs: `tracing::info!` at the start of the sweep with the fixture path. Stderr.
- The acceptance report includes elapsed times for each tool.

## Testing Requirements

### Unit Tests

None new.

### Integration Tests

- `tests/safety/rejections.rs`.
- `tests/safety/truncation.rs`.
- `tests/fuzz/parser_fuzz.rs`.
- `tests/integration/sweep.rs`.

### Architectural Tests

- The phase-2 lint tests must still pass.
- Add a third architectural test: `tests/architecture/no_panic.rs` — walks `src/`, asserts that no `unwrap()` or `expect()` exists outside test code or in the safety/parser layer. (Best-effort lint; allowed exclusions are documented in the test file.)

## Validation Commands

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

All three must pass.

## Acceptance Criteria

- [ ] `cargo test` passes with all unit, integration, safety, fuzz, sweep, and architectural tests.
- [ ] `cargo clippy --all-targets -- -D warnings` produces zero warnings.
- [ ] `cargo fmt --check` reports no formatting issues.
- [ ] `docs/ast-mcp-v1/v1-acceptance.md` exists with one row per success criterion in the contract, each linked to a passing test.
- [ ] The acceptance report includes a "Known limitations" section listing the position-encoding caveat and the timeout model.
- [ ] All 12 tools are listed in `tools/list` and respond to `tools/call` with valid JSON.
- [ ] `tools/list` returns exactly 12 tools (no more, no less).

## Risks

| Risk | Severity | Mitigation |
|---|---|---|
| Fuzz harness panics on a real Tree-sitter bug | medium | Wrap the parser call in `catch_unwind`; convert panics to `parse_failed`. |
| Sweep test fixture diverges from the spec | low | The fixture is checked in; if a tool's response shape changes, the sweep test fails first. |
| `cargo fmt --check` discovers unformatted code | low | The CI run is gated; the developer can run `cargo fmt` to fix. |
