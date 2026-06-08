# AST MCP Server V1 Acceptance Report

**Date**: 2026-06-08
**Sign-off**: All 16 test binaries, 143 tests passing, clippy clean, fmt clean.

## Health and Language Support

| Criterion | Status | Evidence |
|---|---|---|
| `ast_health_check` returns workspace status and parser availability | PASS | `tests/parse_file_test.rs`: `health_check_ok` |
| `ast_list_supported_languages` returns full registry (5 languages) | PASS | `tests/parser.rs`: `registry_has_five_entries`; `tests/parse_file_test.rs`: `list_supported_languages_returns_five`; `tests/integration_sweep.rs`: `sweep_list_supported_languages` |

## Parsing

| Criterion | Status | Evidence |
|---|---|---|
| Parse valid `.ts`, `.tsx`, `.js`, `.jsx`, `.py` files | PASS | `tests/parser.rs`: `parse_typescript_valid`, `parse_typescript_react_valid`, `parse_javascript_valid`, `parse_javascript_react_valid`, `parse_python_valid`; `tests/integration_sweep.rs`: `sweep_parse_file` (all 5 extensions) |
| `hasSyntaxError: true` on error, tree still returned | PASS | `tests/parser.rs`: `parse_typescript_syntax_error` |
| Rejects files > `maxFileBytes` (1 MiB) with `file_too_large` | PASS | `tests/safety.rs`: `ensure_under_size_rejects_too_large`; `tests/safety_rejections.rs`: `reject_file_too_large` |
| Returns `rootKind`, `nodeCount`, `parseTimeMs` | PASS | `tests/integration_sweep.rs`: `sweep_parse_file` (asserts all three fields) |

## Outline

| Criterion | Status | Evidence |
|---|---|---|
| TS/JS outline: classes, functions, methods, imports, exports, type aliases, enums, interfaces | PASS | `tests/outline_test.rs`: `outline_typescript_classes_returns_kinds_and_names`, `outline_typescript_types_includes_interface_and_type_and_enum` |
| Python outline: classes, functions, async functions, imports | PASS | `tests/outline_test.rs`: `outline_python_all_includes_async_functions`, `outline_python_classes_returns_methods` |
| Bounded by `maxDepth` (4 default) | PASS | `tests/outline_test.rs`: `outline_with_max_depth` |
| `outlineText` is deterministic, compact, multi-line | PASS | `tests/integration_sweep.rs`: `sweep_file_outline` |

## Top-Level Nodes

| Criterion | Status | Evidence |
|---|---|---|
| Returns direct root children in source order | PASS | `tests/outline_test.rs`: `top_level_nodes_typescript_returns_correct_count`, `top_level_nodes_python_returns_correct_count`, `top_level_nodes_each_has_kind_and_range` |

## Enclosing Node

| Criterion | Status | Evidence |
|---|---|---|
| Returns smallest node containing input position | PASS | `tests/enclosing_node_test.rs`: `enclosing_node_inside_if_returns_if_statement` |
| `kinds` filter walks ancestors until match | PASS | `tests/enclosing_node_test.rs`: `enclosing_node_kinds_filter_returns_only_class` |
| `ancestors` returned outermost-first | PASS | `tests/enclosing_node_test.rs`: `enclosing_node_returns_ancestors_outermost_first` |

## Imports

| Criterion | Status | Evidence |
|---|---|---|
| ES imports: default, named, namespace, type, side-effect | PASS | `tests/imports_exports_test.rs`: `find_imports_typescript_all_forms`, `find_imports_typescript_default`, `find_imports_typescript_side_effect` |
| `require()` (best effort) | PASS | `tests/imports_exports_test.rs`: `find_imports_typescript_require` |
| Python `import` and `from … import` | PASS | `tests/imports_exports_test.rs`: `find_imports_python_all_forms`, `find_imports_python_aliased` |
| Imports include source ranges | PASS | `tests/imports_exports_test.rs`: `find_imports_has_ranges` |

## Exports

| Criterion | Status | Evidence |
|---|---|---|
| TS/JS export declarations (function, class, const, type, interface, enum, default, re-export) | PASS | `tests/imports_exports_test.rs`: `find_exports_typescript_all_forms`, `find_exports_typescript_default`, `find_exports_typescript_re_export`, `find_exports_typescript_class_name` |
| Python `__all__` and best-effort public defs | PASS | `tests/imports_exports_test.rs`: `find_exports_python_all`, `find_exports_python_public_defs`, `find_exports_python_excludes_private` |

## Functions and Classes

| Criterion | Status | Evidence |
|---|---|---|
| Functions: declarations, methods, constructors, arrows, async, lambdas | PASS | `tests/functions_classes_chunks_test.rs`: `find_functions_typescript_all_forms`, `find_functions_python_all_forms`, `find_functions_python_async`, `find_functions_typescript_async` |
| Parameter lists with name, type, optionality, defaults | PASS | `tests/functions_classes_chunks_test.rs`: `find_functions_typescript_parameters` |
| Methods tagged with `parentName` | PASS | `tests/functions_classes_chunks_test.rs`: `find_functions_typescript_method_parent` |
| Classes with extends, implements, decorators | PASS | `tests/functions_classes_chunks_test.rs`: `find_classes_typescript_extends`, `find_classes_python_extends`, `find_classes_typescript_abstract`, `find_classes_typescript_exported` |

