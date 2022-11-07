use crate::{ Caret, AnsiParser, Position, TextAttribute, TerminalScrolling, parser::tests::{create_buffer, update_buffer, get_string_from_buffer}, BufferType, convert_to_ans, SaveOptions};

#[test]
fn test_ansi_sequence() {
    let (buf, _) = create_buffer(&mut AnsiParser::new(), b"\x1B[0;40;37mFoo-\x1B[1mB\x1B[0ma\x1B[35mr");

    let ch = buf.get_char(Position::from(0, 0)).unwrap_or_default();
    assert_eq!(b'F', ch.char_code as u8);
    assert_eq!(7, ch.attribute.as_u8(BufferType::LegacyDos));

    let ch = buf.get_char(Position::from(1, 0)).unwrap_or_default();
    assert_eq!(b'o', ch.char_code as u8);
    assert_eq!(7, ch.attribute.as_u8(BufferType::LegacyDos));

    let ch = buf.get_char(Position::from(2, 0)).unwrap_or_default();
    assert_eq!(b'o', ch.char_code as u8);
    assert_eq!(7, ch.attribute.as_u8(BufferType::LegacyDos));

    let ch = buf.get_char(Position::from(3, 0)).unwrap_or_default();
    assert_eq!(b'-', ch.char_code as u8);
    assert_eq!(7, ch.attribute.as_u8(BufferType::LegacyDos));

    let ch = buf.get_char(Position::from(4, 0)).unwrap_or_default();
    assert_eq!(b'B', ch.char_code as u8);
    assert_eq!(15, ch.attribute.as_u8(BufferType::LegacyDos));

    let ch = buf.get_char(Position::from(5, 0)).unwrap_or_default();
    assert_eq!(b'a', ch.char_code as u8);
    assert_eq!(7, ch.attribute.as_u8(BufferType::LegacyDos));

    let ch = buf.get_char(Position::from(6, 0)).unwrap_or_default();
    assert_eq!(b'r', ch.char_code as u8);
    assert_eq!(5, ch.attribute.as_u8(BufferType::LegacyDos));
}


#[test]
fn test_ansi_30() {
    let (buf, _) = create_buffer(&mut AnsiParser::new(), b"\x1B[1;35mA\x1B[30mB\x1B[0mC");
    let ch = buf.get_char(Position::from(0, 0)).unwrap_or_default();
    assert_eq!(b'A', ch.char_code as u8);
    assert_eq!(13, ch.attribute.as_u8(BufferType::LegacyDos));
    let ch = buf.get_char(Position::from(1, 0)).unwrap_or_default();
    assert_eq!(b'B', ch.char_code as u8);
    assert_eq!(8, ch.attribute.as_u8(BufferType::LegacyDos));
    let ch = buf.get_char(Position::from(2, 0)).unwrap_or_default();
    assert_eq!(b'C', ch.char_code as u8);
    assert_eq!(7, ch.attribute.as_u8(BufferType::LegacyDos));
}

