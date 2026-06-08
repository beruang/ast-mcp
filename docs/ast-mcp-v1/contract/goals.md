# Goals

## Primary Goals (V1)

The following 12 MCP tools must be implemented and registered:

| # | Tool | Purpose |
|---:|---|---|
| 1 | `ast_health_check` | Report workspace validity, parser availability, and configured limits. |
| 2 | `ast_list_supported_languages` | Return the parser registry and extension routing table. |
| 3 | `ast_parse_file` | Parse a file and return parse status (with optional bounded tree). |
| 4 | `ast_file_outline` | Return a syntax-based structural outline. |
| 5 | `ast_top_level_nodes` | Return direct root children in source order. |
| 6 | `ast_enclosing_node` | Return the smallest syntax node at a position, with optional ancestors and kind filter. |
| 7 | `ast_find_imports` | Extract import statements (ES, CommonJS, dynamic, Python). |
| 8 | `ast_find_exports` | Extract export declarations and best-effort Python public surface. |
| 9 | `ast_find_functions` | Extract functions, methods, arrow functions, async functions, lambdas. |
| 10 | `ast_find_classes` | Extract class declarations with methods, superclasses, interfaces. |
| 11 | `ast_chunk_file` | Chunk a file by syntax structure across four strategies. |
| 12 | `ast_query` | Run a bounded Tree-sitter query with capture normalization. |

## Language Goals (V1)

Parsers must be wired and routed for:

- TypeScript (`.ts`)
- TSX (`.tsx`)
- JavaScript (`.js`, `.mjs`, `.cjs`)
- JSX (`.jsx`)
- Python (`.py`)

The parser registry must be designed so that Go, Rust, Java, and C/C++ can be added later **without changing the public tool contracts**.

## Engineering Goals

- **Single Rust binary** communicating via MCP over stdio.
- **No file mutation.** No tool writes, renames, creates, deletes, or formats files.
- **No LSP dependency.** No `lsp_*` calls, no language server processes, no semantic engines.
- **Workspace safety.** Every input path is normalized and validated against the workspace root.
- **Bounded output.** All list- and text-returning tools respect default limits and return `truncated` flags.
- **UTF-16 public positions.** Conversion helpers bridge Tree-sitter byte positions to LSP-style UTF-16 positions.

## Source

`spec/version-1.md` § 4 (V1 Goals), § 37 (Final V1 Tool Surface), § 6 (Tech Stack), § 8 (Safety Requirements).