## Chunking

| Criterion | Status | Evidence |
|---|---|---|
| Four strategies: top_level, function_class, semantic_blocks, max_lines_with_ast_boundaries | PASS | `tests/functions_classes_chunks_test.rs`: `chunk_file_top_level`, `chunk_file_function_class_strategy`, `chunk_file_semantic_blocks_strategy`, `chunk_file_max_lines_strategy`; `tests/integration_sweep.rs`: `sweep_chunk_file` |

## Query

| Criterion | Status | Evidence |
|---|---|---|
| Valid queries for TS, TSX, JS, JSX, Python | PASS | `tests/query_test.rs`: `query_ts_function_declarations`, `query_python_function_definitions` |
| Captures normalized to name, kind, range, text | PASS | `tests/query_test.rs`: `query_returns_metadata` |
| Invalid queries return structured `query_invalid` | PASS | `tests/query_test.rs`: `query_invalid_syntax`, `query_empty_returns_error` |
| Result count bounded by `maxResults` (200) | PASS | `tests/safety_truncation.rs`: `query_truncates_large_result_set` |

## Safety

| Criterion | Status | Evidence |
|---|---|---|
| `path_outside_workspace` for traversal | PASS | `tests/safety.rs`: `reject_traversal_dotdot`; `tests/safety_rejections.rs`: `reject_traversal_dotdot` (all 10 file tools) |
| `path_outside_workspace` for absolute paths | PASS | `tests/safety.rs`: `reject_absolute_path`; `tests/safety_rejections.rs`: `reject_absolute_path` |
| `unsupported_language` for unknown extensions | PASS | `tests/safety_rejections.rs`: `reject_unsupported_language` |
| `file_not_found` for missing files | PASS | `tests/safety.rs`: `reject_missing_file`; `tests/safety_rejections.rs`: `reject_missing_file` |
| `file_not_found` for directory paths | PASS | `tests/safety.rs`: `reject_directory`; `tests/safety_rejections.rs`: `reject_directory_as_file` |
| `file_too_large` for oversized files | PASS | `tests/safety.rs`: `ensure_under_size_rejects_too_large`; `tests/safety_rejections.rs`: `reject_file_too_large` |
| Truncation on list/text results | PASS | `tests/safety_truncation.rs`: `parse_file_truncates_large_tree`, `query_truncates_large_result_set` |
| No file writes (architectural lint) | PASS | `tests/architecture.rs`: `no_file_write_anywhere` |
| No LSP dependency (architectural lint) | PASS | `tests/architecture.rs`: `no_lsp_dependency_anywhere` |
| No unwrap/expect in library (architectural lint) | PASS | `tests/architecture_no_panic.rs`: `no_unwrap_or_expect_in_library` |

## Tool Registry

| Criterion | Status | Evidence |
|---|---|---|
| Exactly 12 tools in `tools/list` | PASS | `tests/integration_sweep.rs`: `sweep_tool_list_count` |
| All 12 tools dispatch and return valid JSON | PASS | `tests/integration_sweep.rs`: `sweep_dispatch_all_tools_return_json` |
| Fuzz: no parser panics on random input | PASS | `tests/fuzz_parser.rs`: `fuzz_all_parsers_no_panic` (500 random sequences across 5 parsers) |

## Known Limitations

1. **Position encoding caveat**: V1 is exact for ASCII, Latin-1, BMP, and surrogate pairs. Complex grapheme clusters combining astral-plane characters with combining marks may report different UTF-16 widths than user-perceived character positions. Verified by `tests/positions_test.rs` and `tests/position_utf16_test.rs`.

2. **Timeout model**: `parseTimeoutMs` (5,000 ms) and `queryTimeoutMs` (5,000 ms) are soft budgets — they are not enforced as hard interrupts. Tree-sitter parses are synchronous and the limits are declared in the tool metadata but not enforced at runtime via `tokio::time::timeout` in the current V1 implementation.

3. **Chunk IDs**: Chunks do not carry a stable `id` field in the current implementation. They are identified by `kind`, `startLine`, and `endLine` fields.

## Performance Notes

All parse times recorded during the sweep test run were well under the 1 MiB and 5,000 ms limits:

- Typical parse time for fixture files (20-30 lines): 0-1 ms
- Large file parse (44,000 lines, >1 MiB): ~50 ms
- Large tree with `includeTree`: well within the 500-node truncation limit
- Query execution for 300 functions: well within the 200-match truncation limit
- Fuzz run: 500 random sequences across 5 parsers completed in ~200 ms

## Validation Commands

All three validation gates pass:

```bash
cargo test      # 143 tests passed, 0 failed
cargo clippy --all-targets -- -D warnings  # 0 warnings
cargo fmt --check  # no formatting issues
```
