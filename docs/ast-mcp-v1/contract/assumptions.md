# Assumptions

The contract rests on the following assumptions. Any change to these requires reopening the contract.

## A1. Target repository

The implementation lives at the project root `/Volumes/Workspace/rnd/workflow/mcp/ast/`. The spec is at `spec/version-1.md`; the planning artifacts are at `docs/ast-mcp-v1/` and `.agent/contracts/ast-mcp-v1/`. The Rust crate (likely named `ast-mcp` or `mcp-ast`) will be initialized as a sub-crate at the project root, alongside `spec/`, `docs/`, and `.agent/`. No code currently lives in the target directory.

## A2. Tree-sitter versions

The spec recommends `tree-sitter = "0.22"`, `tree-sitter-typescript = "0.20"`, `tree-sitter-javascript = "0.20"`, `tree-sitter-python = "0.20"`. We assume the implementation team verifies these versions exist and resolve at the time of implementation. If the latest versions on crates.io have moved on, the implementation may pin to the closest stable versions and record the choice in `decisions.md`.

## A3. MCP transport

The spec defers the choice of an MCP Rust SDK. We assume the team either adopts an existing SDK (e.g., `mcp-rs`, `rust-mcp`, or the official `rmcp`) or implements a thin JSON-RPC over stdio. The transport is fully behind a `mcp::transport` module, so the choice is reversible.

## A4. Single workspace root

V1 supports one workspace per process. The contract does not contemplate multi-root workspaces, and the `WORKSPACE_PATH` env var supplies the only root. A future V5 may add multi-root.

## A5. UTF-16 conversion is correct for the BMP

We assume that for all V1 input files, the public UTF-16 character offset correctly identifies the column in source text. We do not commit to full non-BMP (astral plane) accuracy in V1; if a non-BMP character appears, the position may be off by one code unit. The implementation must add tests for non-BMP behavior and document any limitation in `docs/ast-mcp-v1/`.

## A6. Tree-sitter is fast enough for V1 files

We assume Tree-sitter parses a 1 MiB file in well under 1 s on commodity hardware, and that the 5-second `parseTimeoutMs` default is a safety net, not a hot path. The implementation will record real parse times during acceptance tests and tune the default if needed.

## A7. Single source of truth for limits

All numeric limits (max nodes, max results, etc.) live in `config/defaults.rs` as constants. Tool handlers read from this module. There is no implicit override mechanism in V1.

## A8. No persistence

The server holds no persistent state across requests. There is no in-memory cache that survives a process restart. (V5 will add caches; V1 must not pre-build them.)

## A9. Test fixtures live in the repo

Unit and integration tests read fixtures from `tests/fixtures/`. A small corpus of representative `.ts`, `.tsx`, `.js`, `.jsx`, and `.py` files is committed, including:

- A normal class with imports and exports.
- A file with a syntax error.
- A file with non-ASCII characters (e.g., emoji in a string literal) to test UTF-16 conversion.
- A file with a `__all__` declaration (Python).
- A file with `require()` (CommonJS).

## A10. CI is not in scope for V1

V1 ships the test suite and the validation commands. CI wiring (GitHub Actions, etc.) is a deployment concern and is deferred. The `cargo test` and `cargo clippy` commands are runnable locally.
