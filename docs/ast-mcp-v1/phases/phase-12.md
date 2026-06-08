# Phase 12: V1 Acceptance Tests

> Spec milestone: `spec/version-1.md` § 33 Milestone 12 + § 34 / § 35

## Goal

Run the full test suite, verify every acceptance criterion in `contract/success-criteria.md`, and produce a V1 acceptance report.

## Dependencies

- Requires: phase-11.
- Produces: V1 acceptance report at `docs/ast-mcp-v1/v1-acceptance.md`.

## Risk

- **low** — the work is running tests, not writing them. New tests are minimal (the spec's required safety tests, the fuzz corpus for tree-sitter, and a final integration sweep).

## Value

Closes the V1 loop. After this phase, V1 is shippable per the contract.

## Implementation Notes

- `tests/safety/` — fill in any safety tests not already covered in earlier phases:
  - reject `../outside.ts`
  - reject `/etc/passwd`
  - reject unsupported extension
  - reject missing file
  - reject directory path
  - reject too-large file
  - truncate large output
  - never write files (architectural lint, already in phase 2)
  - no LSP dependency (architectural lint, already in phase 2)
- `tests/fuzz/` — a minimal fuzz harness using `cargo-fuzz` or a hand-rolled random-input test:
  - Random bytes fed to each parser. The harness asserts no panic and a structured error response.
- `tests/integration/` — final sweep that runs all 12 tools against the `tests/fixtures/` corpus and asserts:
  - All 12 tools respond with valid JSON.
  - All response shapes match the spec.
  - All paths in responses are workspace-relative.
  - All truncated responses carry `truncated: true` and `returned: <n>`.
- `docs/ast-mcp-v1/v1-acceptance.md` — the acceptance report. One row per acceptance criterion from `contract/success-criteria.md`, with a green check and the test that proves it. Plus a "Known limitations" section that calls out:
  - Any non-BMP position limitation (per phase 5 docs).
  - The `parseTimeoutMs` and `queryTimeoutMs` are soft budgets enforced via `tokio::time::timeout`, not interrupts.

## Validation

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

All three must pass before V1 is accepted. The clippy and fmt checks are the gate.

## Acceptance

- [ ] `cargo test` passes with all unit, integration, safety, fuzz, and architectural tests.
- [ ] `cargo clippy --all-targets -- -D warnings` produces zero warnings.
- [ ] `cargo fmt --check` reports no formatting issues.
- [ ] `docs/ast-mcp-v1/v1-acceptance.md` exists with one row per success criterion in the contract, each linked to a passing test.
- [ ] The acceptance report includes a "Known limitations" section that lists the position-encoding caveat and the timeout model.
- [ ] All 12 tools are listed in `tools/list` and respond to `tools/call` with valid JSON.
