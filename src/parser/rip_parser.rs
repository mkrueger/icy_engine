use crate::{Buffer, Caret, AnsiParser, EngineResult, AnsiState, ParserError, Rectangle};
use super::BufferParser;

enum RipState {
    Default,
    GotRipStart,
    ReadCommand
}

pub enum RipWriteMode {
    Normal,
    Xor
}
pub struct RipParser {
    ansi_parser: AnsiParser,
    enable_rip: bool,
    state: RipState,

    text_window: Option<Rectangle>,
    viewport: Option<Rectangle>,
    current_write_mode: RipWriteMode
}

impl RipParser {
    pub fn new() -> Self {
        Self { 
            ansi_parser: AnsiParser::new(),
            enable_rip: true,
            state: RipState::Default,
            text_window: None,
            viewport: None,
            current_write_mode: RipWriteMode::Normal
        }
    }

    pub fn clear(&mut self) {
        // clear viewport
    }
}

static RIP_TERMINAL_ID: &str = "RIPSCRIP01540\0";

impl BufferParser for RipParser {
    fn from_unicode(&self, ch: char) -> char
    {
        self.ansi_parser.from_unicode(ch)
    }
    
    fn to_unicode(&self, ch: char) -> char
    {
        self.ansi_parser.to_unicode(ch)
    }

