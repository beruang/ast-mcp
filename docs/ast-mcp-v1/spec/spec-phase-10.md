# Spec Phase 10: Chunking

## Phase Goal

Ship `ast_chunk_file` with all four strategies.

## Dependencies

- Requires: phase-9.
- Produces: 11 of 12 V1 tools.

## Existing Code References

- Pattern to follow: `src/tools/find_classes::handle` (phase 9).
- Related modules: `extractors::outline` (phase 6), `extractors::imports` (phase 8), `extractors::exports` (phase 8), `extractors::functions` (phase 9), `extractors::classes` (phase 9).
- Test pattern: `tests/integration/functions_classes.rs` (phase 9) — extend with chunk tests.

## Technical Approach

A `ChunkBuilder` that groups structural nodes per strategy, applies limits, and emits stable chunk IDs. Each strategy has a thin grouping closure; the splitting logic is shared.

## File Changes

### New Files

| File | Purpose |
|---|---|
| `src/extractors/chunks.rs` | `ChunkBuilder`, `chunk_file` extractor, `AstChunk` type. |
| `src/tools/chunk_file.rs` | `ast_chunk_file` handler. |
| `tests/fixtures/chunks/mixed.ts` | A TS file with imports, classes, functions, types, exports, and one large method. |
| `tests/fixtures/chunks/mixed.py` | A Python file with similar coverage. |
| `tests/integration/chunking.rs` | End-to-end tests. |

### Modified Files

| File | Change |
|---|---|
| `src/mcp/register_tools.rs` | Register the new tool. |

## Implementation Steps

1. `src/extractors/chunks.rs`:
   ```rust
   pub struct AstChunk {
       pub id: String,                // "{filePath}:{kind}:{name}:{startLine}-{endLine}"
       pub kind: String,
       pub name: Option<String>,
       pub range: Range,
       pub start_line: u32,
       pub end_line: u32,
       pub byte_length: usize,
       pub text: Option<String>,
   }
   pub enum ChunkStrategy { TopLevel, FunctionClass, SemanticBlocks, MaxLinesWithAstBoundaries }
   pub struct ChunkOptions { pub strategy: ChunkStrategy, pub max_chunk_lines: usize, pub max_chunk_bytes: usize, pub include_imports: bool, pub include_text: bool }
   pub fn chunk_file(tree: &Tree, source: &str, file_path: &str, opts: ChunkOptions) -> ChunkResult;
   ```
2. `ChunkBuilder` core:
   - `group(candidates: Vec<OutlineCandidate>, strategy) -> Vec<Vec<OutlineCandidate>>` — the per-strategy grouping closure.
   - `emit(group: Vec<OutlineCandidate>) -> Vec<AstChunk>` — turn a group into chunks, splitting any candidate that exceeds `max_chunk_bytes` on AST boundaries.
   - `id(file_path, kind, name, start_line, end_line)` — stable ID format.
3. Strategy implementations:
   - `TopLevel`: each top-level candidate is its own group.
   - `FunctionClass`: classes and their methods are one group; functions each get a group; type aliases and interfaces are grouped into one `types` group.
   - `SemanticBlocks` (default): imports → group 1; re-exports / type aliases / interfaces / enums → group 2; classes → each its own group; functions → each its own group; large methods (length > `max_chunk_lines`) → each its own group; remaining top-level statements → one "remainder" group.
   - `MaxLinesWithAstBoundaries`: greedy walk. Add children to the current chunk until adding the next would exceed `max_chunk_lines`. Emit and reset. Never split a single node across chunks unless it exceeds `max_chunk_bytes` (then emit it alone with `truncated: true`).
4. `includeImports: true` prepends an `imports` chunk. For `SemanticBlocks`, the imports are absorbed into the first semantic block.
5. `includeText: true` (default) attaches the source slice; otherwise `text` is `None`.
6. Tool handler: resolve → ensure size → read → parse → call extractor → return `AstChunkFileResult`.
7. `chunk_id` is `{filePath}:{kind}:{name}:{startLine}-{endLine}`. When `name` is absent, use the kind twice.

