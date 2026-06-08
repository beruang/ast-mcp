# Spec Phase 9: Functions and Classes

## Phase Goal

Ship `ast_find_functions` and `ast_find_classes` with method, parameter, and return-type extraction. This is the largest extractor in V1 by surface area.

## Dependencies

- Requires: phase-8.
- Produces: 10 of 12 V1 tools.

## Existing Code References

- Pattern to follow: `src/extractors/imports::find_imports` (phase 8).
- Related modules: `parser::parse::parse_source` (phase 3), `languages::*::outline_candidates` (phase 6).
- Test pattern: `tests/integration/imports_exports.rs` (phase 8) — extend with function/class tests.

## Technical Approach

Two extractors. `find_functions` walks for function-like nodes per language; `find_classes` walks for class nodes and (optionally) their methods. The function parameter extractor reads `formal_parameters` (TS/JS) or `parameters` (Python) and pulls each parameter's name, type annotation, and default value.

## File Changes

### New Files

| File | Purpose |
|---|---|
| `src/extractors/functions.rs` | `find_functions` extractor and `AstFunction` type. |
| `src/extractors/classes.rs` | `find_classes` extractor and `AstClass` type. |
| `src/tools/find_functions.rs` | `ast_find_functions` handler. |
| `src/tools/find_classes.rs` | `ast_find_classes` handler. |
| `tests/fixtures/functions/full.ts` | All TS function/method/arrow shapes. |
| `tests/fixtures/functions/full.py` | All Python function shapes. |
| `tests/fixtures/classes/full.ts` | TS class with extends, implements, decorators, getter/setter. |
| `tests/fixtures/classes/full.py` | Python class with base classes, decorator. |
| `tests/integration/functions_classes.rs` | End-to-end tests. |

### Modified Files

| File | Change |
|---|---|
| `src/mcp/register_tools.rs` | Register the 2 new tools. |

## Implementation Steps

1. `src/extractors/functions.rs`:
   ```rust
   pub enum FunctionKind { Function, Method, Constructor, ArrowFunction, FunctionExpression, AsyncFunction, Lambda, Unknown }
   pub struct AstParameter { pub name: Option<String>, pub type_text: Option<String>, pub optional: Option<bool>, pub default_value_text: Option<String> }
   pub struct AstFunction {
       pub name: Option<String>,
       pub kind: FunctionKind,
       pub async_: Option<bool>,
       pub exported: Option<bool>,
       pub parameters: Vec<AstParameter>,
       pub return_type_text: Option<String>,
       pub range: Range,
       pub body_range: Option<Range>,
       pub text: Option<String>,
       pub parent_name: Option<String>,
   }
   pub fn find_functions(tree: &Tree, source: &str, lang: LanguageId, opts: FunctionOptions) -> Vec<AstFunction>;
   ```
2. TS/JS extraction:
   - `function_declaration` / `generator_function_declaration` → `Function` (or `Function` + `async_: true` if prefixed with `async`).
   - `class { method_definition }` → `Method`. Read `parent_name` from the enclosing `class_declaration`'s `name` field. Read `Constructor` for `method_definition` whose `name` is `"constructor"`. Read `Getter` / `Setter` from the `method_definition` kind modifiers (TS supports them as separate field values).
   - `lexical_declaration` whose initializer is an `arrow_function` or `function_expression` → `ArrowFunction` / `FunctionExpression`. `name` is the LHS identifier.
   - For each: walk `formal_parameters` to extract parameters. Each `required_parameter` / `optional_parameter` has a `pattern` (the name) and a `type_annotation` (the type text). `assignment_pattern` adds `default_value_text`.
   - `return_type_text` comes from the `return_type` field on `function_declaration` or `arrow_function`.
3. Python extraction:
   - `function_definition` → `Function`. Detect `async` from the `async` keyword child.
   - `class { function_definition }` → `Method`, `parent_name` from the class.
   - For each: walk `parameters` to extract name, type annotation (from `type` field), and default value.
   - `return_type_text` from the `return` field on the `function_definition` (the `-> T` annotation).
   - Lambdas (`lambda`): best-effort. V1 returns them with `kind: Lambda` and `name: None`.
   - `decorated_definition` is unwrapped and the inner node is processed.
4. `src/extractors/classes.rs`:
   ```rust
   pub enum ClassMethodKind { Method, Constructor, Getter, Setter, Unknown }
   pub struct AstClassMethod { pub name: Option<String>, pub kind: ClassMethodKind, pub range: Range }
   pub struct AstClass {
       pub name: String,
       pub exported: Option<bool>,
       pub extends_text: Option<String>,
       pub implements_text: Vec<String>,
       pub decorators_text: Vec<String>,
       pub methods: Vec<AstClassMethod>,
       pub range: Range,
       pub body_range: Option<Range>,
       pub text: Option<String>,
   }
   pub fn find_classes(tree: &Tree, source: &str, lang: LanguageId, opts: ClassOptions) -> Vec<AstClass>;
   ```