#[test]
fn test_bg_colorrsequence() {
    let (buf, _) = create_buffer(&mut AnsiParser::new(), b"\x1B[1;30m1\x1B[0;34m2\x1B[33m3\x1B[1;41m4\x1B[40m5\x1B[43m6\x1B[40m7");
    let ch = buf.get_char(Position::from(0, 0)).unwrap_or_default();
    assert_eq!(b'1', ch.char_code as u8);
    assert_eq!(8, ch.attribute.as_u8(BufferType::LegacyDos));
    let ch = buf.get_char(Position::from(1, 0)).unwrap_or_default();
    assert_eq!(b'2', ch.char_code as u8);
    assert_eq!(1, ch.attribute.as_u8(BufferType::LegacyDos));
    let ch = buf.get_char(Position::from(2, 0)).unwrap_or_default();
    assert_eq!(b'3', ch.char_code as u8);
    assert_eq!(6, ch.attribute.as_u8(BufferType::LegacyDos));
    let ch = buf.get_char(Position::from(3, 0)).unwrap_or_default();
    assert_eq!(b'4', ch.char_code as u8);
    assert_eq!(14 + (4 << 4), ch.attribute.as_u8(BufferType::LegacyDos));
    let ch = buf.get_char(Position::from(4, 0)).unwrap_or_default();
    assert_eq!(b'5', ch.char_code as u8);
    assert_eq!(14, ch.attribute.as_u8(BufferType::LegacyDos));
    let ch = buf.get_char(Position::from(5, 0)).unwrap_or_default();
    assert_eq!(b'6', ch.char_code as u8);
    assert_eq!(14 + (6 << 4), ch.attribute.as_u8(BufferType::LegacyDos));
    let ch = buf.get_char(Position::from(6, 0)).unwrap_or_default();
    assert_eq!(b'7', ch.char_code as u8);
    assert_eq!(14, ch.attribute.as_u8(BufferType::LegacyDos));
}
#[test]
fn test_char_missing_bug() {
    let (buf, _) = create_buffer(&mut AnsiParser::new(), b"\x1B[1;35mA\x1B[30mB\x1B[0mC");
    
    let ch = buf.get_char(Position::from(0, 0)).unwrap_or_default();
    assert_eq!(b'A', ch.char_code as u8);
    assert_eq!(13, ch.attribute.as_u8(BufferType::LegacyDos));
    let ch = buf.get_char(Position::from(1, 0)).unwrap_or_default();
    assert_eq!(b'B', ch.char_code as u8);
    assert_eq!(8, ch.attribute.as_u8(BufferType::LegacyDos));
    let ch = buf.get_char(Position::from(2, 0)).unwrap_or_default();
    assert_eq!(b'C', ch.char_code as u8);
    assert_eq!(7, ch.attribute.as_u8(BufferType::LegacyDos));
}

#[test]
fn test_caret_forward() {
    let (buf, _) = create_buffer(&mut AnsiParser::new(), b"\x1B[70Ctest_me\x1B[2CF");
    let ch = buf.get_char(Position::from(79, 0)).unwrap_or_default();
    assert_eq!('F', char::from_u32(ch.char_code as u32).unwrap());
}

#[test]
fn test_caret_forward_at_eol() {
    let (buf, _) = create_buffer(&mut AnsiParser::new(), b"\x1B[75CTEST_\x1B[2CF");
    let ch = buf.get_char(Position::from(2, 1)).unwrap_or_default();
    assert_eq!(b'F', ch.char_code as u8);
}

#[test]
fn test_char0_bug() {
    let (buf, _) = create_buffer(&mut AnsiParser::new(), b"\x00A");
    let ch = buf.get_char(Position::from(0, 0)).unwrap_or_default();
    assert_eq!(b'A', ch.char_code as u8);
}

fn test_ansi(data: &[u8])
{
    let (buf, _) = create_buffer(&mut AnsiParser::new(), data);
    let converted = convert_to_ans(&buf, &crate::SaveOptions::new()).unwrap();

    // more gentle output.
    let b : Vec<u8> = converted.iter().map(|&x| if x == 27 { b'x' } else { x }).collect();
    let converted  = String::from_utf8_lossy(b.as_slice());

    let b : Vec<u8> = data.iter().map(|&x| if x == 27 { b'x' } else { x }).collect();
    let expected  = String::from_utf8_lossy(b.as_slice());

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
    assert_eq!(0x16, buf.get_char(Position {x: 1, y: 0}).unwrap_or_default().char_code);
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
    assert_eq!(b' ' , buf.get_char(Position::new()).unwrap().char_code as u8);
}

#[test]
fn test_remove_n_line() {
    let (mut buf, _) = create_buffer(&mut AnsiParser::new(), b"test\ntest\ntest\ntest");
    for i in 0..4  {
        assert_eq!(b't' , buf.get_char(Position::from(0, i)).unwrap().char_code as u8);
    }
    update_buffer(&mut buf, &mut Caret::new(), &mut AnsiParser::new(), b"\x1b[3M");
    assert_eq!(b't' , buf.get_char(Position::from(0, 0)).unwrap().char_code as u8);
    assert_eq!(b' ' , buf.get_char(Position::from(0, 1)).unwrap().char_code as u8);
}

