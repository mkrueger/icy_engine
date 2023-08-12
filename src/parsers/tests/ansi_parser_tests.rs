#![allow(clippy::float_cmp)]
use crate::{
    convert_to_ans,
    parsers::tests::{create_buffer, get_action, update_buffer},
    AnsiMusicOption, AnsiParser, BufferType, CallbackAction, Caret, Color, MusicAction, Position,
    SaveOptions, TerminalScrolling, TextAttribute, XTERM_256_PALETTE,
};

#[test]
fn test_ansi_sequence() {
    let (buf, _) = create_buffer(
        &mut AnsiParser::new(),
        b"\x1B[0;40;37mFoo-\x1B[1mB\x1B[0ma\x1B[35mr",
    );

    let ch = buf.get_char(Position::new(0, 0)).unwrap_or_default();
    assert_eq!(b'F', ch.ch as u8);
    assert_eq!(7, ch.attribute.as_u8(BufferType::LegacyDos));

    let ch = buf.get_char(Position::new(1, 0)).unwrap_or_default();
    assert_eq!(b'o', ch.ch as u8);
    assert_eq!(7, ch.attribute.as_u8(BufferType::LegacyDos));

    let ch = buf.get_char(Position::new(2, 0)).unwrap_or_default();
    assert_eq!(b'o', ch.ch as u8);
    assert_eq!(7, ch.attribute.as_u8(BufferType::LegacyDos));

    let ch = buf.get_char(Position::new(3, 0)).unwrap_or_default();
    assert_eq!(b'-', ch.ch as u8);
    assert_eq!(7, ch.attribute.as_u8(BufferType::LegacyDos));

    let ch = buf.get_char(Position::new(4, 0)).unwrap_or_default();
    assert_eq!(b'B', ch.ch as u8);
    assert_eq!(15, ch.attribute.as_u8(BufferType::LegacyDos));

    let ch = buf.get_char(Position::new(5, 0)).unwrap_or_default();
    assert_eq!(b'a', ch.ch as u8);
    assert_eq!(7, ch.attribute.as_u8(BufferType::LegacyDos));

    let ch = buf.get_char(Position::new(6, 0)).unwrap_or_default();
    assert_eq!(b'r', ch.ch as u8);
    assert_eq!(5, ch.attribute.as_u8(BufferType::LegacyDos));
}

#[test]
fn test_ansi_30() {
    let (buf, _) = create_buffer(&mut AnsiParser::new(), b"\x1B[1;35mA\x1B[30mB\x1B[0mC");
    let ch = buf.get_char(Position::new(0, 0)).unwrap_or_default();
    assert_eq!(b'A', ch.ch as u8);
    assert_eq!(13, ch.attribute.as_u8(BufferType::LegacyDos));
    let ch = buf.get_char(Position::new(1, 0)).unwrap_or_default();
    assert_eq!(b'B', ch.ch as u8);
    assert_eq!(8, ch.attribute.as_u8(BufferType::LegacyDos));
    let ch = buf.get_char(Position::new(2, 0)).unwrap_or_default();
    assert_eq!(b'C', ch.ch as u8);
    assert_eq!(7, ch.attribute.as_u8(BufferType::LegacyDos));
}

#[test]
fn test_bg_colorrsequence() {
    let (buf, _) = create_buffer(
        &mut AnsiParser::new(),
        b"\x1B[1;30m1\x1B[0;34m2\x1B[33m3\x1B[1;41m4\x1B[40m5\x1B[43m6\x1B[40m7",
    );
    let ch = buf.get_char(Position::new(0, 0)).unwrap_or_default();
    assert_eq!('1', ch.ch);
    assert_eq!(8, ch.attribute.as_u8(BufferType::LegacyDos));
    let ch = buf.get_char(Position::new(1, 0)).unwrap_or_default();
    assert_eq!('2', ch.ch);
    assert_eq!(1, ch.attribute.as_u8(BufferType::LegacyDos));
    let ch = buf.get_char(Position::new(2, 0)).unwrap_or_default();
    assert_eq!('3', ch.ch);
    assert_eq!(6, ch.attribute.as_u8(BufferType::LegacyDos));
    let ch = buf.get_char(Position::new(3, 0)).unwrap_or_default();
    assert_eq!('4', ch.ch);
    assert_eq!(14 + (4 << 4), ch.attribute.as_u8(BufferType::LegacyDos));
    let ch = buf.get_char(Position::new(4, 0)).unwrap_or_default();
    assert_eq!('5', ch.ch);
    assert_eq!(14, ch.attribute.as_u8(BufferType::LegacyDos));
    let ch = buf.get_char(Position::new(5, 0)).unwrap_or_default();
    assert_eq!('6', ch.ch);
    assert_eq!(14 + (6 << 4), ch.attribute.as_u8(BufferType::LegacyDos));
    let ch = buf.get_char(Position::new(6, 0)).unwrap_or_default();
    assert_eq!('7', ch.ch);
    assert_eq!(14, ch.attribute.as_u8(BufferType::LegacyDos));
}
#[test]
fn test_char_missing_bug() {
    let (buf, _) = create_buffer(&mut AnsiParser::new(), b"\x1B[1;35mA\x1B[30mB\x1B[0mC");

    let ch = buf.get_char(Position::new(0, 0)).unwrap_or_default();
    assert_eq!(b'A', ch.ch as u8);
    assert_eq!(13, ch.attribute.as_u8(BufferType::LegacyDos));
    let ch = buf.get_char(Position::new(1, 0)).unwrap_or_default();
    assert_eq!(b'B', ch.ch as u8);
    assert_eq!(8, ch.attribute.as_u8(BufferType::LegacyDos));
    let ch = buf.get_char(Position::new(2, 0)).unwrap_or_default();
    assert_eq!(b'C', ch.ch as u8);
    assert_eq!(7, ch.attribute.as_u8(BufferType::LegacyDos));
}

