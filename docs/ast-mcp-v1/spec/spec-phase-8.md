# Spec Phase 8: Imports and Exports

## Phase Goal

Ship `ast_find_imports` and `ast_find_exports` for TypeScript, JavaScript, and Python.

## Dependencies

- Requires: phase-7.
- Produces: 8 of 12 V1 tools.

## Existing Code References

- Pattern to follow: `src/tools/file_outline::handle` (phase 6).
- Related modules: `parser::parse::parse_source` (phase 3), `languages::*::outline_candidates` (phase 6).
- Test pattern: `tests/integration/outline.rs` (phase 6) — extend with import/export tests.

## Technical Approach

A `find_imports` extractor that walks the root for `import_statement` (TS/JS) or `import_statement` / `import_from_statement` (Python), plus a `find_exports` extractor that walks for `export_statement` (TS/JS) or `__all__` and public top-level defs (Python).

## File Changes

### New Files

| File | Purpose |
|---|---|
| `src/extractors/imports.rs` | `find_imports` extractor and `AstImport` type. |
| `src/extractors/exports.rs` | `find_exports` extractor and `AstExport` type. |
| `src/tools/find_imports.rs` | `ast_find_imports` handler. |
| `src/tools/find_exports.rs` | `ast_find_exports` handler. |
| `tests/fixtures/imports/all_forms.ts` | All 5 ES import forms. |
| `tests/fixtures/imports/require.js` | CommonJS require. |
| `tests/fixtures/imports/dynamic.ts` | `await import(...)`. |
| `tests/fixtures/imports/all_forms.py` | All Python import forms. |
| `tests/fixtures/exports/all_forms.ts` | All TS export forms. |
| `tests/fixtures/exports/public.py` | Python `__all__` and public defs. |
| `tests/integration/imports_exports.rs` | End-to-end tests. |

### Modified Files

| File | Change |
|---|---|
| `src/mcp/register_tools.rs` | Register the 2 new tools. |

## Implementation Steps

1. `src/extractors/imports.rs`:
   ```rust
   pub struct AstImport {
       pub source: Option<String>,
       pub kind: ImportKind,           // "import" | "from_import" | "require" | "dynamic_import" | "unknown"
       pub default_import: Option<String>,
       pub namespace_import: Option<String>,
       pub named_imports: Vec<String>,
       pub aliases: Vec<Alias>,        // { imported, local }
       pub is_type_only: Option<bool>,
       pub range: Range,
       pub text: String,
   }
   pub fn find_imports(tree: &Tree, source: &str, lang: LanguageId) -> Vec<AstImport>;
   ```
   - For TS/JS: walk for `import_statement`. For each:
     - `source` = the first `string` child.
     - `default_import` = the first identifier before `{` or `*`.
     - `namespace_import` = `* as foo`.
     - `named_imports` = the list inside `import_clause` → `named_imports` → each `import_specifier` → `name`.
     - `aliases` = the `import_specifier → "as" → name` chains.
     - `is_type_only` = the `import_statement` has a `type` keyword.
   - For CommonJS `require`: walk for `lexical_declaration` whose initializer is a `call_expression` with callee name `require`. Extract the first string argument as `source`, mark `kind: "require"`. Best effort; nested or non-string `require` calls are ignored.
   - For dynamic `import`: walk for `await_expression` whose argument is a `call_expression` with callee name `import`. Mark `kind: "dynamic_import"`.
   - For Python: walk for `import_statement` (`import x`, `import x.y`, `import x as y`) and `import_from_statement` (`from x import y`, `from x import y as z`). Mark `kind: "import"` or `"from_import"`. Extract the module path and the bound names.
2. `src/extractors/exports.rs`:
   ```rust
   pub struct AstExport {
       pub kind: ExportKind,           // "function" | "class" | "const" | ... | "default" | "re_export" | "python_public_definition" | "python_all" | "unknown"
       pub name: Option<String>,
       pub source: Option<String>,     // for re-exports
       pub is_default: Option<bool>,
       pub is_type_only: Option<bool>,
       pub range: Range,
       pub text: String,
   }
   pub fn find_exports(tree: &Tree, source: &str, lang: LanguageId, include_best_effort_python: bool) -> Vec<AstExport>;
   ```
   - For TS/JS: walk for `export_statement`. For each:
     - `export function f` → `kind: "function"`, `name: "f"`.
     - `export class C` → `kind: "class"`, `name: "C"`.
     - `export const x = ...` → `kind: "const"`, `name: "x"`.
     - `export type T = ...` → `kind: "type"`, `is_type_only: true`.
     - `export interface I` → `kind: "interface"`.
     - `export default ...` → `kind: "default"`, `is_default: true`.
     - `export { a, b as c }` → for each, emit one entry with `kind: "const"` and the right name.
     - `export * from "mod"` → `kind: "re_export"`, `source: "mod"`.
   - For Python: if `include_best_effort_python: true`:
     - Find `assignment` whose LHS is `__all__`. Emit one entry with `kind: "python_all"`, `name: "__all__"`, and the array text.
     - Walk top-level definitions whose name does not start with `_`. Emit one `python_public_definition` per definition.
