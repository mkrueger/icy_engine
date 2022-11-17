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
    }

    fn fill_to_eol(&self, buf: &mut Buffer, caret: &Caret) {
        let y = caret.get_position().y;
        for x in caret.get_position().x..buf.get_buffer_width() {

            let p = Position::new(x, y);
            let mut ch = buf.get_char(p).unwrap();
            ch.attribute = caret.attr;
            buf.set_char(0, p, Some(ch));
        }
    }

    fn reset_on_row_change(&mut self, caret: &mut Caret) {
        self.reset_screen();
        caret.attr = TextAttribute::default();
    }
}

impl BufferParser for ViewdataParser {
    fn from_unicode(&self, ch: char) -> char
    {
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
        let mut ch = ch as u8;

        if self.got_esc {
            self.got_esc = false;

            let control_character = if self.hold_graphics { self.held_graphics_character }  else { ' ' };
            let ach = AttributedChar::new(control_character, caret.attr);
            buf.print_char(caret, ach);
            if caret.get_position().x >= buf.get_buffer_width() {
                self.reset_on_row_change(caret);
            }

            caret.attr.set_is_concealed(false); 
            match ch {
                b'A'..=b'G' => {// Alpha Red, Green, Yellow, Blue, Magenta, Cyan, Whita 
                    self.is_in_graphic_mode = false;
                    caret.attr.set_foreground(1 + (ch - b'A') as u32);
                }
                b'Q'..=b'W' => {  // Graphics Red, Green, Yellow, Blue, Magenta, Cyan, Whita
                    self.is_in_graphic_mode = true;
                    caret.attr.set_foreground(1 + (ch - b'Q') as u32);
                },
                b'H' => { caret.attr.set_is_blinking(true);  },
                b'I' => { caret.attr.set_is_blinking(false); },
                
                b'L' => { caret.attr.set_is_double_height(false); },
                b'M' => { caret.attr.set_is_double_height(true); },

                b'X' => { caret.attr.set_is_concealed(true) },
                b'Y' => self.is_contiguous = true,
                b'Z' => self.is_contiguous = false,
                0b101_1100=> caret.attr.set_background(0), // Black Background
                0b101_1101=> caret.attr.set_background(caret.attr.get_foreground()),
                0b101_1110=> self.hold_graphics = true,
                0b101_1111=> { self.hold_graphics = false; self.held_graphics_character = ' '; } ,

                0b001_0001 => {} // DC1 ?
                0b001_0010 => {} // DC2 ?
                0b001_0011 => {} // DC3 ?
                0b001_0100 => {} // DC4 ?
    
                _ => {}
            }

            
            return Ok(None);
        } 

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
            0b000_1000 => caret.left(buf, 1),
            0b000_1001 => caret.right(buf, 1),
            0b000_1010 => {
               /*  if let Some(cur_line) = &buf.layers[0].lines.get(caret.get_position().y as usize) {
                    let mut has_double_height_line = false;
                    for c in &cur_line.chars {
                        if let Some(c) = c {
                            if c.attribute.is_double_height() {
                                has_double_height_line = true;
                                break;
                            }
                        }
                    }
                    if has_double_height_line {
                        caret.lf(buf);
                    }
                }*/
                caret.lf(buf);
                self.reset_on_row_change(caret);
            } 
            0b000_1011 => caret.up(buf, 1), // 11 / 0x0B
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
            0b001_1011 => self.got_esc = true,
            0b001_1100 => { return Ok(None); } // TODO: SS2 - switch to G2 char set
            0b001_1101 => { return Ok(None); } // TODO: SS3 - switch to G3 char set
            0b001_1110 => { // 28 / 0x1C
                self.fill_to_eol(buf, caret);
                caret.home(buf)
            },
            0b001_1111 => {} // ignore
            
            _ => {
                if self.is_in_graphic_mode && (ch >= 0x20 && ch < 0x40 || ch >= 0x60 && ch < 0x80) {
                    if ch < 0x40 {
                        ch -= 0x20;
                    } else {
                        ch -= 0x40;
                    }
                
                    if self.is_contiguous {
                        ch += 0x80;
                    } else {
                        ch += 0xC0;
                    }
                    self.held_graphics_character = unsafe { char::from_u32_unchecked(ch as u32) };
                } else {
                    self.held_graphics_character = ' ';
                }
                let ch = unsafe { char::from_u32_unchecked(ch as u32) };
                let ach = AttributedChar::new(ch, caret.attr);
                buf.print_char(caret, ach);
                if caret.get_position().x >= buf.get_buffer_width() {
                    self.reset_on_row_change(caret);
                }
    
            }
        }
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

    // grapihcs char sextants
    ' ', '\u{1fb00}','\u{1fb01}','\u{1fb02}','\u{1fb03}','\u{1fb04}','\u{1fb05}','\u{1fb06}','\u{1fb07}','\u{1fb08}','\u{1fb09}','\u{1fb0a}','\u{1fb0b}','\u{1fb0c}','\u{1fb0d}','\u{1fb0e}',
    '\u{1fb0f}','\u{1fb10}','\u{1fb11}','\u{1fb12}','\u{1fb13}','\u{1fb14}','\u{1fb15}','\u{1fb16}','\u{1fb17}','\u{1fb18}','\u{1fb19}','\u{1fb1a}','\u{1fb1b}','\u{1fb1c}','\u{1fb1d}','\u{1fb1e}',
    '\u{1fb1f}','\u{1fb20}','\u{1fb21}','\u{1fb22}','\u{1fb23}','\u{1fb24}','\u{1fb25}','\u{1fb26}','\u{1fb27}','\u{1fb28}','\u{1fb29}','\u{1fb2a}','\u{1fb2b}','\u{1fb2c}','\u{1fb2d}','\u{1fb2e}',
    '\u{1fb2f}','\u{1fb30}','\u{1fb31}','\u{1fb32}','\u{1fb33}','\u{1fb34}','\u{1fb35}','\u{1fb36}','\u{1fb37}','\u{1fb38}','\u{1fb39}','\u{1fb3a}','\u{1fb3b}','\u{2588}', ' ', ' ',
    
    // no sextants for this variant :/
    ' ', '\u{1fb00}','\u{1fb01}','\u{1fb02}','\u{1fb03}','\u{1fb04}','\u{1fb05}','\u{1fb06}','\u{1fb07}','\u{1fb08}','\u{1fb09}','\u{1fb0a}','\u{1fb0b}','\u{1fb0c}','\u{1fb0d}','\u{1fb0e}',
    '\u{1fb0f}','\u{1fb10}','\u{1fb11}','\u{1fb12}','\u{1fb13}','\u{1fb14}','\u{1fb15}','\u{1fb16}','\u{1fb17}','\u{1fb18}','\u{1fb19}','\u{1fb1a}','\u{1fb1b}','\u{1fb1c}','\u{1fb1d}','\u{1fb1e}',
    '\u{1fb1f}','\u{1fb20}','\u{1fb21}','\u{1fb22}','\u{1fb23}','\u{1fb24}','\u{1fb25}','\u{1fb26}','\u{1fb27}','\u{1fb28}','\u{1fb29}','\u{1fb2a}','\u{1fb2b}','\u{1fb2c}','\u{1fb2d}','\u{1fb2e}',
    '\u{1fb2f}','\u{1fb30}','\u{1fb31}','\u{1fb32}','\u{1fb33}','\u{1fb34}','\u{1fb35}','\u{1fb36}','\u{1fb37}','\u{1fb38}','\u{1fb39}','\u{1fb3a}','\u{1fb3b}','\u{2588}', ' ', ' ',


];


/* 


*/