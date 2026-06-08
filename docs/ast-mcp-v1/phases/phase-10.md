# Phase 10: Chunking

> Spec milestone: `spec/version-1.md` § 33 Milestone 10 + § 23

## Goal

Ship `ast_chunk_file` with all four strategies. This is the highest-leverage tool for retrieval and prompt context.

## Dependencies

- Requires: phase-9.
- Produces: 11 of 12 V1 tools.

## Risk

- **medium** — four strategies, but they share a chunk-iteration core. The work is in the grouping rules, not the plumbing.

## Value

Used by every "give me a context pack for this file" workflow. The recommended default (`semantic_blocks`) is what most agents will use by default.

## Implementation Notes

- `src/extractors/chunks.rs`:
  - Shared core: a `ChunkBuilder` that walks candidates, applies a strategy-specific grouping, and splits oversized nodes on AST boundaries.
  - `strategy = "top_level"`: one chunk per direct root child.
  - `strategy = "function_class"`: one chunk per function/method/class declaration; type aliases and interfaces are grouped into a `types` chunk.
  - `strategy = "semantic_blocks"` (default): imports block first, then re-exports/type block, then class declarations, then function declarations, then large methods (split if > `maxChunkLines`), then a final "remainder" chunk for misc top-level statements.
  - `strategy = "max_lines_with_ast_boundaries"`: walk children greedily until adding the next child would exceed `maxChunkLines`; emit a chunk; continue. Never split a single node across chunks unless it exceeds `maxChunkBytes`, in which case emit it as a single oversized chunk with `truncated: true`.
- Stable chunk ID format: `{filePath}:{kind}:{name}:{startLine}-{endLine}` per spec § 23. When `name` is absent (e.g., a top-level statement), use the kind twice: `top_level:export_pair:export_pair:42-50`.
- `includeImports: true` prepends an `imports` chunk at the start of the result list (or absorbs the imports into the first semantic-block chunk for `semantic_blocks`).
- `includeText: true` includes the chunk text; otherwise the chunk is metadata-only.
- `src/tools/chunk_file.rs` wires the extractor. Defaults: `strategy: "semantic_blocks"`, `maxChunkLines: 120`, `maxChunkBytes: 30,000`, `includeImports: true`, `includeText: true`.

## Validation

```bash
cargo test --test integration ast_chunk_file
```

The integration test must cover all 4 strategies on a single TS fixture file. For each strategy, assert: the chunk count is sensible; the first chunk contains imports (if requested); oversized chunks are split; chunk IDs are stable.

## Acceptance

- [ ] `ast_chunk_file` with `strategy: "top_level"` returns one chunk per direct root child.
- [ ] `ast_chunk_file` with `strategy: "function_class"` returns one chunk per function/method/class.
- [ ] `ast_chunk_file` with `strategy: "semantic_blocks"` (default) returns imports first, then classes, then functions.
- [ ] `ast_chunk_file` with `strategy: "max_lines_with_ast_boundaries"` splits on AST boundaries and never splits a single node.
- [ ] Each chunk has a stable `id` matching `{filePath}:{kind}:{name}:{startLine}-{endLine}`.
- [ ] Chunks respect `maxChunkLines` (120) and `maxChunkBytes` (30,000).
- [ ] `truncated: true` is set when the result list is capped.
