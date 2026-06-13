# ast-mcp

[![Rust](https://img.shields.io/badge/rust-1.96%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-162%20passed-brightgreen.svg)](.)

Production-grade structural AST analysis MCP server. Exposes **54 `ast_*` tools** over stdio JSON-RPC 2.0, backed by Tree-sitter parsers for TypeScript, TSX, JavaScript, JSX, Python, Go, and Rust.

---

## Architecture

```
MCP Client ── stdio ── JSON-RPC 2.0 ── ServerContext ── Tool Dispatch ── Safety Layer ── Tree-sitter
```

| Layer | Responsibility |
|---|---|
| **Transport** | JSON-RPC 2.0 over stdin/stdout via tower-lsp. Notifications are dropped. |
| **ServerContext** | Shared state: workspace, config store, caches, request tracker, scan registry. |
| **Safety** | Workspace-bounded paths. `..`, absolute paths, and symlink escapes rejected. File size, node count, and text bytes capped. |
| **Parsers** | Tree-sitter 0.22 — each language gets its own grammar. Byte-to-LSP-position conversion via `O(log n)` binary search on a line index. |
| **Positions** | LSP-compatible: zero-based `line` + UTF-16 `character` offset. |

---

## Supported Languages

| Language | Extensions | Parser |
|---|---|---|
| TypeScript | `.ts` | `tree-sitter-typescript` |
| TSX | `.tsx` | `tree-sitter-typescript` (tsx) |
| JavaScript | `.js`, `.mjs`, `.cjs` | `tree-sitter-javascript` |
| JSX | `.jsx` | `tree-sitter-javascript` |
| Python | `.py` | `tree-sitter-python` |
| Go | `.go` | `tree-sitter-go` |
| Rust | `.rs` | `tree-sitter-rust` |

---

## Tools

### V1 — Core Parsing & Extraction (12 tools)

| Tool | Description |
|---|---|
| `ast_health_check` | Workspace path, available parsers, configured limits |
| `ast_list_supported_languages` | Language registry with extensions and availability |
| `ast_parse_file` | Parse a file — root kind, node count, parse time, optional tree |
| `ast_file_outline` | Structured outline — classes, functions, imports, exports |
| `ast_top_level_nodes` | Direct children of the root node with kinds and ranges |
| `ast_enclosing_node` | Smallest node at a position, with filterable ancestor chain |
| `ast_find_imports` | All imports — ES modules, `require()`, Python `import`/`from` |
| `ast_find_exports` | All exports — TS/JS declarations, Python `__all__`, public defs |
| `ast_find_functions` | Functions, methods, arrows, lambdas — with parameters and return types |
| `ast_find_classes` | Classes with extends, implements, decorators, methods |
| `ast_chunk_file` | Split a file into chunks — 4 strategies |
| `ast_query` | Run a Tree-sitter query pattern — captures with kind, name, range, text |

### V2 — Context & Workspace (11 tools)

| Tool | Description |
|---|---|
| `ast_enclosing_scope` | Syntactic scope chain at a position |
| `ast_node_at_range` | Smallest node matching or containing a range |
| `ast_node_text` | Exact source text for a range (byte-budgeted) |
| `ast_context_for_range` | Syntax context around a range — target, parents, siblings |
| `ast_context_pack` | Compact agent-ready structural context pack |
| `ast_find_calls` | Call expressions — filter by callee name |
| `ast_find_member_access` | Member/property access expressions |
| `ast_find_literals` | String, number, boolean, null, regex literals |
| `ast_find_template_literals` | Template literals — filter by tag or content |
| `ast_query_workspace` | Bounded Tree-sitter query across workspace files |
| `ast_file_metrics` | Structural metrics: lines, nodes, functions, nesting depth |

### V3 — Framework-Aware Extraction (7 tools)

| Tool | Description |
|---|---|
| `ast_find_schema_definitions` | Zod, TypeScript interfaces, Pydantic, dataclasses, SQLAlchemy, Go/Rust structs |
| `ast_find_react_components` | Function, arrow, and class components — hooks, JSX summary |
| `ast_find_hooks` | Built-in and custom React hooks — usages and definitions |
| `ast_find_routes` | Express, Fastify, Hono, Next.js, NestJS, FastAPI, Flask, Django, Go, Axum |
| `ast_find_decorators` | TypeScript, Python, and Rust decorators/annotations/attributes |
| `ast_find_tests` | Jest, Vitest, Mocha, Pytest, unittest, Go, Rust tests |
| `ast_dependency_edges` | Syntax-level dependency edges — imports, exports, requires |

### V4 — Structural Rewrite Preview (9 tools)

| Tool | Description |
|---|---|
| `ast_rewrite_preview` | Preview structural rewrites: replace, insert, delete — diff + validation |
| `ast_validate_rewrite` | Validate edits without generating a diff |
| `ast_parse_after_rewrite` | Apply edits in memory, re-parse, check for syntax errors |
| `ast_insert_import_preview` | Preview adding an import — TS/JS/Python |
| `ast_remove_unused_import_preview` | Preview removing syntactically unused imports |
| `ast_rename_local_preview` | Preview renaming a local variable within its scope |
| `ast_wrap_node_preview` | Preview wrapping a node — prefix/suffix, try/catch, call expression |
| `ast_add_decorator_preview` | Preview adding a decorator/attribute — TS/JS/Python/Rust |
| `ast_modify_function_signature_preview` | Preview modifying a function signature |

### V5 — Production Hardening (15 tools)

| Tool | Description |
|---|---|
| `ast_get_config` | Return effective runtime configuration with source breakdown |
| `ast_update_runtime_config` | Update limits, timeouts, cache TTLs, scan settings at runtime |
| `ast_request_log` | Recent request history — filter by tool, status, file path |
| `ast_clear_request_log` | Clear request log entries |
| `ast_cache_status` | Parse tree, query, and framework cache sizes, TTLs, memory |
| `ast_clear_caches` | Clear caches selectively — cascading dependency awareness |
| `ast_readiness` | Readiness probe — workspace, parsers, caches, config |
| `ast_liveness` | Liveness probe — uptime, memory (no parsing, no scanning) |
| `ast_parser_status` | Parser registry health — language, version, query count, errors |
| `ast_rebuild_parser_cache` | Clear and rebuild parser/query caches per language |
| `ast_workspace_scan_status` | Active and recent workspace scan progress |
| `ast_cancel_workspace_scan` | Cancel a running workspace scan cooperatively |
| `ast_complexity_summary` | Structural complexity — branch/loop count, nesting depth, hotspots |
| `ast_detect_large_nodes` | Find oversized functions, classes, modules across the workspace |
| `ast_detect_duplicate_shapes` | Heuristic clone detection — structural fingerprinting with normalization |

---

## Runtime Configuration

V5 introduces layered, in-memory runtime configuration. Precedence:

```
defaults → AST_* environment variables → ast_update_runtime_config overrides
```

### Environment Variables

| Variable | Default | Description |
|---|---|---|
| `WORKSPACE_PATH` | CWD | Workspace root directory |
| `AST_MAX_FILE_BYTES` | `1048576` | Max file size (1 MiB) |
| `AST_MAX_WORKSPACE_FILES` | `500` | Max files per workspace scan |
| `AST_MAX_WORKSPACE_RESULTS` | `5000` | Max results per workspace scan |
| `AST_MAX_CONTEXT_CHARACTERS` | `20000` | Max context characters |
| `AST_MAX_PARALLELISM` | `8` | Parallel workers for workspace scans |
| `AST_RESPECT_GITIGNORE` | `true` | Respect `.gitignore` during scans |
| `AST_INCLUDE_HIDDEN` | `false` | Include hidden files in scans |
| `AST_PARSE_TREE_TTL_MS` | `300000` | Parse tree cache TTL (5 min) |
| `AST_REQUEST_LOG_MAX_ENTRIES` | `500` | Request log ring buffer capacity |
| `AST_MAX_CACHED_FILES` | `1000` | Max cached parse trees |
| `AST_VERBOSE_LOGGING` | `false` | Enable verbose tracing |

Runtime config updates via `ast_update_runtime_config` are in-memory only and cannot change `workspace_path`, parser registry, or language grammar paths.

---

## Limits

| Limit | Default | Configurable |
|---|---|---|
| `maxFileBytes` | 1 MiB | `AST_MAX_FILE_BYTES` / runtime |
| `maxParseTreeNodes` | 200,000 | runtime |
| `maxQueryResults` | 1,000 | runtime |
| `maxWorkspaceFiles` | 500 | `AST_MAX_WORKSPACE_FILES` / runtime |
| `maxWorkspaceResults` | 5,000 | `AST_MAX_WORKSPACE_RESULTS` / runtime |
| `maxContextCharacters` | 20,000 | `AST_MAX_CONTEXT_CHARACTERS` / runtime |
| `maxChunkLines` | 160 | runtime |
| `maxChangedFiles` | 100 | runtime |
| `maxEdits` | 1,000 | runtime |
| `maxDuplicateCandidates` | 200 | runtime |
| `maxParallelism` | 8 | `AST_MAX_PARALLELISM` / runtime |

Results exceeding limits are truncated; responses include `"truncated": true`.

---

## Usage

### Build

```bash
cargo build --release
# Binary: target/release/ast-mcp
```

### MCP Client Configuration

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

If `WORKSPACE_PATH` is not set, the server uses the current working directory. All `file_path` arguments are workspace-relative — absolute paths and `..` are rejected.

### Example Tool Calls

**Complexity summary:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "ast_complexity_summary",
    "arguments": { "glob": "src/**/*.ts", "max_files": 200 }
  }
}
```

**Runtime config update:**

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "ast_update_runtime_config",
    "arguments": {
      "limits": { "max_workspace_files": 1000 },
      "scans": { "max_parallelism": 12 }
    }
  }
}
```

