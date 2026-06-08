# Spec Phase 6: Outline and Top-Level Nodes

## Phase Goal

Ship `ast_file_outline` and `ast_top_level_nodes`. Both lean on language-specific extractors and prove the extractor pattern that phases 7–10 will reuse.

## Dependencies

- Requires: phase-5 (positions).
- Produces: 5 of 12 V1 tools.

## Existing Code References

- Pattern to follow: `src/tools/parse_file::handle` (phase 4) — every tool follows the same shape.
- Related modules: `parser::parse::parse_source` (phase 3), `parser::positions` (phase 5), `safety::paths::resolve_file` (phase 2).
- Test pattern: `tests/integration/parse_file.rs` (phase 4) — extend with outline tests.

## Technical Approach

A `TopLevelNode` extractor that walks the root's named children. An `OutlineExtractor` that walks the tree to `maxDepth` and yields structural nodes. Language modules export a single `outline_candidates(root, source) -> Vec<OutlineCandidate>` helper; the extractors are language-agnostic.

## File Changes

### New Files

| File | Purpose |
|---|---|
| `src/extractors/mod.rs` | Public surface. |
| `src/extractors/top_level.rs` | `top_level_nodes` extractor. |
| `src/extractors/outline.rs` | `file_outline` extractor. |
| `src/tools/file_outline.rs` | `ast_file_outline` handler. |
| `src/tools/top_level_nodes.rs` | `ast_top_level_nodes` handler. |
| `src/languages/typescript.rs` (extend) | `outline_candidates` helper. |
| `src/languages/javascript.rs` (extend) | `outline_candidates` helper. |
| `src/languages/python.rs` (extend) | `outline_candidates` helper. |
| `tests/fixtures/outline/classes.ts` | TS class with methods. |
| `tests/fixtures/outline/classes.py` | Python class with methods. |
| `tests/fixtures/outline/types.ts` | TS file with `type`, `interface`, `enum`. |
| `tests/fixtures/outline/all.py` | Python file with `__all__` and an async function. |
| `tests/integration/outline.rs` | End-to-end tests. |

### Modified Files

| File | Change |
|---|---|
| `src/mcp/register_tools.rs` | Register the 2 new tools. |

## Implementation Steps

1. `src/extractors/top_level.rs`:
   ```rust
   pub struct TopLevelNode { pub kind: String, pub name: Option<String>, pub range: Range }
   pub fn top_level_nodes(tree: &Tree, source: &str) -> Vec<TopLevelNode> {
       let root = tree.root_node();
       let mut out = Vec::new();
       let mut cursor = root.walk();
       for child in root.named_children(&mut cursor) {
           out.push(TopLevelNode {
               kind: child.kind().to_string(),
               name: extract_name(&child, source),
               range: Range {
                   start: ts_point_to_position(child.start_position(), source),
                   end:   ts_point_to_position(child.end_position(), source),
               },
           });
       }
       out
   }
   fn extract_name(node: &Node, source: &str) -> Option<String> { /* see step 4 */ }
   ```
2. `src/languages/typescript.rs::outline_candidates`:
   ```rust
   pub fn outline_candidates(root: Node, source: &str) -> Vec<OutlineCandidate> {
       let mut out = Vec::new();
       let mut cursor = root.walk();
       for child in root.named_children(&mut cursor) {
           match child.kind() {
               "import_statement" | "export_statement" => push(&mut out, child, source, child.kind()),
               "function_declaration" | "generator_function_declaration" => push(&mut out, child, source, "function"),
               "class_declaration" | "class_expression" => push(&mut out, child, source, "class"),
               "interface_declaration" => push(&mut out, child, source, "interface"),
               "type_alias_declaration" => push(&mut out, child, source, "type_alias"),
               "enum_declaration" => push(&mut out, child, source, "enum"),
               "lexical_declaration" => {
                   // const x = () => {}; const x = function() {};
                   for var_desc in field children of "declarator" { /* see step 4 */ }
               }
               _ => {}
           }
       }
       out
   }
   ```
3. `src/languages/javascript.rs::outline_candidates` — same as TS, minus `interface`, `type_alias`, `enum`. Plus a `require(...)` call inside a `lexical_declaration` is treated as a "require" import (also handled by the imports extractor in phase 8).
4. `src/languages/python.rs::outline_candidates`:
   ```python
   import_statement, import_from_statement → "import"
   class_definition → "class"
   function_definition → "function" or "async_function" (if "async" is in the prefix)
   decorated_definition → unwrap and apply the same rules
   ```
5. Name extraction (`extract_name`):
   - `function_declaration` / `class_declaration` / `method_definition`: read the `name` field.
   - `interface_declaration` / `type_alias_declaration` / `enum_declaration`: read the `name` field.
   - `lexical_declaration` with arrow function: walk to the `variable_declarator` and read its `name` field.
   - `import_statement` / `export_statement`: no name (omit).
   - `class_definition` / `function_definition` (Python): read the `name` field.
