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
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"\x1B[0;1;34mArea\x1B[0;1;30m.....\x1B[0;1;34m: \x1B[0;1mUploads\r\n\x1B[17;1H\x1B[0;1;34mCommand \x1B[0;1;30m-> D\r\n\x1B[18;1H\r\n\x1B[19;1H\x1B[0;1;31mDownload queued files? \x1B[0;1;36m\x1B[0;1;34m\x1B[0;1;34;44m Y\x1B[0;1;44mes \x1B[0;1;34;44m\x1B[0;1;34m N\x1B[0;1mo \r\n\x1B[20;1H\r\n\x1B[21;1H\x1B[0m\x1B[36mFiles in Batch \x1B[0;1;36m\x1B[0;1;30m-> \x1B[0;1;33m1\r\n\x1B[22;1H\x1B[0m\x1B[36mBatch Size     \x1B[0;1;36m\x1B[0;1;30m-> \x1B[0;1;32m75,988\r\n\x1B[23;1H\x1B[0m\x1B[36mEstimated Time \x1B[0;1;36m\x1B[0;1;30m-> \x1B[0;1;36m0 min 19 seconds\r\n\x1B[24;1H\r\n\x1B[25;1H\x1B[0;1;33mAvailable Protocols:\r\x1B[0m\n\x1B[1;33m\x1B[25;1H\r\x1B[0m\n\x1B[1;33m\x1B[25;1H[\x1B[0;1mY\x1B[0;1;33m] Ymodem\r\x1B[0m\n\x1B[1;33m\x1B[25;1H[\x1B[0;1mG\x1B[0;1;33m] Ymodem-G\r\x1B[0m\n\x1B[1;33m\x1B[25;1H[\x1B[0;1mZ\x1B[0;1;33m] Zmodem\r\x1B[0m\n\x1B[1;33m\x1B[25;1H[\x1B[0;1m8\x1B[0;1;33m] Zmodem 8K\r\x1B[0m\n\x1B[1;33m\x1B[25;1H\r\x1B[0m\n\x1B[1;33m\x1B[25;1HSelect Protocol [\x1B[0;1mQ/Quit\x1B[0;1;33m]: ");
    //(x: 0, y: 16)
    assert_eq!(31, caret.pos.y);
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