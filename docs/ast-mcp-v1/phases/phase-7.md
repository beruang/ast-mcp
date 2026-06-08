# Phase 7: Enclosing Node

> Spec milestone: `spec/version-1.md` § 33 Milestone 7 + § 18

## Goal

Ship `ast_enclosing_node`. This is the agent's primary "what is around this position?" tool.

## Dependencies

- Requires: phase-6.
- Produces: 6 of 12 V1 tools.

## Risk

- **low** — the algorithm is well-defined; phase 5's position helpers do the heavy lifting.

## Value

Used by every "inspect this region" workflow. Cheap to implement once positions are correct.

## Implementation Notes

- `src/extractors/enclosing_node.rs`:
  - `pub fn enclosing_node(tree: &Tree, source: &str, pos: Position, opts: EnclosingOptions) -> AstEnclosingNodeResult`.
  - Convert `pos` → byte offset using phase-5 helpers.
  - Walk root's named children with `tree.root_node().descendant_for_byte_range(byte, byte)`. Tree-sitter returns the smallest named node containing the range; if the position is on a boundary, prefer the inner node.
  - If `opts.kinds` is non-empty, walk `node.parent()` until a matching kind is found.
  - Collect ancestors from outermost to innermost, capped at 64 entries.
  - Include text only if `includeText: true` and the text fits in `maxTextBytes`.
- `src/tools/enclosing_node.rs` wires the extractor. Defaults: `includeAncestors: true, includeText: true, maxTextBytes: 20,000`.
- Document the ancestor order in the tool description: outermost → innermost. This matches spec § 18.

## Validation

```bash
cargo test --test integration ast_enclosing_node
```

Integration test: a TS file with a class containing a method containing an `if` statement. Three positions: inside the `if`, inside the method, inside the class. Each returns the expected `node` and ancestors.

## Acceptance

- [ ] `ast_enclosing_node` at a position inside a function body returns `kind: "function_declaration"` (or the matching kind for that language).
- [ ] `kinds: ["class_declaration"]` returns the enclosing class even when the position is in a deeply nested method.
- [ ] `ancestors` is returned outermost-first.
- [ ] Out-of-bounds position returns `invalid_position`.
- [ ] Text is bounded; if `maxTextBytes` would be exceeded, `text` is omitted and `truncated: true` is implied.