#[test]
fn test_delete_character_default() {
    let (mut buf, _) = create_buffer(&mut AnsiParser::new(), b"test");
    update_buffer(&mut buf, &mut &mut Caret::from_xy(0, 0), &mut AnsiParser::new(), b"\x1b[P");
    assert_eq!(b'e' , buf.get_char(Position::from(0, 0)).unwrap().char_code as u8);
    update_buffer(&mut buf, &mut &mut Caret::from_xy(0, 0), &mut AnsiParser::new(), b"\x1b[P");
    assert_eq!(b's' , buf.get_char(Position::from(0, 0)).unwrap().char_code as u8);
    update_buffer(&mut buf, &mut &mut Caret::from_xy(0, 0), &mut AnsiParser::new(), b"\x1b[P");
    assert_eq!(b't' , buf.get_char(Position::from(0, 0)).unwrap().char_code as u8);
}

#[test]
fn test_delete_n_character() {
    let (mut buf, _) = create_buffer(&mut AnsiParser::new(), b"testme");
    update_buffer(&mut buf, &mut &mut Caret::from_xy(0, 0), &mut AnsiParser::new(), b"\x1b[4P");
    assert_eq!(b'm' , buf.get_char(Position::from(0, 0)).unwrap().char_code as u8);
}

#[test]
fn test_save_cursor() {
    let (_,caret) = create_buffer(&mut AnsiParser::new(), b"\x1b7testme\x1b8");
    assert_eq!(Position::new(), caret.get_position());
}

#[test]
fn test_save_cursor_more_times() {
    let (_,caret) = create_buffer(&mut AnsiParser::new(), b"\x1b7testme\x1b8testme\x1b8");
    assert_eq!(Position::new(), caret.get_position());
}

#[test]
fn test_reset_cursor() {
    let (mut buf, mut caret) = create_buffer(&mut AnsiParser::new(), b"testme\x1b[1;37m");
    assert_ne!(TextAttribute::DEFAULT, caret.attr);
    assert_ne!(Position::new(), caret.get_position());
    update_buffer(&mut buf, &mut caret, &mut AnsiParser::new(), b"\x1bc");
    assert_eq!(TextAttribute::DEFAULT, caret.attr);
    assert_eq!(Position::new(), caret.get_position());
}

#[test]
fn test_cursor_visibilty() {
    let (mut buf, mut caret) = create_buffer(&mut AnsiParser::new(), b"\x1b[?25l");
    assert_eq!(false, caret.is_visible);
    update_buffer(&mut buf, &mut caret, &mut AnsiParser::new(), b"\x1b[?25h");
    assert_eq!(true, caret.is_visible);
}

#[test]
fn test_cursor_visibilty_reset() {
    let (mut buf, mut caret) = create_buffer(&mut AnsiParser::new(), b"\x1b[?25l");
    assert_eq!(false, caret.is_visible);
    update_buffer(&mut buf, &mut caret, &mut AnsiParser::new(), b"\x0C"); // FF
    assert_eq!(true, caret.is_visible);
}

#[test]
fn test_vert_line_position_absolute_default() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"\n\n\nfoo\x1b[d");
    assert_eq!(Position::from(3, 0) , caret.get_position());
}

#[test]
fn test_vert_line_position_absolute_n() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"test\x1b[5d");
    assert_eq!(Position::from(4, 4), caret.get_position());
}

#[test]
fn test_vert_line_position_relative_default() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"\n\n\nfoo\x1b[e");
    assert_eq!(Position::from(3, 4) , caret.get_position());
}

#[test]
fn test_vert_line_position_relative_n() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"\n\n\x1b[5e");
    assert_eq!(Position::from(0, 7) , caret.get_position());
}

#[test]
fn test_horiz_line_position_absolute_default() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"foo\x1b['");
    assert_eq!(Position::new(), caret.get_position());
}

#[test]
fn test_horiz_line_position_absolute_n() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"testfooo\x1b['\x1b[3'");
    assert_eq!(Position::from(2, 0) , caret.get_position());
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"01234567\x1b['\x1b[100'");
    assert_eq!(Position::from(8, 0) , caret.get_position());
}

