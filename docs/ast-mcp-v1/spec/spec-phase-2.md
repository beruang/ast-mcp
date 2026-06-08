# Spec Phase 2: Workspace Safety

## Phase Goal

Add `WORKSPACE_PATH` loading, path validation, file size limits, and architectural lint tests that enforce "no write" and "no LSP" for the lifetime of the project.

## Dependencies

- Requires: phase-1.
- Produces: `config::workspace`, `safety::paths`, `safety::limits`, `tests/architecture/no_write.rs`, `tests/architecture/no_lsp.rs`, symlink safety test.

## Existing Code References

- Pattern to follow: None (greenfield).
- Related module: `mcp::transport` (phase 1) — never bypasses safety; will use it in phase 4.
- Test pattern: `tests/transport.rs` (phase 1) — extend with a symlink test.

## Technical Approach

Three concerns, three modules:

- `config::workspace` — read env var, validate, expose the absolute root.
- `safety::paths` — normalize input, resolve against workspace, check containment, classify the path (file / dir / missing / symlink / too large).
- `safety::limits` — re-export the V1 default constants from `config::defaults`.

Two architectural lint tests live under `tests/architecture/` and use a path-grep approach so they compile even when the binary is empty.

## File Changes

### New Files

| File | Purpose |
|---|---|
| `src/config/mod.rs` | Public surface of `config` module. |
| `src/config/defaults.rs` | V1 default limit constants. |
| `src/config/workspace.rs` | `Workspace` struct and `Workspace::from_env()`. |
| `src/safety/mod.rs` | Public surface of `safety` module. |
| `src/safety/paths.rs` | Path resolution and containment. |
| `src/safety/limits.rs` | Re-export of limit constants. |
| `src/shared/errors.rs` | `AstToolError` enum and `error_payload()` helper. |
| `tests/architecture/no_write.rs` | Greps `src/` for `fs::write`, `tokio::fs::write`, `OpenOptions::write`, `fs::rename`. Fails on any match. |
| `tests/architecture/no_lsp.rs` | Greps `Cargo.toml` for `lsp` deps; greps `src/` for `use lsp_`. Fails on any match (whitelist: `tree-sitter`). |
| `tests/safety/paths.rs` | Unit tests for the symlink, traversal, and missing-file cases. |

### Modified Files

| File | Change |
|---|---|
| `src/main.rs` | Initialize `Workspace::from_env()` before starting the transport loop; pass it to `register_tools`. |
| `src/mcp/register_tools.rs` | Take a `&Workspace` argument (or store it in a small `AppContext` struct). |

## Implementation Steps

