use crate::{parser::tests::create_buffer, AnsiParser, Position, TextAttribute};


#[test]
fn test_bs() {
    let (buf, caret) = create_buffer(&mut AnsiParser::new(), b"\x1b[1;43mtest\x08\x08\x08\x08");
    assert_eq!(Position::new(), caret.pos);
    for i in 0..4 {
        assert_eq!(TextAttribute::from_color(15, 6), buf.get_char(Position::from(i, 0)).unwrap().attribute);
    }
}

#[test]
fn test_up() {
    let (mut buf, mut caret) = create_buffer(&mut AnsiParser::new(), b"\x1b[10;10H");
    assert_eq!(9, caret.pos.y);
    caret.up(&mut buf, 100);
    assert_eq!(0, caret.pos.y);
}

#[test]
fn test_down() {
    let (mut buf, mut caret) = create_buffer(&mut AnsiParser::new(), b"\x1b[10;10H");
    assert_eq!(9, caret.pos.y);
    caret.down(&mut buf, 100);
    assert_eq!(24, caret.pos.y);
}

#[test]
fn test_lf_beyond_terminal_height() {
    let (mut buf, mut caret) = create_buffer(&mut AnsiParser::new(), b"");
    for _ in 0..100 {
        caret.lf(&mut buf);
    }
    assert_eq!(100, caret.pos.y);
}
