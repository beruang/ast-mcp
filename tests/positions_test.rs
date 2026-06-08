use ast_mcp::parser::positions;
use ast_mcp::shared::position::Position;

#[test]
fn line_index_byte_to_position_multiline() {
    let source = "line1\nline2\nline3";
    // Verify positions on each line via public API
    assert_eq!(positions::byte_offset_to_position(source, 0), Position { line: 0, character: 0 });
    // "line2" starts at byte 6 (after "line1\n")
    assert_eq!(positions::byte_offset_to_position(source, 6), Position { line: 1, character: 0 });
    // "line3" starts at byte 12 (after "line1\nline2\n")
    assert_eq!(positions::byte_offset_to_position(source, 12), Position { line: 2, character: 0 });
}

#[test]
fn byte_to_position_ascii() {
    let source = "hello world";
    let pos = positions::byte_offset_to_position(source, 0);
    assert_eq!(pos.line, 0);
    assert_eq!(pos.character, 0);
    let pos = positions::byte_offset_to_position(source, 5);
    assert_eq!(pos.line, 0);
    assert_eq!(pos.character, 5);
}

#[test]
fn ascii_round_trip() {
    let source = "hello\nworld\n";
    let pos = positions::byte_offset_to_position(source, 0);
    assert_eq!(pos, Position { line: 0, character: 0 });
    let byte = positions::position_to_byte_offset(source, pos).unwrap();
    assert_eq!(byte, 0);

    // "world" starts at byte 6 (after "hello\n")
    let pos = positions::byte_offset_to_position(source, 6);
    assert_eq!(pos, Position { line: 1, character: 0 });
    let byte = positions::position_to_byte_offset(source, pos).unwrap();
    assert_eq!(byte, 6);
}

#[test]
fn latin1_round_trip() {
    // 'é' is 2 bytes UTF-8, 1 UTF-16 code unit
    let source = "café";
    // c=byte 0, a=byte 1, f=byte 2, é=bytes 3-4
    // Position at byte 3 = start of é
    let pos = positions::byte_offset_to_position(source, 3);
    assert_eq!(pos.line, 0);
    assert_eq!(pos.character, 3); // 3 UTF-16 code units before é
    let byte = positions::position_to_byte_offset(source, pos).unwrap();
    assert_eq!(byte, 3);
}

#[test]
fn bmp_cjk_round_trip() {
    // '中' is 3 bytes UTF-8, 1 UTF-16 code unit
    let source = "a中b";
    // a=1 byte, 中=3 bytes, b=1 byte
    let pos = positions::byte_offset_to_position(source, 1); // byte 1 = start of 中
    assert_eq!(pos.line, 0);
    assert_eq!(pos.character, 1); // 1 UTF-16 code unit (a)
    let byte = positions::position_to_byte_offset(source, pos).unwrap();
    assert_eq!(byte, 1);

    let pos = positions::byte_offset_to_position(source, 4); // byte 4 = start of b
    assert_eq!(pos.line, 0);
    assert_eq!(pos.character, 2); // 2 UTF-16 code units (a + 中)
    let byte = positions::position_to_byte_offset(source, pos).unwrap();
    assert_eq!(byte, 4);
}

#[test]
fn surrogate_pair_round_trip() {
    // '😀' is 4 bytes UTF-8, 2 UTF-16 code units
    let source = "a😀b";
    // a=1 byte, 😀=4 bytes, b=1 byte
    let pos = positions::byte_offset_to_position(source, 1); // byte 1 = start of 😀
    assert_eq!(pos.line, 0);
    assert_eq!(pos.character, 1); // 1 UTF-16 code unit (a)
    let byte = positions::position_to_byte_offset(source, pos).unwrap();
    assert_eq!(byte, 1);

    // Position at character 3 (after the surrogate pair)
    let pos = Position { line: 0, character: 3 };
    let byte = positions::position_to_byte_offset(source, pos).unwrap();
    assert_eq!(byte, 5); // byte 5 = start of b
    let round = positions::byte_offset_to_position(source, byte);
    assert_eq!(round, pos);
}

#[test]
fn out_of_bounds_line() {
    let source = "one\ntwo";
    let result = positions::position_to_byte_offset(source, Position { line: 5, character: 0 });
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code(), "invalid_position");
}

#[test]
fn out_of_bounds_character() {
    let source = "hello";
    let result = positions::position_to_byte_offset(source, Position { line: 0, character: 20 });
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code(), "invalid_position");
}

#[test]
fn multi_line_position() {
    let source = "line1\nline2\nline3";
    // "line2" starts at byte 6
    let pos = positions::byte_offset_to_position(source, 6);
    assert_eq!(pos, Position { line: 1, character: 0 });
    // "line3" starts at byte 12
    let pos = positions::byte_offset_to_position(source, 12);
    assert_eq!(pos, Position { line: 2, character: 0 });
}

#[test]
fn byte_to_position_at_end() {
    let source = "hello";
    let pos = positions::byte_offset_to_position(source, source.len());
    assert_eq!(pos.line, 0);
    assert_eq!(pos.character, 5);
}

#[test]
fn position_to_byte_at_end_of_line() {
    let source = "hello\nworld";
    // Position at the newline
    let pos = Position { line: 0, character: 5 };
    let byte = positions::position_to_byte_offset(source, pos).unwrap();
    assert_eq!(byte, 5); // byte 5 is '\n'
    assert_eq!(&source[byte..byte + 1], "\n");
}
