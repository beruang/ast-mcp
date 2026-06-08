# Spec Phase 7: Enclosing Node

## Phase Goal

Ship `ast_enclosing_node`. This is the agent's primary "what is around this position?" tool.

## Dependencies

- Requires: phase-6.
- Produces: 6 of 12 V1 tools.

## Existing Code References

- Pattern to follow: `src/tools/parse_file::handle` (phase 4).
- Related modules: `parser::positions` (phase 5), `extractors::top_level` (phase 6).
- Test pattern: `tests/integration/outline.rs` (phase 6) — extend with an enclosing-node test.

## Technical Approach

Use Tree-sitter's `descendant_for_byte_range` to find the smallest named node containing the position. Walk ancestors for kind filtering. Cap ancestor list at 64.

## File Changes

### New Files

| File | Purpose |
|---|---|
| `src/extractors/enclosing_node.rs` | `enclosing_node` extractor. |
| `src/tools/enclosing_node.rs` | `ast_enclosing_node` handler. |
| `tests/fixtures/enclosing/nested.ts` | A class with a method with an `if` statement. |
| `tests/integration/enclosing_node.rs` | End-to-end tests. |

### Modified Files

| File | Change |
|---|---|
| `src/mcp/register_tools.rs` | Register the new tool. |

## Implementation Steps

1. `src/extractors/enclosing_node.rs`:
   ```rust
   pub struct EnclosingOptions {
       pub kinds: Vec<String>,           // empty = no filter
       pub include_ancestors: bool,      // default true
       pub include_text: bool,           // default true
       pub max_text_bytes: usize,        // default 20,000
   }
   pub struct EnclosingResult {
       pub node: Option<NodeSummary>,    // smallest node, or matching-kind ancestor if filter is set
       pub ancestors: Vec<NodeSummary>,  // outermost → innermost
   }
   pub struct NodeSummary { pub kind: String, pub name: Option<String>, pub range: Range, pub text: Option<String> }
   pub fn enclosing_node(tree: &Tree, source: &str, pos: Position, opts: EnclosingOptions) -> Result<EnclosingResult, AstToolError> {
       let byte = position_to_byte_offset(source, pos)?;
       let root = tree.root_node();
       let smallest = root.descendant_for_byte_range(byte, byte);
       let Some(smallest) = smallest else { return Ok(EnclosingResult { node: None, ancestors: vec![] }); };
       // Apply kind filter
       let target = if opts.kinds.is_empty() {
           Some(smallest)
       } else {
           let mut cur = Some(smallest);
           loop {
               let n = cur?;
               if opts.kinds.iter().any(|k| k == n.kind()) { break Some(n); }
               cur = n.parent();
           }
       };
       // Collect ancestors (outermost → innermost) up to 64
       let mut ancestors = Vec::new();
       if opts.include_ancestors {
           let mut chain = Vec::new();
           let mut cur = target.and_then(|n| n.parent());
           while let Some(n) = cur {
               chain.push(n);
               if chain.len() >= 64 { break; }
               cur = n.parent();
           }
           chain.reverse();   // outermost first
           for n in chain { ancestors.push(to_summary(n, source, opts.include_text, opts.max_text_bytes)); }
       }
       let node = target.map(|n| to_summary(n, source, opts.include_text, opts.max_text_bytes));
       Ok(EnclosingResult { node, ancestors })
   }
   fn to_summary(n: Node, source: &str, include_text: bool, max_text_bytes: usize) -> NodeSummary { /* range + optional text */ }
   ```
2. `src/tools/enclosing_node.rs`:
   - `AstEnclosingNodeInput { file_path, position, kinds?, include_ancestors?, include_text?, max_text_bytes? }`.
   - Defaults: `include_ancestors: true, include_text: true, max_text_bytes: 20,000`.
   - Pipeline: resolve → ensure size → read → parse → call extractor → return `AstEnclosingNodeResult`.
3. Tests:
   - `nested.ts` contains: `class C { m() { if (x) { return; } } }`.
   - Position inside the `if` body → `node.kind == "if_statement"`, ancestors: `class_declaration, method_definition, if_statement` (outermost first).
   - With `kinds: ["class_declaration"]` → `node.kind == "class_declaration"`, ancestors: empty.
   - Out-of-bounds position → `invalid_position`.

## Data / API / Interface Contract

```rust
// extractors::enclosing_node
pub struct EnclosingOptions { pub kinds: Vec<String>, pub include_ancestors: bool, pub include_text: bool, pub max_text_bytes: usize }
pub struct NodeSummary { pub kind: String, pub name: Option<String>, pub range: Range, pub text: Option<String> }
pub struct EnclosingResult { pub node: Option<NodeSummary>, pub ancestors: Vec<NodeSummary> }
pub fn enclosing_node(tree: &Tree, source: &str, pos: Position, opts: EnclosingOptions) -> Result<EnclosingResult, AstToolError>;
```

Tool response shape (matches spec § 18):

```jsonc
{
  "filePath": "src/user.ts",
  "language": "typescript",
  "position": { "line": 20, "character": 12 },
  "node": { "kind": "if_statement", "range": { /* ... */ }, "text": "if (x) { return; }" },
  "ancestors": [
    { "kind": "class_declaration", "name": "C" },
    { "kind": "method_definition", "name": "m" }
  ]
}
```

## Error Handling

- `invalid_position` — out of bounds.
- `file_not_found`, `file_too_large`, `unsupported_language` — propagated.

## Observability

- Logs: `tracing::debug!` with `file_path`, `position`, `node.kind`, `ancestors.len()`. Stderr.

## Testing Requirements

### Unit Tests

- `enclosing_node` for a position exactly on a node boundary returns the inner node.
- `enclosing_node` for a position at `(0, 0)` returns the root child containing the start.
- Kind filter walks ancestors correctly.
- `ancestors` is reversed to outermost-first.

### Integration Tests

- `tests/integration/enclosing_node.rs` — the 3 cases in step 3.

## Validation Commands

```bash
cargo test --lib extractors::enclosing_node
cargo test --test integration enclosing_node
```

## Acceptance Criteria

- [ ] `ast_enclosing_node` at a position inside a function body returns the function node.
- [ ] `kinds: ["class_declaration"]` returns the enclosing class even from a deeply nested position.
- [ ] `ancestors` is outermost-first.
- [ ] Out-of-bounds position returns `invalid_position`.
- [ ] Text is bounded; if `maxTextBytes` is exceeded, `text` is omitted.
- [ ] `cargo clippy --all-targets -- -D warnings` passes.

## Risks

| Risk | Severity | Mitigation |
|---|---|---|
| Position is exactly at a node boundary; smallest node is ambiguous | low | Tree-sitter's `descendant_for_byte_range` returns the leftmost containing node for a zero-width range. Document the behavior in the tool description. |
| Long ancestor chain blows a `Vec` cap | low | The 64-entry cap is generous; we truncate silently. |
