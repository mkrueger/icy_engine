#![allow(clippy::float_cmp)]
use crate::{
    convert_to_ans,
    parsers::tests::{create_buffer, get_action, update_buffer},
    AnsiMusicOption, AnsiParser, AttributedChar, BufferType, CallbackAction, Caret, Color,
    MusicAction, Position, SaveOptions, TerminalScrolling, TextAttribute, XTERM_256_PALETTE,
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
    assert_eq!(Some((4, 9)), buf.terminal_state.margins_up_down);
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

#[test]
fn test_left_right_margin_mode() {
    let mut parser = AnsiParser::new();
    let (mut buf, mut caret) = create_buffer(&mut parser, b"\x1B[?69h");
    assert!(buf.terminal_state.dec_margin_mode_left_right);
    update_buffer(&mut buf, &mut caret, &mut parser, b"\x1B[5;10s");
    assert_eq!(Some((4, 9)), buf.terminal_state.margins_left_right);

    update_buffer(&mut buf, &mut caret, &mut parser, b"\x1B[?69l");
    assert!(!buf.terminal_state.dec_margin_mode_left_right);
}

#[test]
fn test_scroll_left() {
    let mut parser = AnsiParser::new();
    let (mut buf, mut caret) = create_buffer(&mut parser, b"");

    for y in 0..buf.get_buffer_height() {
        for x in 0..buf.get_buffer_width() {
            buf.set_char(
                0,
                Position::new(x, y),
                Some(AttributedChar::new(
                    unsafe { char::from_u32_unchecked((b'0' as i32 + (x % 10)) as u32) },
                    TextAttribute::default(),
                )),
            );
        }
    }
    for y in 0..buf.get_buffer_height() {
        assert_eq!('9', buf.get_char_xy(79, y).unwrap().ch);
    }
    update_buffer(&mut buf, &mut caret, &mut parser, b"\x1B[ @");
    for y in 0..buf.get_buffer_height() {
        assert_eq!(' ', buf.get_char_xy(79, y).unwrap().ch);
    }
}

#[test]
fn test_scroll_left_with_margins() {
    let mut parser = AnsiParser::new();
    let (mut buf, mut caret) = create_buffer(&mut parser, b"\x1B[?69h\x1B[5;10r\x1B[5;10s");

    for y in 0..buf.get_buffer_height() {
        for x in 0..buf.get_buffer_width() {
            buf.set_char(
                0,
                Position::new(x, y),
                Some(AttributedChar::new(
                    unsafe { char::from_u32_unchecked((b'0' as i32 + (x % 10)) as u32) },
                    TextAttribute::default(),
                )),
            );
        }
    }
    update_buffer(&mut buf, &mut caret, &mut parser, b"\x1B[ @");
    for y in 0..buf.get_buffer_height() {
        if (4..=9).contains(&y) {
            assert_eq!(' ', buf.get_char_xy(9, y).unwrap().ch);
        } else {
            assert_eq!('9', buf.get_char_xy(9, y).unwrap().ch);
        }
    }
}

#[test]
fn test_scroll_right() {
    let mut parser = AnsiParser::new();
    let (mut buf, mut caret) = create_buffer(&mut parser, b"");

    for y in 0..buf.get_buffer_height() {
        for x in 0..buf.get_buffer_width() {
            buf.set_char(
                0,
                Position::new(x, y),
                Some(AttributedChar::new(
                    unsafe { char::from_u32_unchecked((b'0' as i32 + (x % 10)) as u32) },
                    TextAttribute::default(),
                )),
            );
        }
    }
    for y in 0..buf.get_buffer_height() {
        assert_eq!('0', buf.get_char_xy(0, y).unwrap().ch);
    }
    update_buffer(&mut buf, &mut caret, &mut parser, b"\x1B[ A");
    for y in 0..buf.get_buffer_height() {
        assert_eq!(' ', buf.get_char_xy(0, y).unwrap().ch);
    }
}

#[test]
fn test_scroll_right_with_margins() {
    let mut parser = AnsiParser::new();
    let (mut buf, mut caret) = create_buffer(&mut parser, b"\x1B[?69h\x1B[5;10r\x1B[5;10s");

    for y in 0..buf.get_buffer_height() {
        for x in 0..buf.get_buffer_width() {
            buf.set_char(
                0,
                Position::new(x, y),
                Some(AttributedChar::new(
                    unsafe { char::from_u32_unchecked((b'0' as i32 + (x % 10)) as u32) },
                    TextAttribute::default(),
                )),
            );
        }
    }
    update_buffer(&mut buf, &mut caret, &mut parser, b"\x1B[ A");
    for y in 0..buf.get_buffer_height() {
        if (4..=9).contains(&y) {
            assert_eq!(' ', buf.get_char_xy(4, y).unwrap().ch);
        } else {
            assert_eq!('4', buf.get_char_xy(4, y).unwrap().ch);
        }
    }
}

#[test]
fn test_scroll_up() {
    let mut parser = AnsiParser::new();
    let (mut buf, mut caret) = create_buffer(&mut parser, b"");

    for y in 0..buf.get_buffer_height() {
        for x in 0..buf.get_buffer_width() {
            buf.set_char(
                0,
                Position::new(x, y),
                Some(AttributedChar::new(
                    unsafe { char::from_u32_unchecked((b'0' as i32 + (y % 10)) as u32) },
                    TextAttribute::default(),
                )),
            );
        }
    }
    for x in 0..buf.get_buffer_width() {
        assert_ne!(' ', buf.get_char_xy(x, 24).unwrap().ch);
    }
    update_buffer(&mut buf, &mut caret, &mut parser, b"\x1B[S");
    for x in 0..buf.get_buffer_width() {
        assert_eq!(' ', buf.get_char_xy(x, 24).unwrap().ch);
    }
}

#[test]
fn test_scroll_up_with_margins() {
    let mut parser = AnsiParser::new();
    let (mut buf, mut caret) = create_buffer(&mut parser, b"\x1B[?69h\x1B[5;10r\x1B[5;10s");

    for y in 0..buf.get_buffer_height() {
        for x in 0..buf.get_buffer_width() {
            buf.set_char(
                0,
                Position::new(x, y),
                Some(AttributedChar::new(
                    unsafe { char::from_u32_unchecked((b'0' as i32 + (x % 10)) as u32) },
                    TextAttribute::default(),
                )),
            );
        }
    }
    update_buffer(&mut buf, &mut caret, &mut parser, b"\x1B[S");
    for x in 0..buf.get_buffer_width() {
        if (4..=9).contains(&x) {
            assert_eq!(' ', buf.get_char_xy(x, 9).unwrap().ch);
        } else {
            assert_ne!(' ', buf.get_char_xy(x, 9).unwrap().ch);
        }
    }
}

#[test]
fn test_scroll_down() {
    let mut parser = AnsiParser::new();
    let (mut buf, mut caret) = create_buffer(&mut parser, b"");

    for y in 0..buf.get_buffer_height() {
        for x in 0..buf.get_buffer_width() {
            buf.set_char(
                0,
                Position::new(x, y),
                Some(AttributedChar::new(
                    unsafe { char::from_u32_unchecked((b'0' as i32 + (y % 10)) as u32) },
                    TextAttribute::default(),
                )),
            );
        }
    }
    for x in 0..buf.get_buffer_width() {
        assert_ne!(' ', buf.get_char_xy(x, 0).unwrap().ch);
    }
    update_buffer(&mut buf, &mut caret, &mut parser, b"\x1B[T");
    for x in 0..buf.get_buffer_width() {
        assert_eq!(' ', buf.get_char_xy(x, 0).unwrap().ch);
    }
}

#[test]
fn test_scroll_down_with_margins() {
    let mut parser = AnsiParser::new();
    let (mut buf, mut caret) = create_buffer(&mut parser, b"\x1B[?69h\x1B[5;10r\x1B[5;10s");

    for y in 0..buf.get_buffer_height() {
        for x in 0..buf.get_buffer_width() {
            buf.set_char(
                0,
                Position::new(x, y),
                Some(AttributedChar::new(
                    unsafe { char::from_u32_unchecked((b'0' as i32 + (x % 10)) as u32) },
                    TextAttribute::default(),
                )),
            );
        }
    }
    update_buffer(&mut buf, &mut caret, &mut parser, b"\x1B[T");
    for x in 0..buf.get_buffer_width() {
        if (4..=9).contains(&x) {
            assert_eq!(' ', buf.get_char_xy(x, 4).unwrap().ch);
        } else {
            assert_ne!(' ', buf.get_char_xy(x, 4).unwrap().ch);
        }
    }
}
