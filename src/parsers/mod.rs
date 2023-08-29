use crate::{EngineResult, Line};
use std::cmp::{max, min};

use self::ansi::sound::AnsiMusic;

use super::{AttributedChar, Buffer, Caret, Position};

mod parser_errors;
pub use parser_errors::*;

pub mod ansi;
pub mod ascii;
pub mod atascii;
pub mod avatar;
pub mod pcboard;
pub mod petscii;
pub mod rip;
pub mod viewdata;

pub const BEL: char = '\x07';
pub const LF: char = '\n';
pub const CR: char = '\r';
pub const BS: char = '\x08';
pub const FF: char = '\x0C';

#[derive(Debug, PartialEq)]
pub enum CallbackAction {
    None,
    Beep,
    SendString(String),
    PlayMusic(AnsiMusic),
    ChangeBaudEmulation(ansi::BaudEmulation),
    ResizeTerminal(usize, usize),
}

pub trait BufferParser {
    fn convert_from_unicode(&self, ch: char, font_page: usize) -> char;
    fn convert_to_unicode(&self, attributed_char: AttributedChar) -> char;

    /// Prints a character to the buffer. Gives back an optional string returned to the sender (in case for terminals).
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    fn print_char(
        &mut self,
        buffer: &mut Buffer,
        current_layer: usize,
        caret: &mut Caret,
        c: char,
    ) -> EngineResult<CallbackAction>;
}

impl Caret {
    /// (line feed, LF, \n, ^J), moves the print head down one line, or to the left edge and down. Used as the end of line marker in most UNIX systems and variants.
    pub fn lf(&mut self, buf: &mut Buffer, current_layer: usize) {
        let was_ooe = self.pos.y > buf.get_last_editable_line();

        self.pos.x = 0;
        self.pos.y += 1;
        while self.pos.y >= buf.layers[current_layer].lines.len() as i32 {
            let len = buf.layers[current_layer].lines.len();
            let buffer_width = buf.get_width() as usize;
            buf.layers[current_layer]
                .lines
                .insert(len, Line::with_capacity(buffer_width));
        }
        if !buf.is_terminal_buffer {
            return;
        }
        if was_ooe {
            buf.terminal_state.limit_caret_pos(buf, self);
        } else {
            self.check_scrolling_on_caret_down(buf, current_layer, false);
        }
    }

    /// (form feed, FF, \f, ^L), to cause a printer to eject paper to the top of the next page, or a video terminal to clear the screen.
    pub fn ff(&mut self, buf: &mut Buffer, current_layer: usize) {
        buf.reset_terminal();
        buf.layers[current_layer].clear();
        buf.stop_sixel_threads();
        self.pos = Position::default();
        self.is_visible = true;
        self.reset_color_attribute();
    }

    /// (carriage return, CR, \r, ^M), moves the printing position to the start of the line.
    pub fn cr(&mut self, _buf: &Buffer) {
        self.pos.x = 0;
    }

    pub fn eol(&mut self, buf: &Buffer) {
        self.pos.x = buf.get_width() - 1;
    }

    pub fn home(&mut self, buf: &Buffer) {
        self.pos = buf.upper_left_position();
    }

    /// (backspace, BS, \b, ^H), may overprint the previous character
    pub fn bs(&mut self, buf: &mut Buffer, current_layer: usize) {
        self.pos.x = max(0, self.pos.x - 1);
        buf.layers[current_layer].set_char(self.pos, AttributedChar::new(' ', self.attribute));
    }

    pub fn del(&mut self, buf: &mut Buffer, current_layer: usize) {
        if let Some(line) = buf.layers[current_layer].lines.get_mut(self.pos.y as usize) {
            let i = self.pos.x as usize;
            if i < line.chars.len() {
                line.chars.remove(i);
            }
        }
    }

    pub fn ins(&mut self, buf: &mut Buffer, current_layer: usize) {
        if let Some(line) = buf.layers[current_layer].lines.get_mut(self.pos.y as usize) {
            let i = self.pos.x as usize;
            if i < line.chars.len() {
                line.chars
                    .insert(i, AttributedChar::new(' ', self.attribute));
            }
        }
    }