#[test]
fn test_caret_forward() {
    let (buf, _) = create_buffer(&mut AnsiParser::new(), b"\x1B[70Ctest_me\x1B[2CF");
    let ch = buf.get_char(Position::new(79, 0)).unwrap_or_default();
    assert_eq!('F', char::from_u32(ch.ch as u32).unwrap());
}

#[test]
fn test_caret_forward_at_eol() {
    let (buf, _) = create_buffer(&mut AnsiParser::new(), b"\x1B[75CTEST_\x1B[2CF");
    let ch = buf.get_char(Position::new(2, 1)).unwrap_or_default();
    assert_eq!(b'F', ch.ch as u8);
}

#[test]
fn test_char0_bug() {
    let (buf, _) = create_buffer(&mut AnsiParser::new(), b"\x00A");
    let ch = buf.get_char(Position::new(0, 0)).unwrap_or_default();
    assert_eq!(b'A', ch.ch as u8);
}

fn test_ansi(data: &[u8]) {
    let (buf, _) = create_buffer(&mut AnsiParser::new(), data);
    let converted = convert_to_ans(&buf, &crate::SaveOptions::new()).unwrap();

    // more gentle output.
    let b: Vec<u8> = converted
        .iter()
        .map(|&x| if x == 27 { b'x' } else { x })
        .collect();
    let converted = String::from_utf8_lossy(b.as_slice());

    let b: Vec<u8> = data
        .iter()
        .map(|&x| if x == 27 { b'x' } else { x })
        .collect();
    let expected = String::from_utf8_lossy(b.as_slice());

    assert_eq!(expected, converted);
}

#[test]
fn test_space_compression() {
    let data = b"\x1B[0mA A  A   A    A\x1B[5CA\x1B[6CA\x1B[8CA";
    test_ansi(data);
}

#[test]
fn test_fg_color_change() {
    let data = b"\x1B[0ma\x1B[32ma\x1B[33ma\x1B[1ma\x1B[35ma\x1B[0;35ma\x1B[1;32ma\x1B[0;36ma";
    test_ansi(data);
}

#[test]
fn test_bg_color_change() {
    let data = b"\x1B[0mA\x1B[44mA\x1B[45mA\x1B[31;40mA\x1B[42mA\x1B[40mA\x1B[1;46mA\x1B[0mA\x1B[1;47mA\x1B[0;47mA";
    test_ansi(data);
}
/*
#[test]
fn test_blink_change() {
    let data = b"\x1B[0mA\x1B[5mA\x1B[0mA\x1B[1;5;42mA\x1B[0;1;42mA\x1B[0;5mA\x1B[0;36mA\x1B[5;33mA\x1B[0;1mA";
    test_ansi(data);
}*/

#[test]
fn test_eol_skip() {
    let data = b"\x1B[0;1m\x1B[79Cdd";
    test_ansi(data);
}

#[test]
fn test_eol() {
    let data = b"\x1B[0mfoo\r\n";
    test_ansi(data);
}

#[test]
fn test_noeol() {
    let data = b"\x1B[0mfoo";
    test_ansi(data);
}

#[test]
fn test_emptyeol() {
    let data = b"\r\n";
    test_ansi(data);
    let data = b"\r\n\r\n";
    test_ansi(data);
    let data = b"\r\n\r\n\r\n";
    test_ansi(data);
}

