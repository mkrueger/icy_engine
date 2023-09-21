use super::BufferParser;
use crate::{
    AttributedChar, Buffer, CallbackAction, Caret, EngineResult, ParserError, TextAttribute,
};
use std::cmp::{max, min};

#[derive(Debug)]
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

pub struct Parser {
    ansi_parser: super::ansi::Parser,

    avt_state: AvtReadState,
    avatar_state: i32,
    avt_repeat_char: char,
}

impl Default for Parser {
    fn default() -> Self {
        Self {
            ansi_parser: super::ansi::Parser::default(),
            avatar_state: 0,
            avt_state: AvtReadState::Chars,
            avt_repeat_char: ' ',
        }
    }
}

impl Parser {
    fn print_fallback(
        &mut self,
        buf: &mut Buffer,
        current_layer: usize,
        caret: &mut Caret,
        ch: char,
    ) -> EngineResult<CallbackAction> {
        self.ansi_parser.print_char(buf, current_layer, caret, ch)
    }
}

impl BufferParser for Parser {
    fn convert_from_unicode(&self, ch: char, font_page: usize) -> char {
        self.ansi_parser.convert_from_unicode(ch, font_page)
    }

    fn convert_to_unicode(&self, attributed_char: AttributedChar) -> char {
        self.ansi_parser.convert_to_unicode(attributed_char)
    }

    fn print_char(
        &mut self,
        buf: &mut Buffer,
        current_layer: usize,
        caret: &mut Caret,
        ch: char,
    ) -> EngineResult<CallbackAction> {
        match self.avt_state {
            AvtReadState::Chars => {
                match ch {
                    AVT_CLR => caret.ff(buf, current_layer), // clear & reset attributes
                    AVT_REP => {
                        self.avt_state = AvtReadState::RepeatChars;
                        self.avatar_state = 1;
                    }
                    AVT_CMD => {
                        self.avt_state = AvtReadState::ReadCommand;
                    }
                    _ => return self.print_fallback(buf, current_layer, caret, ch),
                }
                Ok(CallbackAction::None)
            }
            AvtReadState::ReadCommand => {
                match ch as u16 {
                    1 => {
                        self.avt_state = AvtReadState::ReadColor;
                        return Ok(CallbackAction::None);
                    }
                    2 => {
                        caret.attribute.set_is_blinking(true);
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
                        return Err(ParserError::Description("todo: avt cleareol").into());
                    }
                    8 => {
                        self.avt_state = AvtReadState::MoveCursor;
                        self.avatar_state = 1;
                        return Ok(CallbackAction::None);
                    }
                    // TODO implement commands from FSC0025.txt & FSC0037.txt
                    _ => {
                        self.avt_state = AvtReadState::Chars;
                        return Err(ParserError::Description("unsupported avatar command").into());
                    }
                }
                self.avt_state = AvtReadState::Chars;
                Ok(CallbackAction::None)
            }
            AvtReadState::RepeatChars => match self.avatar_state {
                1 => {
                    self.avt_repeat_char = ch;
                    self.avatar_state = 2;
                    Ok(CallbackAction::None)
                }
                2 => {
                    self.avatar_state = 3;
                    let repeat_count = ch as usize;
                    for _ in 0..repeat_count {
                        self.ansi_parser.print_char(
                            buf,
                            current_layer,
                            caret,
                            self.avt_repeat_char,
                        )?;
                    }
                    self.avt_state = AvtReadState::Chars;
                    Ok(CallbackAction::None)
                }
                _ => {
                    self.avt_state = AvtReadState::Chars;
                    Err(ParserError::Description("error in reading avt state").into())
                }
            },
            AvtReadState::ReadColor => {
                caret.attribute = TextAttribute::from_u8(ch as u8, buf.ice_mode);
                self.avt_state = AvtReadState::Chars;
                Ok(CallbackAction::None)
            }
            AvtReadState::MoveCursor => match self.avatar_state {
                1 => {
                    self.avt_repeat_char = ch;
                    self.avatar_state = 2;
                    Ok(CallbackAction::None)
                }
                2 => {
                    caret.pos.x = self.avt_repeat_char as i32;
                    caret.pos.y = ch as i32;

                    self.avt_state = AvtReadState::Chars;
                    Ok(CallbackAction::None)
                }
                _ => Err(ParserError::Description("error in reading avt avt_gotoxy").into()),
            },
        }
    }
}
