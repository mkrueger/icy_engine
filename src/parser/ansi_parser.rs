// Useful description: https://vt100.net/docs/vt510-rm/chapter4.html

use std::{io, cmp::{max, min}, sync::Arc};

use crate::{Position, Buffer, TextAttribute, Caret, TerminalScrolling, OriginMode, AutoWrapMode};

use super::{BufferParser, AsciiParser};

pub struct AnsiParser {
    ascii_parser: AsciiParser,
    got_escape: bool,
    ans_code: bool,
    saved_pos: Position,
    saved_cursor_opt: Option<Caret>,
    parsed_numbers: Vec<i32>,
    current_sequence: String,
    in_custom_command: bool
}

const ANSI_CSI: u8 = b'[';
const ANSI_ESC: u8 = 27;

const COLOR_OFFSETS : [u8; 8] = [ 0, 4, 2, 6, 1, 5, 3, 7 ];

impl AnsiParser {
    pub fn new() -> Self {
        AnsiParser {
            ascii_parser: AsciiParser::new(),
            ans_code: false,
            got_escape: false,
            saved_pos: Position::new(),
            parsed_numbers: Vec::new(),
            current_sequence: String::new(),
            saved_cursor_opt: None,
            in_custom_command: false
        }
    }

    fn start_sequence(&mut self, buf: &mut Buffer, caret: &mut Caret, ch: u8) -> io::Result<Option<String>> {
        self.got_escape = false;
        match ch {
            ANSI_CSI => {
                self.current_sequence.push(char::from_u32(ch as u32).unwrap());
                self.ans_code = true;
                self.parsed_numbers.clear();
                Ok(None)
            }
            b'7' => {
                self.saved_cursor_opt = Some(caret.clone());
                Ok(None)
            }
            b'8' => {
                if let Some(saved_caret) = &self.saved_cursor_opt {
                    *caret = saved_caret.clone();
                }
                Ok(None)
            }

            b'c' => { // RIS—Reset to Initial State see https://vt100.net/docs/vt510-rm/RIS.html
                caret.ff(buf);
                Ok(None)
            }

            b'D' => { // Index
                caret.index(buf);
                Ok(None)
            }
            b'M' => { // Reverse Index
                caret.reverse_index(buf);
                Ok(None)
            }
            
            b'E' => { // Next Line
                caret.next_line(buf);
                Ok(None)
            }
            
            _ => {
                Err(io::Error::new(io::ErrorKind::InvalidData, format!("Unknown escape char 0x{:x}('{:?}')", ch, char::from_u32(ch as u32))))
            }
        }
    }
}

impl BufferParser for AnsiParser {
    fn from_unicode(&self, ch: char) -> u8
    {
        self.ascii_parser.from_unicode(ch)
    }

