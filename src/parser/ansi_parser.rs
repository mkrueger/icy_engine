// Useful description: https://vt100.net/docs/vt510-rm/chapter4.html

use core::num;
use std::{cmp::{max, min}};

use crate::{Position, Buffer, TextAttribute, Caret, TerminalScrolling, OriginMode, AutoWrapMode, EngineResult, ParserError};

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

const ANSI_CSI: char = '[';
const ANSI_ESC: char = '\x1B';

const COLOR_OFFSETS : [u8; 8] = [ 0, 4, 2, 6, 1, 5, 3, 7 ];

impl AnsiParser {
    pub fn new() -> Self {
        AnsiParser {
            ascii_parser: AsciiParser::new(),
            ans_code: false,
            got_escape: false,
            saved_pos: Position::default(),
            parsed_numbers: Vec::new(),
            current_sequence: String::new(),
            saved_cursor_opt: None,
            in_custom_command: false
        }
    }

    fn start_sequence(&mut self, buf: &mut Buffer, caret: &mut Caret, ch: char) -> EngineResult<Option<String>> {
        self.got_escape = false;
        match ch {
            ANSI_CSI => {
                self.current_sequence.push('[');
                self.ans_code = true;
                self.parsed_numbers.clear();
                Ok(None)
            }
            '7' => {
                self.saved_cursor_opt = Some(caret.clone());
                Ok(None)
            }
            '8' => {
                if let Some(saved_caret) = &self.saved_cursor_opt {
                    *caret = saved_caret.clone();
                }
                Ok(None)
            }

            'c' => { // RIS—Reset to Initial State see https://vt100.net/docs/vt510-rm/RIS.html
                caret.ff(buf);
                Ok(None)
            }

            'D' => { // Index
                caret.index(buf);
                Ok(None)
            }
            'M' => { // Reverse Index
                caret.reverse_index(buf);
                Ok(None)
            }
            
            'E' => { // Next Line
                caret.next_line(buf);
                Ok(None)
            }
            
            _ => {
                Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())))
            }
        }
    }
}

impl BufferParser for AnsiParser {
    fn from_unicode(&self, ch: char) -> char
    {
        self.ascii_parser.from_unicode(ch)
    }
    
    fn to_unicode(&self, ch: char) -> char
    {
        self.ascii_parser.to_unicode(ch)
    }

