# MCP AST — Code Intelligence Server

[![Rust](https://img.shields.io/badge/rust-1.96%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-162%20passed-brightgreen.svg)](.)

Production-grade structural code intelligence over MCP. **54 tools** across parsing, extraction, context selection, framework detection, structural rewrites, complexity analysis, and operational observability. Backed by Tree-sitter parsers for TypeScript, TSX, JavaScript, JSX, Python, Go, and Rust.

---

## Architecture

```
                            ┌─────────────────────────────────────┐
                            │           MCP Client (LLM)          │
                            └──────────────┬──────────────────────┘
                                           │  JSON-RPC 2.0
                                           │  stdin / stdout
                                           ▼
┌──────────────────────────────────────────────────────────────────────────────┐
│                              ast-mcp Server                                  │
│                                                                              │
│   ┌──────────┐    ┌───────────────┐    ┌─────────────┐    ┌──────────────┐  │
│   │ Transport │───▶│  ServerContext │───▶│  Dispatch   │───▶│ Safety Layer │  │
│   │ tower-lsp │    │  · workspace   │    │  54 tools   │    │ · path bound │  │
│   │   stdio   │    │  · config      │    │  registered  │    │ · size caps  │  │
│   │           │    │  · caches      │    │              │    │ · truncation │  │
│   └──────────┘    │  · req tracker  │    └─────────────┘    └──────┬───────┘  │
│                   │  · scan registry│                              │          │
│                   └───────────────┘                                ▼          │
│                                                           ┌──────────────┐  │
│                                                           │  Tree-sitter  │  │
│                                                           │   0.22 × 7    │  │
│                                                           │ TS/JS/Py/Go/Rs│  │
│                                                           └──────────────┘  │
└──────────────────────────────────────────────────────────────────────────────┘
```

| Layer | Role |
|---|---|
| **Transport** | JSON-RPC 2.0 over stdin/stdout via `tower-lsp`. Notifications are dropped. |
| **ServerContext** | Shared state: workspace resolver, layered runtime config, TTL caches, request log ring buffer, scan registry with cancellation. |
| **Dispatch** | 54 registered tools. Each handler validates input, enforces limits, and delegates to the appropriate engine. |
| **Safety** | Workspace-bounded paths — `..`, absolute paths, and symlink escapes rejected. File size, node count, and text bytes capped. Truncation flagged in responses. |
| **Tree-sitter** | 7 language grammars at 0.22. Byte offsets → LSP positions via `O(log n)` binary search on pre-built line index. |

Positions are LSP-compatible: zero-based `line` + UTF-16 `character` offset.

---

## Supported Languages

| Language | Extensions | Parser Crate |
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

### Diagnostics & Health

| Tool | Description |
|---|---|
| `ast_health_check` | Workspace path, available parsers, configured limits |
| `ast_list_supported_languages` | Language registry — extensions, parser name, availability |
| `ast_readiness` | Readiness probe — workspace, parser registry, cache initialization, config validity |
| `ast_liveness` | Liveness probe — uptime, memory usage. No parsing or scanning. |
| `ast_parser_status` | Parser registry health — language, version, query count, last error |
| `ast_rebuild_parser_cache` | Clear and reinitialize parser/query caches per language |

### Parsing

| Tool | Description |
|---|---|
| `ast_parse_file` | Parse a file — root kind, node count, parse time, optional depth-limited tree |
| `ast_query` | Run a Tree-sitter query pattern — captures with kind, name, range, text |
| `ast_file_outline` | Structured outline — classes, functions, methods, imports, exports |
| `ast_file_metrics` | Structural metrics — lines, nodes, functions, nesting depth |

### Navigation

| Tool | Description |
|---|---|
| `ast_top_level_nodes` | Direct children of the root node with kinds and ranges |
| `ast_enclosing_node` | Smallest node at a position, filterable ancestor chain (outermost first) |
| `ast_enclosing_scope` | Syntactic scope chain at a position — module, class, function, block |
| `ast_node_at_range` | Smallest node that exactly matches or contains a source range |
| `ast_node_text` | Exact source text for a range — byte-budgeted, no whole-file read |

### Context Selection

| Tool | Description |
|---|---|
| `ast_context_for_range` | Syntax context around a range — target node, parent chain, optional siblings |
| `ast_context_pack` | Compact agent-ready context pack — imports, exports, scope, outline, nearby symbols |

### Extraction

| Tool | Description |
|---|---|
| `ast_find_imports` | All imports — ES modules, `require()`, Python `import`/`from` |
| `ast_find_exports` | All exports — TS/JS declarations, re-exports, Python `__all__`, public defs |
| `ast_find_functions` | Functions, methods, constructors, arrows, lambdas — params, return types |
| `ast_find_classes` | Class definitions — extends, implements, decorators, methods |
| `ast_find_calls` | Call expressions — filter by callee name or substring |
| `ast_find_member_access` | Member/property access expressions — filter by property or object |
| `ast_find_literals` | String, number, boolean, null, regex literals — filter by kind or value |
| `ast_find_template_literals` | Template literals — filter by tag name (e.g. `sql`, `gql`) or content |
| `ast_dependency_edges` | Syntax-level dependency edges — imports, exports, requires across files |

### Structural Chunking

| Tool | Description |
|---|---|
| `ast_chunk_file` | Split a file into chunks — 4 strategies: `top_level`, `function_class`, `semantic_blocks`, `max_lines` |

### Framework-Aware Detection

| Tool | Description |
|---|---|
| `ast_find_schema_definitions` | Zod schemas, TypeScript interfaces, Pydantic models, dataclasses, SQLAlchemy, Go/Rust structs |
| `ast_find_react_components` | Function, arrow, and class components — props, hooks used, JSX root |
| `ast_find_hooks` | Built-in and custom React hooks — usages and definitions |
| `ast_find_routes` | Express, Fastify, Hono, Next.js, NestJS, FastAPI, Flask, Django, Go `net/http`, Rust Axum |
| `ast_find_decorators` | TypeScript decorators, Python decorators, Rust `#[attributes]` — with target attachment |
| `ast_find_tests` | Jest, Vitest, Mocha, Pytest, unittest, Go tests, Rust `#[test]` — suite/test/fixture/hook |
| `ast_query_workspace` | Bounded Tree-sitter query across workspace files — glob, limits, parallelism |

### Structural Rewrites (Preview-Only)

| Tool | Description |
|---|---|
| `ast_rewrite_preview` | Preview structural rewrites — replace, insert before/after, delete. Returns diff + validation. |
| `ast_validate_rewrite` | Validate rewrite operations — safety, overlap, limits. No diff generated. |
| `ast_parse_after_rewrite` | Apply edits in memory, re-parse, report syntax errors. |
| `ast_insert_import_preview` | Preview adding an import — TS/JS/Python. Merges with existing imports. |
| `ast_remove_unused_import_preview` | Preview removing syntactically unused imports. Side-effect imports preserved. |
| `ast_rename_local_preview` | Preview renaming a local variable within its scope. Exported/top-level rejected. |
| `ast_wrap_node_preview` | Preview wrapping a node — prefix/suffix, try/catch, or call expression. |
| `ast_add_decorator_preview` | Preview adding a decorator/attribute — TS/JS/Python/Rust. |
| `ast_modify_function_signature_preview` | Preview modifying function signature — add/remove/rename params or replace. |

### Structural Analysis

| Tool | Description |
|---|---|
| `ast_complexity_summary` | Per-file and workspace-wide complexity — branch/loop count, nesting depth, hotspot ranking with risk heuristics |
| `ast_detect_large_nodes` | Find oversized functions, classes, modules, tests — configurable `min_lines` threshold |
| `ast_detect_duplicate_shapes` | Heuristic clone detection — structural fingerprinting with identifier/literal normalization |

### Operations & Observability

| Tool | Description |
|---|---|
| `ast_get_config` | Return effective runtime config — optional source breakdown (defaults, env, overrides) |
| `ast_update_runtime_config` | Update limits, timeouts, cache TTLs, scan parallelism at runtime. Immutable fields rejected. |
| `ast_request_log` | Recent request history — filter by tool name, status, file path. Ring buffer. |
| `ast_clear_request_log` | Clear request log entries — optionally filtered by tool. |
| `ast_cache_status` | Cache sizes, TTLs, estimated memory per cache — parse trees, queries, framework results, request log |
| `ast_clear_caches` | Clear caches selectively. Clearing parse trees cascades to dependent query/framework caches. |
| `ast_workspace_scan_status` | Active and recent workspace scan progress — files discovered, processed, results found |
| `ast_cancel_workspace_scan` | Cooperatively cancel a running workspace scan by ID. |

---

## Runtime Configuration

Layered, in-memory configuration. Precedence:

```
defaults  →  AST_* environment variables  →  ast_update_runtime_config overrides
```

### Environment Variables

| Variable | Default | Description |
|---|---|---|
| `WORKSPACE_PATH` | CWD | Workspace root directory |
| `AST_MAX_FILE_BYTES` | `1048576` | Max file size (1 MiB) |
| `AST_MAX_WORKSPACE_FILES` | `500` | Max files per workspace scan |
| `AST_MAX_WORKSPACE_RESULTS` | `5000` | Max results per workspace scan |
| `AST_MAX_CONTEXT_CHARACTERS` | `20000` | Max context characters in responses |
| `AST_MAX_PARALLELISM` | `8` | Parallel workers for workspace scans |
| `AST_RESPECT_GITIGNORE` | `true` | Respect `.gitignore` during file discovery |
| `AST_INCLUDE_HIDDEN` | `false` | Include hidden files in scans |
| `AST_PARSE_TREE_TTL_MS` | `300000` | Parse tree cache TTL (5 min) |
| `AST_REQUEST_LOG_MAX_ENTRIES` | `500` | Request log ring buffer capacity |
| `AST_MAX_CACHED_FILES` | `1000` | Max cached parse trees |
| `AST_VERBOSE_LOGGING` | `false` | Enable verbose tracing output |

Runtime updates via `ast_update_runtime_config` are in-memory only. `workspace_path`, parser registry, and language grammar paths are immutable at runtime.

---

## Limits

| Limit | Default | Configurable Via |
|---|---|---|
| `maxFileBytes` | 1 MiB | `AST_MAX_FILE_BYTES` / runtime update |
| `maxParseTreeNodes` | 200,000 | Runtime update |
| `maxQueryResults` | 1,000 | Runtime update |
| `maxWorkspaceFiles` | 500 | `AST_MAX_WORKSPACE_FILES` / runtime update |
| `maxWorkspaceResults` | 5,000 | `AST_MAX_WORKSPACE_RESULTS` / runtime update |
| `maxContextCharacters` | 20,000 | `AST_MAX_CONTEXT_CHARACTERS` / runtime update |
| `maxChunkLines` | 160 | Runtime update |
| `maxChangedFiles` | 100 | Runtime update |
| `maxEdits` | 1,000 | Runtime update |
| `maxDuplicateCandidates` | 200 | Runtime update |
| `maxParallelism` | 8 | `AST_MAX_PARALLELISM` / runtime update |

Results exceeding limits are truncated. Responses include `"truncated": true`.

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

**Complexity analysis before a refactor:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "ast_complexity_summary",
    "arguments": { "glob": "src/**/*.ts", "max_files": 200, "max_results": 100 }
  }
}
```

**Adjust scan parallelism at runtime:**

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

**Inspect cache health:**

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

**Check readiness before a workspace scan:**

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "ast_readiness",
    "arguments": { "require_languages": ["typescript", "python"] }
  }
}
```

