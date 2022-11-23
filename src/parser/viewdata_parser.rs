use crate::{Buffer, Caret, EngineResult, AttributedChar, TextAttribute, Position};
use super::BufferParser;

/// https://www.blunham.com/Radar/Teletext/PDFs/Viewdata1976Spec.pdf
#[derive(Default)]
pub struct ViewdataParser {
    got_esc: bool,
    
    hold_graphics: bool,
    held_graphics_character: char,

    is_contiguous: bool,

    is_in_graphic_mode : bool,

    graphics_bg: u32,
    alpha_bg: u32
}

impl ViewdataParser {
    pub fn new() -> Self {
        let mut res = ViewdataParser::default();
        res.reset_screen();
        res
    }

    fn reset_screen(&mut self) {
        self.got_esc = false;

        self.hold_graphics = false;
        self.held_graphics_character = ' ';
      
        self.is_contiguous = true;
        self.is_in_graphic_mode = false;
        self.graphics_bg = 0;
        self.alpha_bg = 0;
    }

    fn fill_to_eol(&self, buf: &mut Buffer, caret: &Caret) {
        if caret.get_position().x <= 0 {
            return;
        }
        let sx = caret.get_position().x;
        let sy = caret.get_position().y;

        let attr = buf.get_char(Position::new(sx - 1, sy)).unwrap().attribute;
//        let attr = caret.attr;

        for x in sx..buf.get_buffer_width() {
            let p = Position::new(x, sy);
            let mut ch = buf.get_char(p).unwrap();
            ch.attribute = attr;
            buf.set_char(0, p, Some(ch));
        }
    }

    fn reset_on_row_change(&mut self, caret: &mut Caret) {
        self.reset_screen();
        caret.attr = TextAttribute::default();
    }

    fn print_char(&mut self, buf: &mut Buffer, caret: &mut Caret, ch: AttributedChar) {
        buf.set_char(0, caret.pos, Some(ch));
        self.caret_right(buf, caret);
    }

    fn caret_down(&mut self, buf: &mut Buffer, caret: &mut Caret) {
        caret.pos.y += 1;
        if caret.pos.y >= buf.get_buffer_height() {
            caret.pos.y = 0;
        }
        self.reset_on_row_change(caret);
    }

    fn caret_up(&mut self, buf: &mut Buffer, caret: &mut Caret) {
        if caret.pos.y > 0 {
            caret.pos.y = caret.pos.y.saturating_sub(1);
        } else {
            caret.pos.y = buf.get_buffer_height() - 1;
        }
    }

    fn caret_right(&mut self, buf: &mut Buffer, caret: &mut Caret) {
        caret.pos.x += 1;
        if caret.pos.x >= buf.get_buffer_width() {
            caret.pos.x = 0;
            self.caret_down(buf, caret);
        }
    } 

    fn caret_left(&mut self, buf: &mut Buffer, caret: &mut Caret) {
        if caret.pos.x > 0 {
            caret.pos.x = caret.pos.x.saturating_sub(1);
        } else {
            caret.pos.x = buf.get_buffer_width() - 1;
            self.caret_up(buf, caret);
        }
    }