## Data / API / Interface Contract

```rust
// extractors::chunks
pub struct AstChunk { pub id: String, pub kind: String, pub name: Option<String>, pub range: Range, pub start_line: u32, pub end_line: u32, pub byte_length: usize, pub text: Option<String> }
pub enum ChunkStrategy { TopLevel, FunctionClass, SemanticBlocks, MaxLinesWithAstBoundaries }
pub struct ChunkOptions { pub strategy: ChunkStrategy, pub max_chunk_lines: usize, pub max_chunk_bytes: usize, pub include_imports: bool, pub include_text: bool }
pub struct ChunkResult { pub chunks: Vec<AstChunk>, pub truncated: bool }
pub fn chunk_file(tree: &Tree, source: &str, file_path: &str, opts: ChunkOptions) -> ChunkResult;
```

Tool response shape (matches spec § 23):

```jsonc
{
  "filePath": "src/user.ts",
  "language": "typescript",
  "strategy": "semantic_blocks",
  "chunks": [
    { "id": "src/user.ts:imports:imports:1-5", "kind": "imports", "range": { /* ... */ }, "startLine": 1, "endLine": 5, "byteLength": 240, "text": "/* ... */" }
  ],
  "returned": 8,
  "truncated": false
}
```

## Error Handling

- `file_not_found`, `file_too_large`, `unsupported_language` — propagated.
- `result_limit_exceeded` not raised; the result list is silently truncated.

## Observability

- Logs: `tracing::debug!` with file path, strategy, chunk count, total bytes. Stderr.

## Testing Requirements

### Unit Tests

- Each strategy returns the expected grouping on a tiny fixture.
- Chunk IDs are stable.
- A chunk whose candidate exceeds `max_chunk_bytes` is emitted alone with `truncated: true`.

### Integration Tests

- `tests/integration/chunking.rs`:
  - `ast_chunk_file mixed.ts` with `strategy: "top_level"` → one chunk per top-level statement.
  - `ast_chunk_file mixed.ts` with `strategy: "function_class"` → one chunk per function/class; types grouped.
  - `ast_chunk_file mixed.ts` with `strategy: "semantic_blocks"` (default) → imports first, then types, then classes, then functions.
  - `ast_chunk_file mixed.ts` with `strategy: "max_lines_with_ast_boundaries"` → chunks near `maxChunkLines`; never splits a single node.
  - A `mixed.ts` containing a 200-line method is split into a single oversized chunk with `truncated: true`.

## Validation Commands

```bash
cargo test --lib extractors::chunks
cargo test --test integration chunking
```

## Acceptance Criteria

- [ ] `ast_chunk_file` with `strategy: "top_level"` returns one chunk per direct root child.
- [ ] `ast_chunk_file` with `strategy: "function_class"` returns one chunk per function/method/class.
- [ ] `ast_chunk_file` with `strategy: "semantic_blocks"` returns imports first, then classes, then functions.
- [ ] `ast_chunk_file` with `strategy: "max_lines_with_ast_boundaries"` splits on AST boundaries and never splits a single node.
- [ ] Each chunk has a stable `id` matching `{filePath}:{kind}:{name}:{startLine}-{endLine}`.
- [ ] Chunks respect `maxChunkLines` (120) and `maxChunkBytes` (30,000).
- [ ] `truncated: true` is set when the result list is capped.
- [ ] `cargo clippy --all-targets -- -D warnings` passes.

## Risks

| Risk | Severity | Mitigation |
|---|---|---|
| `max_lines_with_ast_boundaries` produces a single chunk that exceeds `maxChunkBytes` | low | The strategy's "never split a node" rule allows oversized chunks; the response carries `truncated: true` and the byte length so the caller can decide. |
| `name` extraction differs from `outline_candidates` | low | Both extractors share the same `extract_name` helper (defined in phase 6). |
| `__all__` not recognized as a Python top-level | low | Python's `assignment` with LHS `__all__` is grouped with the imports block, not the remainder. |
