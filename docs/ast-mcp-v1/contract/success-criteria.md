# Success Criteria

V1 is acceptable when **every** item below is observably true.

## Health and Language Support

- [ ] `ast_health_check` returns workspace status and parser availability for all 5 V1 languages.
- [ ] `ast_list_supported_languages` returns the full registry (TypeScript, TSX, JavaScript, JSX, Python) with extension routing and parser crate name.

## Parsing

- [ ] `ast_parse_file` successfully parses valid `.ts`, `.tsx`, `.js`, `.jsx`, and `.py` files.
- [ ] `ast_parse_file` reports `hasSyntaxError: true` when Tree-sitter detects errors and continues to return a tree.
- [ ] `ast_parse_file` rejects files larger than `maxFileBytes` (1 MiB default) with `file_too_large`.
- [ ] `ast_parse_file` returns `rootKind`, `nodeCount`, and `parseTimeMs`.

## Outline

- [ ] `ast_file_outline` returns classes, functions, methods, imports, exports, type aliases, enums, and interfaces for TypeScript/JavaScript.
- [ ] `ast_file_outline` returns classes, functions, async functions, and imports for Python.
- [ ] Outline output is bounded by `maxNodes` (500 default) and `maxDepth` (4 default).
- [ ] `outlineText` is a deterministic, compact, multi-line string.

## Top-Level Nodes

- [ ] `ast_top_level_nodes` returns direct root children in source order.
- [ ] Text is omitted by default; included only when `includeText: true`, and truncated to `maxTextBytes` (20,000 default).

## Enclosing Node

- [ ] `ast_enclosing_node` returns the smallest node whose range contains the input position.
- [ ] `kinds` filter walks ancestors until a matching kind is found.
- [ ] `ancestors` is returned outermost-first when `includeAncestors: true`.
- [ ] Text is bounded and optional.

## Imports

- [ ] `ast_find_imports` detects ES imports (`import x from "mod"`, `import { a, b as c }`, `import * as ns`, `import type { T }`).
- [ ] `ast_find_imports` detects `const x = require("mod")` (best effort).
- [ ] `ast_find_imports` detects `await import("mod")` (best effort).
- [ ] `ast_find_imports` detects Python `import` and `from … import` statements.

## Exports

- [ ] `ast_find_exports` detects TypeScript/JavaScript `export` declarations (function, class, const, type, interface, enum, default, re-export).
- [ ] `ast_find_exports` detects Python `__all__` and best-effort public top-level definitions.

## Functions and Classes

- [ ] `ast_find_functions` returns functions, methods, constructors, arrow functions, function expressions, async functions, and Python lambdas.
- [ ] Parameter lists include name, type text (where structurally present), optionality, and default value text.
- [ ] Return type text is included when structurally available.
- [ ] Methods are tagged with `parentName` when the enclosing class is detectable.
- [ ] `ast_find_classes` returns classes with name, exported flag, superclass/extends text, implements list, and decorators text.

## Chunking

- [ ] `ast_chunk_file` supports all four strategies: `top_level`, `function_class`, `semantic_blocks`, `max_lines_with_ast_boundaries`.
- [ ] Each chunk has a stable `id` in the form `{filePath}:{kind}:{name}:{startLine}-{endLine}` (or nearest equivalent when `name` is absent).
- [ ] Chunks respect `maxChunkLines` (120) and `maxChunkBytes` (30,000) and are split on AST boundaries when oversized.
- [ ] The import block is prepended when `includeImports: true`.

## Query

- [ ] `ast_query` compiles and runs valid Tree-sitter queries for TypeScript, TSX, JavaScript, JSX, and Python.
- [ ] Captures are normalized to `name`, `kind`, `range`, and optional `text`.
- [ ] Invalid queries return a structured `query_invalid` error with `language` in `details`.
- [ ] Result count is bounded by `maxResults` (200 default).

## Safety

- [ ] Paths outside the workspace return `path_outside_workspace`.
- [ ] Unsupported extensions return `unsupported_language`.
- [ ] Missing files return `file_not_found`.
- [ ] Directory paths where a file is required are rejected.
- [ ] Files exceeding `maxFileBytes` return `file_too_large`.
- [ ] All list- and text-returning tools honor limits and set `truncated: true` when capping.
- [ ] No tool ever writes, renames, creates, deletes, or formats files (verified by an architectural lint test).
- [ ] No tool imports or references an LSP crate, language server, or semantic engine (verified by an architectural lint test).

## Source

`spec/version-1.md` § 34 (Acceptance Criteria), § 35 (Testing Requirements).
