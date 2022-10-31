// Useful description: https://vt100.net/docs/vt510-rm/chapter4.html

use std::{io, cmp::{max, min}};

use crate::{Position, Buffer, TextAttribute, Caret, Line};

use super::{BufferParser, AsciiParser};

pub struct AnsiParser {
    ascii_parser: AsciiParser,
    got_escape: bool,
    ans_code: bool,
    saved_pos: Position,
    saved_cursor_opt: Option<Caret>,
    parsed_numbers: Vec<i32>,
    current_sequence: String
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
            saved_cursor_opt: None
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
                self.saved_cursor_opt = None;
                Ok(None)
            }

            b'c' => { // RIS—Reset to Initial State see https://vt100.net/docs/vt510-rm/RIS.html
                caret.ff(buf);
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

            match ch {
                b'm' => { // Select Graphic Rendition 
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
                            7 => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Negative image not supported: {}", self.current_sequence))),
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
                    self.ans_code = false;
                    return Ok(None);
                }
                b'H' | b'f' => { // Cursor Position + Horizontal Vertical Position ('f')
                    if !self.parsed_numbers.is_empty() {
                        if self.parsed_numbers[0] > 0 { 
                            // always be in terminal mode for gotoxy
                            caret.pos.y =  min(buf.get_first_visible_line() + buf.height - 1, max(0, buf.get_first_visible_line() + self.parsed_numbers[0] - 1));
                            println!("parsed line:{} first visible: {}, last line: {} caret line:{}", self.parsed_numbers[0], buf.get_first_visible_line(), buf.get_first_visible_line() + buf.height, caret.pos.y)
                        }
                        if self.parsed_numbers.len() > 1 {
                            if self.parsed_numbers[1] > 0 {
                                caret.pos.x =  max(0, self.parsed_numbers[1] - 1);
                            }
                        } else {
                            caret.pos.x = 0;
                        }
                    } else {
                        caret.pos = Position::new();
                    }
                    self.ans_code = false;
                    return Ok(None);
                }
                b'C' => { // Cursor Forward 
                    if self.parsed_numbers.is_empty() {
                        caret.right(buf, 1);
                    } else {
                        caret.right(buf, self.parsed_numbers[0]);
                    }
                    self.ans_code = false;
                    return Ok(None);
                }
                b'D' => { // Cursor Back 
                    if self.parsed_numbers.is_empty() {
                        caret.left(buf, 1);
                    } else {
                        caret.left(buf, self.parsed_numbers[0]);
                    }
                    self.ans_code = false;
                    return Ok(None);
                }
                b'A' => { // Cursor Up 
                    if self.parsed_numbers.is_empty() {
                        caret.up(buf, 1);
                    } else {
                        caret.up(buf, self.parsed_numbers[0]);
                    }
                    caret.pos.y = max(0, caret.pos.y);
                    self.ans_code = false;
                    return Ok(None);
                }
                b'B' => { // Cursor Down 
                    if self.parsed_numbers.is_empty() {
                        caret.down(buf, 1);
                    } else {
                        caret.down(buf, self.parsed_numbers[0]);
                    }
                    self.ans_code = false;
                    return Ok(None);
                }
                b's' => { // Save Current Cursor Position
                    self.saved_pos = caret.pos;
                    self.ans_code = false;
                    return Ok(None);
                }
                b'u' => { // Restore Saved Cursor Position 
                    caret.pos = self.saved_pos;
                    self.ans_code = false;
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
                            let s = format!("\x1b[{};{}R", min(buf.height as i32, caret.pos.y + 1), min(buf.width as i32, caret.pos.x + 1));
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
                        buf.layers[0].remove_line(caret.pos.y);
                    } else {
                        if self.parsed_numbers.len() != 1 {
                            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid parameters sequence {}", self.parsed_numbers[0])));
                        }
                        if let Some(number) = self.parsed_numbers.get(0) {
                            for _ in 0..*number {
                                buf.layers[0].remove_line(caret.pos.y);
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
                        buf.layers[0].insert_line(caret.pos.y, Line::create(buf.width));
                    } else {
                        if self.parsed_numbers.len() != 1 {
                            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid parameters sequence {}", self.parsed_numbers[0])));
                        }
                        if let Some(number) = self.parsed_numbers.get(0) {
                            for _ in 0..*number {
                                buf.layers[0].insert_line(caret.pos.y, Line::create(buf.width));
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
                
                b'K' => { // Erase in line
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
                    self.ans_code = false;
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
    fn test_insert_line_default() {
        let (buf, _) = create_buffer(&mut AnsiParser::new(), b"\x1b[L");
        assert_eq!(26, buf.layers[0].lines.len());
    }
    
    #[test]
    fn test_insert_n_line() {
        let (buf, _) = create_buffer(&mut AnsiParser::new(), b"\x1b[10L");
        assert_eq!(35, buf.layers[0].lines.len());
    }

    #[test]
    fn test_remove_line_default() {
        let (buf, _) = create_buffer(&mut AnsiParser::new(), b"test\x1b[M");
        assert_eq!(b' ' , buf.get_char(Position::new()).unwrap().char_code as u8);
    }
    
    #[test]
    fn test_remove_n_line() {
        let (mut buf, _) = create_buffer(&mut AnsiParser::new(), b"test\ntest\ntest\ntest");
        for i in 0..4  {
            assert_eq!(b't' , buf.get_char(Position::from(0, i)).unwrap().char_code as u8);
        }
        run_parser(&mut buf, &mut Caret::new(), &mut AnsiParser::new(), b"\x1b[3M");
        assert_eq!(b't' , buf.get_char(Position::from(0, 0)).unwrap().char_code as u8);
        assert_eq!(b' ' , buf.get_char(Position::from(0, 1)).unwrap().char_code as u8);
    }

    #[test]
    fn test_delete_character_default() {
        let (mut buf, _) = create_buffer(&mut AnsiParser::new(), b"test");
        run_parser(&mut buf, &mut &mut Caret::from_xy(0, 0), &mut AnsiParser::new(), b"\x1b[P");
        assert_eq!(b'e' , buf.get_char(Position::from(0, 0)).unwrap().char_code as u8);
        run_parser(&mut buf, &mut &mut Caret::from_xy(0, 0), &mut AnsiParser::new(), b"\x1b[P");
        assert_eq!(b's' , buf.get_char(Position::from(0, 0)).unwrap().char_code as u8);
        run_parser(&mut buf, &mut &mut Caret::from_xy(0, 0), &mut AnsiParser::new(), b"\x1b[P");
        assert_eq!(b't' , buf.get_char(Position::from(0, 0)).unwrap().char_code as u8);
    }

    #[test]
    fn test_delete_n_character() {
        let (mut buf, _) = create_buffer(&mut AnsiParser::new(), b"testme");
        run_parser(&mut buf, &mut &mut Caret::from_xy(0, 0), &mut AnsiParser::new(), b"\x1b[4P");
        assert_eq!(b'm' , buf.get_char(Position::from(0, 0)).unwrap().char_code as u8);
    }


    #[test]
    fn test_save_cursor() {
        let (_,caret) = create_buffer(&mut AnsiParser::new(), b"\x1b7testme\x1b8");
        assert_eq!(Position::new() , caret.get_position());
    }

    #[test]
    fn test_reset_cursor() {
        let (mut buf, mut caret) = create_buffer(&mut AnsiParser::new(), b"testme\x1b[1;37m");
        assert_ne!(TextAttribute::DEFAULT, caret.attr);
        assert_ne!(Position::new() , caret.get_position());
        run_parser(&mut buf, &mut caret, &mut AnsiParser::new(), b"\x1bc");
        assert_eq!(TextAttribute::DEFAULT, caret.attr);
        assert_eq!(Position::new() , caret.get_position());
    }
}