# Phase 1: Rust Project Skeleton

> Spec milestone: `spec/version-1.md` § 33 Milestone 1

## Goal

Stand up a minimal Rust crate that runs as an MCP stdio server, accepts JSON-RPC requests, and replies with one dummy tool. This proves the toolchain, the stdio transport, and the JSON response shape before any AST logic lands.

## Dependencies

- Requires: None.
- Produces: a runnable binary `ast-mcp` that responds to `tools/list` and `tools/call` for one placeholder tool.

## Risk

- **medium** — toolchain and transport are well understood, but a wrong stdio framing (e.g., extra newlines, missing flushes) will fail the rest of the project. The first commit is the riskiest.

## Value

A green `cargo run` that responds to JSON-RPC. Without this, no other phase can be tested end-to-end.

## Implementation Notes

- `cargo new --bin ast-mcp` at the project root.
- `Cargo.toml` pins:
  - `serde = { version = "1", features = ["derive"] }`
  - `serde_json = "1"`
  - `tokio = { version = "1", features = ["full"] }`
  - `anyhow = "1"`
  - `thiserror = "1"`
- `src/main.rs` reads stdin line-by-line, parses each line as a JSON-RPC request, dispatches to a registry, and writes one JSON line to stdout.
- `src/mcp/transport.rs` owns the stdio loop. `src/mcp/register_tools.rs` declares the tool list. `src/mcp/responses.rs` shapes the `content[0].text` envelope.
- One dummy tool, e.g., `ast_health_check`, returns a stub `{ "ok": true }` JSON. It will be replaced in phase 4.
- Logging goes to **stderr** (MCP convention); never to stdout.

## Validation

```bash
cargo build
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | ./target/debug/ast-mcp
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"ast_health_check","arguments":{}}}' | ./target/debug/ast-mcp
```

Both commands must produce a single-line JSON response on stdout. The second must return `content[0].text` containing a valid JSON object with `"ok": true`.

## Acceptance

- [ ] `cargo build` produces a binary `target/debug/ast-mcp` with no warnings.
- [ ] `tools/list` returns exactly one tool named `ast_health_check`.
- [ ] `tools/call ast_health_check` returns a structured `content[0].text` payload.
- [ ] All JSON responses are single-line, with no trailing garbage.
- [ ] No code writes to stdout other than the response writer.