    pub fn erase_charcter(&mut self, buf: &mut Buffer, current_layer: usize, number: i32) {
        let mut i = self.pos.x as usize;
        let number = min(buf.get_width() - i as i32, number);
        if number <= 0 {
            return;
        }
        if let Some(line) = buf.layers[current_layer].lines.get_mut(self.pos.y as usize) {
            for _ in 0..number {
                line.set_char(i as i32, AttributedChar::new(' ', self.attribute));
                i += 1;
            }
        }
    }

    pub fn left(&mut self, buf: &Buffer, num: i32) {
        self.pos.x = self.pos.x.saturating_sub(num);
        buf.terminal_state.limit_caret_pos(buf, self);
    }

    pub fn right(&mut self, buf: &Buffer, num: i32) {
        self.pos.x = self.pos.x.saturating_add(num);
        buf.terminal_state.limit_caret_pos(buf, self);
    }

    pub fn up(&mut self, buf: &mut Buffer, current_layer: usize, num: i32) {
        self.pos.y = self.pos.y.saturating_sub(num);
        self.check_scrolling_on_caret_up(buf, current_layer, false);
        buf.terminal_state.limit_caret_pos(buf, self);
    }

    pub fn down(&mut self, buf: &mut Buffer, current_layer: usize, num: i32) {
        self.pos.y += num;
        self.check_scrolling_on_caret_down(buf, current_layer, false);
        buf.terminal_state.limit_caret_pos(buf, self);
    }

    /// Moves the cursor down one line in the same column. If the cursor is at the bottom margin, the page scrolls up.
    pub fn index(&mut self, buf: &mut Buffer, current_layer: usize) {
        self.pos.y += 1;
        self.check_scrolling_on_caret_down(buf, current_layer, true);
        buf.terminal_state.limit_caret_pos(buf, self);
    }

    /// Moves the cursor up one line in the same column. If the cursor is at the top margin, the page scrolls down.
    pub fn reverse_index(&mut self, buf: &mut Buffer, current_layer: usize) {
        self.pos.y = self.pos.y.saturating_sub(1);
        self.check_scrolling_on_caret_up(buf, current_layer, true);
        buf.terminal_state.limit_caret_pos(buf, self);
    }

    pub fn next_line(&mut self, buf: &mut Buffer, current_layer: usize) {
        self.pos.y += 1;
        self.pos.x = 0;
        self.check_scrolling_on_caret_down(buf, current_layer, true);
        buf.terminal_state.limit_caret_pos(buf, self);
    }

    fn check_scrolling_on_caret_up(&mut self, buf: &mut Buffer, current_layer: usize, force: bool) {
        if buf.needs_scrolling() || force {
            let last = buf.get_first_editable_line();
            while self.pos.y < last {
                buf.scroll_down(current_layer);
                self.pos.y += 1;
            }
        }
    }

    fn check_scrolling_on_caret_down(
        &mut self,
        buf: &mut Buffer,
        current_layer: usize,
        force: bool,
    ) {
        if (buf.needs_scrolling() || force) && self.pos.y > buf.get_last_editable_line() {
            buf.scroll_up(current_layer);
            self.pos.y -= 1;
        }
    }
}

impl Buffer {
    fn print_value(&mut self, layer: usize, caret: &mut Caret, ch: u16) {
        let ch = AttributedChar::new(char::from_u32(ch as u32).unwrap(), caret.attribute);
        self.print_char(layer, caret, ch);
    }

    fn print_char(&mut self, layer: usize, caret: &mut Caret, ch: AttributedChar) {
        let buffer_width = self.get_width();
        if caret.insert_mode {
            let layer = &mut self.layers[layer];
            if layer.lines.len() < caret.pos.y as usize + 1 {
                layer.lines.resize(
                    caret.pos.y as usize + 1,
                    Line::with_capacity(buffer_width as usize),
                );
            }
            layer.lines[caret.pos.y as usize].insert_char(caret.pos.x, AttributedChar::default());
        }

        self.layers[layer].set_char(caret.pos, ch);
        caret.pos.x += 1;
        if caret.pos.x >= buffer_width {
            if let crate::AutoWrapMode::AutoWrap = self.terminal_state.auto_wrap_mode {
                caret.lf(self, layer);
            } else {
                caret.pos.x -= 1;
            }
        }
    }

