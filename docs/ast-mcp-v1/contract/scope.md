# Scope

## In Scope (V1)

```text
Rust stdio MCP server
12 ast_* tools
5 V1 languages: TypeScript, TSX, JavaScript, JSX, Python
Tree-sitter parsing
WORKSPACE_PATH-based single-root workspace model
path safety, file size limits, bounded output
UTF-16 ↔ byte-offset conversion
language-specific extractors for imports/exports/functions/classes
4 chunking strategies
bounded Tree-sitter query execution
structured JSON responses and errors
unit tests, integration tests, safety tests
```

## Out of Scope (V1)

```text
LSP integration
semantic references
type resolution
compiler diagnostics
hover information
implementation lookup
call hierarchy
type hierarchy
workspace-wide indexing
workspace-wide AST queries
framework-aware route extraction
React component extraction
test extraction
schema/model extraction
AST rewrite previews
file mutation (write, rename, create, delete, format)
patch application
persistent database / state
remote workspace support
multi-root workspace support
runtime configuration (deferred to V5)
caching layer (deferred to V5)
telemetry / request logs (deferred to V5)
```

## Deferred to Later Versions

The V1 design must not block the following, which are explicitly described in the multi-version spec set (`spec/version-2.md` … `spec/version-5.md`):

- **V2** — `ast_enclosing_scope`, `ast_node_at_range`, `ast_node_text`, `ast_context_for_range`, `ast_context_pack`, `ast_find_calls`, `ast_find_member_access`, `ast_find_literals`, `ast_find_template_literals`, `ast_query_workspace`, `ast_file_metrics`.
- **V3** — `ast_find_routes`, `ast_find_react_components`, `ast_find_hooks`, `ast_find_tests`, `ast_find_decorators`, `ast_find_schema_definitions`, `ast_dependency_edges`.
- **V4** — `ast_rewrite_preview`, `ast_insert_import_preview`, `ast_remove_unused_import_preview`, `ast_rename_local_preview`, `ast_wrap_node_preview`, `ast_add_decorator_preview`, `ast_modify_function_signature_preview`, `ast_validate_rewrite`, `ast_parse_after_rewrite`.
- **V5** — `ast_complexity_summary`, `ast_detect_large_nodes`, `ast_detect_duplicate_shapes`, `ast_cache_status`, `ast_clear_caches`, `ast_request_log`, `ast_clear_request_log`, `ast_get_config`, `ast_update_runtime_config`, `ast_readiness`, `ast_liveness`, `ast_workspace_scan_status`, `ast_cancel_workspace_scan`, `ast_parser_status`, `ast_rebuild_parser_cache`.

V1's design — particularly the `extractors/`, `parser/`, and `shared/` modules — must keep these extension points open.

## Composite Workflows (Out of AST MCP)

The following are explicitly **not** part of the AST MCP, even though they may consume AST output:

```text
code_context_pack
code_inspect_symbol
code_prepare_safe_rename
code_analyze_change_impact
code_prepare_diagnostic_fix
```

These belong to Agent Skills, Code Composite MCP, or a client-side orchestration layer.

## Source

`spec/version-1.md` § 5 (V1 Non-Goals), § 2 (Architectural Boundary), § 36 (Future Version Hooks).