---

## Development

```bash
cargo build --release                          # Release build
cargo test                                     # All tests across 19 binaries
cargo clippy --all-targets -- -D warnings       # Strict lint
cargo fmt --check                              # Format check
```

### Test Suite

| Suite | Tests | Coverage |
|---|---|---|
| Unit (lib) | 29 | Parser, positions, fingerprints, request log, rewrite engine |
| Integration sweep | 14 | All V1 tools end-to-end against fixture files |
| V3 framework | 45 | Route, schema, React, decorator, test, dependency extractors |
| V4 rewrites | 12 | Rewrite preview, import insert, rename, wrap, decorator, signature |
| Per-tool | ~50 | Enclosing nodes, functions/classes, imports/exports, outline, parse, query, positions |
| Safety & rejection | 8 | Path traversal, missing files, unsupported languages, oversize truncation |
| Architecture lints | 3 | No `unwrap`/`expect` in library code, no file writes, no LSP dependency |
| Fuzz | 2 | 500 random byte sequences across 5 parsers with reproducible seeds |
| **Total** | **~162** | |

---

## Project Structure

```
src/
├── main.rs                  # Entry point — workspace + transport loop
├── lib.rs                   # Crate root
├── config/                  # Workspace resolver, runtime config, env parsing, validation
├── mcp/                     # JSON-RPC transport, tool registry, ServerContext
├── parser/                  # Tree-sitter wrappers (0.22), line index, query engine
├── languages/               # Per-language node kinds — TS, JS, Python, Go, Rust
├── shared/                  # Position, Range, AST node, errors, V2–V5 type schemas
├── safety/                  # Path resolution, range validation, size limits, truncation
├── text/                    # Position encoding, UTF-16 conversion, indentation, byte budget
├── workspace/               # File scanner, glob matching, .gitignore rules
│
├── extractors/              # V1 — AST extractors (outline, imports, exports, functions, classes)
├── context/                 # V2 — Scope chain, node-at-range, context pack
├── extraction/              # V2 — Calls, literals, member access, template literals
├── metrics/                 # V2 — File/function metrics, nesting depth
├── frameworks/              # V3 — Routes, React, schemas, decorators, tests, dependencies
├── rewrite/                 # V4 — Diff engine, edit overlap, parse-after-rewrite
├── rewrite_tools/           # V4 — Import merge, rename, wrap, decorator, signature
├── analysis/                # V5 — Complexity summary, large node detection, duplicate shapes
├── cache/                   # V5 — TTL caches (parse tree, query, framework results)
├── observability/           # V5 — Request log ring buffer, request tracker
├── ops/                     # V5 — Readiness, liveness, parser status, cache rebuild
├── scan/                    # V5 — Scan registry, cooperative cancellation
│
└── tools/                   # 54 tool handlers — one module per tool
```

