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

impl Caret {
    /// (line feed, LF, \n, ^J), moves the print head down one line, or to the left edge and down. Used as the end of line marker in most UNIX systems and variants.
    pub fn lf(&mut self, buf: &mut Buffer) {
        let was_ooe = self.pos.y > buf.get_last_editable_line();
        self.pos.x = 0;
        self.pos.y += 1;
        while self.pos.y >= buf.layers[0].lines.len() as i32 {
            let len = buf.layers[0].lines.len();
            buf.layers[0].lines.insert(len, Line::new());
        }
        if was_ooe {
            buf.terminal_state.limit_caret_pos(buf, self);
        } else {
            self.check_scrolling_on_caret_down(buf, false);
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
    }

    pub fn right(&mut self, buf: &mut Buffer, num: i32) {
        let old_x = self.pos.x;
        self.pos.x = self.pos.x + num;
        if self.pos.x > buf.get_buffer_width() && self.pos.y < buf.get_last_editable_line() {
            self.pos.y += self.pos.x / buf.get_buffer_width();
            self.pos.x %= buf.get_buffer_width();
        }
        buf.terminal_state.limit_caret_pos(buf, self);
    }

    pub fn up(&mut self, buf: &mut Buffer, num: i32) {
        self.pos.y = self.pos.y.saturating_sub(num);
        self.check_scrolling_on_caret_up(buf, false);
        buf.terminal_state.limit_caret_pos(buf, self);
    }

    pub fn down(&mut self, buf: &mut Buffer, num: i32) {
        self.pos.y = self.pos.y + num;
        self.check_scrolling_on_caret_down(buf, false);
        buf.terminal_state.limit_caret_pos(buf, self);
    }

    /// Moves the cursor down one line in the same column. If the cursor is at the bottom margin, the page scrolls up.
    pub fn index(&mut self, buf: &mut Buffer) {
        self.pos.y = self.pos.y + 1;
        self.check_scrolling_on_caret_down(buf, true);
        buf.terminal_state.limit_caret_pos(buf, self);
    }
    
    /// Moves the cursor up one line in the same column. If the cursor is at the top margin, the page scrolls down.
    pub fn reverse_index(&mut self, buf: &mut Buffer) {
        self.pos.y = self.pos.y.saturating_sub(1);
        self.check_scrolling_on_caret_up(buf, true);
        buf.terminal_state.limit_caret_pos(buf, self);
    }

    pub fn next_line(&mut self, buf: &mut Buffer) {
        self.pos.y = self.pos.y + 1;
        self.pos.x = 0;
        self.check_scrolling_on_caret_down(buf, true);
        buf.terminal_state.limit_caret_pos(buf, self);
    }

    fn check_scrolling_on_caret_up(&mut self, buf: &mut Buffer, force: bool) {
        if buf.needs_scrolling() || force {
            while self.pos.y < buf.get_first_editable_line() {
                buf.scroll_up();
                self.pos.y += 1;
            }
        }
    }

    fn check_scrolling_on_caret_down(&mut self, buf: &mut Buffer, force: bool) {
        if buf.needs_scrolling() || force {
            while self.pos.y >= buf.get_last_editable_line() {
                buf.scroll_down();
                self.pos.y -= 1;
            }
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
        if caret.insert_mode {
            let layer = &mut self.layers[0];
            layer.lines[caret.pos.y as usize].insert_char(caret.pos.x, Some(DosChar::new()));
        }

        self.set_char(0, caret.pos, Some(ch));
        caret.pos.x += 1;
        if caret.pos.x > self.get_buffer_width() as i32 {
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
        let start = self.get_first_editable_line();
        let end = self.get_last_editable_line();
        for layer in &mut self.layers {
            if layer.lines.len() as i32 > start {
                layer.lines.remove(start as usize);
            }
            if (layer.lines.len() as i32) >= end {
                layer.lines.insert(end as usize, Line::new());
            }
        }
    }

    fn scroll_up(&mut self)
    {
        let start = self.get_first_editable_line();
        let end = self.get_last_editable_line();
        for layer in &mut self.layers {
            if (layer.lines.len() as i32) <= end {
                layer.lines.resize(end as usize + 1, Line::new());
            }
            layer.lines.remove(end as usize);
            layer.lines.insert(start as usize, Line::new());
        }
    }

    fn clear_screen(&mut self, caret: &mut Caret)
    {
        caret.pos = Position::new();
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

        if let Some((_, end)) = self.terminal_state.margins {
            if end < self.layers[0].lines.len() as i32 {
                self.layers[0].lines.remove(end as usize);
            }
        }
        self.layers[0].insert_line(line, Line::new());
    }
}