    fn scroll_up(&mut self, layer: usize) {
        let start_line: i32 = self.get_first_editable_line();
        let end_line = self.get_last_editable_line();

        let start_column = self.get_first_editable_column();
        let end_column = self.get_last_editable_column();

        let layer = &mut self.layers[layer];
        for x in start_column..=end_column {
            (start_line..end_line).for_each(|y| {
                let ch = layer.get_char(Position::new(x, y + 1));
                layer.set_char(Position::new(x, y), ch);
            });
            layer.set_char(Position::new(x, end_line), AttributedChar::default());
        }
    }

    fn scroll_down(&mut self, layer: usize) {
        let start_line: i32 = self.get_first_editable_line();
        let end_line = self.get_last_editable_line();

        let start_column = self.get_first_editable_column();
        let end_column = self.get_last_editable_column();

        let layer = &mut self.layers[layer];
        for x in start_column..=end_column {
            ((start_line + 1)..=end_line).rev().for_each(|y| {
                let ch = layer.get_char(Position::new(x, y - 1));
                layer.set_char(Position::new(x, y), ch);
            });
            layer.set_char(Position::new(x, start_line), AttributedChar::default());
        }
    }

    fn scroll_left(&mut self, layer: usize) {
        let start_line: i32 = self.get_first_editable_line();
        let end_line = self.get_last_editable_line();

        let start_column = self.get_first_editable_column() as usize;
        let end_column = self.get_last_editable_column() as usize + 1;

        let layer = &mut self.layers[layer];
        for i in start_line..=end_line {
            let line = &mut layer.lines[i as usize];
            if line.chars.len() > start_column {
                line.chars.insert(end_column, AttributedChar::default());
                line.chars.remove(start_column);
            }
        }
    }

    fn scroll_right(&mut self, layer: usize) {
        let start_line = self.get_first_editable_line();
        let end_line = self.get_last_editable_line();

        let start_column = self.get_first_editable_column() as usize;
        let end_column = self.get_last_editable_column() as usize;

        let layer = &mut self.layers[layer];
        for i in start_line..=end_line {
            let line = &mut layer.lines[i as usize];
            if line.chars.len() > start_column {
                line.chars.insert(start_column, AttributedChar::default());
                line.chars.remove(end_column + 1);
            }
        }
    }

    pub fn clear_screen(&mut self, layer: usize, caret: &mut Caret) {
        caret.pos = Position::default();
        let layer = &mut self.layers[layer];
        layer.clear();
        self.stop_sixel_threads();
    }

    fn clear_buffer_down(&mut self, layer: usize, caret: &Caret) {
        let pos = caret.get_position();
        let ch: AttributedChar = AttributedChar {
            attribute: caret.attribute,
            ..Default::default()
        };

        for y in pos.y..self.get_last_visible_line() {
            for x in 0..self.get_width() {
                self.layers[layer].set_char(Position::new(x, y), ch);
            }
        }
    }

    fn clear_buffer_up(&mut self, layer: usize, caret: &Caret) {
        let pos = caret.get_position();
        let ch: AttributedChar = AttributedChar {
            attribute: caret.attribute,
            ..Default::default()
        };

        for y in self.get_first_visible_line()..pos.y {
            for x in 0..self.get_width() {
                self.layers[layer].set_char(Position::new(x, y), ch);
            }
        }
    }

    fn clear_line(&mut self, layer: usize, caret: &Caret) {
        let mut pos = caret.get_position();
        let ch: AttributedChar = AttributedChar {
            attribute: caret.attribute,
            ..Default::default()
        };
        for x in 0..self.get_width() {
            pos.x = x;
            self.layers[layer].set_char(pos, ch);
        }
    }

