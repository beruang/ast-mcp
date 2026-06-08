# Constraints

## File Mutation

The AST MCP V1 server **must never write files**. It may read files and return parsed structure. It must not:

- write source files
- apply patches
- rename files
- create files
- delete files
- format files
- execute commands

The "never write files" rule is enforced by a structural test (no `std::fs::write`, no `tokio::fs::write`, no `OpenOptions::write` anywhere in the codebase).

## LSP and Semantic Engines

The AST MCP V1 server **must never** call:

- `lsp_*` tools
- the LSP MCP
- any language server (TypeScript language service, Pyright, gopls, rust-analyzer, etc.)
- any semantic reference engine

It must remain usable when the LSP MCP is unavailable. The "no LSP dependency" rule is enforced by a structural test (no `lsp_*` crate in `Cargo.toml`, no `lsp` module, no subprocess of a language server).

## Workspace Model

- The server runs against exactly one workspace root in V1.
- Workspace root is supplied via `WORKSPACE_PATH=/absolute/path/to/repo` or falls back to the current working directory.
- Every tool input path is normalized, resolved against the workspace, and checked for containment.
- Response paths are **workspace-relative**.
- Rejected inputs include `../outside.ts`, `/etc/passwd`, and `/another-project/file.ts`.

## Position and Encoding

- The public API uses **UTF-16** character positions (the LSP contract), e.g.:

  ```ts
  type Position = { line: number; character: number };
  type Range    = { start: Position; end: Position };
  ```

- Tree-sitter is byte-based. Conversion helpers are required:

  ```text
  UTF-16 position → byte offset
  byte offset     → UTF-16 position
  Tree-sitter point → public Position
  public Range    → byte range
  ```

- Implementation may document limitations for non-BMP characters in V1, but the public contract is UTF-16.

## Default Limits (V1 Constants)

```text
maxFileBytes      = 1 MiB
maxNodes          = 500
maxResults        = 200
maxTextBytes      = 20,000
maxChunkLines     = 120
maxChunkBytes     = 30,000
maxQueryMatches   = 200
parseTimeoutMs    = 5,000
queryTimeoutMs    = 5,000
```

When a result is capped, the response includes:

```json
{ "truncated": true, "returned": 200 }
```

Runtime configuration of these limits is deferred to V5.

## Response Format

- All MCP tool responses return one JSON payload in the `content[0].text` slot.
- Errors are returned as structured JSON with the shape `{ "error": { "code", "message", "details" } }`.
- No prose in tool payloads.
- The public error codes listed in section 10 of the spec are mandatory: `workspace_not_found`, `path_outside_workspace`, `file_not_found`, `file_too_large`, `unsupported_language`, `parser_unavailable`, `parse_failed`, `syntax_error`, `invalid_position`, `invalid_range`, `query_invalid`, `query_execution_failed`, `result_limit_exceeded`, `internal_error`.

## Tech Stack (V1)

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
anyhow = "1"
tokio = { version = "1", features = ["full"] }
tree-sitter = "0.22"
tree-sitter-typescript = "0.20"
tree-sitter-javascript = "0.20"
tree-sitter-python = "0.20"
ignore = "0.4"
globset = "0.4"
walkdir = "2"
uuid = { version = "1", features = ["v4"] }
```

An MCP SDK crate may be added by the implementation team. If no SDK is selected, the server implements MCP stdio JSON-RPC directly behind a small transport abstraction.

## Source

`spec/version-1.md` § 6 (Tech Stack), § 7 (Workspace Model), § 8 (Safety Requirements), § 9 (Position and Range Model), § 27 (Runtime Limits).