    fn print_char(&mut self, buf: &mut Buffer, caret: &mut Caret, ch: char) -> EngineResult<Option<String>> {

        match self.state {
            RipState::ReadCommand => {
                match ch {
                    'w' => {  // RIP_TEXT_WINDOW
                        todo!();
                    }
                    'v' => {  // RIP_VIEWPORT
                        todo!();
                    }
                    '*' => {  // RIP_RESET_WINDOWS
                        self.state = RipState::Default;
                        self.text_window = None;
                        self.viewport = None;
                        return Ok(None);
                    }
                    'e' => { // RIP_ERASE_VIEW
                        self.state = RipState::Default;
                        self.clear();
                        return Ok(None);
                    }
                    'E' => { // RIP_ERASE_WINDOW
                        // level1: RIP_END_TEXT
                        self.state = RipState::Default;
                        buf.clear();
                        return Ok(None);
                    }
                    'g' => { // RIP_GOTOXY
                        todo!();
                    }
                    'H' => { // RIP_HOME
                        self.state = RipState::Default;
                        caret.home(buf);
                        return Ok(None);
                    }
                    '>' => { // RIP_ERASE_EOL
                        self.state = RipState::Default;
                        buf.clear_line_end(&caret.get_position());
                        return Ok(None);
                    }
                    'c' => { // RIP_COLOR
                        todo!();
                    }
                    'Q' => { // RIP_SET_PALETTE
                        todo!();
                    }
                    'a' => { // RIP_ONE_PALETTE
                        todo!();
                    }
                    'W' => { // RIP_WRITE_MODE
                        // level 1: RIP_WRITE_ICON
                        todo!();
                    }
                    'm' => { // RIP_MOVE
                        todo!();
                    }
                    'T' => { // RIP_TEXT
                        // level1: RIP_REGION_TEXT
                        todo!();
                    }
                    '@' => { // RIP_TEXT_XY
                        todo!();
                    }
                    'Y' => { // RIP_FONT_STYLE
                        todo!();
                    }
                    'X' => { // RIP_PIXEL
                        todo!();
                    }
                    'L' => { // RIP_LINE
                        todo!();
                    }
                    'R' => { // RIP_RECTANGLE
                        // RIP_READ_SCENE level 1
                        todo!();
                    }
                    'B' => { // RIP_BAR
                        // level 1: RIP_BUTTON_STYLE
                        todo!();
                    }
                    'C' => { // RIP_CIRCLE
                        // level 1: RIP_GET_IMAGE
                        todo!();
                    }
                    'O' => { // RIP_OVAL
                        todo!();
                    }
                    'o' => { // RIP_FILLED_OVAL
                        todo!();
                    }
                    'A' => { // RIP_ARC
                        todo!();
                    }
                    'V' => { // RIP_OVAL_ARC
                        todo!();
                    }
                    'I' => { // RIP_PIE_SLICE
                        // level 1: RIP_LOAD_ICON
                        todo!();
                    }
                    'i' => { // RIP_OVAL_PIE_SLICE
                        todo!();
                    }
                    'Z' => { // RIP_BEZIER
                        todo!();
                    }
                    'P' => { // RIP_POLYGON
                        // level 1: RIP_PUT_IMAGE
                        todo!();
                    }
                    'p' => { // RIP_FILL_POLYGON
                        todo!();
                    }
                    'l' => { // RIP_POLYLINE
                        todo!();
                    }
                    'F' => { // RIP_FILL
                        // level 1: RIP_FILE_QUERY
                        todo!();
                    }
                    '=' => { // RIP_LINE_STYLE
                        todo!();
                    }
                    'S' => { // RIP_FILL_STYLE
                        todo!();
                    }
                    's' => { // RIP_FILL_PATTERN
                        todo!();
                    }
                    'M' => { // RIP_MOUSE
                        todo!();
                    }
                    'K' => { // RIP_KILL_MOUSE_FIELDS
                        todo!();
                    }
                    't' => { // RIP_REGION_TEXT
                        todo!();
                    }
                    'U' => { // RIP_BUTTON level 1
                        todo!();
                    }
                    'D' => { // RIP_DEFINE level 1
                        todo!();
                    }
                    '\x1B' => { // RIP_QUERY level 1
                        // level 9: RIP_ENTER_BLOCK_MODE
                        todo!();
                    }
                    'G' => { // RIP_COPY_REGION level 1
                        todo!();
                    }
                    '#' => { // RIP_NO_MORE
                        self.state = RipState::Default;
                        return Ok(None);
                    }
                    _ => {
                        self.state = RipState::Default;
                        self.ansi_parser.print_char(buf, caret, '!')?;
                        self.ansi_parser.print_char(buf, caret, '|')?;
                        return self.ansi_parser.print_char(buf, caret, ch);
                    }
                }
            }
            RipState::GotRipStart => { // got !
                if ch != '|' {
                    self.state = RipState::Default;
                    self.ansi_parser.print_char(buf, caret, '!')?;
                    return self.ansi_parser.print_char(buf, caret, ch);
                }
                self.state = RipState::ReadCommand;
                return Ok(None);
            }
            _ => {
                match self.ansi_parser.state {
                    crate::AnsiState::ReadSequence => {
                        match ch {
                            '!' => { // Select Graphic Rendition 
                                self.ansi_parser.state = AnsiState::Default;
                                if self.ansi_parser.parsed_numbers.is_empty() {
                                    return Ok(Some(RIP_TERMINAL_ID.to_string()));
                                }
        
                                match self.ansi_parser.parsed_numbers[0] {
                                    0 => {
                                        return Ok(Some(RIP_TERMINAL_ID.to_string()));
                                    }
                                    1 => {
                                        self.enable_rip = false;
                                    }
                                    2 => {
                                        self.enable_rip = true;
                                    }
                                    _ => {
                                        return Err(Box::new(ParserError::InvalidRipAnsiQuery(self.ansi_parser.parsed_numbers[0])));
                                    }
                                }
                                return Ok(None);
                            }
                            _ => {}
                        }
                    }
                    crate::AnsiState::Default => {
                        if !self.enable_rip {
                            return self.ansi_parser.print_char(buf, caret, ch);
                        }
        
                        match ch {
                            '!' => {
                                self.state = RipState::GotRipStart;
                                return Ok(None);
                            }
                            _=> {}
                        }
        
                    }
                    _ => {}
                }
            }
        }

        
        self.ansi_parser.print_char(buf, caret, ch)
    }
}