---

## Architectural Boundaries

MCP AST is **structural only**. It does not own semantic compiler truth.

| May Do | Must Not Do |
|---|---|
| Parse files and walk syntax trees | Call LSP MCP |
| Run Tree-sitter queries | Resolve semantic references |
| Extract imports, exports, functions, classes, routes, tests, decorators | Infer full type information |
| Chunk files structurally | Perform semantic rename |
| Preview structural rewrites (read-only, in-memory) | Apply patches to disk |
| Compute structural metrics and complexity | Execute shell commands |
| Cache parse trees, query results, framework extractions | Run tests or typecheck |
| Scan workspaces with bounded parallelism and cancellation | |

For semantic analysis, use LSP MCP. For cross-service orchestration, use Agent Skills.

---

## Error Handling

All tools return structured errors:

```json
{
  "error": {
    "code": "path_outside_workspace",
    "message": "path outside workspace: ../outside.ts"
  }
}
```

V5 adds 16 error codes for cache operations (`cache_unavailable`, `cache_clear_failed`), config validation (`invalid_runtime_config`, `config_update_rejected`), scan lifecycle (`workspace_scan_not_found`, `scan_timeout`), analysis failures (`complexity_analysis_failed`, `duplicate_shape_detection_failed`), and health checks (`readiness_check_failed`, `parser_rebuild_failed`).

---

## Known Limitations

- **Timeout enforcement**: Declared in metadata and enforced at request level; Tree-sitter parse operations are synchronous (no hard interrupt mid-parse).
- **Position encoding**: Exact for ASCII, Latin-1, BMP, and surrogate pairs. Complex grapheme clusters may diverge from user-perceived positions.
- **Duplicate detection**: Heuristic structural fingerprinting — not a full clone-detection engine. Normalization may produce false positives.
- **Complexity metrics**: Node-kind counting heuristics — not compiler-grade cyclomatic complexity. Risk classifications (`low`/`medium`/`high`) are advisory.
