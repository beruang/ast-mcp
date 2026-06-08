# Position Encoding (V1)

## Model

The public API uses **UTF-16 code units** for character offsets in `Position.character`. This matches the LSP model.

- **line**: 0-based line number.
- **character**: 0-based UTF-16 code-unit offset from the start of the line.

## Coverage

| Encoding | UTF-8 bytes | UTF-16 code units | Tested |
|---|---|---|---|
| ASCII | 1 | 1 | Yes |
| Latin-1 (e.g. `é`) | 2 | 1 | Yes |
| BMP CJK (e.g. `中`) | 3 | 1 | Yes |
| Surrogate pairs (e.g. `😀`) | 4 | 2 | Yes |

The round-trip tests in `tests/positions_test.rs` assert that `byte_offset_to_position` and `position_to_byte_offset` are exact inverses for all four classes.

## Limitations

- Strings combining astral-plane characters with combining marks are not specifically tested in V1.
- Non-BMP characters outside the Basic Multilingual Plane are supported via surrogate pairs, but complex grapheme clusters may report different UTF-16 widths than user-perceived character positions.

## Implementation

`LineIndex` (in `src/parser/positions.rs`) builds a table of byte offsets where each line begins. Lookups are `O(log n)` via binary search. The index is built once per source string.

Conversion between byte offsets and UTF-16 positions walks the source line character-by-character, counting `char::len_utf16()` for each code point.

## Test Fixtures

- `tests/fixtures/utf16/emoji.ts` — TypeScript source containing `😀`
- `tests/fixtures/utf16/emoji.py` — Python source containing `😀`
- `tests/positions_test.rs` — Round-trip unit tests
- `tests/position_utf16_test.rs` — Integration tests via `ast_parse_file`