3. Tool handlers are thin wrappers; they enforce `MAX_RESULTS` (200) and set `truncated: true` if hit.
4. Tests assert exact entries for each fixture.

## Data / API / Interface Contract

```rust
// extractors::imports
pub enum ImportKind { Import, FromImport, Require, DynamicImport, Unknown }
pub struct Alias { pub imported: String, pub local: String }
pub struct AstImport { pub source: Option<String>, pub kind: ImportKind, pub default_import: Option<String>, pub namespace_import: Option<String>, pub named_imports: Vec<String>, pub aliases: Vec<Alias>, pub is_type_only: Option<bool>, pub range: Range, pub text: String }
pub fn find_imports(tree: &Tree, source: &str, lang: LanguageId) -> Vec<AstImport>;
// extractors::exports
pub enum ExportKind { Function, Class, Const, Let, Var, Type, Interface, Enum, ReExport, Default, PythonPublicDefinition, PythonAll, Unknown }
pub struct AstExport { pub kind: ExportKind, pub name: Option<String>, pub source: Option<String>, pub is_default: Option<bool>, pub is_type_only: Option<bool>, pub range: Range, pub text: String }
pub fn find_exports(tree: &Tree, source: &str, lang: LanguageId, include_best_effort_python: bool) -> Vec<AstExport>;
```

Tool response shapes match spec § 19 / § 20.

## Error Handling

- `file_not_found`, `file_too_large`, `unsupported_language` — propagated.
- `result_limit_exceeded` not raised; results are silently truncated with `truncated: true`.

## Observability

- Logs: `tracing::debug!` with file path, language, count. Stderr.

## Testing Requirements

### Unit Tests

- `find_imports` for each TS/JS/Python form is asserted to extract the right `source`, `kind`, and bindings.
- `find_exports` for each TS/JS/Python form is asserted to extract the right `kind`, `name`, and `source`.

### Integration Tests

- `tests/integration/imports_exports.rs`:
  - `ast_find_imports all_forms.ts` → 5 entries (default, named, namespace, type-only, side-effect).
  - `ast_find_imports require.js` → 1 entry with `kind: "require"`.
  - `ast_find_imports dynamic.ts` → 1 entry with `kind: "dynamic_import"`.
  - `ast_find_imports all_forms.py` → 4 entries.
  - `ast_find_exports all_forms.ts` → 8 entries.
  - `ast_find_exports public.py` → 1 `python_all` + 2 `python_public_definition`.

## Validation Commands

```bash
cargo test --lib extractors::imports
cargo test --lib extractors::exports
cargo test --test integration imports_exports
```

## Acceptance Criteria

- [ ] `ast_find_imports` on `all_forms.ts` returns 5 entries with correct `source`, `defaultImport`, `namespaceImport`, `namedImports`, `aliases`, and `isTypeOnly`.
- [ ] `ast_find_imports` on `require.js` returns 1 entry with `kind: "require"`.
- [ ] `ast_find_imports` on `dynamic.ts` returns 1 entry with `kind: "dynamic_import"`.
- [ ] `ast_find_imports` on `all_forms.py` returns 4 entries.
- [ ] `ast_find_exports` on `all_forms.ts` returns 8 entries.
- [ ] `ast_find_exports` on `public.py` returns 1 `python_all` + 2 `python_public_definition`.
- [ ] Both tools honor `MAX_RESULTS` (200) and set `truncated: true` if hit.
- [ ] `cargo clippy --all-targets -- -D warnings` passes.

## Risks

| Risk | Severity | Mitigation |
|---|---|---|
| Python `from x import y as z` alias mis-extracted | low | Unit test covers the case explicitly. |
| `export { a, b as c }` reports `b` as the name instead of `c` | low | The `aliases` field captures the local name; the entry's `name` is the local name (`c`). |
| Best-effort Python public definition flags a private name | low | The filter is "name does not start with `_`" and is exactly what the spec describes. |
| `require` inside an arrow function or a non-`const` declaration is missed | low | Best-effort; documented in the tool description. |