#[test]
fn test_first_char_color() {
    let data = b"\x1B[0;1;36mA";
    test_ansi(data);
    let data = b"\x1B[0;31mA";
    test_ansi(data);
    let data = b"\x1B[0;33;45mA";
    test_ansi(data);
    let data = b"\x1B[0;1;33;45mA";
    test_ansi(data);
}
/*
#[test]
fn test_bgcolor_change() {
    let data = b"\x1B[0mA\x1B[44m \x1B[40m ";
    test_ansi(data);
} */

#[test]
fn test_bgcolor_change2() {
    let data = b"\x1B[0m\x1B[69C\x1B[44m           ";
    test_ansi(data);
}

/*
#[test]
fn test_emptylastline_roundtrip() {
    let mut vec = Vec::new();
    vec.resize(80, b'-');
    vec.resize(80 * 2, b' ');
    let (buf, _) = create_buffer(&mut AnsiParser::new(), &vec);
    assert_eq!(2, buf.get_buffer_height());
    let vec2 = buf.to_bytes("ans", &SaveOptions::new()).unwrap();
    let (buf2, _) = create_buffer(&mut AnsiParser::new(), &vec2);
    assert_eq!(2, buf2.get_buffer_height());
}
*/
#[test]
fn test_linebreak_bug() {
    let (buf, _) = create_buffer(&mut AnsiParser::new(), b"XX");
    assert_eq!(
        '\x16',
        buf.get_char(Position { x: 1, y: 0 }).unwrap_or_default().ch
    );
}

#[test]
fn test_insert_line_default() {
    let (buf, _) = create_buffer(&mut AnsiParser::new(), b"\x1b[L");
    assert_eq!(1, buf.layers[0].lines.len());
}

#[test]
fn test_insert_n_line() {
    let (buf, _) = create_buffer(&mut AnsiParser::new(), b"\x1b[10L");
    assert_eq!(10, buf.layers[0].lines.len());
}

#[test]
fn test_remove_line_default() {
    let (buf, _) = create_buffer(&mut AnsiParser::new(), b"test\x1b[M");
    assert_eq!(b' ', buf.get_char(Position::default()).unwrap().ch as u8);
}

#[test]
fn test_remove_n_line() {
    let (mut buf, _) = create_buffer(&mut AnsiParser::new(), b"test\ntest\ntest\ntest");
    for i in 0..4 {
        assert_eq!(b't', buf.get_char(Position::new(0, i)).unwrap().ch as u8);
    }
    update_buffer(
        &mut buf,
        &mut Caret::default(),
        &mut AnsiParser::new(),
        b"\x1b[3M",
    );
    assert_eq!(b't', buf.get_char(Position::new(0, 0)).unwrap().ch as u8);
    assert_eq!(b' ', buf.get_char(Position::new(0, 1)).unwrap().ch as u8);
}

#[test]
fn test_delete_character_default() {
    let (mut buf, _) = create_buffer(&mut AnsiParser::new(), b"test");
    update_buffer(
        &mut buf,
        &mut Caret::new_xy(0, 0),
        &mut AnsiParser::new(),
        b"\x1b[P",
    );
    assert_eq!(b'e', buf.get_char(Position::new(0, 0)).unwrap().ch as u8);
    update_buffer(
        &mut buf,
        &mut Caret::new_xy(0, 0),
        &mut AnsiParser::new(),
        b"\x1b[P",
    );
    assert_eq!(b's', buf.get_char(Position::new(0, 0)).unwrap().ch as u8);
    update_buffer(
        &mut buf,
        &mut Caret::new_xy(0, 0),
        &mut AnsiParser::new(),
        b"\x1b[P",
    );
    assert_eq!(b't', buf.get_char(Position::new(0, 0)).unwrap().ch as u8);
}

#[test]
fn test_delete_n_character() {
    let (mut buf, _) = create_buffer(&mut AnsiParser::new(), b"testme");
    update_buffer(
        &mut buf,
        &mut Caret::new_xy(0, 0),
        &mut AnsiParser::new(),
        b"\x1b[4P",
    );
    assert_eq!(b'm', buf.get_char(Position::new(0, 0)).unwrap().ch as u8);
}

#[test]
fn test_save_cursor() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"\x1b7testme\x1b8");
    assert_eq!(Position::default(), caret.get_position());
}

#[test]
fn test_save_cursor_more_times() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"\x1b7testme\x1b8testme\x1b8");
    assert_eq!(Position::default(), caret.get_position());
}

#[test]
fn test_reset_cursor() {
    let (mut buf, mut caret) = create_buffer(&mut AnsiParser::new(), b"testme\x1b[1;37m");
    assert_ne!(TextAttribute::default(), caret.attr);
    assert_ne!(Position::default(), caret.get_position());
    update_buffer(&mut buf, &mut caret, &mut AnsiParser::new(), b"\x1bc");
    assert_eq!(TextAttribute::default(), caret.attr);
    assert_eq!(Position::default(), caret.get_position());
}