1. `src/shared/errors.rs`:
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum AstToolError {
       #[error("workspace not found: {0}")] WorkspaceNotFound(String),
       #[error("path outside workspace: {0}")] PathOutsideWorkspace(String),
       #[error("file not found: {0}")] FileNotFound(String),
       #[error("file too large: {0} bytes exceeds {1}")] FileTooLarge(u64, u64),
       #[error("unsupported language: {0}")] UnsupportedLanguage(String),
       #[error("parser unavailable: {0}")] ParserUnavailable(String),
       #[error("parse failed: {0}")] ParseFailed(String),
       #[error("syntax error")] SyntaxError,
       #[error("invalid position: {0}")] InvalidPosition(String),
       #[error("invalid range")] InvalidRange,
       #[error("query invalid: {0}")] QueryInvalid(String),
       #[error("query execution failed: {0}")] QueryExecutionFailed(String),
       #[error("result limit exceeded")] ResultLimitExceeded,
       #[error("internal error: {0}")] InternalError(String),
   }
   impl AstToolError {
       pub fn code(&self) -> &'static str { /* map to spec code strings */ }
       pub fn payload(&self) -> serde_json::Value { /* { error: { code, message, details? } } */ }
   }
   ```
2. `src/config/defaults.rs`:
   ```rust
   pub const MAX_FILE_BYTES: u64 = 1_048_576;       // 1 MiB
   pub const MAX_NODES: usize = 500;
   pub const MAX_RESULTS: usize = 200;
   pub const MAX_TEXT_BYTES: usize = 20_000;
   pub const MAX_CHUNK_LINES: usize = 120;
   pub const MAX_CHUNK_BYTES: usize = 30_000;
   pub const MAX_QUERY_MATCHES: usize = 200;
   pub const PARSE_TIMEOUT_MS: u64 = 5_000;
   pub const QUERY_TIMEOUT_MS: u64 = 5_000;
   ```
3. `src/safety/limits.rs` — re-exports `config::defaults::*` under names matching the spec (`maxFileBytes`, `maxNodes`, etc.). The `safety::limits` module is the public surface that tool handlers import.
4. `src/config/workspace.rs`:
   ```rust
   pub struct Workspace { root: PathBuf }
   impl Workspace {
       pub fn from_env() -> Result<Self, AstToolError> { /* read WORKSPACE_PATH or CWD */ }
       pub fn root(&self) -> &Path { &self.root }
   }
   ```
5. `src/safety/paths.rs`:
   ```rust
   pub struct ResolvedFile {
       pub absolute: PathBuf,        // canonical, absolute
       pub workspace_relative: String, // forward-slash, relative to workspace root
   }
   pub fn resolve_file(workspace: &Workspace, input: &str) -> Result<ResolvedFile, AstToolError>;
   pub fn file_size(p: &Path) -> Result<u64, AstToolError>;
   pub fn ensure_under_size(size: u64) -> Result<(), AstToolError>;
   ```
   - Reject if `input` is absolute (when not equal to the workspace root).
   - Reject `..` segments.
   - Reject if `canonicalize` escapes the workspace.
   - Reject if the canonical path is a directory.
   - Reject if the canonical path does not exist.
   - Return `workspace_relative` as a string with forward slashes.
6. `tests/architecture/no_write.rs`:
   - Walk `src/` recursively. For each `.rs` file, read it. Match the regex `(tokio::)?fs::write|OpenOptions::(?:new\([^)]*\))?\.write|\.rename\(`.
   - **Fail** the test if any match is found. The first phase ships zero matches; the test passes.
7. `tests/architecture/no_lsp.rs`:
   - Read `Cargo.toml`. For each `[dependencies]` entry, lowercase the name. Fail if it contains `lsp` but is not in the allowlist `[tree-sitter, tree-sitter-typescript, tree-sitter-javascript, tree-sitter-python, tree-sitter-rust, ...]`.
   - Walk `src/`. For each `.rs` file, read it. Match `use lsp_` (the underscore prevents `lsp` substrings inside identifiers). Fail on any match.
8. `tests/safety/paths.rs`:
   - `resolve_file` on `../outside.ts` → `PathOutsideWorkspace`.
   - `resolve_file` on `/etc/passwd` → `PathOutsideWorkspace`.
   - `resolve_file` on `Cargo.toml` (a directory in the test fixture) → `FileNotFound` (because the canonical path is a directory; or, if it is a file, a separate test asserts the size check). Set up the test fixture so a directory exists at the same name.
   - `resolve_file` on a symlink to `/etc/passwd` → `PathOutsideWorkspace` (canonical path is outside).
   - `resolve_file` on a valid file inside the workspace → success.
   - `ensure_under_size(MAX_FILE_BYTES + 1)` → `FileTooLarge`.

## Data / API / Interface Contract

```rust
// Public surface of safety.
pub fn resolve_file(workspace: &Workspace, input: &str) -> Result<ResolvedFile, AstToolError>;
pub fn ensure_under_size(size: u64) -> Result<(), AstToolError>;

// Public surface of config.
pub struct Workspace { /* private */ }
impl Workspace {
    pub fn from_env() -> Result<Self, AstToolError>;
    pub fn root(&self) -> &Path;
}

// Public surface of shared::errors.
pub enum AstToolError { /* see impl above */ }
impl AstToolError {
    pub fn code(&self) -> &'static str;
    pub fn payload(&self) -> serde_json::Value;
}
```

## Error Handling

This phase introduces the `AstToolError` enum and the `code()` mapping. Tool handlers in later phases will use `?` and `.payload()` to produce structured errors. Codes that may appear in this phase:

- `workspace_not_found`
- `path_outside_workspace`
- `file_not_found`
- `file_too_large`
- `internal_error`

## Observability

- Logs: `tracing::warn!` when a path is rejected (e.g., symlink escape), to **stderr**.
- Metrics: none.
- Traces: none.

## Testing Requirements

### Unit Tests

- `tests/safety/paths.rs` — all of the cases in step 8.
- `tests/safety/limits.rs` — `ensure_under_size` boundary.

### Integration Tests

None new in this phase (the symlink test counts as a unit test because it operates on a tempdir).

### Architectural Tests

- `tests/architecture/no_write.rs` — must pass on day one (zero matches).
- `tests/architecture/no_lsp.rs` — must pass on day one.

## Validation Commands

```bash
cargo build
cargo test
cargo test --test architecture
cargo test --test safety
```

## Acceptance Criteria

- [ ] `Workspace::from_env()` reads `WORKSPACE_PATH` and falls back to CWD.
- [ ] `Workspace::from_env()` returns `WorkspaceNotFound` for a non-existent path.
- [ ] `resolve_file` rejects all 6 cases enumerated in step 8.
- [ ] `resolve_file` returns the workspace-relative path on success (forward slashes).
- [ ] All V1 limit constants exist in `config::defaults` and are re-exported from `safety::limits`.
- [ ] `AstToolError::code()` returns the spec-mandated strings.
- [ ] Both architectural lint tests pass.
- [ ] `cargo clippy --all-targets -- -D warnings` passes.

## Risks

| Risk | Severity | Mitigation |
|---|---|---|
| Symlink race: between canonicalize and read | low | The symlink check happens at validation time, not at read time. The file is read immediately after, on the same task. For V1 we accept the small TOCTOU window. |
| Architectural lint test false-positive on `tree-sitter` containing `lsp` | medium | The allowlist in `no_lsp.rs` whitelists `tree-sitter*`. The substring check is on lowercase crate name and explicitly excludes the whitelist. |
| Architectural lint test false-positive on `OpenOptions::new().read(true)` | low | The regex matches only `OpenOptions::new(...)` calls that include `.write` or `.append` after them. The read-only case is fine. |