5. TS/JS extraction:
   - `class_declaration` → name from `name` field, `extends_text` from `class_heritage`'s `extends` + `extends_type` (or `extends_clause` depending on TS version), `implements_text` from the `implements_clause` (TS only — list of `type_identifier`s), `decorators_text` from each preceding `decorator` (best effort).
   - Methods: walk the `class_body` for `method_definition`. Tag with `Method`, `Constructor` (name == "constructor"), `Getter` / `Setter` (per method kind field).
6. Python extraction:
   - `class_definition` → name from `name` field, `extends_text` from the `superclasses` argument list (joined with `,`), `decorators_text` from preceding `decorator` nodes.
   - Methods: walk the `block` for `function_definition`. Tag with `Method`.
7. Tool handlers are thin wrappers. `includeMethods: true` (default) populates the `methods` array; `includeText: true` (default false) populates `text`.

## Data / API / Interface Contract

```rust
// extractors::functions
pub enum FunctionKind { Function, Method, Constructor, ArrowFunction, FunctionExpression, AsyncFunction, Lambda, Unknown }
pub struct AstParameter { pub name: Option<String>, pub type_text: Option<String>, pub optional: Option<bool>, pub default_value_text: Option<String> }
pub struct AstFunction { pub name: Option<String>, pub kind: FunctionKind, pub async_: Option<bool>, pub exported: Option<bool>, pub parameters: Vec<AstParameter>, pub return_type_text: Option<String>, pub range: Range, pub body_range: Option<Range>, pub text: Option<String>, pub parent_name: Option<String> }
pub struct FunctionOptions { pub include_methods: bool, pub include_anonymous: bool }
pub fn find_functions(tree: &Tree, source: &str, lang: LanguageId, opts: FunctionOptions) -> Vec<AstFunction>;
// extractors::classes
pub enum ClassMethodKind { Method, Constructor, Getter, Setter, Unknown }
pub struct AstClassMethod { pub name: Option<String>, pub kind: ClassMethodKind, pub range: Range }
pub struct AstClass { pub name: String, pub exported: Option<bool>, pub extends_text: Option<String>, pub implements_text: Vec<String>, pub decorators_text: Vec<String>, pub methods: Vec<AstClassMethod>, pub range: Range, pub body_range: Option<Range>, pub text: Option<String> }
pub struct ClassOptions { pub include_methods: bool }
pub fn find_classes(tree: &Tree, source: &str, lang: LanguageId, opts: ClassOptions) -> Vec<AstClass>;
```

Tool response shapes match spec § 21 / § 22.

## Error Handling

- `file_not_found`, `file_too_large`, `unsupported_language` — propagated.
- `result_limit_exceeded` not raised; results are silently truncated.

## Observability

- Logs: `tracing::debug!` with file path, language, function count, class count. Stderr.

## Testing Requirements

### Unit Tests

- TS function parameter extraction: required, optional, default, typed, untyped.
- TS method parent_name is the enclosing class.
- Python async detection.
- Python `decorated_definition` unwraps.
- TS class `implements` list.

### Integration Tests

- `tests/integration/functions_classes.rs`:
  - `ast_find_functions full.ts` → 4 entries (function, async, arrow, method).
  - `ast_find_functions full.py` → 3 entries (function, async, method).
  - `ast_find_classes full.ts` → 1 entry with `extendsText: "Base"`, `implementsText: ["IService"]`, 1 method + 1 constructor + 1 getter + 1 setter.
  - `ast_find_classes full.py` → 1 entry with `extendsText: "Base"`, 1 method.

## Validation Commands

```bash
cargo test --lib extractors::functions
cargo test --lib extractors::classes
cargo test --test integration functions_classes
```

## Acceptance Criteria

- [ ] `ast_find_functions` on `full.ts` returns 4 entries with correct kinds, parameter lists, return types, async flags, and `parentName` on the method.
- [ ] `ast_find_functions` on `full.py` returns 3 entries.
- [ ] `ast_find_classes` on `full.ts` returns 1 entry with `extendsText: "Base"` and `implementsText: ["IService"]`.
- [ ] `ast_find_classes` on `full.py` returns 1 entry with `extendsText: "Base"`.
- [ ] Both tools respect `includeText` and `maxTextBytes`.
- [ ] `cargo clippy --all-targets -- -D warnings` passes.

## Risks

| Risk | Severity | Mitigation |
|---|---|---|
| Parameter `type_text` reads too much (e.g., includes comments) | low | Slice from `node.start_byte()` to `node.end_byte()` of the type annotation node only. |
| Python `decorated_definition` is double-counted | medium | The extractor unwraps `decorated_definition` and applies rules to the inner node. The fixture and the unit test pin this. |
| `class_heritage` shape varies between TS versions | medium | Pin TS parser version; if a shape change breaks the test, update the extractor and record the change in `decisions.md`. |
| Overloaded constructors: only one is reported as `Constructor` | low | The extractor reports the method_definition whose name is `"constructor"`; overloading in TS doesn't change the name. |
