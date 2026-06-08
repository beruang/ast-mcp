# Phase 5: Position Conversion

> Spec milestone: `spec/version-1.md` § 33 Milestone 5 + § 9 (Position and Range Model)

## Goal

Stabilize UTF-16 ↔ byte-offset conversion helpers and propagate them into `Range`. After this phase, every tool that returns a position or range uses the helpers, not ad-hoc math.

## Dependencies

- Requires: phase-4.
- Produces: `parser::positions` module, populated `Range` fields in `ast_parse_file`, and a fuzz test for non-BMP characters.

## Risk

- **high** — UTF-16 ↔ byte math is one of the higher-risk areas. Off-by-one is easy and only visible for non-ASCII inputs.

## Value

Every range-returning tool (8 of 12 V1 tools) depends on this being correct. Getting it right once prevents an entire class of bugs.

## Implementation Notes

- `src/shared/position.rs`:
  - `pub struct Position { pub line: u32, pub character: u32 }` (0-based; matches LSP).
  - `pub struct Range { pub start: Position, pub end: Position }`.
  - `pub fn normalize_range(r: Range) -> Range` (clamps to start ≤ end; both inclusive; 0-based).
- `src/parser/positions.rs`:
  - `pub fn byte_offset_to_position(source: &str, byte: usize) -> Position`
  - `pub fn position_to_byte_offset(source: &str, pos: Position) -> Result<usize, AstToolError>` (returns `invalid_position` if out of bounds)
  - `pub fn ts_point_to_position(p: tree_sitter::Point) -> Position` (Tree-sitter columns are bytes; we translate to UTF-16 using the current line's bytes)
  - `pub fn range_to_byte_range(source: &str, r: Range) -> Result<(usize, usize), AstToolError>`
- Algorithm: build a line-start table once per source (`Vec<usize>` of byte offsets where each line begins). Lookup is `O(log n)` with `binary_search`.
- Update `src/tools/parse_file.rs` to populate `range` on every emitted `AstNodeSummary` (replace the phase-4 placeholder).
- Add non-BMP test fixtures in `tests/fixtures/utf16/`:
  - A TS file with `const greeting = "😀 hello";` — a string containing an emoji (U+1F600).
  - A Python file with the same content.
  - Assert that `ast_parse_file` returns ranges where the line/character of the emoji-bearing token matches the UTF-16 column (not the byte column).
- Document known limitations in `docs/ast-mcp-v1/position-encoding.md`:
  - For V1, we commit to BMP + surrogate pairs correct.
  - Full non-BMP correctness is asserted by the test suite.

## Validation

```bash
cargo test --lib parser::positions
cargo test --test integration ast_parse_file_utf16
```

The UTF-16 integration test must pass for both a TS fixture and a Python fixture.

## Acceptance

- [ ] `byte_offset_to_position` and `position_to_byte_offset` are exact inverses for ASCII, Latin-1, BMP, and surrogate-pair inputs.
- [ ] `ts_point_to_position` returns the correct UTF-16 column for a tree-sitter node sitting in a line that contains a multi-byte character.
- [ ] `range_to_byte_range` returns `(start, end)` byte offsets that match the Tree-sitter node's `start_byte` and `end_byte` for a representative TS source.
- [ ] `ast_parse_file` with `includeTree: true` returns ranges whose `character` is in UTF-16 code units (not bytes).
- [ ] The non-BMP test fixtures pass without rounding.
- [ ] Out-of-bounds positions return `invalid_position`.
