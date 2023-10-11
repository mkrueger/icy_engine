use super::{ansi, BufferParser};
use crate::{AttributedChar, Buffer, CallbackAction, Caret, EngineResult};

#[derive(Default, Clone, Copy, PartialEq)]
enum State {
    #[default]
    Normal,
    ParseFirstColor,
    ParseSecondColor(u8),
}

#[derive(Default)]
pub struct Parser {
    ansi_parser: ansi::Parser,
    state: State,
}

impl BufferParser for Parser {
    fn convert_from_unicode(&self, ch: char, font_page: usize) -> char {
        self.ansi_parser.convert_from_unicode(ch, font_page)
    }

    fn convert_to_unicode(&self, attributed_char: AttributedChar) -> char {
        self.ansi_parser.convert_to_unicode(attributed_char)
    }

    fn print_char(&mut self, buf: &mut Buffer, current_layer: usize, caret: &mut Caret, ch: char) -> EngineResult<CallbackAction> {
        match self.state {
            State::Normal => match ch {
                '|' => {
                    self.state = State::ParseFirstColor;
                    Ok(CallbackAction::NoUpdate)
                }
                _ => self.ansi_parser.print_char(buf, current_layer, caret, ch),
            },
            State::ParseFirstColor => {
                let code = ch as u8;
                if !(b'0'..=b'3').contains(&code) {
                    self.state = State::Normal;
                    return Err(anyhow::anyhow!("Invalid color code: {}", ch));
                }
                self.state = State::ParseSecondColor((code - b'0') * 10);
                Ok(CallbackAction::NoUpdate)
            }
            State::ParseSecondColor(first) => {
                self.state = State::Normal;

                let code = ch as u8;
                if !(b'0'..=b'9').contains(&code) {
                    return Err(anyhow::anyhow!("Invalid color code: {}", ch));
                }
                let color = first + (code - b'0');
                if color < 16 {
                    caret.attribute.set_foreground(color as u32);
                } else {
                    caret.attribute.set_background((color - 16) as u32);
                }
                Ok(CallbackAction::NoUpdate)
            }
        }
    }
}
