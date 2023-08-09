use crate::{
    convert_to_ans,
    parsers::tests::{create_buffer, get_action, update_buffer},
    AnsiMusic, AnsiMusicOption, AnsiParser, BufferType, CallbackAction, Caret, Color, MusicAction,
    Position, SaveOptions, TerminalScrolling, TextAttribute, XTERM_256_PALETTE, Buffer,
};

#[test]
fn test_simple_sixel() {
    let (buf, _) = create_buffer(&mut AnsiParser::new(), b"\x1BPq#0;2;0;0;0#1;2;100;100;0#2;2;0;100;0#1~~@@vv@@~~@@~~$43#2??}}GG}}??}}??-#1!14@\x1B\\");
    assert_eq!(1, buf.layers[0].sixels.len());
    assert_eq!(Position::new(0, 0), buf.layers[0].sixels[0].position);
    assert_eq!(14, buf.layers[0].sixels[0].width());
    assert_eq!(12, buf.layers[0].sixels[0].height());
}

#[test]
fn test_simple_position_sixel() {
    let (buf, _) = create_buffer(&mut AnsiParser::new(), b"\x1B[4;13H\x1BPq#0;2;0;0;0#1;2;100;100;0#2;2;0;100;0#1~~@@vv@@~~@@~~$43#2??}}GG}}??}}??-#1!14@\x1B\\");
    assert_eq!(1, buf.layers[0].sixels.len());
    assert_eq!(Position::new(12, 3), buf.layers[0].sixels[0].position);
    assert_eq!(14, buf.layers[0].sixels[0].width());
    assert_eq!(12, buf.layers[0].sixels[0].height());
}

#[test]
fn test_overwrite_sixel() {
    let (mut buf, mut caret) = create_buffer(&mut AnsiParser::new(), b"\x1B[4;13H\x1BPq#0;2;0;0;0#1;2;100;100;0#2;2;0;100;0#1~~@@vv@@~~@@~~$43#2??}}GG}}??}}??-#1!14@\x1B\\");
    update_buffer(&mut buf, &mut caret, &mut AnsiParser::new(), b"\x1B[4;13H\x1BPq#0;2;0;0;0#1;2;100;100;0#2;2;0;100;0#1~~@@vv@@~~@@~~$43#2??}}GG}}??}}??-#1!14@\x1B\\");
    update_buffer(&mut buf, &mut caret, &mut AnsiParser::new(), b"\x1B[4;13H\x1BPq#0;2;0;0;0#1;2;100;100;0#2;2;0;100;0#1~~@@vv@@~~@@~~$43#2??}}GG}}??}}??-#1!14@\x1B\\");
    update_buffer(&mut buf, &mut caret, &mut AnsiParser::new(), b"\x1B[4;13H\x1BPq#0;2;0;0;0#1;2;100;100;0#2;2;0;100;0#1~~@@vv@@~~@@~~$43#2??}}GG}}??}}??-#1!14@\x1B\\");
    update_buffer(&mut buf, &mut caret, &mut AnsiParser::new(), b"\x1B[4;13H\x1BPq#0;2;0;0;0#1;2;100;100;0#2;2;0;100;0#1~~@@vv@@~~@@~~$43#2??}}GG}}??}}??-#1!14@\x1B\\");
    update_buffer(&mut buf, &mut caret, &mut AnsiParser::new(), b"\x1B[4;13H\x1BPq#0;2;0;0;0#1;2;100;100;0#2;2;0;100;0#1~~@@vv@@~~@@~~$43#2??}}GG}}??}}??-#1!14@\x1B\\");

    assert_eq!(1, buf.layers[0].sixels.len());
    assert_eq!(Position::new(12, 3), buf.layers[0].sixels[0].position);
    assert_eq!(14, buf.layers[0].sixels[0].width());
    assert_eq!(12, buf.layers[0].sixels[0].height());
}
