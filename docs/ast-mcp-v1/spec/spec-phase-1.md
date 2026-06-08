# Spec Phase 1: Rust Project Skeleton

## Phase Goal

Stand up a runnable Rust binary that responds to JSON-RPC over stdio for `tools/list` and one dummy `ast_health_check` tool.

## Dependencies

- Requires: None.
- Produces: a binary at `target/debug/ast-mcp` (or `target/release/ast-mcp`) that responds to JSON-RPC.

## Existing Code References

- Pattern to follow: None (greenfield).
- Related module: None.
- Test pattern: None yet.
- Config pattern: None yet.

## Technical Approach

A minimal async binary that reads JSON-RPC requests from stdin, dispatches to a hand-written registry, and writes JSON-RPC responses to stdout. Logging goes to stderr.

### Module layout (this phase)

```text
ast-mcp/
  Cargo.toml
  src/
    main.rs
    mcp/
      mod.rs
      transport.rs
      register_tools.rs
      responses.rs
```

## File Changes

### New Files

| File | Purpose |
|---|---|
| `Cargo.toml` | Crate manifest. Pins serde, serde_json, tokio, anyhow, thiserror. |
| `src/main.rs` | Binary entry. Spawns the transport loop. |
| `src/mcp/mod.rs` | Public surface of the `mcp` module. |
| `src/mcp/transport.rs` | Stdio read loop, dispatch, response writer. |
| `src/mcp/register_tools.rs` | One dummy tool, schema, and dispatch table. |
| `src/mcp/responses.rs` | `content[0].text` envelope helpers. |

### Modified Files

None.

## Implementation Steps

1. Run `cargo new --bin ast-mcp` (or initialize manually). Place the crate at the project root `/Volumes/Workspace/rnd/workflow/mcp/ast/`. Adjust `Cargo.toml` to declare a binary named `ast-mcp`.
2. Add the V1 dependencies (phase-1 set only):
   ```toml
   [dependencies]
   serde = { version = "1", features = ["derive"] }
   serde_json = "1"
   tokio = { version = "1", features = ["full"] }
   anyhow = "1"
   thiserror = "1"
   ```
3. `src/mcp/transport.rs` — async fn `run() -> anyhow::Result<()>`:
   - Loop: read a line from stdin (`tokio::io::AsyncBufReadExt::lines`).
   - Parse the line as `serde_json::Value` (or a `JsonRpcRequest` struct).
   - Match on `method`:
     - `"initialize"` → return `ServerInfo { name: "ast-mcp", version: "0.1.0" }`.
     - `"tools/list"` → return the tools from `register_tools::tools()`.
     - `"tools/call"` → dispatch to the named tool with `arguments`.
     - `"notifications/initialized"` → ignore (no response).
   - On any error, return a JSON-RPC error response with `code: -32603` and the error message.
   - On success, write a single JSON line to stdout, flushed.
4. `src/mcp/responses.rs` — helper:
   ```rust
   pub fn text_envelope(payload: serde_json::Value) -> serde_json::Value {
     json!({ "content": [{ "type": "text", "text": payload.to_string() }] })
   }
   pub fn error_envelope(code: i32, message: &str) -> serde_json::Value { /* ... */ }
   ```
5. `src/mcp/register_tools.rs` — declares one tool:
   ```rust
   pub struct ToolSpec {
     pub name: &'static str,
     pub description: &'static str,
     pub input_schema: serde_json::Value,
     pub handler: fn(serde_json::Value) -> serde_json::Value,
   }
   pub fn tools() -> Vec<ToolSpec> { vec![ /* ast_health_check stub */ ] }
   ```
   The `ast_health_check` handler returns `json!({ "ok": true })` (no parsing yet).
6. `src/main.rs`:
   ```rust
   #[tokio::main]
   async fn main() -> anyhow::Result<()> { mcp::transport::run().await }
   ```
7. Build: `cargo build`. Run: pipe a `tools/list` request to the binary, confirm one JSON line of response on stdout.

## Data / API / Interface Contract

```rust
// Public tool surface (this phase: one tool).
pub fn tools() -> Vec<ToolSpec>;

// ToolSpec
pub struct ToolSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema: serde_json::Value, // JSON Schema
    pub handler: fn(serde_json::Value) -> serde_json::Value,
}
```

`ast_health_check` (this phase, stub):

```jsonc
// input:  { }
// output: { "ok": true }
```

JSON-RPC envelope:

```jsonc
// request:  { "jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": { "name": "ast_health_check", "arguments": {} } }
// response: { "jsonrpc": "2.0", "id": 1, "result": { "content": [{ "type": "text", "text": "{ \"ok\": true }" }] } }
```

## Error Handling

- JSON-RPC parse errors: return `code: -32700` (`ParseError`).
- Method not found: `code: -32601` (`MethodNotFound`).
- Tool not found: `code: -32602` (`InvalidParams`) with message naming the missing tool.
- Internal error: `code: -32603` with the error message (no leaked secrets).

## Observability

- Logs: `eprintln!` and `tracing` macros to **stderr only**. Never to stdout.
- Metrics: none in V1.
- Traces: none in V1.
- Alerts: none in V1.

## Testing Requirements

### Unit Tests

This phase has no tool logic to unit-test. A placeholder test asserts `text_envelope(json!({"ok": true}))` round-trips through serde.

### Integration Tests

A small `tests/transport.rs` that spawns the binary, writes a `tools/list` request, and asserts the response is valid JSON containing the tool name `ast_health_check`.

## Validation Commands

```bash
cargo build
cargo test
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | ./target/debug/ast-mcp
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"ast_health_check","arguments":{}}}' | ./target/debug/ast-mcp
```

Both echo commands must produce a single JSON line on stdout and nothing else.

## Acceptance Criteria

- [ ] `cargo build` produces a binary with no warnings.
- [ ] `tools/list` returns exactly one tool named `ast_health_check`.
- [ ] `tools/call ast_health_check` returns a structured `content[0].text` payload with `ok: true`.
- [ ] All JSON responses are single-line, with no trailing garbage.
- [ ] No code writes to stdout other than the response writer.
- [ ] `cargo clippy --all-targets -- -D warnings` passes.

## Risks

| Risk | Severity | Mitigation |
|---|---|---|
| Stdio framing bug (extra newlines, missing flushes) | medium | Hand-test both `tools/list` and `tools/call` in the validation step. Use `BufWriter` and `flush()` after each response. |
| Logging accidentally goes to stdout | high | Architectural convention: only `mcp::transport::write_response` writes to stdout. Lint test in phase 2 will grep for `println!` in `src/mcp/`. |
| Wrong JSON-RPC framing for `notifications/initialized` | low | Treat as a no-op (no response) per JSON-RPC 2.0. |