    fn interpret_char(&mut self, buf: &mut Buffer, caret: &mut Caret, ch: u8) -> EngineResult<Option<String>>  {
        if self.got_esc {
            match ch {
                b'\\' => { // Black Background
                    caret.attr.set_is_concealed(false);
                    caret.attr.set_background(0);
                },
                b']' => { 
                    caret.attr.set_background(caret.attr.get_foreground());
                },
                b'I' => { caret.attr.set_is_blinking(false); },
                b'L' => { caret.attr.set_is_double_height(false); },
                b'X' => { if !self.is_in_graphic_mode { caret.attr.set_is_concealed(true) } },
                b'Y' => { self.is_contiguous = true; self.is_in_graphic_mode = true; },
                b'Z' => self.is_contiguous = false,
                b'^' => {self.hold_graphics = true; self.is_in_graphic_mode = true; } ,
                _ => {}
            }
        }
        if !self.hold_graphics {

            self.held_graphics_character = ' ';
        }

        let mut print_ch = ch;
        if self.got_esc || ch < 0x20  {
            print_ch = if self.hold_graphics { self.held_graphics_character as u8 } else { b' ' };
        } else {
            if self.is_in_graphic_mode {
                if ch >= 0x20 && ch < 0x40 || ch >= 0x60 && ch < 0x80 {
                    if print_ch < 0x40 {
                        print_ch -= 0x20;
                    } else {
                        print_ch -= 0x40;
                    }

                    if self.is_contiguous {
                        print_ch += 0x80;
                    } else {
                        print_ch += 0xC0;
                    }
                } 
                self.held_graphics_character = unsafe { char::from_u32_unchecked(print_ch as u32) };
            } 
        }
        // println!("print : '{}' ({}) esc:{}  attr:{}", unsafe { char::from_u32_unchecked(print_ch as u32) }, ch, self.got_esc, caret.attr);
        let ach = AttributedChar::new(unsafe { char::from_u32_unchecked(print_ch as u32) }, caret.attr);
        self.print_char(buf, caret, ach);

        if self.got_esc {
            match ch {
          
                b'A'..=b'G' => {// Alpha Red, Green, Yellow, Blue, Magenta, Cyan, White
                    self.is_in_graphic_mode = false;
                    caret.attr.set_is_concealed(false);
                    self.held_graphics_character = ' ';
                    caret.attr.set_foreground(1 + (ch - b'A') as u32);
                }
                b'Q'..=b'W' => {  // Graphics Red, Green, Yellow, Blue, Magenta, Cyan, White
                     if !self.is_in_graphic_mode {
                        self.is_in_graphic_mode = true;
                        self.held_graphics_character = ' ';
                    }
                    caret.attr.set_is_concealed(false);
                    caret.attr.set_foreground(1 + (ch - b'Q') as u32);
                },
                b'H' => { caret.attr.set_is_blinking(true);  },
        
                b'M' => {
                    caret.attr.set_is_double_height(true); 
                },
        
                b'_' => { self.hold_graphics = false;} ,

                _ => {}
            }
            self.got_esc = false;
        }
        Ok(None)
    }


}

impl BufferParser for ViewdataParser {
    fn from_unicode(&self, ch: char) -> char
    {
        if ch == ' ' {
            return ' ';
        }
        match UNICODE_TO_VIEWDATA.get(&ch) {
            Some(out_ch) => *out_ch,
            _ => ch
        }
    }

    fn to_unicode(&self, ch: char) -> char
    {
        match VIEWDATA_TO_UNICODE.get(ch as usize) {
            Some(out_ch) => *out_ch,
            _ => ch
        }
    }

    fn print_char(&mut self, buf: &mut Buffer, caret: &mut Caret, ch: char) -> EngineResult<Option<String>> {
        let ch = ch as u8;
        match ch {
            // control codes 0
            0b000_0000 => {} // ignore
            0b000_0001 => {} // ignore
            0b000_0010 => {} // STX
            0b000_0011 => {} // ETX
            0b000_0100 => {} // ignore
            0b000_0101 => { /*return Ok(Some("1\0".to_string())); */ } // ENQ - send identity number <= 16 digits - ignore doesn't work properly 2022
            0b000_0110 => {} // ACK
            0b000_0111 => {} // ignore
            0b000_1000 => {  // Caret left 0x08 
                self.caret_left(buf, caret);
            },
            0b000_1001 => { // Caret right 0x09
                self.caret_right(buf, caret);
            },
            0b000_1010 => { // Caret down 0x0A
                self.caret_down(buf, caret);
            } 
            0b000_1011 => {  // Caret up 0x0B
                self.caret_up(buf, caret);
            },
            0b000_1100 => { // 12 / 0x0C
                caret.ff(buf); 
                self.reset_screen();
            },
            0b000_1101 => {  // 13 / 0x0D
                self.fill_to_eol(buf, caret);
                caret.cr(buf);
            },
            0b000_1110 => { return Ok(None); } // TODO: SO - switch to G1 char set
            0b000_1111 => { return Ok(None); } // TODO: SI - switch to G0 char set

            // control codes 1
            0b001_0000 => {} // ignore
            0b001_0001 => caret.is_visible = true,
            0b001_0010 => {} // ignore
            0b001_0011 => {} // ignore
            0b001_0100 => caret.is_visible = false,
            0b001_0101 => {} // NAK
            0b001_0110 => {} // ignore
            0b001_0111 => {} // ignore
            0b001_1000 => {} // CAN
            0b001_1001 => {} // ignore
            0b001_1010 => {} // ignore
            0b001_1011 => { self.got_esc = true; return Ok(None); } // 0x1B ESC
            0b001_1100 => { return Ok(None); } // TODO: SS2 - switch to G2 char set
            0b001_1101 => { return Ok(None); } // TODO: SS3 - switch to G3 char set
            0b001_1110 => { // 28 / 0x1E
              //  self.fill_to_eol(buf, caret);
                caret.home(buf)
            },
            0b001_1111 => {} // ignore
            _ => {
                return self.interpret_char(buf, caret, ch);
            }
        }
        self.got_esc = false; 
        Ok(None)
    }
}


