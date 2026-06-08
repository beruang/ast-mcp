# Phase 8: Imports and Exports

> Spec milestone: `spec/version-1.md` § 33 Milestone 8 + § 19 / § 20

## Goal

Ship `ast_find_imports` and `ast_find_exports` for TypeScript, JavaScript, and Python. The TS/JS extractors share a code path; Python gets its own.

## Dependencies

- Requires: phase-7.
- Produces: 8 of 12 V1 tools.

## Risk

- **medium** — many edge cases (default imports, namespace imports, `import type`, `require`, dynamic `import`, Python `as` aliases, `__all__`).

## Value

Used by every "what does this file depend on / expose?" workflow.

## Implementation Notes

- `src/extractors/imports.rs`:
  - For TS/JS: walk root for `import_statement`. Extract `source` (string literal), `default`, `namespace`, named imports, aliases, `import type` flag. Handle `lexical_declaration` with `require(...)` (best effort). Handle `await import(...)` (best effort).
  - For Python: walk root for `import_statement` and `import_from_statement`. Extract module and bound names.
- `src/extractors/exports.rs`:
  - For TS/JS: walk root for `export_statement` (function, class, const, let, var, type, interface, enum, default, re-export `export { a, b as c }`, `export * from "mod"`).
  - For Python: if `includeBestEffortPythonExports: true`, look for `__all__` assignment and any top-level definition whose name does not start with `_`.
- `src/tools/find_imports.rs` and `src/tools/find_exports.rs` are thin wrappers.
- Fixtures: `tests/fixtures/imports/imports.ts`, `imports.py`, `exports.ts`, `exports.py`.

## Validation

```bash
cargo test --test integration ast_find_imports
cargo test --test integration ast_find_exports
```

Each fixture is parsed and the response is asserted to contain every expected import/export.

## Acceptance

- [ ] `ast_find_imports` on a TS file with all 5 ES import forms returns 5 `AstImport` entries with correct `source`, `defaultImport`, `namespaceImport`, `namedImports`, and `aliases`.
- [ ] `ast_find_imports` on a TS file with `const x = require("mod")` returns one entry with `kind: "require"`.
- [ ] `ast_find_imports` on a Python file with `import os`, `import numpy as np`, `from pathlib import Path`, and `from package.module import A, B as C` returns 4 entries.
- [ ] `ast_find_exports` on a TS file with `export function f`, `export class C`, `export const x = 1`, `export type T`, `export interface I`, `export default function`, `export { a, b as c }`, and `export * from "mod"` returns 8 entries.
- [ ] `ast_find_exports` on a Python file with `__all__ = ["User", "get_user"]` and two top-level public definitions returns at least 3 entries (one `python_all` + two `python_public_definition`).
