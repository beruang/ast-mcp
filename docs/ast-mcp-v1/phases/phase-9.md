# Phase 9: Functions and Classes

> Spec milestone: `spec/version-1.md` § 33 Milestone 9 + § 21 / § 22

## Goal

Ship `ast_find_functions` and `ast_find_classes` with method, parameter, and return-type extraction. This is the largest extractor in V1 by surface area.

## Dependencies

- Requires: phase-8.
- Produces: 10 of 12 V1 tools.

## Risk

- **medium** — many node kinds to handle (function decl, function expression, arrow function, method, constructor, getter, setter, async, Python lambda, decorated definition).

## Value

Most-used structural tool. "What functions live in this file?" is the agent's most common structural question.

## Implementation Notes

- `src/extractors/functions.rs`:
  - For TS/JS: walk for `function_declaration`, `function_expression`, `arrow_function`, `method_definition`, `generator_function_declaration`. Tag `kind` accordingly.
  - For Python: walk for `function_definition` and `async_function_definition`. Detect lambdas.
  - For each function: extract `name`, `parameters` (each with `name`, `typeText?`, `optional?`, `defaultValueText?`), `returnTypeText?` (TS only — pulled from the `return_type` annotation if present), `async` flag, `exported` flag (parent `export_statement`), and `parentName` for methods.
  - `includeMethods: true` includes class methods; `includeAnonymous: true` includes anonymous function expressions and arrow functions.
- `src/extractors/classes.rs`:
  - For TS/JS: walk for `class_declaration` and `class_expression`. Extract name, `extendsText?` from the `class_heritage`, `implementsText` (TypeScript only), `decoratorsText` (best effort).
  - For Python: walk for `class_definition` (handle decorated_definition wrapper). Extract name, base classes (from `argument_list` of superclass).
  - For each class, list `methods` (constructor, method, getter, setter, async method) with `name` and `range`. Methods are not recursively expanded.
  - `includeMethods: true` includes the methods array; otherwise only the class top-level info is returned.
- `src/tools/find_functions.rs` and `src/tools/find_classes.rs` wire the extractors. Both honor `includeText` and `maxTextBytes`.
- Fixtures: `tests/fixtures/functions/classes.ts` (with extends, implements, decorators, getter, setter), `classes.py` (with base classes and decorated class).

## Validation

```bash
cargo test --test integration ast_find_functions
cargo test --test integration ast_find_classes
```

## Acceptance

- [ ] `ast_find_functions` on a TS file with `function f(a: string): number`, `async function g()`, `const h = () => {}`, and `class C { method(x: string) {} }` returns 4 entries with correct kinds, parameter lists, return types, async flags, and `parentName` on the method.
- [ ] `ast_find_functions` on a Python file with `def f(x: str) -> int`, `async def g()`, and a class with a `def method(self)` returns 3 entries.
- [ ] `ast_find_classes` on a TS file with `class UserService extends Base implements IService` returns one entry with `extendsText: "Base"`, `implementsText: ["IService"]`.
- [ ] `ast_find_classes` on a Python file with `class UserService(Base):` returns one entry with `extendsText: "Base"`.
- [ ] Both tools respect `includeText` and `maxTextBytes` and set `truncated: true` if capped.
