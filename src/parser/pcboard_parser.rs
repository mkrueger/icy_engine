use std::{io};
use crate::{Buffer, Caret,  AnsiParser, TextAttribute};
use super::BufferParser;

#[allow(clippy::struct_excessive_bools)]
pub struct PCBoardParser {
    ansi_parser: AnsiParser,

    // PCB
    pub pcb_code: bool,
    pub pcb_color: bool,
    pub pcb_value: u8,
    pub pcb_pos: i32,
}

impl PCBoardParser {
    pub fn new() -> Self {
        PCBoardParser {
            ansi_parser: AnsiParser::new(),
            pcb_code: false,
            pcb_color: false,
            pcb_value: 0,
            pcb_pos: 0,
        }
    }
}

impl BufferParser for PCBoardParser {
    fn from_unicode(&self, ch: char) -> u8
    {
        self.ansi_parser.from_unicode(ch)
    }

    fn to_unicode(&self, ch: u16) -> char
    {
        self.ansi_parser.to_unicode(ch)
    }
    
    fn print_char(&mut self, buf: &mut Buffer, caret: &mut Caret, ch: u8) -> io::Result<Option<String>> {
        if self.pcb_color {
            self.pcb_pos += 1;
            if self.pcb_pos < 3 {
                match self.pcb_pos {
                    1 => {
                        self.pcb_value = conv_ch(ch);
                        return Ok(None);
                    }
                    2 => {
                        self.pcb_value = (self.pcb_value << 4) + conv_ch(ch);
                        caret.attr = TextAttribute::from_u8(self.pcb_value, buf.buffer_type);
                    }
                    _ => {}
                }
            }
            self.pcb_color = false;
            self.pcb_code = false;
            return Ok(None);
        }
        
        if self.pcb_code {
            match ch {
                b'@' => {
                    self.pcb_code = false;
                }
                b'X' => {
                    self.pcb_color = true;
                    self.pcb_pos = 0;
                }
                _ => {}
            }
            return Ok(None);
        }
        match ch {
            b'@' => {
                self.pcb_code = true;
                Ok(None)
            }
            _ => self.ansi_parser.print_char(buf, caret, ch),
        }
    }
}

fn conv_ch(ch: u8) -> u8 {
    if (b'0'..=b'9').contains(&ch) {
        return ch - b'0';
    }
    if (b'a'..=b'f').contains(&ch) {
        return 10 + ch - b'a';
    }
    if (b'A'..=b'F').contains(&ch) {
        return 10 + ch - b'A';
    }
    0
}
