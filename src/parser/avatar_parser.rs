use super::BufferParser;
use crate::{
    AnsiParser, AsciiParser, Buffer, CallbackAction, Caret, EngineResult, ParserError,
    TextAttribute,
};
use std::cmp::{max, min};

enum AvtReadState {
    Chars,
    RepeatChars,
    ReadCommand,
    MoveCursor,
    ReadColor,
}

/// Starts Avatar command
const AVT_CMD: char = '\x16';
/// clear the current window and set current attribute to default.
const AVT_CLR: char = '\x0C';
///  Read two bytes from the modem. Send the first one to the screen as many times as the binary value
///  of the second one. This is the exception where the two bytes may have their high bit set. Do not reset it here!
const AVT_REP: char = '\x19';

pub struct AvatarParser {
    ascii_parser: AsciiParser,
    ansi_parser: AnsiParser,

    use_ansi_parser: bool,

    avt_state: AvtReadState,
    avatar_state: i32,
    avt_repeat_char: char,
}

impl AvatarParser {
    pub fn new(use_ansi_parser: bool) -> Self {
        Self {
            ascii_parser: AsciiParser::new(),
            ansi_parser: AnsiParser::new(),
            use_ansi_parser,

            avatar_state: 0,
            avt_state: AvtReadState::Chars,
            avt_repeat_char: ' ',
        }
    }

    fn print_fallback(
        &mut self,
        buf: &mut Buffer,
        caret: &mut Caret,
        ch: char,
    ) -> EngineResult<CallbackAction> {
        if self.use_ansi_parser {
            self.ansi_parser.print_char(buf, caret, ch)
        } else {
            self.ascii_parser.print_char(buf, caret, ch)
        }
    }
}

impl BufferParser for AvatarParser {
    fn from_unicode(&self, ch: char) -> char {
        self.ascii_parser.from_unicode(ch)
    }

    fn to_unicode(&self, ch: char) -> char {
        self.ascii_parser.to_unicode(ch)
    }

    fn print_char(
        &mut self,
        buf: &mut Buffer,
        caret: &mut Caret,
        ch: char,
    ) -> EngineResult<CallbackAction> {
        match self.avt_state {
            AvtReadState::Chars => {
                match ch {
                    AVT_CLR => caret.ff(buf), // clear & reset attributes
                    AVT_REP => {
                        self.avt_state = AvtReadState::RepeatChars;
                        self.avatar_state = 1;
                    }
                    AVT_CMD => {
                        self.avt_state = AvtReadState::ReadCommand;
                    }
                    _ => return self.print_fallback(buf, caret, ch),
                }
                return Ok(CallbackAction::None);
            }
            AvtReadState::ReadCommand => {
                match ch as u16 {
                    1 => {
                        self.avt_state = AvtReadState::ReadColor;
                        return Ok(CallbackAction::None);
                    }
                    2 => {
                        caret.attr.set_is_blinking(true);
                    }
                    3 => {
                        caret.pos.y = max(0, caret.pos.y - 1);
                    }
                    4 => {
                        caret.pos.y += 1;
                    }

                    5 => {
                        caret.pos.x = max(0, caret.pos.x - 1);
                    }
                    6 => {
                        caret.pos.x = min(79, caret.pos.x + 1);
                    }
                    7 => {
                        return Err(Box::new(ParserError::Description("todo: avt cleareol")));
                    }
                    8 => {
                        self.avt_state = AvtReadState::MoveCursor;
                        self.avatar_state = 1;
                        return Ok(CallbackAction::None);
                    }
                    // TODO implement commands from FSC0025.txt & FSC0037.txt
                    _ => {
                        self.avt_state = AvtReadState::Chars;
                        return Err(Box::new(ParserError::Description(
                            "unsupported avatar command",
                        )));
                    }
                }
                self.avt_state = AvtReadState::Chars;
                return Ok(CallbackAction::None);
            }
            AvtReadState::RepeatChars => match self.avatar_state {
                1 => {
                    self.avt_repeat_char = ch;
                    self.avatar_state = 2;
                    return Ok(CallbackAction::None);
                }
                2 => {
                    self.avatar_state = 3;
                    let repeat_count = ch as usize;
                    for _ in 0..repeat_count {
                        self.ascii_parser
                            .print_char(buf, caret, self.avt_repeat_char)?;
                    }
                    self.avt_state = AvtReadState::Chars;
                    return Ok(CallbackAction::None);
                }
                _ => {
                    self.avt_state = AvtReadState::Chars;
                    return Err(Box::new(ParserError::Description(
                        "error in reading avt state",
                    )));
                }
            },
            AvtReadState::ReadColor => {
                caret.attr = TextAttribute::from_u8(ch as u8, buf.buffer_type);
                self.avt_state = AvtReadState::Chars;
                return Ok(CallbackAction::None);
            }
            AvtReadState::MoveCursor => match self.avatar_state {
                1 => {
                    self.avt_repeat_char = ch;
                    self.avatar_state = 2;
                    return Ok(CallbackAction::None);
                }
                2 => {
                    caret.pos.x = self.avt_repeat_char as i32;
                    caret.pos.y = ch as i32;

                    self.avt_state = AvtReadState::Chars;
                    return Ok(CallbackAction::None);
                }
                _ => {
                    return Err(Box::new(ParserError::Description(
                        "error in reading avt avt_gotoxy",
                    )));
                }
            },
        }
    }
}
