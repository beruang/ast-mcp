# Spec Phase 5: Position Conversion

## Phase Goal

Stabilize UTF-16 ↔ byte-offset conversion helpers and propagate them into `Range`. Every range-returning tool from this point on uses the helpers, not ad-hoc math.

## Dependencies

- Requires: phase-4.
- Produces: `shared::position`, `shared::range`, `parser::positions`, UTF-16 tests, and a `docs/ast-mcp-v1/position-encoding.md` limitation doc.

## Existing Code References

- Pattern to follow: `src/shared/errors::AstToolError` (phase 2) — used for out-of-bounds positions.
- Related module: `src/tools/parse_file::walk_tree` (phase 4) — replace the placeholder range with the real one.
- Test pattern: `tests/integration/parse_file.rs` (phase 4) — extend with a UTF-16 fixture.

## Technical Approach

Two helper types and four conversion functions. The line-start table is built once per source. Out-of-bounds positions return `invalid_position`. Non-BMP behavior is asserted by tests; any limitation is documented.

## File Changes

### New Files

| File | Purpose |
|---|---|
| `src/shared/position.rs` | `Position` and `Range` types. |
| `src/shared/range.rs` | `Range` impl, `normalize_range`. (Splitting from `position.rs` is optional; the spec calls for both.) |
| `src/parser/positions.rs` | Conversion helpers and line-start cache. |
| `tests/fixtures/utf16/emoji.ts` | A TS file with a string containing `😀`. |
| `tests/fixtures/utf16/emoji.py` | A Python file with the same content. |
| `tests/integration/position_utf16.rs` | End-to-end UTF-16 tests. |
| `docs/ast-mcp-v1/position-encoding.md` | Limitation doc. |

### Modified Files

| File | Change |
|---|---|
| `src/shared/ast_node.rs` | Use the real `Range` type from `shared::position`. |
| `src/tools/parse_file.rs` | `walk_tree` populates real `Range` values via `parser::positions`. |

## Implementation Steps

1. `src/shared/position.rs`:
   ```rust
   #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
   pub struct Position { pub line: u32, pub character: u32 }   // 0-based
   ```
2. `src/shared/range.rs`:
   ```rust
   #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
   pub struct Range { pub start: Position, pub end: Position }
   pub fn normalize_range(r: Range) -> Range { /* clamp start ≤ end; both inclusive; 0-based */ }
   ```
3. `src/parser/positions.rs`:
   ```rust
   use crate::shared::position::{Position, Range};
   use crate::shared::errors::AstToolError;

   pub struct LineIndex { starts: Vec<usize> }   // byte offset where each line begins
   impl LineIndex {
       pub fn new(source: &str) -> Self {
           let mut starts = vec![0usize];
           for (i, b) in source.bytes().enumerate() {
               if b == b'\n' { starts.push(i + 1); }
           }
           Self { starts }
       }
       pub fn position_to_byte(&self, source: &str, pos: Position) -> Result<usize, AstToolError> { /* binary search line, then walk char-by-char counting UTF-16 code units, then add line start */ }
       pub fn byte_to_position(&self, source: &str, byte: usize) -> Position { /* binary search line, then count UTF-16 code units in that line up to the byte */ }
   }

   pub fn byte_offset_to_position(source: &str, byte: usize) -> Position { /* builds LineIndex once */ }
   pub fn position_to_byte_offset(source: &str, pos: Position) -> Result<usize, AstToolError> { /* ditto */ }
   pub fn ts_point_to_position(p: tree_sitter::Point, source: &str) -> Position { /* uses byte_to_position on the line's start + p.column (bytes) */ }
   pub fn range_to_byte_range(source: &str, r: Range) -> Result<(usize, usize), AstToolError> { /* (position_to_byte_offset(start), position_to_byte_offset(end)) */ }
   ```
   Implementation notes for `position_to_byte_offset`:
   - Look up the start byte of `pos.line`. If the line index is out of bounds, return `InvalidPosition`.
   - Walk the line char-by-char (`source[line_start..].chars()`), counting UTF-16 code units. BMP chars = 1; surrogate pairs = 2.
   - Stop at `pos.character` UTF-16 code units. The byte offset is `line_start + bytes_consumed`.
4. `walk_tree` update: each `AstNodeSummary` now uses `Range { start: ts_point_to_position(node.start_position(), source), end: ts_point_to_position(node.end_position(), source) }`.
5. UTF-16 fixtures — write them by hand so the byte positions are deterministic:
   ```ts
   // emoji.ts
   const greeting = "😀 hello";
   function greet() { return greeting; }
   ```
   ```py
   # emoji.py
   GREETING = "😀 hello"
   def greet(): return GREETING
   ```
