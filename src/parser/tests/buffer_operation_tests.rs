use crate::{parser::tests::create_buffer, AnsiParser, Position, TextAttribute};


#[test]
fn test_bs() {
    let (buf, caret) = create_buffer(&mut AnsiParser::new(), b"\x1b[1;43mtest\x08\x08\x08\x08");
    assert_eq!(Position::new(), caret.pos);
    for i in 0..4 {
        assert_eq!(TextAttribute::from_color(15, 6), buf.get_char(Position::from(i, 0)).unwrap().attribute);
    }
}
