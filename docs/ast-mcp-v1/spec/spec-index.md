# Spec Index: AST MCP Server V1

## Summary

12 implementation specs, one per phase. Each spec turns the phase's "what and why" into the "how" an implementation agent needs to write code without re-reading the contract.

## Specs

| Spec | Title | Depends On | Purpose | Status |
|---|---|---|---|---|
| [`spec-phase-1.md`](./spec-phase-1.md) | Rust project skeleton | — | Cargo crate, stdio transport, one dummy tool. | Draft |
| [`spec-phase-2.md`](./spec-phase-2.md) | Workspace safety | phase-1 | `WORKSPACE_PATH`, path validation, file size limits, architectural lint tests. | Draft |
| [`spec-phase-3.md`](./spec-phase-3.md) | Parser registry | phase-2 | 5 V1 Tree-sitter parsers, extension routing, `parser::registry` + `parser::parse`. | Draft |
| [`spec-phase-4.md`](./spec-phase-4.md) | Basic parsing tools | phase-3 | `ast_health_check`, `ast_list_supported_languages`, `ast_parse_file`. | Draft |
| [`spec-phase-5.md`](./spec-phase-5.md) | Position conversion | phase-4 | UTF-16 ↔ byte-offset helpers; non-BMP tests. | Draft |
| [`spec-phase-6.md`](./spec-phase-6.md) | Outline and top-level nodes | phase-5 | `ast_file_outline`, `ast_top_level_nodes`. | Draft |
| [`spec-phase-7.md`](./spec-phase-7.md) | Enclosing node | phase-6 | `ast_enclosing_node` with kind filter and ancestors. | Draft |
| [`spec-phase-8.md`](./spec-phase-8.md) | Imports and exports | phase-7 | `ast_find_imports`, `ast_find_exports` for TS/JS/Python. | Draft |
| [`spec-phase-9.md`](./spec-phase-9.md) | Functions and classes | phase-8 | `ast_find_functions`, `ast_find_classes` with method/parameter extraction. | Draft |
| [`spec-phase-10.md`](./spec-phase-10.md) | Chunking | phase-9 | `ast_chunk_file` with all 4 strategies. | Draft |
| [`spec-phase-11.md`](./spec-phase-11.md) | Tree-sitter query | phase-10 | `ast_query` with timeout and capture normalization. | Draft |
| [`spec-phase-12.md`](./spec-phase-12.md) | V1 acceptance tests | phase-11 | Run all tests, produce acceptance report. | Draft |

## Recommended Reading Order

1. [`../contract.md`](../contract.md) — locked intent and constraints.
2. [`../phases.md`](../phases.md) — implementation breakdown and dependencies.
3. The relevant `spec-phase-N.md` for the phase you are implementing.
4. The phase's `../phases/phase-N.md` for the "why" behind the spec.

## Agent Loading Guidance

Implementation agents should start with:

1. `.agent/contracts/ast-mcp-v1/manifest.json`
2. `.agent/contracts/ast-mcp-v1/specs.index.ndjson`
3. The specific phase spec assigned to them (`spec/spec-phase-N.md`)

## Spec Conventions

Each spec follows the same shape:

- **Phase Goal** — one sentence.
- **Dependencies** — phase IDs and the artifacts this phase writes.
- **Existing Code References** — `Unknown` (no prior Rust code in the project).
- **Technical Approach** — module-by-module description.
- **File Changes** — new and modified files.
- **Implementation Steps** — ordered list, runnable top-to-bottom.
- **Data / API / Interface Contract** — public types and function signatures.
- **Error Handling** — which error codes this phase may return.
- **Observability** — logs (stderr), no metrics, no traces in V1.
- **Testing Requirements** — unit, integration, and (where relevant) safety tests.
- **Validation Commands** — `cargo build`, `cargo test`, `cargo clippy`.
- **Acceptance Criteria** — checkboxes, one per observable condition.
- **Risks** — table with severity and mitigation.

## Architectural Invariants (Apply to Every Spec)

1. **No file writes.** No `fs::write`, `tokio::fs::write`, `OpenOptions::write`, or `fs::rename` anywhere in `src/`. Verified by `tests/architecture/no_write.rs` (phase 2).
2. **No LSP.** No `lsp` crate in `Cargo.toml`, no `use lsp_` in `src/`. Verified by `tests/architecture/no_lsp.rs` (phase 2).
3. **Single workspace root.** Every path input flows through `safety::paths::resolve`. No tool reads a path that has not been resolved.
4. **Bounded output.** Every list- and text-returning tool honors the V1 limits and sets `truncated: true` if capped.
5. **Structured errors.** All tool errors go through `shared::errors::error_payload` with the spec-mandated `code` strings.