    fn clear_line_end(&mut self, layer: usize, caret: &Caret) {
        let mut pos = caret.get_position();
        let ch: AttributedChar = AttributedChar {
            attribute: caret.attribute,
            ..Default::default()
        };
        for x in pos.x..self.get_width() {
            pos.x = x;
            self.layers[layer].set_char(pos, ch);
        }
    }

    fn clear_line_start(&mut self, layer: usize, caret: &Caret) {
        let mut pos = caret.get_position();
        let ch: AttributedChar = AttributedChar {
            attribute: caret.attribute,
            ..Default::default()
        };
        for x in 0..pos.x {
            pos.x = x;
            self.layers[layer].set_char(pos, ch);
        }
    }

    fn remove_terminal_line(&mut self, layer: usize, line: i32) {
        if line >= self.layers[layer].get_line_count() {
            return;
        }
        self.layers[layer].remove_line(line);
        if let Some((_, end)) = self.terminal_state.get_margins_top_bottom() {
            let buffer_width = self.layers[layer].get_width() as usize;
            self.layers[layer].insert_line(end, Line::with_capacity(buffer_width));
        }
    }

    fn insert_terminal_line(&mut self, layer: usize, line: i32) {
        if let Some((_, end)) = self.terminal_state.get_margins_top_bottom() {
            if end < self.layers[layer].get_line_count() {
                self.layers[layer].lines.remove(end as usize);
            }
        }
        let buffer_width = self.layers[layer].get_width() as usize;
        self.layers[layer].insert_line(line, Line::with_capacity(buffer_width));
    }
}

fn _get_string_from_buffer(buf: &Buffer) -> String {
    let converted = crate::convert_to_asc(buf, &crate::SaveOptions::new()).unwrap(); // test code
    let b: Vec<u8> = converted
        .iter()
        .map(|&x| if x == 27 { b'x' } else { x })
        .collect();
    let converted = String::from_utf8_lossy(b.as_slice());

    converted.to_string()
}

#[cfg(test)]
fn create_buffer<T: BufferParser>(parser: &mut T, input: &[u8]) -> (Buffer, Caret) {
    let mut buf = Buffer::create(80, 25);
    let mut caret = Caret::default();
    // remove editing layer
    buf.is_terminal_buffer = true;
    buf.layers.remove(0);
    buf.layers[0].is_locked = false;
    buf.layers.first_mut().unwrap().lines.clear();

    update_buffer(&mut buf, &mut caret, parser, input);

    (buf, caret)
}

#[cfg(test)]
fn update_buffer<T: BufferParser>(
    buf: &mut Buffer,
    caret: &mut Caret,
    parser: &mut T,
    input: &[u8],
) {
    for b in input {
        if let Some(ch) = char::from_u32(*b as u32) {
            parser.print_char(buf, 0, caret, ch).unwrap(); // test code
        }
    }
}

#[cfg(test)]
fn update_buffer_force<T: BufferParser>(
    buf: &mut Buffer,
    caret: &mut Caret,
    parser: &mut T,
    input: &[u8],
) {
    for b in input {
        if let Some(ch) = char::from_u32(*b as u32) {
            let _ = parser.print_char(buf, 0, caret, ch); // test code
        }
    }
}

#[cfg(test)]
fn get_simple_action<T: BufferParser>(parser: &mut T, input: &[u8]) -> CallbackAction {
    let mut buf = Buffer::create(80, 25);
    let mut caret = Caret::default();
    buf.is_terminal_buffer = true;
    buf.layers.remove(0);
    buf.layers[0].is_locked = false;

    get_action(&mut buf, &mut caret, parser, input)
}

#[cfg(test)]
fn get_action<T: BufferParser>(
    buf: &mut Buffer,
    caret: &mut Caret,
    parser: &mut T,
    input: &[u8],
) -> CallbackAction {
    // remove editing layer

    let mut action = CallbackAction::None;
    for b in input {
        if let Some(ch) = char::from_u32(*b as u32) {
            action = parser.print_char(buf, 0, caret, ch).unwrap(); // test code
        }
    }

    action
}