**Check cache health:**

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "ast_cache_status",
    "arguments": {}
  }
}
```

---

## Development

```bash
cargo build --release              # Release build
cargo test                         # All 162 tests across 19 binaries
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

### Test Suite

| Suite | Tests | Description |
|---|---|---|
| Unit (lib) | 29 | Parser, positions, fingerprints, request log, rewrite engine |
| Integration sweep | 14 | All V1 tools end-to-end with fixture files |
| V3 | 45 | Framework extractors across TypeScript, Python, Go, Rust |
| V4 | 12 | Rewrite preview tools — safety, insert, modify, rename |
| Per-tool tests | ~50 | Enclosing nodes, functions/classes, imports/exports, outline, parse, query |
| Safety & rejection | 8 | Path traversal, missing files, unsupported languages, truncation |
| Architecture lints | 3 | No `unwrap` in library, no file writes, no LSP dependency |
| Fuzz | 2 | 500 random byte sequences across 5 parsers |
| **Total** | **~162** | |

---

## Project Structure

```
src/
├── main.rs                  # Entry point — workspace + transport loop
├── lib.rs                   # Crate root
├── config/                  # Workspace, runtime config, env parsing, validation
├── mcp/                     # JSON-RPC transport, tool registry, ServerContext
├── parser/                  # Tree-sitter wrappers, line index, query runner
├── extractors/              # Language-agnostic AST extractors (V1)
├── context/                 # Scope chain, node-at-range, context pack (V2)
├── extraction/              # Calls, literals, member access, templates (V2)
├── metrics/                 # File and function metrics, nesting depth (V2)
├── frameworks/              # Routes, React, schemas, decorators, tests, deps (V3)
├── rewrite/                 # Rewrite engine — diff, overlap, parse-after (V4)
├── rewrite_tools/           # Import merge, rename, wrap, decorator, signature (V4)
├── analysis/                # Complexity, large nodes, duplicate shapes (V5)
├── cache/                   # TTL caches — parse tree, query, framework results (V5)
├── observability/           # Request log ring buffer, request tracker (V5)
├── ops/                     # Readiness, liveness, parser status, rebuild (V5)
├── scan/                    # Scan registry, cooperative cancellation (V5)
├── tools/                   # One module per tool — 54 handlers total
├── languages/               # Language-specific node kinds (TS, JS, Python, Go, Rust)
├── safety/                  # Path resolution, range validation, limits, violations
├── text/                    # Position encoding, UTF-16, indentation, byte budget
├── workspace/               # File scanner, glob matching, ignore rules
└── shared/                  # Position, Range, errors, AST node, V2–V5 types
```