lazy_static::lazy_static!{
    static ref UNICODE_TO_VIEWDATA: std::collections::HashMap<char,char> = {
        let mut res = std::collections::HashMap::new();
        for a in 0..256 { 
            res.insert(VIEWDATA_TO_UNICODE[a], char::from_u32(a as u32).unwrap());
        }
        res
    };
}
pub const VIEWDATA_TO_UNICODE: [char; 256] = [
    ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', 
    ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', 
    ' ', '!', '"', 'f', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/', 
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?', 
    '@', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 
    'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '←', '½', '→', '↑', '#', 
    '\u{23AF}', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 
    'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '¼', '⏸', '¾','÷', '▉', 

    // graphics char sextants
    ' ',        '\u{1fb00}','\u{1fb01}','\u{1fb02}','\u{1fb03}','\u{1fb04}','\u{1fb05}','\u{1fb06}','\u{1fb07}','\u{1fb08}','\u{1fb09}','\u{1fb0a}','\u{1fb0b}','\u{1fb0c}','\u{1fb0d}','\u{1fb0e}',
    '\u{1fb0f}','\u{1fb10}','\u{1fb11}','\u{1fb12}','\u{1fb13}',        '▌','\u{1fb14}','\u{1fb15}','\u{1fb16}','\u{1fb17}','\u{1fb18}','\u{1fb19}','\u{1fb1a}','\u{1fb1b}','\u{1fb1c}','\u{1fb1d}',
    '\u{1fb1e}','\u{1fb1f}','\u{1fb20}','\u{1fb21}','\u{1fb22}','\u{1fb23}','\u{1fb24}','\u{1fb25}','\u{1fb26}','\u{1fb27}',        '▐','\u{1fb28}','\u{1fb29}','\u{1fb2a}','\u{1fb2b}','\u{1fb2c}',
    '\u{1fb2d}','\u{1fb2e}','\u{1fb2f}','\u{1fb30}','\u{1fb31}','\u{1fb32}','\u{1fb33}','\u{1fb34}','\u{1fb35}','\u{1fb36}','\u{1fb37}','\u{1fb38}','\u{1fb39}','\u{1fb3a}','\u{1fb3b}',        '█', 

    // no sextants for this variant :/
    ' ',        '\u{1fb00}','\u{1fb01}','\u{1fb02}','\u{1fb03}','\u{1fb04}','\u{1fb05}','\u{1fb06}','\u{1fb07}','\u{1fb08}','\u{1fb09}','\u{1fb0a}','\u{1fb0b}','\u{1fb0c}','\u{1fb0d}','\u{1fb0e}',
    '\u{1fb0f}','\u{1fb10}','\u{1fb11}','\u{1fb12}','\u{1fb13}',        '▌','\u{1fb14}','\u{1fb15}','\u{1fb16}','\u{1fb17}','\u{1fb18}','\u{1fb19}','\u{1fb1a}','\u{1fb1b}','\u{1fb1c}','\u{1fb1d}',
    '\u{1fb1e}','\u{1fb1f}','\u{1fb20}','\u{1fb21}','\u{1fb22}','\u{1fb23}','\u{1fb24}','\u{1fb25}','\u{1fb26}','\u{1fb27}',        '▐','\u{1fb28}','\u{1fb29}','\u{1fb2a}','\u{1fb2b}','\u{1fb2c}',
    '\u{1fb2d}','\u{1fb2e}','\u{1fb2f}','\u{1fb30}','\u{1fb31}','\u{1fb32}','\u{1fb33}','\u{1fb34}','\u{1fb35}','\u{1fb36}','\u{1fb37}','\u{1fb38}','\u{1fb39}','\u{1fb3a}','\u{1fb3b}',        '█', 
];


/* 


*/