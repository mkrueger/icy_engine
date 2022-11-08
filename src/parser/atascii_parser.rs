use std::io;
use crate::{Buffer, Caret, AsciiParser};
use super::BufferParser;

pub struct AtasciiParser {
    got_escape: bool,
}

impl AtasciiParser {
    pub fn new() -> Self {
        Self {
            got_escape: false
        }
    }
}

const ATASCII_CR: u8 = 155;

impl BufferParser for AtasciiParser {
    fn from_unicode(&self, ch: char) -> u8
    {
        if ch == '\r' {
            return ATASCII_CR;
        }
         ch as u8
    }

    fn to_unicode(&self, ch: u16) -> char
    {
        // TODO
        AsciiParser::new().to_unicode(ch)
    }

    fn print_char(&mut self, buf: &mut Buffer, caret: &mut Caret, ch: u8) -> io::Result<Option<String>> {

        if self.got_escape {
            self.got_escape = false;
            buf.print_value(caret, ch as u16);
            return Ok(None);
        }

        match ch {
            0x1C => caret.up(buf, 1),
            0x1D => caret.down(buf, 1),
            0x1E => caret.left(buf, 1),
            0x1F => caret.right(buf, 1),
            0x7D => buf.clear_screen(caret),
            0x7E => caret.bs(buf),
            0x7F => { /* TAB TODO */ },
            0x9B => caret.lf(buf),
            0x9C => buf.remove_terminal_line(caret.pos.y),
            0x9D => buf.insert_terminal_line(caret.pos.y),
            0x9E => { /* clear TAB stops TODO */ },
            0x9F => { /* set TAB stops TODO */ },
            0xFD => { /* Buzzer TODO */ },
            0xFE => caret.del(buf),
            0xFF => caret.ins(buf),
            0x1B => {
                self.got_escape = true;
            }
            _ => buf.print_value(caret, ch as u16)
        }
        Ok(None)
    }
}