    fn print_char(&mut self, buf: &mut Buffer, caret: &mut Caret, ch: char) -> EngineResult<Option<String>>
    {
        if self.got_escape {
            return self.start_sequence(buf, caret, ch);
        }
    
        if self.ans_code {
            if let Some(ch) = char::from_u32(ch as u32) {
                self.current_sequence.push(ch);
            } else {
                return Err(Box::new(ParserError::InvalidChar('\0')));
            }

            if self.in_custom_command && !(ch >= '0' && ch <= '9') {
                self.in_custom_command  = false; 
                self.ans_code = false;
                self.got_escape = false;
                match ch {
                    'p' => { // [!p Soft Teminal Reset
                        buf.terminal_state.reset();
                    }
                    'l' => {
                        if self.parsed_numbers.len() != 1 {
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                        }
                        match self.parsed_numbers.get(0) {
                            Some(4) => buf.terminal_state.scroll_state = TerminalScrolling::Fast,
                            Some(6) => {
                               //  buf.terminal_state.origin_mode = OriginMode::WithinMargins;
                            }
                            Some(7) => buf.terminal_state.auto_wrap_mode = AutoWrapMode::NoWrap,
                            Some(25) => caret.is_visible = false,
                            _ => { 
                                return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                            }
                        } 
                    }
                    'h' => {
                        if self.parsed_numbers.len() != 1 {
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                        }
                        match self.parsed_numbers.get(0) {
                            Some(4) => buf.terminal_state.scroll_state = TerminalScrolling::Smooth,
                            Some(6) => buf.terminal_state.origin_mode = OriginMode::UpperLeftCorner,
                            Some(7) => buf.terminal_state.auto_wrap_mode = AutoWrapMode::AutoWrap,
                            Some(25) => caret.is_visible = true,
                            _ => { 
                                return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                            }
                        } 
                    }
                    _ => {
                        // error in control sequence, terminate reading
                        return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                    }
                }
                return Ok(None);
            }
            match ch {
                'm' => { // Select Graphic Rendition 
                    self.ans_code = false;
                    if self.parsed_numbers.len() == 0 {
                        caret.attr = TextAttribute::default(); // Reset or normal 
                    } 
                    for n in &self.parsed_numbers {
                        match n {
                            0 => caret.attr = TextAttribute::default(), // Reset or normal 
                            1 => caret.attr.set_is_bold(true),
                            2 => caret.attr.set_is_faint(true),
                            3 => caret.attr.set_is_italic(true),
                            4 => caret.attr.set_is_underlined(true), 
                            5 | 6 => caret.attr.set_is_blinking(true),
                            7 => {
                                let fg = caret.attr.get_foreground();
                                caret.attr.set_foreground(caret.attr.get_background());
                                caret.attr.set_background(fg);
                            }
                            8 => caret.attr.set_is_concealed(true),
                            9 => caret.attr.set_is_crossed_out(true),
                            10 => return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone()))),
                            11..=19 => return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone()))),
                            21 => { caret.attr.set_is_double_underlined(true) },
                            22 => { caret.attr.set_is_bold(false); caret.attr.set_is_faint(false) },
                            23 => caret.attr.set_is_italic(false),
                            24 => caret.attr.set_is_underlined(false),
                            25 => caret.attr.set_is_blinking(false),
                            27 => return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone()))),
                            28 => caret.attr.set_is_concealed(false),
                            29 => caret.attr.set_is_crossed_out(false),
                            // set foreaground color
                            30..=37 => caret.attr.set_foreground(COLOR_OFFSETS[*n as usize - 30]),
                            // set background color
                            40..=47 => caret.attr.set_background(COLOR_OFFSETS[*n as usize - 40]),
                            _ => { 
                                return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                            }
                        }
                    }
                    return Ok(None);
                }
                'H' | 'f' => { // Cursor Position + Horizontal Vertical Position ('f')
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
                'C' => { // Cursor Forward 
                    self.ans_code = false;
                    if self.parsed_numbers.is_empty() {
                        caret.right(buf, 1);
                    } else {
                        caret.right(buf, self.parsed_numbers[0]);
                    }
                    return Ok(None);
                }
                'D' => { // Cursor Back 
                    self.ans_code = false;
                    if self.parsed_numbers.is_empty() {
                        caret.left(buf, 1);
                    } else {
                        caret.left(buf, self.parsed_numbers[0]);
                    }
                    return Ok(None);
                }
                'A' => { // Cursor Up 
                    self.ans_code = false;
                    if self.parsed_numbers.is_empty() {
                        caret.up(buf, 1);
                    } else {
                        caret.up(buf, self.parsed_numbers[0]);
                    }
                    return Ok(None);
                }
                'b' => { // Cursor Down 
                    self.ans_code = false;
                    if self.parsed_numbers.is_empty() {
                        caret.down(buf, 1);
                    } else {
                        caret.down(buf, self.parsed_numbers[0]);
                    }
                    return Ok(None);
                }
                's' => { // Save Current Cursor Position
                    self.ans_code = false;
                    self.saved_pos = caret.pos;
                    return Ok(None);
                }
                'u' => { // Restore Saved Cursor Position 
                    self.ans_code = false;
                    caret.pos = self.saved_pos;
                    return Ok(None);
                }
                
                'd' => { // Vertical Line Position Absolute
                    self.ans_code = false;
                    let num  = match self.parsed_numbers.get(0) { 
                        Some(n) => n - 1,
                        _ => 0
                    };
                    caret.pos.y =  buf.get_first_visible_line() + num;
                    buf.terminal_state.limit_caret_pos(buf, caret);
                    return Ok(None);
                }
                'e' => { // Vertical Line Position Relative
                    self.ans_code = false;
                    let num  = match self.parsed_numbers.get(0) { 
                        Some(n) => *n,
                        _ => 1
                    };
                    caret.pos.y = buf.get_first_visible_line() + caret.pos.y + num;
                    buf.terminal_state.limit_caret_pos(buf, caret);
                    return Ok(None);
                }
                '\'' => { // Horizontal Line Position Absolute
                    self.ans_code = false;
                    let num = match self.parsed_numbers.get(0) { 
                        Some(n) => n - 1,
                        _ => 0
                    };
                    if let Some(layer) = &buf.layers.get(0) {
                        if let Some(line) = layer.lines.get(caret.pos.y as usize) {
                            caret.pos.x = min(line.get_line_length() as i32 + 1, max(0, num));
                            buf.terminal_state.limit_caret_pos(buf, caret);
                        }
                    } else {
                        return Err(Box::new(ParserError::InvalidBuffer));
                    }
                    return Ok(None);
                }
                'a' => { // Horizontal Line Position Relative
                    self.ans_code = false;
                    let num = match self.parsed_numbers.get(0) { 
                        Some(n) => *n,
                        _ => 1
                    };
                    if let Some(layer) = &buf.layers.get(0) {
                        if let Some(line) = layer.lines.get(caret.pos.y as usize) {
                            caret.pos.x = min(line.get_line_length() as i32 + 1, caret.pos.x + num);
                            buf.terminal_state.limit_caret_pos(buf, caret);
                        }
                    } else {
                        return Err(Box::new(ParserError::InvalidBuffer));
                    }

                    return Ok(None);
                }
                
                'G' => { // Cursor Horizontal Absolute
                    self.ans_code = false;
                    let num = match self.parsed_numbers.get(0) { 
                        Some(n) => n - 1,
                        _ => 0
                    };
                    caret.pos.x = num;
                    buf.terminal_state.limit_caret_pos(buf, caret);
                    return Ok(None);
                }
                'E' => { // Cursor Next Line
                    self.ans_code = false;
                    let num  = match self.parsed_numbers.get(0) { 
                        Some(n) => *n,
                        _ => 1
                    };
                    caret.pos.y = buf.get_first_visible_line() + caret.pos.y + num;
                    caret.pos.x = 0;
                    buf.terminal_state.limit_caret_pos(buf, caret);
                    return Ok(None);
                }
                'F' => { // Cursor Previous Line
                    self.ans_code = false;
                    let num  = match self.parsed_numbers.get(0) { 
                        Some(n) => *n,
                        _ => 1
                    };
                    caret.pos.y = buf.get_first_visible_line() + caret.pos.y - num;
                    caret.pos.x = 0;
                    buf.terminal_state.limit_caret_pos(buf, caret);
                    return Ok(None);
                }
                        
                'n' => { // Device Status Report 
                    self.ans_code = false;
                    if self.parsed_numbers.is_empty() {
                        return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                    }
                    if self.parsed_numbers.len() != 1 {
                        return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
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
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
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

                'M' => { // Delete Line
                    self.ans_code = false;
                    if self.parsed_numbers.is_empty() {
                        if let Some(layer) = buf.layers.get(0) {
                            if caret.pos.y  < layer.lines.len() as i32 {
                                buf.remove_terminal_line(caret.pos.y);
                            }
                        } else {
                            return Err(Box::new(ParserError::InvalidBuffer));
                        }
                    } else {
                        if self.parsed_numbers.len() != 1 {
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                        }
                        if let Some(number) =self.parsed_numbers.get(0) {
                            let mut number = *number;
                            if let Some(layer) = buf.layers.get(0) {
                                number = min(number, layer.lines.len() as i32 - caret.pos.y);
                            } else {
                                return Err(Box::new(ParserError::InvalidBuffer));
                            }
                            for _ in 0..number {
                                buf.remove_terminal_line(caret.pos.y);
                            }
                        } else {
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                        }
                    }
                    return Ok(None);
                }
                
                'P' => { // Delete character
                    self.ans_code = false;
                    if self.parsed_numbers.is_empty() {
                        caret.del(buf);
                    } else {
                        if self.parsed_numbers.len() != 1 {
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                        }
                        if let Some(number) = self.parsed_numbers.get(0) {
                            for _ in 0..*number {
                                caret.del(buf);
                            }
                        } else {
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                        }
                    }
                    return Ok(None);
                }

                'L' => { // Insert line 
                    self.ans_code = false;
                    if self.parsed_numbers.is_empty() {
                        buf.insert_terminal_line(caret.pos.y);
                    } else {
                        if self.parsed_numbers.len() != 1 {
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                        }
                        if let Some(number) = self.parsed_numbers.get(0) {
                            for _ in 0..*number {
                                buf.insert_terminal_line(caret.pos.y);
                            }
                        } else {
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                        }
                    }
                    return Ok(None);
                }
                
                'J' => { // Erase in Display 
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
                                    return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                                }
                            }
                        } else {
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                        }
                    }
                    return Ok(None);
                }
                
                '?' => { // read custom command
                    self.in_custom_command = true;
                    return Ok(None);
                }

                'K' => { // Erase in line
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
                                return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                            }
                        }
                    } else {
                        buf.clear_line_end(&caret.pos);
                    }
                    return Ok(None);
                }
                
                'c' => { // device attributes
                    self.ans_code = false;
                    return Ok(Some("\x1b[?1;0c".to_string()));
                }

                'r' => { // Set Top and Bottom Margins
                    self.ans_code = false;
                    if self.parsed_numbers.len() != 2 {
                        return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
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
                'h' => {
                    self.ans_code = false;
                    if self.parsed_numbers.len() != 1 {
                        return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                    }
                    match self.parsed_numbers.get(0) {
                        Some(4) => { caret.insert_mode = true; }
                        _ => return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())))
                    }
                    return Ok(None);
                }

                'l' => {
                    self.ans_code = false;
                    if self.parsed_numbers.len() != 1 {
                        return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                    }
                    match self.parsed_numbers.get(0)  {
                        Some(4) => { caret.insert_mode = false; }
                        _ => return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())))
                    }
                    return Ok(None);
                }
                '~' => {
                    self.ans_code = false;
                    if self.parsed_numbers.len() != 1 {
                        return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                    }
                    match self.parsed_numbers.get(0) {
                        Some(1) => { caret.pos.x = 0; } // home
                        Some(2) => { caret.ins(buf); } // home
                        Some(3) => { caret.del(buf); }
                        Some(4) => { caret.eol(buf); }
                        Some(5) => {} // pg up 
                        Some(6) => {} // pg dn
                        _ => return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())))
                    }
                    return Ok(None);
                }

                _ => {
                    if ('\x40'..='\x7E').contains(&ch) {
                        // unknown control sequence, terminate reading
                        self.ans_code = false;
                        self.got_escape = false;
                        return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                    }
    
                    if ('0'..='9').contains(&ch) {
                        let d = match self.parsed_numbers.pop() {
                            Some(number) => number,
                            _ => 0
                        };
                        self.parsed_numbers.push(d * 10 + ch as i32 - b'0' as i32);
                    } else if ch == ';' {
                        self.parsed_numbers.push(0);
                        return Ok(None);
                    } else {
                        self.ans_code = false;
                        self.got_escape = false;
                        // error in control sequence, terminate reading
                        return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
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