#[test]
fn test_cursor_visibilty() {
    let (mut buf, mut caret) = create_buffer(&mut AnsiParser::new(), b"\x1b[?25l");
    assert!(!caret.is_visible);
    update_buffer(&mut buf, &mut caret, &mut AnsiParser::new(), b"\x1b[?25h");
    assert!(caret.is_visible);
}

#[test]
fn test_cursor_visibilty_reset() {
    let (mut buf, mut caret) = create_buffer(&mut AnsiParser::new(), b"\x1b[?25l");
    assert!(!caret.is_visible);
    update_buffer(&mut buf, &mut caret, &mut AnsiParser::new(), b"\x0C"); // FF
    assert!(caret.is_visible);
}

#[test]
fn test_vert_line_position_absolute_default() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"\n\n\nfoo\x1b[d");
    assert_eq!(Position::new(3, 0), caret.get_position());
}

#[test]
fn test_vert_line_position_absolute_n() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"test\x1b[5d");
    assert_eq!(Position::new(4, 4), caret.get_position());
}

#[test]
fn test_vert_line_position_relative_default() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"\n\n\nfoo\x1b[e");
    assert_eq!(Position::new(3, 4), caret.get_position());
}

#[test]
fn test_vert_line_position_relative_n() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"\n\n\x1b[5e");
    assert_eq!(Position::new(0, 7), caret.get_position());
}

#[test]
fn test_horiz_line_position_absolute_default() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"foo\x1b['");
    assert_eq!(Position::default(), caret.get_position());
}

#[test]
fn test_horiz_line_position_absolute_n() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"testfooo\x1b['\x1b[3'");
    assert_eq!(Position::new(2, 0), caret.get_position());
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"01234567\x1b['\x1b[100'");
    assert_eq!(Position::new(8, 0), caret.get_position());
}

#[test]
fn test_horiz_line_position_relative_default() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"testfooo\x1b['\x1b[a");
    assert_eq!(Position::new(1, 0), caret.get_position());
}

#[test]
fn test_horiz_line_position_relative_n() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"testfooo\x1b['\x1b[3a");
    assert_eq!(Position::new(3, 0), caret.get_position());
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"01234567\x1b['\x1b[100a");
    assert_eq!(Position::new(8, 0), caret.get_position());
}

#[test]
fn test_cursor_horiz_absolute_default() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"testfooo\x1b[G");
    assert_eq!(Position::new(0, 0), caret.get_position());
}

#[test]
fn test_cursor_horiz_absolute_n() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"testfooo\x1b['\x1b[3G");
    assert_eq!(Position::new(2, 0), caret.get_position());
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"01234567\x1b['\x1b[100G");
    assert_eq!(Position::new(79, 0), caret.get_position());
}

#[test]
fn test_cursor_next_line_default() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"\n\n\nfoo\x1b[E");
    assert_eq!(Position::new(0, 4), caret.get_position());
}

#[test]
fn test_cursor_next_line_n() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"test\x1b[5E");
    assert_eq!(Position::new(0, 5), caret.get_position());
}

#[test]
fn test_cursor_previous_line_default() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"\n\n\nfoo\x1b[F");
    assert_eq!(Position::new(0, 2), caret.get_position());
}

#[test]
fn test_cursor_previous_line_n() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"\n\n\nfoo\x1b[2F");
    assert_eq!(Position::new(0, 1), caret.get_position());
}

#[test]
fn test_set_top_and_bottom_margins() {
    let (buf, _) = create_buffer(&mut AnsiParser::new(), b"\x1b[5;10r");
    assert_eq!(Some((4, 9)), buf.terminal_state.margins);
}

#[test]
fn test_scrolling_terminal_state() {
    let (mut buf, mut caret) = create_buffer(&mut AnsiParser::new(), b"");
    assert_eq!(TerminalScrolling::Smooth, buf.terminal_state.scroll_state);
    update_buffer(&mut buf, &mut caret, &mut AnsiParser::new(), b"\x1b[?4l");
    assert_eq!(TerminalScrolling::Fast, buf.terminal_state.scroll_state);
    update_buffer(&mut buf, &mut caret, &mut AnsiParser::new(), b"\x1b[?4h");
    assert_eq!(TerminalScrolling::Smooth, buf.terminal_state.scroll_state);
}

#[test]
fn test_reset_empty_colors() {
    let (buf, _) = create_buffer(
        &mut AnsiParser::new(),
        b"\x1B[m\x1B[33mN\x1B[1m\x1B[33ma\x1B[m\x1B[33mCHR\x1B[1m\x1B[33mi\x1B[m\x1B[33mCHT",
    );
    assert_eq!(
        buf.get_char(Position::new(0, 0)).unwrap().attribute,
        buf.get_char(Position::new(2, 0)).unwrap().attribute
    );
    assert_eq!(
        buf.get_char(Position::new(1, 0)).unwrap().attribute,
        buf.get_char(Position::new(5, 0)).unwrap().attribute
    );
    assert_eq!(
        buf.get_char(Position::new(2, 0)).unwrap().attribute,
        buf.get_char(Position::new(8, 0)).unwrap().attribute
    );
}