    fn print_char(&mut self, buf: &mut Buffer, caret: &mut Caret, ch: u8) -> io::Result<Option<String>>
    {
        if self.got_escape {
            return self.start_sequence(buf, caret, ch);
        }
    
        if self.ans_code {
            if let Some(ch) = char::from_u32(ch as u32) {
                self.current_sequence.push(ch);
            } else {
                return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Can't convert char {}", ch)));
            }

            if self.in_custom_command && !(ch >= b'0' && ch <= b'9') {
                self.in_custom_command  = false; 
                self.ans_code = false;
                self.got_escape = false;
                match ch {
                    b'p' => { // [!p Soft Teminal Reset
                        buf.terminal_state.reset();
                    }
                    b'l' => {
                        if self.parsed_numbers.len() != 1 {
                            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Unsuppored custom command: {}, {:?}!", self.current_sequence, char::from_u32(ch as u32))));
                        }
                        match self.parsed_numbers[0] {
                            4 => { buf.terminal_state.scroll_state = TerminalScrolling::Fast; }
                            6 => {
                               //  buf.terminal_state.origin_mode = OriginMode::WithinMargins;
                            }
                            7 => { buf.terminal_state.auto_wrap_mode = AutoWrapMode::NoWrap; }
                            25 => {
                                caret.is_visible = false;
                            }
                            _ => { 
                                return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Unsuppored custom command: {}, {:?}!", self.current_sequence, char::from_u32(ch as u32))));
                            }
                        } 
                    }
                    b'h' => {
                        if self.parsed_numbers.len() != 1 {
                            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Unsuppored custom command: {}, {:?}!", self.current_sequence, char::from_u32(ch as u32))));
                        }
                        match self.parsed_numbers[0] {
                            4 => { buf.terminal_state.scroll_state = TerminalScrolling::Smooth; }
                            6 => { buf.terminal_state.origin_mode = OriginMode::UpperLeftCorner; }
                            7 => { buf.terminal_state.auto_wrap_mode = AutoWrapMode::AutoWrap; }
                            25 => {
                                caret.is_visible = true;
                            }
                            _ => { 
                                return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Unsuppored custom command: {}, {:?}!", self.current_sequence, char::from_u32(ch as u32))));
                            }
                        } 
                    }
                    _ => {
                        // error in control sequence, terminate reading
                        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Unsuppored custom command: {}, {:?}!", self.current_sequence, char::from_u32(ch as u32))));
                    }
                }
                return Ok(None);
            }
            match ch {
                b'm' => { // Select Graphic Rendition 
                    self.ans_code = false;
                    if self.parsed_numbers.len() == 0 {
                        caret.attr = TextAttribute::DEFAULT; // Reset or normal 
                    } 
                    for n in &self.parsed_numbers {
                        match n {
                            0 => caret.attr = TextAttribute::DEFAULT, // Reset or normal 
                            1 => caret.attr.set_foreground_bold(true),    // Bold or increased intensity 
                            4 => caret.attr.set_is_underlined(true), 
                            24 =>caret.attr.set_is_underlined(false),
                            5 => if buf.buffer_type.use_ice_colors() { 
                                caret.attr.set_background_bold(true);
                            }  else  {
                                caret.attr.set_is_blinking(true);  // Slow blink 
                            }
                            7 => {
                                let fg = caret.attr.get_foreground();
                                caret.attr.set_foreground(caret.attr.get_background());
                                caret.attr.set_background(fg);
                            }
                            //return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Negative image not supported: {}", self.current_sequence))),
                            8 => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invisible image not supported: {}", self.current_sequence))),
                            10 => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("ASCII char set (SCO only) not supported: {}", self.current_sequence))),
                            11 => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Map 00-7F not supported: {}", self.current_sequence))),
                            12 => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Map 80-FF not supported: {}", self.current_sequence))),
                            22 => caret.attr.set_foreground_bold(false),    // Bold off
                            25 => if buf.buffer_type.use_ice_colors() {  // blink off
                                caret.attr.set_background_bold(false);
                            }  else  {
                                caret.attr.set_is_blinking(false);
                            }
                            27 => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Negative image off not supported: {}", self.current_sequence))),
                            28 => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invisible image off not supported: {}", self.current_sequence))),
                            // set foreaground color
                            30..=37 => caret.attr.set_foreground_without_bold(COLOR_OFFSETS[*n as usize - 30]),
                            // set background color
                            40..=47 => caret.attr.set_background_without_bold(COLOR_OFFSETS[*n as usize - 40]),
                            _ => { 
                                return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Unsupported ANSI graphic code {} in seq {}", n, self.current_sequence)));
                            }
                        }
                    }
                    return Ok(None);
                }
                b'H' | b'f' => { // Cursor Position + Horizontal Vertical Position ('f')
                self.ans_code = false;
                    if !self.parsed_numbers.is_empty() {
                        if self.parsed_numbers[0] > 0 { 
                            // always be in terminal mode for gotoxy
                            caret.pos.y =  buf.get_first_visible_line() + self.parsed_numbers[0] - 1;
                        }
                        if self.parsed_numbers.len() > 1 {
                            if self.parsed_numbers[1] > 0 {
                                caret.pos.x = self.parsed_numbers[1] - 1;
                            }
                        } else {
                            caret.pos.x = 0;
                        }
                    } else {
                        caret.pos = buf.upper_left_position();
                    }
                    buf.terminal_state.limit_caret_pos(buf, caret);
                    return Ok(None);
                }
                b'C' => { // Cursor Forward 
                    self.ans_code = false;
                    if self.parsed_numbers.is_empty() {
                        caret.right(buf, 1);
                    } else {
                        caret.right(buf, self.parsed_numbers[0]);
                    }
                    return Ok(None);
                }
                b'D' => { // Cursor Back 
                    self.ans_code = false;
                    if self.parsed_numbers.is_empty() {
                        caret.left(buf, 1);
                    } else {
                        caret.left(buf, self.parsed_numbers[0]);
                    }
                    return Ok(None);
                }
                b'A' => { // Cursor Up 
                    self.ans_code = false;
                    if self.parsed_numbers.is_empty() {
                        caret.up(buf, 1);
                    } else {
                        caret.up(buf, self.parsed_numbers[0]);
                    }
                    return Ok(None);
                }
                b'B' => { // Cursor Down 
                    self.ans_code = false;
                    if self.parsed_numbers.is_empty() {
                        caret.down(buf, 1);
                    } else {
                        caret.down(buf, self.parsed_numbers[0]);
                    }
                    return Ok(None);
                }
                b's' => { // Save Current Cursor Position
                    self.ans_code = false;
                    self.saved_pos = caret.pos;
                    return Ok(None);
                }
                b'u' => { // Restore Saved Cursor Position 
                    self.ans_code = false;
                    caret.pos = self.saved_pos;
                    return Ok(None);
                }
                
                b'd' => { // Vertical Line Position Absolute
                    self.ans_code = false;
                    let num  = if !self.parsed_numbers.is_empty() {
                        self.parsed_numbers[0] - 1
                    } else {
                        0
                    };
                    caret.pos.y =  buf.get_first_visible_line() + num;
                    buf.terminal_state.limit_caret_pos(buf, caret);
                    return Ok(None);
                }
                b'e' => { // Vertical Line Position Relative
                    self.ans_code = false;
                    let num  = if !self.parsed_numbers.is_empty() {
                        self.parsed_numbers[0]
                    } else {
                        1
                    };
                    caret.pos.y = buf.get_first_visible_line() + caret.pos.y + num;
                    buf.terminal_state.limit_caret_pos(buf, caret);
                    return Ok(None);
                }
                b'\'' => { // Horizontal Line Position Absolute
                    self.ans_code = false;
                    let num = if !self.parsed_numbers.is_empty() {
                        self.parsed_numbers[0] - 1
                    } else {
                        0
                    };
                    if let Some(line) = buf.layers[0].lines.get(caret.pos.y as usize) {
                        caret.pos.x = min(line.get_line_length() as i32 + 1, max(0, num));
                        buf.terminal_state.limit_caret_pos(buf, caret);
                    }
                    return Ok(None);
                }
                b'a' => { // Horizontal Line Position Relative
                    self.ans_code = false;
                    let num = if !self.parsed_numbers.is_empty() {
                        self.parsed_numbers[0]
                    } else {
                        1
                    };
                    if let Some(line) = buf.layers[0].lines.get(caret.pos.y as usize) {
                        caret.pos.x = min(line.get_line_length() as i32 + 1, caret.pos.x + num);
                        buf.terminal_state.limit_caret_pos(buf, caret);
                    }
                    return Ok(None);
                }
                
                b'G' => { // Cursor Horizontal Absolute
                    self.ans_code = false;
                    let num = if !self.parsed_numbers.is_empty() {
                        self.parsed_numbers[0] - 1
                    } else {
                        0
                    };
                    caret.pos.x = num;
                    buf.terminal_state.limit_caret_pos(buf, caret);
                    return Ok(None);
                }
                b'E' => { // Cursor Next Line
                    self.ans_code = false;
                    let num  = if !self.parsed_numbers.is_empty() {
                        self.parsed_numbers[0]
                    } else {
                        1
                    };
                    caret.pos.y = buf.get_first_visible_line() + caret.pos.y + num;
                    caret.pos.x = 0;
                    buf.terminal_state.limit_caret_pos(buf, caret);
                    return Ok(None);
                }
                b'F' => { // Cursor Previous Line
                    self.ans_code = false;
                    let num  = if !self.parsed_numbers.is_empty() {
                        self.parsed_numbers[0]
                    } else {
                        1
                    };
                    caret.pos.y = buf.get_first_visible_line() + caret.pos.y - num;
                    caret.pos.x = 0;
                    buf.terminal_state.limit_caret_pos(buf, caret);
                    return Ok(None);
                }
                        
                b'n' => { // Device Status Report 
                    self.ans_code = false;
                    if self.parsed_numbers.is_empty() {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("empty number")));
                    }
                    if self.parsed_numbers.len() != 1 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("too many 'n' params in ANSI escape sequence: {}", self.parsed_numbers.len())));
                    }
                    match self.parsed_numbers[0] {
                        5 => { // Device status report
                            return Ok(Some("\x1b[0n".to_string()));
                        },
                        6 => { // Get cursor position
                            let s = format!("\x1b[{};{}R", min(buf.get_buffer_height() as i32, caret.pos.y + 1), min(buf.get_buffer_width() as i32, caret.pos.x + 1));
                            return Ok(Some(s));
                        },
                        _ => {
                            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("unknown ANSI n sequence {}", self.parsed_numbers[0])));
                        }
                    }
                }
                
                /*  TODO: 
                    Insert Character  CSI Pn @
                    Insert Column 	  CSI Pn ' }
                    Erase Character   CSI Pn X
                    Delete Line       CSI Pn M
                    Delete Column 	  CSI Pn ' ~
*/

                b'M' => { // Delete Line
                    self.ans_code = false;
                    if self.parsed_numbers.is_empty() {
                        if caret.pos.y  < buf.layers[0].lines.len() as i32 {
                            buf.remove_terminal_line(caret.pos.y);
                        }
                    } else {
                        if self.parsed_numbers.len() != 1 {
                            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid parameters sequence {}", self.parsed_numbers[0])));
                        }
                        if let Some(number) = self.parsed_numbers.get(0) {
                            for _ in 0..*number {
                                if caret.pos.y >= buf.layers[0].lines.len() as i32 {
                                    break;
                                }
                                buf.remove_terminal_line(caret.pos.y);
                            }
                        } else {
                            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid number in sequence {}", self.parsed_numbers[0])));
                        }
                    }
                    return Ok(None);
                }
                
                b'P' => { // Delete character
                    self.ans_code = false;
                    if self.parsed_numbers.is_empty() {
                        caret.del(buf);
                    } else {
                        if self.parsed_numbers.len() != 1 {
                            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid parameters sequence {}", self.parsed_numbers[0])));
                        }
                        if let Some(number) = self.parsed_numbers.get(0) {
                            for _ in 0..*number {
                                caret.del(buf);
                            }
                        } else {
                            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid number in sequence {}", self.parsed_numbers[0])));
                        }
                    }
                    return Ok(None);
                }

                b'L' => { // Insert line 
                    self.ans_code = false;
                    if self.parsed_numbers.is_empty() {
                        buf.insert_terminal_line(caret.pos.y);
                    } else {
                        if self.parsed_numbers.len() != 1 {
                            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid parameters sequence {}", self.parsed_numbers[0])));
                        }
                        if let Some(number) = self.parsed_numbers.get(0) {
                            for _ in 0..*number {
                                buf.insert_terminal_line(caret.pos.y);
                            }
                        } else {
                            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid number in sequence {}", self.parsed_numbers[0])));
                        }
                    }
                    return Ok(None);
                }
                
                b'J' => { // Erase in Display 
                    self.ans_code = false;
                    if self.parsed_numbers.is_empty() {
                        buf.clear_buffer_down(caret.pos.y);
                    } else {
                        if let Some(number) = self.parsed_numbers.get(0) {
                            match *number {
                                0 => {
                                    buf.clear_buffer_down(caret.pos.y);
                                }
                                1 => {
                                    buf.clear_buffer_up(caret.pos.y);
                                }
                                2 |  // clear entire screen
                                3 
                                => {
                                    buf.clear_screen(caret);
                                } 
                                _ => {
                                    buf.clear_buffer_down(caret.pos.y);
                                    return Err(io::Error::new(io::ErrorKind::InvalidData, format!("unknown ANSI J sequence {} in {}", self.parsed_numbers[0], self.current_sequence)));
                                }
                            }
                        } else {
                            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid number in sequence {}", self.parsed_numbers[0])));
                        }
                    }
                    return Ok(None);
                }
                
                b'?' => { // read custom command
                    self.in_custom_command = true;
                    return Ok(None);
                }

                b'K' => { // Erase in line
                    self.ans_code = false;
                    if self.parsed_numbers.len() > 0 {
                        match self.parsed_numbers[0] {
                            0 => { 
                                buf.clear_line_end(&caret.pos);
                            },
                            1 => {
                                buf.clear_line_start(&caret.pos);
                            },
                            2 => {
                                buf.clear_line(caret.pos.y);
                            },
                            _ => {
                                return Err(io::Error::new(io::ErrorKind::InvalidData, format!("unknown ANSI K sequence {}", self.parsed_numbers[0])));
                            }
                        }
                    } else {
                        buf.clear_line_end(&caret.pos);
                    }
                    return Ok(None);
                }
                
                b'c' => { // device attributes
                    self.ans_code = false;
                    return Ok(Some("\x1b[?1;0c".to_string()));
                }

                b'r' => { // Set Top and Bottom Margins
                    self.ans_code = false;
                    if self.parsed_numbers.len() != 2 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid set top and bottom margin sequence {}", self.current_sequence)));
                    }
                    let start = self.parsed_numbers[0] - 1;
                    let end = self.parsed_numbers[1] - 1;

                    if start > end {
                        // undocumented behavior but CSI 1; 0 r seems to turn off on some terminals.
                        buf.terminal_state.margins  = None;
                        return Ok(None);
                    }
                    caret.pos = buf.upper_left_position();
                    buf.terminal_state.margins  = Some((start, end));
                    return Ok(None);
                }
                b'h' => {
                    self.ans_code = false;
                    if self.parsed_numbers.len() != 1 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid h sequence {}", self.current_sequence)));
                    }
                    match self.parsed_numbers[0] {
                        4 => { caret.insert_mode = true; }
                        _ => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Unknown h sequence {}", self.current_sequence)))
                    }
                    return Ok(None);
                }

                b'l' => {
                    self.ans_code = false;
                    if self.parsed_numbers.len() != 1 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid l sequence {}", self.current_sequence)));
                    }
                    match self.parsed_numbers[0] {
                        4 => { caret.insert_mode = false; }
                        _ => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Unknown l sequence {}", self.current_sequence)))
                    }
                    return Ok(None);
                }
                _ => {
                    if (0x40..=0x7E).contains(&ch) {
                        // unknown control sequence, terminate reading
                        self.ans_code = false;
                        self.got_escape = false;
                        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("unknown control sequence {}/char:{:?} in {}", ch, char::from_u32(ch as u32), self.current_sequence)));
                    }
    
                    if (b'0'..=b'9').contains(&ch) {
                        if self.parsed_numbers.is_empty() {
                            self.parsed_numbers.push(0);
                        }
                        let d = self.parsed_numbers.pop().unwrap();
                        self.parsed_numbers.push(d * 10 + (ch - b'0') as i32);
                    } else if ch == b';' {
                        self.parsed_numbers.push(0);
                        return Ok(None);
                    } else {
                        self.ans_code = false;
                        self.got_escape = false;
                        // error in control sequence, terminate reading
                        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("error in ANSI control sequence: {}, {}!", self.current_sequence, ch)));
                    }
                    return Ok(None);
                }
            }
        }
    
        if ch == ANSI_ESC {
            self.current_sequence.clear();
            self.current_sequence.push_str("<ESC>");
            self.ans_code = false;
            self.got_escape = true;
            return Ok(None)
        } 
        
        self.ascii_parser.print_char(buf, caret, ch) 
    }
}