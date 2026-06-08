# Phase 6: Outline and Top-Level Nodes

> Spec milestone: `spec/version-1.md` § 33 Milestone 6 + § 16 / § 17

## Goal

Ship `ast_file_outline` and `ast_top_level_nodes`. Both lean on language-specific extractors and prove the extractor pattern that the next 4 phases will reuse.

## Dependencies

- Requires: phase-5 (positions).
- Produces: 5 of 12 V1 tools.

## Risk

- **medium** — the extractor pattern is straightforward; the work is enumerating the structural nodes per language.

## Value

Two of the most-used tools. Outline output is consumed by both human agents and downstream chunking/orchestration.

## Implementation Notes

- `src/extractors/top_level.rs`:
  - `pub fn top_level_nodes(tree: &Tree, source: &str) -> Vec<TopLevelNode>`.
  - Iterates the root's named children in source order. Normalizes kind and extracts name where structurally available.
- `src/extractors/outline.rs`:
  - `pub fn file_outline(tree: &Tree, source: &str, opts: OutlineOptions) -> AstFileOutlineResult`.
  - Walks the tree to `maxDepth` (default 4) and produces an `AstOutlineNode` per structural node.
  - For TS/JS: imports, exports, function declarations, class declarations, methods, lexical declarations with arrow functions, interfaces, type aliases, enums.
  - For Python: imports, from-imports, class definitions, function definitions, async function definitions.
  - Renders `outlineText` as a compact, multi-line, deterministic string.
  - Respects `MAX_NODES`. If the cap is hit, sets `truncated: true`.
- `src/languages/typescript.rs` and `src/languages/javascript.rs` and `src/languages/python.rs` each export a `outline_candidates(root: Node, source: &str) -> Vec<OutlineNode>` helper. The extractors call into these; the extractor itself is language-agnostic.
- `src/tools/file_outline.rs` and `src/tools/top_level_nodes.rs` wire the extractors into the tool layer. Both respect the bounded-output rules from phase 4.

## Validation

```bash
cargo test --test integration ast_file_outline
cargo test --test integration ast_top_level_nodes
```

Integration tests must cover: a TS class with methods, a Python class with methods, a TS file with a `type` alias and an `interface`, and a Python file with `__all__` and a top-level async function.

## Acceptance

- [ ] `ast_file_outline` for a TS class file returns the class, its methods, and any imports/exports/types in source order.
- [ ] `ast_file_outline` for a Python class file returns the class, its methods, and any imports in source order.
- [ ] `outlineText` is deterministic — same input → same output, byte-for-byte.
- [ ] `ast_top_level_nodes` returns direct root children in source order, with `kind`, optional `name`, and `range`.
- [ ] `ast_top_level_nodes` with `includeText: true` returns text capped at `maxTextBytes` (default 20,000) and sets `truncated: true` if capped.
- [ ] Both tools return `MAX_NODES`-bounded output and a `truncated` flag.
