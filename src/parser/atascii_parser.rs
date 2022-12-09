use super::BufferParser;
use crate::{Buffer, CallbackAction, Caret, EngineResult};

pub struct AtasciiParser {
    got_escape: bool,
}

impl AtasciiParser {
    pub fn new() -> Self {
        Self { got_escape: false }
    }
}

impl BufferParser for AtasciiParser {
    fn from_unicode(&self, ch: char) -> char {
        match UNICODE_TO_ATARI.get(&ch) {
            Some(out_ch) => *out_ch,
            _ => ch,
        }
    }

    fn to_unicode(&self, ch: char) -> char {
        match ATARI_TO_UNICODE.get(ch as usize) {
            Some(out_ch) => *out_ch,
            _ => ch,
        }
    }

    fn print_char(
        &mut self,
        buf: &mut Buffer,
        caret: &mut Caret,
        ch: char,
    ) -> EngineResult<CallbackAction> {
        if self.got_escape {
            self.got_escape = false;
            buf.print_value(caret, ch as u16);
            return Ok(CallbackAction::None);
        }

        match ch {
            '\x1C' => caret.up(buf, 1),
            '\x1D' => caret.down(buf, 1),
            '\x1E' => caret.left(buf, 1),
            '\x1F' => caret.right(buf, 1),
            '\x7D' => buf.clear_screen(caret),
            '\x7E' => caret.bs(buf),
            '\x7F' => { /* TAB TODO */ }
            '\u{009B}' => caret.lf(buf),
            '\u{009C}' => buf.remove_terminal_line(caret.pos.y),
            '\u{009D}' => buf.insert_terminal_line(caret.pos.y),
            '\u{009E}' => { /* clear TAB stops TODO */ }
            '\u{009F}' => { /* set TAB stops TODO */ }
            '\u{00FD}' => return Ok(CallbackAction::Beep),
            '\u{00FE}' => caret.del(buf),
            '\u{00FF}' => caret.ins(buf),
            '\x1B' => {
                self.got_escape = true;
            }
            _ => buf.print_value(caret, ch as u16),
        }
        Ok(CallbackAction::None)
    }
}

lazy_static::lazy_static! {
    static ref UNICODE_TO_ATARI: std::collections::HashMap<char, char> = {
        let mut res = std::collections::HashMap::new();
        for a in 0..128 {
            if let Some(ch) = char::from_u32(a as u32) {
                res.insert(ATARI_TO_UNICODE[a], ch);
            }
        }
        res
    };
}

pub const ATARI_TO_UNICODE: [char; 256] = [
    '♥', '├', '🮇', '┘', '┤', '┐', '╱', '╲', '◢', '▗', '◣', '▝', '▘', '🮂', '▂', '▖', '♣', '┌', '─',
    '┼', '•', '▄', '▎', '┬', '┴', '▌', '└', '␛', '↑', '↓', '←', '→', ' ', '!', '"', '#', '$', '%',
    '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/', '0', '1', '2', '3', '4', '5', '6', '7', '8',
    '9', ':', ';', '<', '=', '>', '?', '@', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K',
    'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '[', '\\', ']', '^',
    '_', '♦', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q',
    'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '♠', '|', '🢰', '◀', '▶', '♥', '├', '▊', '┘', '┤',
    '┐', '╱', '╲', '◤', '▛', '◥', '▙', '▟', '▆', '▂', '▜', '♣', '┌', '─', '┼', '•', '▀', '▎', '┬',
    '┴', '▐', '└', '\x08', '↑', '↓', '←', '→', '█', '!', '"', '#', '$', '%', '&', '\'', '(', ')',
    '*', '+', ',', '-', '.', '/', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '<',
    '=', '>', '?', '@', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
    'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '[', '\\', ']', '^', '_', '♦', 'a', 'b',
    'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u',
    'v', 'w', 'x', 'y', 'z', '♠', '-', '🢰', '◀', '▶',
];