#[test]
fn test_print_char_extension() {
    let (mut buf, mut caret) = create_buffer(&mut AnsiParser::new(), b"");
    for _ in 0..30 {
        update_buffer(&mut buf, &mut caret, &mut AnsiParser::new(), b"a\n");
    }
    assert_eq!(31, buf.layers[0].lines.len());
}

#[test]
fn test_insert_mode() {
    let (buf, _) = create_buffer(
        &mut AnsiParser::new(),
        b"test\x1B[H\x1B[4lhelp\x1B[H\x1B[4hnewtest",
    );
    let converted = crate::convert_to_asc(&buf, &SaveOptions::new()).unwrap();

    // more gentle output.
    let b: Vec<u8> = converted
        .iter()
        .map(|&x| if x == 27 { b'x' } else { x })
        .collect();
    let converted = String::from_utf8_lossy(b.as_slice());
    assert_eq!("newtesthelp", converted);
}

#[test]
fn test_index_line() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"test\x1BD\x1BD\x1BD");
    assert_eq!(Position::new(4, 3), caret.get_position());
}

#[test]
fn test_reverse_index_line() {
    let (buf, caret) = create_buffer(&mut AnsiParser::new(), b"test\x1BM\x1BM\x1BM");
    assert_eq!(Position::new(4, 0), caret.get_position());
    let ch = buf.get_char(Position::new(0, 3)).unwrap_or_default();
    assert_eq!('t', ch.ch);
}

#[test]
fn test_next_line() {
    let (buf, caret) = create_buffer(&mut AnsiParser::new(), b"\x1B[25;1Htest\x1BE\x1BE\x1BE");
    assert_eq!(Position::new(0, 24), caret.get_position());
    let ch = buf.get_char(Position::new(0, 24 - 3)).unwrap_or_default();
    assert_eq!('t', ch.ch);
}

#[test]
fn test_insert_character() {
    let (buf, caret) = create_buffer(&mut AnsiParser::new(), b"foo\x1B[1;1H\x1B[5@");
    assert_eq!(Position::new(0, 0), caret.get_position());
    let ch = buf.get_char(Position::new(5, 0)).unwrap_or_default();
    assert_eq!('f', ch.ch);
}

#[test]
fn test_erase_character() {
    let (buf, caret) = create_buffer(&mut AnsiParser::new(), b"foobar\x1B[1;1H\x1B[3X");
    assert_eq!(Position::new(0, 0), caret.get_position());
    assert_eq!(' ', buf.get_char(Position::new(0, 0)).unwrap().ch);
    assert_eq!(' ', buf.get_char(Position::new(1, 0)).unwrap().ch);
    assert_eq!(' ', buf.get_char(Position::new(2, 0)).unwrap().ch);
    assert_eq!('b', buf.get_char(Position::new(3, 0)).unwrap().ch);
}

#[test]
fn test_xterm_256_colors() {
    let (buf, _) = create_buffer(&mut AnsiParser::new(), b"\x1B[38;5;232m\x1B[48;5;42mf");
    let fg = buf
        .get_char(Position::new(0, 0))
        .unwrap()
        .attribute
        .get_foreground();
    let bg = buf
        .get_char(Position::new(0, 0))
        .unwrap()
        .attribute
        .get_background();
    assert_eq!(XTERM_256_PALETTE[232], buf.palette.colors[fg as usize]);
    assert_eq!(XTERM_256_PALETTE[42], buf.palette.colors[bg as usize]);
}

#[test]
fn test_xterm_24bit_colors() {
    let (buf, _) = create_buffer(
        &mut AnsiParser::new(),
        b"\x1B[38;2;12;13;14m\x1B[48;2;55;54;19mf",
    );
    let fg = buf
        .get_char(Position::new(0, 0))
        .unwrap()
        .attribute
        .get_foreground();
    let bg = buf
        .get_char(Position::new(0, 0))
        .unwrap()
        .attribute
        .get_background();
    assert_eq!(Color::new(12, 13, 14), buf.palette.colors[fg as usize]);
    assert_eq!(Color::new(55, 54, 19), buf.palette.colors[bg as usize]);
}