#[test]
fn test_horiz_line_position_relative_default() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"testfooo\x1b['\x1b[a");
    assert_eq!(Position::from(1, 0) , caret.get_position());
}

#[test]
fn test_horiz_line_position_relative_n() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"testfooo\x1b['\x1b[3a");
    assert_eq!(Position::from(3, 0) , caret.get_position());
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"01234567\x1b['\x1b[100a");
    assert_eq!(Position::from(8, 0) , caret.get_position());
}

#[test]
fn test_cursor_horiz_absolute_default() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"testfooo\x1b[G");
    assert_eq!(Position::from(0, 0) , caret.get_position());
}

#[test]
fn test_cursor_horiz_absolute_n() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"testfooo\x1b['\x1b[3G");
    assert_eq!(Position::from(2, 0) , caret.get_position());
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"01234567\x1b['\x1b[100G");
    assert_eq!(Position::from(79, 0) , caret.get_position());
}

#[test]
fn test_cursor_next_line_default() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"\n\n\nfoo\x1b[E");
    assert_eq!(Position::from(0, 4) , caret.get_position());
}

#[test]
fn test_cursor_next_line_n() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"test\x1b[5E");
    assert_eq!(Position::from(0, 5), caret.get_position());
}

#[test]
fn test_cursor_previous_line_default() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"\n\n\nfoo\x1b[F");
    assert_eq!(Position::from(0, 2) , caret.get_position());
}

#[test]
fn test_cursor_previous_line_n() {
    let (_, caret) = create_buffer(&mut AnsiParser::new(), b"\n\n\nfoo\x1b[2F");
    assert_eq!(Position::from(0, 1), caret.get_position());
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
    let (buf, _) = create_buffer(&mut AnsiParser::new(), b"\x1B[m\x1B[33mN\x1B[1m\x1B[33ma\x1B[m\x1B[33mCHR\x1B[1m\x1B[33mi\x1B[m\x1B[33mCHT");
    assert_eq!(buf.get_char(Position::from(0, 0)).unwrap().attribute, buf.get_char(Position::from(2, 0)).unwrap().attribute);
    assert_eq!(buf.get_char(Position::from(1, 0)).unwrap().attribute, buf.get_char(Position::from(5, 0)).unwrap().attribute);
    assert_eq!(buf.get_char(Position::from(2, 0)).unwrap().attribute, buf.get_char(Position::from(8, 0)).unwrap().attribute);
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
    let (buf, _) = create_buffer(&mut AnsiParser::new(), b"test\x1B[H\x1B[4lhelp\x1B[H\x1B[4hnewtest");
    let converted = crate::convert_to_asc(&buf, &SaveOptions::new()).unwrap();

    // more gentle output.
    let b : Vec<u8> = converted.iter().map(|&x| if x == 27 { b'x' } else { x }).collect();
    let converted  = String::from_utf8_lossy(b.as_slice());
    assert_eq!("newtesthelp", converted);
}

#[test]
fn test_index_line() {
    let (buf, caret) = create_buffer(&mut AnsiParser::new(), b"test\x1BD\x1BD\x1BD");
    assert_eq!(Position::from(4, 3) , caret.get_position());
}


#[test]
fn test_reverse_index_line() {
    let (buf, caret) = create_buffer(&mut AnsiParser::new(), b"test\x1BM\x1BM\x1BM");
    assert_eq!(Position::from(4, 0) , caret.get_position());
    let ch = buf.get_char(Position::from(0, 3)).unwrap_or_default();
    assert_eq!(b't', ch.char_code as u8);
}

#[test]
fn test_next_line() {
    let (buf, caret) = create_buffer(&mut AnsiParser::new(), b"\x1B[25;1Htest\x1BE\x1BE\x1BE");
    assert_eq!(Position::from(0, 24) , caret.get_position());
    let ch = buf.get_char(Position::from(0, 24 - 3)).unwrap_or_default();
    assert_eq!(b't', ch.char_code as u8);
}
