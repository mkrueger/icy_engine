#![allow(clippy::match_same_arms)]
use super::BufferParser;
use crate::{AttributedChar, Buffer, CallbackAction, Caret, EngineResult, Position, TextPane, UnicodeConverter};

mod constants;

#[cfg(test)]
mod tests;

/// <https://www.blunham.com/Radar/Teletext/PDFs/Viewdata1976Spec.pdf>
pub struct Parser {
    got_esc: bool,

    hold_graphics: bool,
    held_graphics_character: char,

    is_contiguous: bool,

    is_in_graphic_mode: bool,

    graphics_bg: u32,
    alpha_bg: u32,
}

impl Default for Parser {
    fn default() -> Self {
        Self {
            got_esc: false,
            hold_graphics: false,
            held_graphics_character: ' ',
            is_contiguous: true,
            is_in_graphic_mode: false,
            graphics_bg: 0,
            alpha_bg: 0,
        }
    }
}

impl Parser {
    fn reset_screen(&mut self) {
        self.got_esc = false;

        self.hold_graphics = false;
        self.held_graphics_character = ' ';

        self.is_contiguous = true;
        self.is_in_graphic_mode = false;
        self.graphics_bg = 0;
        self.alpha_bg = 0;
    }

    fn fill_to_eol(buf: &mut Buffer, caret: &Caret) {
        if caret.get_position().x <= 0 {
            return;
        }
        let sx = caret.get_position().x;
        let sy = caret.get_position().y;

        let attr = buf.get_char((sx, sy)).attribute;

        for x in sx..buf.terminal_state.get_width() {
            let p = Position::new(x, sy);
            let mut ch = buf.get_char(p);
            if ch.attribute != attr {
                break;
            }
            ch.attribute = caret.attribute;
            buf.layers[0].set_char(p, ch);
        }
    }

    fn reset_on_row_change(&mut self, caret: &mut Caret) {
        self.reset_screen();
        caret.reset_color_attribute();
    }

    fn print_char(&mut self, buf: &mut Buffer, caret: &mut Caret, ch: AttributedChar) {
        buf.layers[0].set_char(caret.pos, ch);
        self.caret_right(buf, caret);
    }

    fn caret_down(&mut self, buf: &Buffer, caret: &mut Caret) {
        caret.pos.y += 1;
        if caret.pos.y >= buf.terminal_state.get_height() {
            caret.pos.y = 0;
        }
        self.reset_on_row_change(caret);
    }

    fn caret_up(buf: &Buffer, caret: &mut Caret) {
        if caret.pos.y > 0 {
            caret.pos.y = caret.pos.y.saturating_sub(1);
        } else {
            caret.pos.y = buf.terminal_state.get_height() - 1;
        }
    }

    fn caret_right(&mut self, buf: &Buffer, caret: &mut Caret) {
        caret.pos.x += 1;
        if caret.pos.x >= buf.terminal_state.get_width() {
            caret.pos.x = 0;
            self.caret_down(buf, caret);
        }
    }

    #[allow(clippy::unused_self)]
    fn caret_left(&self, buf: &Buffer, caret: &mut Caret) {
        if caret.pos.x > 0 {
            caret.pos.x = caret.pos.x.saturating_sub(1);
        } else {
            caret.pos.x = buf.terminal_state.get_width() - 1;
            Parser::caret_up(buf, caret);
        }
    }
}

#[derive(Default)]
pub struct CharConverter {}

impl UnicodeConverter for CharConverter {
    fn convert_from_unicode(&self, ch: char, _font_page: usize) -> char {
        if ch == ' ' {
            return ' ';
        }
        match constants::UNICODE_TO_VIEWDATA.get(&ch) {
            Some(out_ch) => *out_ch,
            _ => ch,
        }
    }

    fn convert_to_unicode(&self, attributed_char: AttributedChar) -> char {
        match constants::VIEWDATA_TO_UNICODE.get(attributed_char.ch as usize) {
            Some(out_ch) => *out_ch,
            _ => attributed_char.ch,
        }
    }
}

impl BufferParser for Parser {
    fn print_char(&mut self, buf: &mut Buffer, current_layer: usize, caret: &mut Caret, ch: char) -> EngineResult<CallbackAction> {
        let ch = ch as u8;
        match ch {
            129..=135 => {
                // Alpha Red, Green, Yellow, Blue, Magenta, Cyan, White
                self.is_in_graphic_mode = false;
                caret.attribute.set_is_concealed(false);
                self.held_graphics_character = ' ';
                caret.attribute.set_foreground(1 + (ch - 129) as u32);
                Parser::fill_to_eol(buf, caret);
            }
            // Flash
            136 => {
                caret.attribute.set_is_blinking(true);
                Parser::fill_to_eol(buf, caret);
            }
            // Steady
            135 => {
                caret.attribute.set_is_blinking(false);
                Parser::fill_to_eol(buf, caret);
            }

            // normal height
            140 => {
                caret.attribute.set_is_double_height(false);
                Parser::fill_to_eol(buf, caret);
            }

            // double height
            141 => {
                caret.attribute.set_is_double_height(true);
                Parser::fill_to_eol(buf, caret);
            }

            145..=151 => {
                // Graphics Red, Green, Yellow, Blue, Magenta, Cyan, White
                if !self.is_in_graphic_mode {
                    self.is_in_graphic_mode = true;
                    self.held_graphics_character = ' ';
                }
                caret.attribute.set_is_concealed(false);
                caret.attribute.set_foreground(1 + (ch - 145) as u32);
                Parser::fill_to_eol(buf, caret);
            }

            // conceal
            152 => {
                if !self.is_in_graphic_mode {
                    caret.attribute.set_is_concealed(true);
                    Parser::fill_to_eol(buf, caret);
                }
            }

            // Contiguous Graphics
            153 => {
                self.is_contiguous = true;
                self.is_in_graphic_mode = true;
            }
            // Separated Graphics
            154 => {
                self.is_contiguous = false;
                self.is_in_graphic_mode = true;
            }

            // Black Background
            156 => {
                caret.attribute.set_is_concealed(false);
                caret.attribute.set_background(0);
                Parser::fill_to_eol(buf, caret);
            }

            // New Background
            157 => {
                caret.attribute.set_background(caret.attribute.get_foreground());
                Parser::fill_to_eol(buf, caret);
            }

            // Hold Graphics
            158 => {
                self.hold_graphics = true;
                self.is_in_graphic_mode = true;
            }

            // Release Graphics
            159 => {
                self.hold_graphics = false;
                self.is_in_graphic_mode = false;
            }

            _ => {
                return Ok(self.interpret_char(buf, caret, ch));
            }
        }
        self.got_esc = false;
        Ok(CallbackAction::Update)
    }
}
