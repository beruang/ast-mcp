# Phase 2: Workspace Safety

> Spec milestone: `spec/version-1.md` § 33 Milestone 2 + § 8 (Safety Requirements)

## Goal

Lock down path validation, file size limits, and the architectural invariants ("no write", "no LSP") before any tool reads a real file.

## Dependencies

- Requires: phase-1.
- Produces: `safety::paths` and `safety::limits` modules plus two architectural lint tests.

## Risk

- **critical** — every later phase depends on this. A containment bug here leaks `/etc/passwd`; a missing lint test lets a future refactor reintroduce writes.

## Value

The foundation that lets every other tool be safely run against an arbitrary workspace. The architectural lint tests are the lowest-cost insurance in the project.

## Implementation Notes

- `src/config/workspace.rs` reads `WORKSPACE_PATH` env var, falls back to CWD, and validates it is an existing directory. Exposes `Workspace::root()` returning an absolute `PathBuf`.
- `src/safety/paths.rs`:
  - `resolve(workspace: &Workspace, input: &str) -> Result<WorkspaceRelativePath, AstToolError>`.
  - Reject `..` traversal. Reject absolute paths. Reject symlinks pointing outside the workspace (via `canonicalize` + recheck).
  - Reject directories where a file is required.
  - Reject missing files (return `file_not_found`).
  - Reject unsupported extensions (return `unsupported_language`).
- `src/safety/limits.rs` exposes the V1 default constants from `config/defaults.rs`:
  - `MAX_FILE_BYTES = 1 MiB`
  - `MAX_NODES = 500`
  - `MAX_RESULTS = 200`
  - `MAX_TEXT_BYTES = 20,000`
  - `MAX_CHUNK_LINES = 120`
  - `MAX_CHUNK_BYTES = 30,000`
  - `MAX_QUERY_MATCHES = 200`
  - `PARSE_TIMEOUT_MS = 5,000`
  - `QUERY_TIMEOUT_MS = 5,000`
- `tests/architecture/no_write.rs`: a test that walks `src/`, greps for `fs::write`, `tokio::fs::write`, `OpenOptions::new().write`, and `fs::rename`, and **fails** if any match is found. The test must compile even when the binary is empty (use a `path` glob, not a Rust symbol import).
- `tests/architecture/no_lsp.rs`: a test that reads `Cargo.toml` and fails if any dependency name contains `lsp` (case-insensitive) **except** the legitimate use of `tree-sitter` (which contains the substring). A second part of the test greps `src/` for `use lsp_` and fails on any match.
- Symlink safety test: a unit test in `safety::tests` that creates a symlink inside the workspace pointing to `/etc/passwd` and asserts `resolve()` rejects it.

## Validation

```bash
cargo test --test architecture
cargo test --lib safety
```

The architectural tests must pass on day one. They are the canary.

## Acceptance

- [ ] `Workspace::from_env()` resolves the workspace and exposes the absolute root.
- [ ] `safety::paths::resolve` rejects `../outside.ts`, `/etc/passwd`, `/another-project/file.ts`, a directory path, a missing file, and a symlink escaping the workspace.
- [ ] `safety::paths::resolve` returns workspace-relative paths on success.
- [ ] All V1 limit constants exist in `safety::limits` and are imported by `config/defaults.rs`.
- [ ] `tests/architecture/no_write.rs` passes (no `fs::write` etc. in `src/`).
- [ ] `tests/architecture/no_lsp.rs` passes (no `lsp` dep in `Cargo.toml`, no `use lsp_` in `src/`).
- [ ] A safety unit test for symlink rejection passes.
