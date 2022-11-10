use std::io;
use crate::{Buffer, Caret};
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

impl BufferParser for AtasciiParser {
    fn from_unicode(&self, ch: char) -> char
    {
        if let Some(tch) = UNICODE_TO_ATARI.get(&ch) {
            *tch
        } else {
            ch 
        }
    }

    fn to_unicode(&self, ch: char) -> char
    {
        if (ch as usize) < ATARI_TO_UNICODE.len() {
            ATARI_TO_UNICODE[ch as usize]
        } else {
            ch
        }
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

lazy_static::lazy_static!{
    static ref UNICODE_TO_ATARI: std::collections::HashMap<char, char> = {
        let mut res = std::collections::HashMap::new();
        for a in 0..128 { 
            res.insert(ATARI_TO_UNICODE[a], char::from_u32(a as u32).unwrap());
        }
        res
    };
}

pub const ATARI_TO_UNICODE: [char; 256] = [
    '♥', '├', '🮇', '┘', '┤', '┐', '╱', '╲', '◢', '▗', '◣', '▝', '▘', '🮂', '▂', '▖', 
    '♣', '┌', '─', '┼', '•', '▄', '▎', '┬', '┴', '▌', '└', '␛', '↑', '↓', '←', '→', 
    ' ', '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/', 
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?', 
    '@', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 
    'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '[', '\\', ']', '^', '_', 
    '♦', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 
    'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '♠', '|', '🢰','◀', '▶', 
    '♥', '├', '▊', '┘', '┤', '┐', '╱', '╲', '◤', '▛', '◥', '▙', '▟', '▆', '▂', '▜', 
    '♣', '┌', '─', '┼', '•', '▀', '▎', '┬', '┴', '▐', '└', '\x08', '↑', '↓', '←', '→', 
    '█', '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/', 
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?', 
    '@', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 
    'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '[', '\\', ']', '^', '_', 
    '♦', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 
    'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '♠', '-', '🢰', '◀', '▶'
];