#[test]
fn test_alt_24bit_colors() {
    let (buf, _) = create_buffer(&mut AnsiParser::new(), b"\x1B[1;12;13;14t\x1B[0;55;54;19tf");
    let fg = buf
        .get_char(Position::new(0, 0))
        .unwrap()
        .attribute
        .get_foreground();
    let bg = buf
        .get_char(Position::new(0, 0))
        .unwrap()
        .attribute
        .get_background();
    assert_eq!(Color::new(12, 13, 14), buf.palette.colors[fg as usize]);
    assert_eq!(Color::new(55, 54, 19), buf.palette.colors[bg as usize]);
}

#[test]
fn test_cursor_position_with0() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"\x1B[10;10H\x1B[24;0H");
    assert_eq!(Position::new(0, 23), caret.get_position());
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"\x1B[10;10H\x1B[24;1H");
    assert_eq!(Position::new(0, 23), caret.get_position());
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"\x1B[10;10H\x1B[0;10H");
    assert_eq!(Position::new(9, 0), caret.get_position());
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"\x1B[10;10H\x1B[1;10H");
    assert_eq!(Position::new(9, 0), caret.get_position());
}

#[test]
fn test_font_switch() {
    let (buf, _) = create_buffer(&mut AnsiParser::new(), b"foo\x1B[0;40 Dbar");
    let ch = buf.get_char(Position::new(2, 0)).unwrap_or_default();
    assert_eq!(0, ch.get_font_page());
    let ch = buf.get_char(Position::new(3, 0)).unwrap_or_default();
    assert_eq!(1, ch.get_font_page());
}

#[test]
fn test_music() {
    let mut p = AnsiParser::new();
    p.ansi_music = AnsiMusicOption::Both;
    let action = get_action(&mut p, b"\x1B[NC\x0E");
    let CallbackAction::PlayMusic(music) = action else {
        panic!();
    };
    assert_eq!(1, music.music_actions.len());
    let MusicAction::PlayNote(f, len) = music.music_actions[0] else {
        panic!();
    };
    assert_eq!(523.2511, f);
    assert_eq!(4 * 120, len);
}

#[test]
fn test_set_length() {
    let mut p = AnsiParser::new();
    p.ansi_music = AnsiMusicOption::Both;
    let action = get_action(&mut p, b"\x1B[NNL8C\x0E");
    let CallbackAction::PlayMusic(music) = action else {
        panic!();
    };
    assert_eq!(2, music.music_actions.len());
    let MusicAction::PlayNote(f, len) = music.music_actions[1] else {
        panic!();
    };
    assert_eq!(523.2511, f);
    assert_eq!(8 * 120, len);
}

#[test]
fn test_tempo() {
    let mut p = AnsiParser::new();
    p.ansi_music = AnsiMusicOption::Both;
    let action = get_action(&mut p, b"\x1B[NT123C\x0E");
    let CallbackAction::PlayMusic(music) = action else {
        panic!();
    };
    assert_eq!(1, music.music_actions.len());
}

#[test]
fn test_pause() {
    let mut p = AnsiParser::new();
    p.ansi_music = AnsiMusicOption::Both;
    let action = get_action(&mut p, b"\x1B[NP32.\x0E");
    let CallbackAction::PlayMusic(music) = action else {
        panic!();
    };
    assert_eq!(1, music.music_actions.len());
    let MusicAction::Pause(t) = music.music_actions[0] else {
        panic!();
    };
    assert_eq!(5760, t);
}

#[test]
fn test_melody() {
    let mut p = AnsiParser::new();
    p.ansi_music = AnsiMusicOption::Both;
    let action = get_action(
        &mut p,
        b"\x1B[MFT225O3L8GL8GL8GL2E-P8L8FL8FL8FMLL2DL2DMNP8\x0E",
    );
    let CallbackAction::PlayMusic(music) = action else {
        panic!();
    };
    assert_eq!(14, music.music_actions.len());
}

#[test]
fn test_macro() {
    let mut parser = AnsiParser::new();
    let (mut buf, mut caret) = create_buffer(&mut parser, b"\x1BP0;0;0!zHello\x1B\\");
    let ch = buf.get_char(Position::new(0, 0)).unwrap_or_default();
    assert_eq!(b' ', ch.ch as u8);
    update_buffer(&mut buf, &mut caret, &mut parser, b"\x1b[0*z");

    let ch = buf.get_char(Position::new(0, 0)).unwrap_or_default();
    assert_eq!(b'H', ch.ch as u8);
    let ch = buf
        .get_char(Position::new("Hello".len() as i32, 0))
        .unwrap_or_default();
    assert_eq!(b' ', ch.ch as u8);
    update_buffer(&mut buf, &mut caret, &mut parser, b"\x1b[0*z");

    let ch = buf
        .get_char(Position::new("Hello".len() as i32, 0))
        .unwrap_or_default();
    assert_eq!(b'H', ch.ch as u8);
}

