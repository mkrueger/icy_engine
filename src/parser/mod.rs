use std::{io, cmp::{max}};
use crate::Line;

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

#[cfg(test)]
mod tests;

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
        if self.pos.y >= buf.get_last_editable_line() {
            buf.scroll_down();
            self.pos.y = buf.get_last_editable_line();
        }
        while self.pos.y >= buf.get_real_buffer_height() {
            let i = buf.layers[0].lines.len() as i32;
            buf.layers[0].insert_line(i, Line::new());
        }
    }
    
    /// (form feed, FF, \f, ^L), to cause a printer to eject paper to the top of the next page, or a video terminal to clear the screen.
    pub fn ff(&mut self, buf: &mut Buffer) {
        buf.terminal_state.reset();
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
        self.pos.x = buf.get_buffer_width() as i32 - 1;
    }

    pub fn home(&mut self, buf: &mut Buffer) {
        self.pos = buf.upper_left_position();
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
        self.pos.x = self.pos.x.saturating_sub(num);
        buf.terminal_state.limit_caret_pos(buf, self);
        fill_line(buf, self.pos.y, self.pos.x, old_x);
    }

    pub fn right(&mut self, buf: &mut Buffer, num: i32) {
        let old_x = self.pos.x;
        self.pos.x = self.pos.x + num;
        buf.terminal_state.limit_caret_pos(buf, self);
        fill_line(buf, self.pos.y, old_x, self.pos.x);
        println!("{}", self.pos);
    }

    pub fn up(&mut self, buf: &mut Buffer, num: i32) {
        self.pos.y = self.pos.y.saturating_sub(num);
        buf.terminal_state.limit_caret_pos(buf, self);
        if self.pos.y < buf.get_first_editable_line() {
            buf.scroll_up();
            self.pos.y = buf.get_first_editable_line();
        }
    }

    pub fn down(&mut self, buf: &mut Buffer, num: i32) {
        self.pos.y = self.pos.y + num;
        buf.terminal_state.limit_caret_pos(buf, self);
        if self.pos.y >= buf.get_last_editable_line() {
            buf.scroll_down();
            self.pos.y = buf.get_last_editable_line();
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
        caret.pos.x += 1;
        if caret.pos.x >= self.get_buffer_width() as i32 {
            if let crate::AutoWrapMode::AutoWrap = self.terminal_state.auto_wrap_mode  {
                caret.lf(self);
            } else {
                caret.pos.x -=  1;

            }
        }
    }

    /*fn get_buffer_last_line(&mut self) -> i32 
    {
        if let Some((_, end)) = self.terminal_state.margins {
            self.get_first_visible_line() + end
        } else {
            max(self.terminal_state.height, self.layers[0].lines.len() as i32)
        }
    }*/

    fn scroll_down(&mut self)
    {
        if let Some((start, end)) = self.terminal_state.margins {
            for layer in &mut self.layers {
                if (layer.lines.len() as i32) < start {
                    continue;
                }
                layer.lines.remove(start as usize);
                if (layer.lines.len() as i32) >= end {
                    layer.lines.insert(end as usize, Line::new());
                }
            }
        }
    }

    fn scroll_up(&mut self)
    {
        if let Some((start, end)) = self.terminal_state.margins {
            for layer in &mut self.layers {
                if (layer.lines.len() as i32) <= end {
                    layer.lines.resize(end as usize + 1, Line::new());
                }
                layer.lines.remove(end as usize);
                layer.lines.insert(start as usize, Line::new());
            }
        }
    }

    fn clear_screen(&mut self, caret: &mut Caret)
    {
        // TODO: how to handle margins here?
        caret.pos = self.upper_left_position();
        self.clear();
    }

    fn clear_buffer_down(&mut self, y: i32) {
        for y in y..self.get_last_editable_line() as i32 {
            for x in 0..self.get_buffer_width() as i32 {
                self.set_char(0, Position::from(x, y), Some(DosChar::new()));
            }
        }
    }

    fn clear_buffer_up(&mut self, y: i32) {
        for y in self.get_first_editable_line()..y {
            for x in 0..self.get_buffer_width() as i32 {
                self.set_char(0, Position::from(x, y), Some(DosChar::new()));
            }
        }
    }

    fn clear_line(&mut self, y: i32) {
        for x in 0..self.get_buffer_width() as i32 {
            self.set_char(0, Position::from(x, y), Some(DosChar::new()));
        }
    }

    fn clear_line_end(&mut self, pos: &Position) {
        for x in pos.x..self.get_buffer_width() as i32 {
            self.set_char(0, Position::from(x, pos.y), Some(DosChar::new()));
        }
    }

    fn clear_line_start(&mut self, pos: &Position) {
        for x in 0..pos.x {
            self.set_char(0, Position::from(x, pos.y), Some(DosChar::new()));
        }
    }

    fn remove_terminal_line(&mut self, line: i32) {
        if line >= self.layers[0].lines.len() as i32 {
            return;
        }
        self.layers[0].remove_line(line);
        if let Some((_, end)) = self.terminal_state.margins {
            self.layers[0].insert_line(end, Line::new());
        } 
    }
    
    fn insert_terminal_line(&mut self, line: i32) {
        if line >= self.layers[0].lines.len() as i32 {
            self.layers[0].lines.resize(line as usize + 1, Line::new());
        }

        if let Some((_, end)) = self.terminal_state.margins {
            if end < self.layers[0].lines.len() as i32 {
                self.layers[0].lines.remove(end as usize);
            }
        }
        self.layers[0].insert_line(line, Line::new());
    }
}