---

## Architectural Boundaries

AST MCP is **structural only**. It does not own semantic compiler truth.

| May Do | Must Not Do |
|---|---|
| Parse files and walk syntax trees | Call LSP MCP |
| Run Tree-sitter queries | Resolve semantic references |
| Extract imports, exports, functions, classes, routes, tests, decorators | Infer full type information |
| Chunk files structurally | Perform semantic rename |
| Preview structural rewrites (read-only) | Apply patches to disk |
| Compute structural metrics and complexity | Execute shell commands |
| Cache parse trees, query results, framework extractions | Run tests or typecheck |
| Scan workspaces with bounded parallelism | |

For semantic analysis, use LSP MCP. For cross-service workflows, use Agent Skills.

---

## Error Handling

All tools return structured errors with a `code` and `message`. Example:

```json
{
  "error": {
    "code": "path_outside_workspace",
    "message": "path outside workspace: ../outside.ts"
  }
}
```

V5 adds 16 new error codes for cache operations, config validation, scan lifecycle, and analysis failures.

---

## Known Limitations

- **Timeout enforcement**: Timeouts are declared in tool metadata and enforced at the request level; Tree-sitter parse operations are synchronous.
- **Position encoding**: Exact for ASCII, Latin-1, BMP, and surrogate pairs. Complex grapheme clusters may report different UTF-16 widths than user-perceived positions.
- **Duplicate detection**: Heuristic structural fingerprinting only — not a full clone-detection engine. Normalization may produce false positives.
- **Complexity metrics**: Structural heuristics based on node kind counting — not compiler-grade cyclomatic complexity. Risk classifications are advisory.
