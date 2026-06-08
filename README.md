# ast-mcp

Read-only AST analysis MCP server. Provides 12 `ast_*` tools over stdio JSON-RPC 2.0, backed by Tree-sitter parsers for TypeScript, TSX, JavaScript, JSX, and Python.

## Architecture

```
MCP Client ── stdio ── JSON-RPC 2.0 ── Tool Dispatch ── Safety Layer ── Parser ── Tree-sitter
```

- **Transport**: JSON-RPC 2.0 over stdin/stdout. Notifications are dropped (no response).
- **Safety**: Paths are resolved relative to a configurable workspace root. Traversal (`..`), absolute paths, and symlink escapes are rejected. File size, node count, and text bytes are capped.
- **Parsers**: Tree-sitter 0.20 — each language gets its own grammar. Parsing is synchronous; `parseTimeoutMs` and `queryTimeoutMs` are declared in tool metadata.
- **Positions**: LSP-compatible UTF-16 code unit offsets (`line` + `character`, both 0-based). Byte-to-position conversion uses `O(log n)` binary search on a pre-built line index.

## Supported Languages

| Language | Extensions |
|---|---|
| TypeScript | `.ts` |
| TSX | `.tsx` |
| JavaScript | `.js`, `.mjs`, `.cjs` |
| JSX | `.jsx` |
| Python | `.py` |

## Tools

### Diagnostic

| Tool | Description |
|---|---|
| `ast_health_check` | Workspace path, available parsers, and configured limits |
| `ast_list_supported_languages` | Full language registry with extensions and parser availability |

### Parsing

| Tool | Description |
|---|---|
| `ast_parse_file` | Parse a file — returns root kind, node count, parse time, optional depth-limited syntax tree |
| `ast_query` | Run a Tree-sitter query pattern — returns captures with kind, name, range, and text |
| `ast_file_outline` | Structured outline of a file — classes, functions, methods, imports, exports in a compact, deterministic format |

### Navigation

| Tool | Description |
|---|---|
| `ast_top_level_nodes` | Direct children of the root node with kinds and ranges |
| `ast_enclosing_node` | Smallest node at a position, with filterable ancestor chain (outermost first) |

### Extraction

| Tool | Description |
|---|---|
| `ast_find_imports` | All imports — ES modules (`import`, `import()`, `require`) and Python (`import`, `from … import`) |
| `ast_find_exports` | All exports — TS/JS declarations, defaults, re-exports; Python `__all__` and public defs |
| `ast_find_functions` | Function declarations, methods, constructors, arrows, async functions, lambdas — with parameters and return types |
| `ast_find_classes` | Class definitions with extends, implements, decorators, and nested methods |

### Chunking

| Tool | Description |
|---|---|
| `ast_chunk_file` | Split a file into chunks — four strategies: `top_level`, `function_class`, `semantic_blocks`, `max_lines_with_ast_boundaries` |

## Limits

| Limit | Value |
|---|---|
| `maxFileBytes` | 1 MiB (1,048,576 bytes) |
| `maxNodes` | 500 |
| `maxResults` | 200 |
| `maxTextBytes` | 20,000 bytes |
| `maxChunkLines` | 120 lines |
| `maxChunkBytes` | 30,000 bytes |
| `maxQueryMatches` | 200 |
| `parseTimeoutMs` | 5,000 ms (declared) |
| `queryTimeoutMs` | 5,000 ms (declared) |

Results exceeding these limits are truncated and the response includes `"truncated": true`.

## Usage

### Build from source

```bash
cargo build --release
```

The binary is `target/release/ast-mcp`.

### MCP client configuration

Configure your MCP client to launch the binary with a workspace path:

```json
{
  "mcpServers": {
    "ast": {
      "command": "/path/to/target/release/ast-mcp",
      "env": {
        "WORKSPACE_PATH": "/absolute/path/to/your/project"
      }
    }
  }
}
```

If `WORKSPACE_PATH` is not set, the server uses the current working directory.

### Example tool calls

**Parse a file:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "ast_parse_file",
    "arguments": { "file_path": "src/main.rs", "include_tree": true, "max_depth": 2 }
  }
}
```

**Query function declarations:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "ast_query",
    "arguments": {
      "file_path": "src/lib.rs",
      "query": "(function_declaration name: (identifier) @name) @f"
    }
  }
}
```

**Get enclosing node at a position:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "ast_enclosing_node",
    "arguments": { "file_path": "src/lib.rs", "line": 42, "character": 4 }
  }
}
```

All `file_path` arguments are workspace-relative. Absolute paths and paths containing `..` are rejected.

## Development

```bash
# Run all tests
cargo test

# Lint
cargo clippy --all-targets -- -D warnings

# Format
cargo fmt --check
```

The test suite covers 143 tests across 16 test binaries:
- **Unit tests**: parser behavior, position encoding, extractors, safety guards
- **Integration sweep**: all 12 tools dispatched end-to-end with real fixture files
- **Rejection tests**: path traversal, missing files, unsupported languages, oversized files — verified across all 10 file-path tools
- **Truncation tests**: large tree and query result truncation
- **Fuzz harness**: 500 random byte sequences across 5 parsers with reproducible seeds
- **Architecture lints**: no `unwrap`/`expect` in library code, no file writes, no LSP dependency

## Project Structure

```
src/
├── main.rs              # Entry point — workspace init + transport loop
├── lib.rs               # Crate root
├── config/              # Workspace resolution, default limits
├── mcp/                 # JSON-RPC transport, tool registry, dispatch
├── parser/              # Tree-sitter wrappers, line index, query runner
├── extractors/          # Language-agnostic AST extractors
├── tools/               # One module per tool — validates args, calls extractors
├── languages/           # Language-specific node kinds and helpers
├── shared/              # Shared types: position, errors, language IDs, AST node
└── safety/              # Path resolution, size limits, truncation
```

## Known Limitations

- **Timeout enforcement**: `parseTimeoutMs` and `queryTimeoutMs` are declared in tool metadata but not enforced as hard interrupts. Tree-sitter parses are synchronous.
- **Position encoding**: Exact for ASCII, Latin-1, BMP, and surrogate pairs. Complex grapheme clusters (astral-plane characters + combining marks) may report different UTF-16 widths than user-perceived positions.
- **Chunk IDs**: Chunks are identified by `kind`, `startLine`, and `endLine` rather than a stable `id` field.
