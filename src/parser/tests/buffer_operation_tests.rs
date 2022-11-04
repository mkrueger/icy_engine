use crate::{parser::tests::{create_buffer, update_buffer}, AnsiParser, Position, TextAttribute};

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
    for _ in 0..30 {
        caret.lf(&mut buf);
    }
    assert_eq!(30, caret.pos.y);
    assert_eq!(6, buf.get_first_visible_line());
}

#[test]
fn test_margin_up() {
    let (mut buf, mut caret) = create_buffer(&mut AnsiParser::new(), b"\x1b[10;10H");
    assert_eq!(9, caret.pos.y);
    caret.up(&mut buf, 100);
    assert_eq!(0, caret.pos.y);
}

#[test]
fn test_margin_scroll_up() {
    let (mut buf, mut caret) = create_buffer(&mut AnsiParser::new(), b"\x1B[1;25r1\n2\n3\n4\n");
    caret.up(&mut buf, 5);
    assert_eq!(0, caret.pos.y);
    assert_eq!(b'1' as u16, buf.get_char(Position::from(0, 1)).unwrap().char_code);
}

#[test]
fn test_margin_scroll_down() {
    let (mut buf, mut caret) = create_buffer(&mut AnsiParser::new(), b"\x1B[1;25r");
    assert_eq!(0, buf.get_real_buffer_height());
    for i in 0..24 {
        update_buffer(&mut buf, &mut caret, &mut AnsiParser::new(), format!("{}",i + 1).as_bytes());
        caret.lf(&mut buf);
    }
    update_buffer(&mut buf, &mut caret, &mut AnsiParser::new(), b"25");
    assert_eq!(25, buf.get_real_buffer_height());
    assert_eq!(b'1' as u16, buf.get_char(Position::from(0, 0)).unwrap().char_code);

    caret.down(&mut buf, 5);
    assert_eq!(25, buf.get_real_buffer_height());
    assert_eq!(b'6' as u16, buf.get_char(Position::from(0, 0)).unwrap().char_code);
    assert_eq!(b'5' as u16, buf.get_char(Position::from(1, 19)).unwrap().char_code);
}

#[test]
fn test_margin_scroll_down_bug() {
    let (mut buf, mut caret) = create_buffer(&mut AnsiParser::new(), b"1\x1b[5;19r\x1b[17;1Hfoo\nbar");

    let ch = buf.get_char(Position::from(0, 16)).unwrap_or_default();
    assert_eq!(b'f', ch.char_code as u8);
    let ch = buf.get_char(Position::from(0, 17)).unwrap_or_default();
    assert_eq!(b'b', ch.char_code as u8);

    assert_eq!(17, caret.pos.y);
    update_buffer(&mut buf, &mut caret, &mut AnsiParser::new(), b"\x1B[19H\r\n");
    update_buffer(&mut buf, &mut caret, &mut AnsiParser::new(), b"\x1B[19H\r\n");
    update_buffer(&mut buf, &mut caret, &mut AnsiParser::new(), b"\x1B[19H\r\n");

    assert_eq!(18, caret.pos.y);
    
    let ch = buf.get_char(Position::from(0, 16 - 3)).unwrap_or_default();
    assert_eq!(b'f', ch.char_code as u8);
    let ch = buf.get_char(Position::from(0, 17 - 3)).unwrap_or_default();
    assert_eq!(b'b', ch.char_code as u8);
}

#[test]
fn test_clear_screen_reset() {
    let (mut buf, mut caret) = create_buffer(&mut AnsiParser::new(), b"");
    for _ in 0..100 {
        caret.lf(&mut buf);
    }
    buf.clear_screen(&mut caret);
    assert_eq!(Position::new(), caret.pos);
    assert_eq!(0, buf.get_first_visible_line());
}