#[test]
fn test_macro_hex() {
    let mut parser = AnsiParser::new();
    let (mut buf, mut caret) = create_buffer(&mut parser, b"\x1BP0;0;1!z4848484848\x1B\\");
    let ch = buf.get_char(Position::new(0, 0)).unwrap_or_default();
    assert_eq!(b' ', ch.ch as u8);
    update_buffer(&mut buf, &mut caret, &mut parser, b"\x1b[0*z");

    let ch = buf.get_char(Position::new(0, 0)).unwrap_or_default();
    assert_eq!(b'H', ch.ch as u8);
    let ch = buf
        .get_char(Position::new("Hello".len() as i32, 0))
        .unwrap_or_default();
    assert_eq!(b' ', ch.ch as u8);
    update_buffer(&mut buf, &mut caret, &mut parser, b"\x1b[0*z");

    let ch = buf
        .get_char(Position::new("Hello".len() as i32, 0))
        .unwrap_or_default();
    assert_eq!(b'H', ch.ch as u8);
}

#[test]
fn test_macro_repeat_hex() {
    let mut parser = AnsiParser::new();
    let (mut buf, mut caret) = create_buffer(&mut parser, b"\x1BP0;0;1!z!5;48;\x1B\\");
    let ch = buf.get_char(Position::new(0, 0)).unwrap_or_default();
    assert_eq!(b' ', ch.ch as u8);
    update_buffer(&mut buf, &mut caret, &mut parser, b"\x1b[0*z");

    let ch = buf.get_char(Position::new(0, 0)).unwrap_or_default();
    assert_eq!(b'H', ch.ch as u8);
    let ch = buf
        .get_char(Position::new("Hello".len() as i32, 0))
        .unwrap_or_default();
    assert_eq!(b' ', ch.ch as u8);
    update_buffer(&mut buf, &mut caret, &mut parser, b"\x1b[0*z");

    let ch = buf
        .get_char(Position::new("Hello".len() as i32, 0))
        .unwrap_or_default();
    assert_eq!(b'H', ch.ch as u8);
}
/*
#[test]
fn test_macro_sixels() {
    let mut parser = AnsiParser::new();
    let (mut buf, mut caret) = create_buffer(&mut parser, b"\x1B[0m\x1B[31m\x1B[2J\x0C\x1B[?25l\x1B[20;1fLoading graphic macro 1 of 2\x1BP11;0;0!z0;1;0q\"1;1;64;32#0;2;0;0;0#1;2;3;3;3#2;2;6;6;6#3;2;7;7;7#4;2;11;11;11#5;2;13;13;13#6;2;14;14;14#7;2;17;17;17#8;2;17;17;17#9;2;19;19;19#10;2;20;20;20#11;2;23;23;23#12;2;25;25;25#13;2;26;26;26#14;2;28;28;28#15;2;29;29;29#16;2;33;33;33#17;2;35;35;35#18;2;36;36;36#19;2;39;39;39#20;2;42;42;42#21;2;42;42;42#22;2;44;44;44#23;2;47;47;47#24;2;48;48;48#25;2;49;49;49#26;2;51;51;51#27;2;52;52;52#28;2;54;54;54#29;2;55;55;55#30;2;57;57;57#31;2;59;59;59#32;2;63;63;63#33;2;65;65;65#34;2;67;67;67#35;2;68;68;68#36;2;70;70;70#37;2;71;71;71#38;2;73;73;73#39;2;75;75;75#40;2;78;78;78#41;2;79;79;79#42;2;81;81;81#43;2;82;82;82#44;2;84;84;84#45;2;85;85;85#46;2;87;87;87#47;2;88;88;88#48;2;90;90;90#49;2;91;91;91#50;2;93;93;93#51;2;95;95;95#52;2;96;96;96#53;2;97;97;97#54;2;100;100;100#0!14~FF^!47~$#54!14?_?_$#22!14?O#4_$#26!14?G#41G$#48!15?O-#0~!4xrrvFf!4NO@E?@~^^NNNnfbrr!34~$#1?A??A#18G#53G!7?C#8_#18G#38C!8?G#4C#33G#15C$#8?C#26A#14A#54C??G?OOO?_bGoqO???__?O?G#46C$??C#51C#40?C#16C#39?G!6?C#48@#44@#50_!5?O#40?O#6O$#4!8?_#17G#20_??O#25G#9A#45?G#51G??_#13O#23O#26_$#42!8?O#30??_#32_??O#11??A#2?_!8?G$#43!12?O#31!5?C-#0!5~^NNMKK#40@O#53@#52?@#12???@!5?@#0@B!36~$#10!5?_#54_?@PO_l}~}~~~}~{!4}{W$#6!6?O#13O???C#27A#45!8?@#34@#26@#25@#41?A#6_$#45!7?_#20_#14a?G#50!9?A#24!5?C$#35!8?O#26?@#28A$#42!10?A#47O$#49!10?_-#0~~vrrrxx{{}]E_ow{}wo?K?A}}}!37~$#25??G?C!9?A!6?A#1_#49??@#11@$#54???G?C?A?@??PHDA@?@FX`Ah@$#12???C#45G#14G#36A?A#2A#48@#20@G#37C!9?O$#30!6?C#1C#34@!7?A#43?A???@C$#51!11?_#33_#13A???@???O#2G$#21!13?O#22G#41C#7??C#35G??C$#42!15?@#6!4?_#29?O$#9!20?A$#28!20?C-#0!9~xw{!9~}wx!40~$#9!9?A#54A@#41!9?@#54AC$#35!9?C#11@#19A#2!10?C#21A$#34!10?C#42!11?@-#0!64B-\x1B\\\r\n");
    update_buffer(&mut buf, &mut caret, &mut parser, b"\x1B[20;1fLoading graphic macro 2 of 2\x1BP12;0;0!z0;1;0q\"1;1;64;32#0;2;0;0;0#1;2;2;2;2#2;2;4;4;4#3;2;4;4;4#4;2;7;7;7#5;2;8;8;8#6;2;10;10;10#7;2;12;12;12#8;2;13;13;13#9;2;15;15;15#10;2;16;16;16#11;2;18;18;18#12;2;20;20;20#13;2;22;22;22#14;2;24;24;24#15;2;27;27;27#16;2;29;29;29#17;2;31;31;31#18;2;31;31;31#19;2;33;33;33#20;2;36;36;36#21;2;38;38;38#22;2;40;40;40#23;2;41;41;41#24;2;42;42;42#25;2;44;44;44#26;2;45;45;45#27;2;49;49;49#28;2;51;51;51#29;2;53;53;53#30;2;53;53;53#31;2;56;56;56#32;2;57;57;57#33;2;59;59;59#34;2;62;62;62#35;2;63;63;63#36;2;65;65;65#37;2;65;65;65#38;2;68;68;68#39;2;70;70;70#40;2;71;71;71#41;2;72;72;72#42;2;74;74;74#43;2;76;76;76#44;2;77;77;77#45;2;79;79;79#46;2;80;80;80#47;2;82;82;82#48;2;85;85;85#49;2;86;86;86#50;2;88;88;88#51;2;91;91;91#52;2;93;93;93#53;2;95;95;95#54;2;96;96;96#55;2;98;98;98#56;2;100;100;100#0!20~^^!42~$#4!20?_#56_-#0!5~rAKWrfFN^~~NFbO?M!42~$#6!5?G#56O`ACGO_!4?oGAh#5_$#20!5?C#5_#19O#35C??_#38O#54_#1??O#28G#32O?A#37@$#12!6?C#39A#4@#37G#45O#9G#27!4?_#49?C#2@#24C#40O$#50!6?@#16?_#6!10?_#55O$#52!6?G#14!12?G$#43!19?C-#0~~~!4rqO!6?@!5?!4@B!38~$#56???CC???@Eq{}~}{~~}|!5{O$#9???G#21G!4?W#46@#52B#43??@#26!4?A#13@#44A#49A#41A#2A#6C$#39!5?C!9?A#29!4?A#50!4?G$#15!5?G#28G#29CG#41@C?@$#32!6?C#18@#47C!9?@!6?_$#40!7?G#4_#17_#55G$#8!8?A-#0!4~NFbooowG?oo_?C{wwoooceN^^!35~$#56!4?_OG?DDEFNFFB~@BABFD@HOO__$#5!4?O#2GC#9@!9?O#25?C??G???_$#22!5?_#34O!8?C!7?A#5O#8@$#54!7?C!4?O#16?G#18G#10?_#49?@#51C#24??C#38?G$#33!7?A#4G#1G!5?O!5?G??A$#50!7?G#49A#45A???G!8?A#48G$#32!10?@#24O#20_!4?A$#42!11?_#12!5?G-#0~~{{}!6~oq~~po!11~}!35~$#8??A#56@!7?@!4?B#14!11?@$#12??@#19A#46@#17!6?G#2G#7??G#22G$#33!11?C#4@#25??A#43C$#53!11?A#6C#30??C-#0!64B-\x1B\\\r\n");
    assert_eq!(2, parser.macros.len());
    assert_eq!(0, buf.layers[0].sixels.len());

    update_buffer(&mut buf, &mut caret, &mut parser, b"\x1B[2J\x0C\x1B[10;59f\x1BP\x1B[11*z\x1B\\\r\n");

    assert_eq!(1, buf.layers[0].sixels.len());
}
*/