6. `src/extractors/outline.rs`:
   - Walks `outline_candidates` and builds a tree at `maxDepth` (default 4).
   - For each class node, walks its `class_body` to find `method_definition` children (TS/JS) or its block to find `function_definition` children (Python).
   - Renders `outlineText` as a deterministic multi-line string. Format: `kind name\n  child_kind child_name\n  child_kind child_name`.
   - Honors `MAX_NODES` (500). If hit, sets `truncated: true`.
7. `src/tools/file_outline.rs`:
   - `AstFileOutlineInput { file_path, max_depth?, include_ranges?, include_imports?, include_exports? }`.
   - Defaults: `max_depth: 4, include_ranges: true, include_imports: true, include_exports: true`.
   - Pipeline: resolve → ensure size → read → parse → call `outline` extractor → return `AstFileOutlineResult`.
8. `src/tools/top_level_nodes.rs`:
   - `AstTopLevelNodesInput { file_path, include_text?, max_text_bytes? }`.
   - Defaults: `include_text: false, max_text_bytes: 20,000`.
   - Pipeline: resolve → ensure size → read → parse → call `top_level` extractor → conditionally attach `text` to each node (sliced from source).
9. Tests: assert specific node counts, kinds, and names for each fixture.

## Data / API / Interface Contract

```rust
// extractors::top_level
pub struct TopLevelNode { pub kind: String, pub name: Option<String>, pub range: Range }
pub fn top_level_nodes(tree: &Tree, source: &str) -> Vec<TopLevelNode>;
// extractors::outline
pub struct OutlineNode { pub kind: String, pub name: Option<String>, pub range: Option<Range>, pub children: Option<Vec<OutlineNode>> }
pub struct AstFileOutlineResult { pub file_path: String, pub language: String, pub outline_text: String, pub nodes: Vec<OutlineNode>, pub truncated: bool }
pub fn file_outline(tree: &Tree, source: &str, opts: OutlineOptions) -> AstFileOutlineResult;
// languages::*
pub struct OutlineCandidate { pub kind: String, pub name: Option<String>, pub range: Range, pub children: Vec<OutlineCandidate> }
pub fn outline_candidates(root: Node, source: &str) -> Vec<OutlineCandidate>;
```

Tool response shapes match spec § 16 / § 17.

## Error Handling

- `file_not_found`, `file_too_large`, `unsupported_language`, `parse_failed` — propagated from earlier phases.
- `result_limit_exceeded` — when the node count exceeds `MAX_NODES`; the response carries `truncated: true` and a `returned` field. (The `result_limit_exceeded` error code is reserved for callers that pass an explicit `limit`; in V1 we just truncate.)

## Observability

- Logs: `tracing::debug!` with file path, language, node count, render time. Stderr.
- No metrics. No traces.

## Testing Requirements

### Unit Tests

- `outline_candidates` for each language returns the expected kinds and names for the fixtures.
- `outline_text` is byte-stable for a given fixture (snapshot test).

### Integration Tests

- `tests/integration/outline.rs`:
  - `ast_file_outline classes.ts` → 1 class, 3 methods, 0 imports (none in fixture).
  - `ast_file_outline classes.py` → 1 class, 3 methods.
  - `ast_file_outline types.ts` → 1 interface, 1 type alias, 1 enum.
  - `ast_file_outline all.py` → `__all__` is included in the outline text.
  - `ast_top_level_nodes` on a 10-statement TS file → 10 entries in source order.

## Validation Commands

```bash
cargo build
cargo test
cargo test --test integration outline
```

## Acceptance Criteria

- [ ] `ast_file_outline` for `classes.ts` returns the class and its 3 methods.
- [ ] `ast_file_outline` for `classes.py` returns the class and its 3 methods.
- [ ] `ast_file_outline` for `types.ts` returns 1 interface, 1 type alias, 1 enum.
- [ ] `ast_file_outline` for `all.py` includes `__all__` and the async function.
- [ ] `outlineText` is deterministic (snapshot test passes).
- [ ] `ast_top_level_nodes` returns direct root children in source order.
- [ ] `ast_top_level_nodes` with `includeText: true` returns text capped at `maxTextBytes` (20,000).
- [ ] Both tools honor `MAX_NODES` and set `truncated: true` if capped.
- [ ] `cargo clippy --all-targets -- -D warnings` passes.

## Risks

| Risk | Severity | Mitigation |
|---|---|---|
| Outline renderer formatting changes between runs | low | Snapshot test pins the exact bytes. |
| Python decorated definitions missed | medium | The `outline_candidates` for Python unwraps `decorated_definition` and applies the same rules to the inner node. |
| `lexical_declaration` with multiple declarators (e.g., `const a = 1, b = 2;`) | low | In V1, we emit one outline candidate per declarator. Phase 9 will revisit if `ast_find_functions` requires per-declarator data. |