6. `tests/parser/positions.rs`:
   - ASCII round-trip: `byte_offset_to_position(source, 0)` ↔ `position_to_byte_offset(source, Position { 0, 0 })`.
   - Latin-1 (e.g., `é` is 2 bytes UTF-8, 1 UTF-16 code unit): round-trip is exact.
   - BMP CJK (e.g., `中` is 3 bytes UTF-8, 1 UTF-16 code unit): round-trip is exact.
   - Surrogate pair (e.g., `😀` is 4 bytes UTF-8, 2 UTF-16 code units): `Position { 0, N }` where `N` reflects the 2-code-unit width.
   - Out-of-bounds line → `InvalidPosition`.
   - Out-of-bounds character on a valid line → `InvalidPosition`.
7. `tests/integration/position_utf16.rs`:
   - `ast_parse_file` on `emoji.ts` with `includeTree: true`. Locate the `string` node containing `😀`. Assert `range.start.character` is 17 (the position right after `const greeting = "`) and the byte offset to character conversion is exact.
   - Same for `emoji.py`.
8. `docs/ast-mcp-v1/position-encoding.md`:
   - State that the public API uses UTF-16 code units.
   - State that the implementation is exact for ASCII, Latin-1, BMP, and surrogate pairs.
   - State that strings combining astral-plane characters with combining marks are not specifically tested in V1.
   - Cross-link to the test fixtures and the `LineIndex` algorithm.

## Data / API / Interface Contract

```rust
// shared::position
pub struct Position { pub line: u32, pub character: u32 }   // 0-based
// shared::range
pub struct Range { pub start: Position, pub end: Position }
pub fn normalize_range(r: Range) -> Range;
// parser::positions
pub struct LineIndex { /* private */ }
impl LineIndex { pub fn new(source: &str) -> Self; pub fn position_to_byte(&self, source: &str, pos: Position) -> Result<usize, AstToolError>; pub fn byte_to_position(&self, source: &str, byte: usize) -> Position; }
pub fn byte_offset_to_position(source: &str, byte: usize) -> Position;
pub fn position_to_byte_offset(source: &str, pos: Position) -> Result<usize, AstToolError>;
pub fn ts_point_to_position(p: tree_sitter::Point, source: &str) -> Position;
pub fn range_to_byte_range(source: &str, r: Range) -> Result<(usize, usize), AstToolError>;
```

## Error Handling

- `invalid_position` — line or character is out of bounds.

## Observability

- Logs: `tracing::trace!` inside `LineIndex::new` with the line count. Stderr.
- No metrics. No traces.

## Testing Requirements

### Unit Tests

- `tests/parser/positions.rs` — round-trip tests for ASCII, Latin-1, BMP, surrogate pairs.
- Out-of-bounds tests.
- `LineIndex` correctness on multi-line sources.

### Integration Tests

- `tests/integration/position_utf16.rs` — emoji fixtures through `ast_parse_file`.

## Validation Commands

```bash
cargo test --lib parser::positions
cargo test --test integration position_utf16
```

## Acceptance Criteria

- [ ] `byte_offset_to_position` and `position_to_byte_offset` are exact inverses for ASCII, Latin-1, BMP, and surrogate-pair inputs.
- [ ] `ts_point_to_position` returns the correct UTF-16 character offset for a Tree-sitter point on a line containing a multi-byte character.
- [ ] `range_to_byte_range` returns byte offsets matching `node.start_byte()` and `node.end_byte()` for representative fixtures.
- [ ] `ast_parse_file` on `emoji.ts` returns ranges in UTF-16 characters, not bytes.
- [ ] Out-of-bounds positions return `invalid_position`.
- [ ] `docs/ast-mcp-v1/position-encoding.md` exists and documents V1's position model.
- [ ] `cargo clippy --all-targets -- -D warnings` passes.

## Risks

| Risk | Severity | Mitigation |
|---|---|---|
| Surrogate-pair width is wrong by 1 | medium | The unit tests in step 6 cover the case explicitly. |
| `LineIndex` is O(n) per source; large files pay the cost | low | The index is built once per request. Reuse across tools via `Arc<LineIndex>` if profiling shows it matters. |
| Out-of-bounds character past the end of a line returns success instead of error | medium | After walking `pos.character` UTF-16 code units, verify the resulting byte offset is on a `char_boundary`. If not, return `InvalidPosition`. |
