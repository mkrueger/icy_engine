use std::{io, cmp::{max, min}};
use super::{Buffer, Caret, Position, DosChar};
mod ascii_parser;
pub use ascii_parser::*;
mod ansi_parser;
pub use ansi_parser::*;
mod avatar_parser;
pub use avatar_parser::*;
mod petscii_parser;
pub use petscii_parser::*;
mod pcboard_parser;
pub use pcboard_parser::*;

pub trait BufferParser {

    fn from_unicode(&self, ch: char) -> u8;

    /// Prints a character to the buffer. Gives back an optional string returned to the sender (in case for terminals).
    fn print_char(&mut self, buffer: &mut Buffer, caret: &mut Caret, c: u8) -> io::Result<Option<String>>;
}


fn fill_line(buf: &mut Buffer, line:i32, from: i32, to: i32) {
    for x in from..=to {
        let p = Position::from(x, line);
        if buf.get_char(p).is_none() {
            buf.set_char( 0, p, Some(DosChar::new()));
        }
    }
}

impl Caret {
    /// (line feed, LF, \n, ^J), moves the print head down one line, or to the left edge and down. Used as the end of line marker in most UNIX systems and variants.
    pub fn lf(&mut self, buf: &mut Buffer) {
        self.pos.x = 0;
        self.pos.y += 1;
        if self.pos.y >= buf.height {
            fill_line(buf, self.pos.y, 0, buf.width as i32);
        }
    }
    
    /// (form feed, FF, \f, ^L), to cause a printer to eject paper to the top of the next page, or a video terminal to clear the screen.
    pub fn ff(&mut self, buf: &mut Buffer) {
        buf.clear();
        self.pos = Position::new();
        self.is_visible = true;
        self.attr = super::TextAttribute::DEFAULT;
    }

    /// (carriage return, CR, \r, ^M), moves the printing position to the start of the line.
    pub fn cr(&mut self, _buf: &mut Buffer) {
        self.pos.x = 0;
    }

    pub fn eol(&mut self, buf: &mut Buffer) {
        self.pos.x = buf.width as i32 - 1;
    }

    pub fn home(&mut self, _buf: &mut Buffer) {
        self.pos = Position::new();
    }

    /// (backspace, BS, \b, ^H), may overprint the previous character
    pub fn bs(&mut self, buf: &mut Buffer) {
        self.pos.x = max(0, self.pos.x - 1);
        buf.set_char(0, self.pos, Some(DosChar::from(b' ' as u16, self.attr)));
    }

    pub fn del(&mut self, buf: &mut Buffer) {
        if let Some(line) = buf.layers[0].lines.get_mut(self.pos.y as usize) {
            let i = self.pos.x as usize ;
            if i < line.chars.len(){ 
                line.chars.remove(i);
            }
        }
    }
    
    pub fn left(&mut self, buf: &mut Buffer, num: i32) {
        let old_x = self.pos.x;
        self.pos.x = max(0, self.pos.x.saturating_sub(num));
        fill_line(buf, self.pos.y, self.pos.x, old_x);
    }

    pub fn right(&mut self, buf: &mut Buffer, num: i32) {
        let old_x = self.pos.x;
        self.pos.x = min(buf.width as i32 - 1, self.pos.x + num);
        fill_line(buf, self.pos.y, old_x, self.pos.x);
    }

    pub fn up(&mut self, buf: &mut Buffer, num: i32) {
        self.pos.y = max(buf.get_first_visible_line(), self.pos.y.saturating_sub(num));
    }

    pub fn down(&mut self, buf: &mut Buffer, num: i32) {
        if buf.is_terminal_buffer {
            self.pos.y = min(buf.get_first_visible_line() + buf.height - 1, self.pos.y + num);
        } else { 
            self.pos.y = self.pos.y + num;
        }
    }
}

impl Buffer {
    fn print_value(&mut self, caret: &mut Caret, ch: u16)
    {
        let ch = DosChar::from(ch, caret.attr);
        self.print_char(caret, ch);
    }

    fn print_char(&mut self, caret: &mut Caret, ch: DosChar)
    {
        self.set_char(0, caret.pos, Some(ch));
        caret.pos.x = caret.pos.x + 1;
        if caret.pos.x >= self.width as i32 {
            caret.lf(self);
        }
    }

    fn clear_screen(&mut self, caret: &mut Caret)
    {
        caret.pos = Position::new();
        self.clear();
    }

    fn clear_buffer_down(&mut self, y: i32) {
        for y in y..self.height as i32 {
            for x in 0..self.width as i32 {
                self.set_char(0, Position::from(x, y), Some(DosChar::new()));
            }
        }
    }

    fn clear_buffer_up(&mut self, y: i32) {
        for y in 0..y {
            for x in 0..self.width as i32 {
                self.set_char(0, Position::from(x, y), Some(DosChar::new()));
            }
        }
    }

    fn clear_line(&mut self, y: i32) {
        for x in 0..self.width as i32 {
            self.set_char(0, Position::from(x, y), Some(DosChar::new()));
        }
    }

    fn clear_line_end(&mut self, pos: &Position) {
        for x in pos.x..self.width as i32 {
            self.set_char(0, Position::from(x, pos.y), Some(DosChar::new()));
        }
    }

    fn clear_line_start(&mut self, pos: &Position) {
        for x in 0..pos.x {
            self.set_char(0, Position::from(x, pos.y), Some(DosChar::new()));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Buffer, Caret, AnsiParser, BufferParser, Position, TextAttribute};

    fn create_buffer<T: BufferParser>(parser: &mut T, input: &[u8]) -> (Buffer, Caret) 
    {
        let mut buf = Buffer::create(80, 25);
        let mut caret  = Caret::new();
        // remove editing layer
        buf.layers.remove(0);
        buf.layers[0].is_locked = false;
        buf.layers[0].is_transparent = false;
      
        assert_eq!(25, buf.layers[0].lines.len());
        
        run_parser(&mut buf, &mut caret, parser, input);
        
        (buf, caret)
    }

    fn run_parser<T: BufferParser>(buf: &mut Buffer, caret: &mut Caret, parser: &mut T, input: &[u8])
    {
        for b in input {
            parser.print_char(buf,caret, *b).unwrap();
        }
    }
    
    #[test]
    fn test_bs() {
        let (buf, caret) = create_buffer(&mut AnsiParser::new(), b"\x1b[1;43mtest\x08\x08\x08\x08");
        assert_eq!(Position::new(), caret.pos);
        for i in 0..4 {
            assert_eq!(TextAttribute::from_color(15, 6), buf.get_char(Position::from(i, 0)).unwrap().attribute);
        }
    }
}