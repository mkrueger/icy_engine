use super::{ansi, BufferParser};
use crate::{
    ansi::EngineState, Buffer, CallbackAction, Caret, EngineResult, ParserError, Rectangle, AttributedChar,
};

#[derive(Default)]
enum State {
    #[default]
    Default,
    GotRipStart,
    ReadCommand,
}

#[derive(Default)]
pub enum WriteMode {
    #[default]
    Normal,
    Xor,
}

#[derive(Default)]
pub struct Parser {
    ansi_parser: ansi::Parser,
    enable_rip: bool,
    state: State,

    text_window: Option<Rectangle>,
    viewport: Option<Rectangle>,
    _current_write_mode: WriteMode,
}

impl Parser {
    pub fn clear(&mut self) {
        // clear viewport
    }
}

static RIP_TERMINAL_ID: &str = "RIPSCRIP01540\0";

impl BufferParser for Parser {
    fn convert_from_unicode(&self, ch: char, font_page: usize) -> char {
        self.ansi_parser.convert_from_unicode(ch, font_page)
    }

    fn convert_to_unicode(&self, ch: AttributedChar) -> char {
        self.ansi_parser.convert_to_unicode(ch)
    }

    fn print_char(
        &mut self,
        buf: &mut Buffer,
        caret: &mut Caret,
        ch: char,
    ) -> EngineResult<CallbackAction> {
        match self.state {
            State::ReadCommand => {
                match ch {
                    'w' => {
                        // RIP_TEXT_WINDOW
                        todo!();
                    }
                    'v' => {
                        // RIP_VIEWPORT
                        todo!();
                    }
                    '*' => {
                        // RIP_RESET_WINDOWS
                        self.state = State::Default;
                        self.text_window = None;
                        self.viewport = None;
                        return Ok(CallbackAction::None);
                    }
                    'e' => {
                        // RIP_ERASE_VIEW
                        self.state = State::Default;
                        self.clear();
                        return Ok(CallbackAction::None);
                    }
                    'E' => {
                        // RIP_ERASE_WINDOW
                        // level1: RIP_END_TEXT
                        self.state = State::Default;
                        buf.clear();
                        return Ok(CallbackAction::None);
                    }
                    'g' => {
                        // RIP_GOTOXY
                        todo!();
                    }
                    'H' => {
                        // RIP_HOME
                        self.state = State::Default;
                        caret.home(buf);
                        return Ok(CallbackAction::None);
                    }
                    '>' => {
                        // RIP_ERASE_EOL
                        self.state = State::Default;
                        buf.clear_line_end(caret);
                        return Ok(CallbackAction::None);
                    }
                    'c' => {
                        // RIP_COLOR
                        todo!();
                    }
                    'Q' => {
                        // RIP_SET_PALETTE
                        todo!();
                    }
                    'a' => {
                        // RIP_ONE_PALETTE
                        todo!();
                    }
                    'W' => {
                        // RIP_WRITE_MODE
                        // level 1: RIP_WRITE_ICON
                        todo!();
                    }
                    'm' => {
                        // RIP_MOVE
                        todo!();
                    }
                    'T' => {
                        // RIP_TEXT
                        // level1: RIP_REGION_TEXT
                        todo!();
                    }
                    '@' => {
                        // RIP_TEXT_XY
                        todo!();
                    }
                    'Y' => {
                        // RIP_FONT_STYLE
                        todo!();
                    }
                    'X' => {
                        // RIP_PIXEL
                        todo!();
                    }
                    'L' => {
                        // RIP_LINE
                        todo!();
                    }
                    'R' => {
                        // RIP_RECTANGLE
                        // RIP_READ_SCENE level 1
                        todo!();
                    }
                    'B' => {
                        // RIP_BAR
                        // level 1: RIP_BUTTON_STYLE
                        todo!();
                    }
                    'C' => {
                        // RIP_CIRCLE
                        // level 1: RIP_GET_IMAGE
                        todo!();
                    }
                    'O' => {
                        // RIP_OVAL
                        todo!();
                    }
                    'o' => {
                        // RIP_FILLED_OVAL
                        todo!();
                    }
                    'A' => {
                        // RIP_ARC
                        todo!();
                    }
                    'V' => {
                        // RIP_OVAL_ARC
                        todo!();
                    }
                    'I' => {
                        // RIP_PIE_SLICE
                        // level 1: RIP_LOAD_ICON
                        todo!();
                    }
                    'i' => {
                        // RIP_OVAL_PIE_SLICE
                        todo!();
                    }
                    'Z' => {
                        // RIP_BEZIER
                        todo!();
                    }
                    'P' => {
                        // RIP_POLYGON
                        // level 1: RIP_PUT_IMAGE
                        todo!();
                    }
                    'p' => {
                        // RIP_FILL_POLYGON
                        todo!();
                    }
                    'l' => {
                        // RIP_POLYLINE
                        todo!();
                    }
                    'F' => {
                        // RIP_FILL
                        // level 1: RIP_FILE_QUERY
                        todo!();
                    }
                    '=' => {
                        // RIP_LINE_STYLE
                        todo!();
                    }
                    'S' => {
                        // RIP_FILL_STYLE
                        todo!();
                    }
                    's' => {
                        // RIP_FILL_PATTERN
                        todo!();
                    }
                    'M' => {
                        // RIP_MOUSE
                        todo!();
                    }
                    'K' => {
                        // RIP_KILL_MOUSE_FIELDS
                        todo!();
                    }
                    't' => {
                        // RIP_REGION_TEXT
                        todo!();
                    }
                    'U' => {
                        // RIP_BUTTON level 1
                        todo!();
                    }
                    'D' => {
                        // RIP_DEFINE level 1
                        todo!();
                    }
                    '\x1B' => {
                        // RIP_QUERY level 1
                        // level 9: RIP_ENTER_BLOCK_MODE
                        todo!();
                    }
                    'G' => {
                        // RIP_COPY_REGION level 1
                        todo!();
                    }
                    '#' => {
                        // RIP_NO_MORE
                        self.state = State::Default;
                        return Ok(CallbackAction::None);
                    }
                    _ => {
                        self.state = State::Default;
                        self.ansi_parser.print_char(buf, caret, '!')?;
                        self.ansi_parser.print_char(buf, caret, '|')?;
                        return self.ansi_parser.print_char(buf, caret, ch);
                    }
                }
            }
            State::GotRipStart => {
                // got !
                if ch != '|' {
                    self.state = State::Default;
                    self.ansi_parser.print_char(buf, caret, '!')?;
                    return self.ansi_parser.print_char(buf, caret, ch);
                }
                self.state = State::ReadCommand;
                return Ok(CallbackAction::None);
            }
            State::Default => {
                match self.ansi_parser.state {
                    EngineState::ReadCSISequence(_) => {
                        if let '!' = ch {
                            // Select Graphic Rendition
                            self.ansi_parser.state = EngineState::Default;
                            if self.ansi_parser.parsed_numbers.is_empty() {
                                return Ok(CallbackAction::SendString(RIP_TERMINAL_ID.to_string()));
                            }

                            match self.ansi_parser.parsed_numbers.first() {
                                Some(0) => {
                                    return Ok(CallbackAction::SendString(
                                        RIP_TERMINAL_ID.to_string(),
                                    ));
                                }
                                Some(1) => {
                                    self.enable_rip = false;
                                }
                                Some(2) => {
                                    self.enable_rip = true;
                                }
                                _ => {
                                    return Err(Box::new(ParserError::InvalidRipAnsiQuery(
                                        self.ansi_parser.parsed_numbers[0],
                                    )));
                                }
                            }
                            return Ok(CallbackAction::None);
                        }
                    }
                    EngineState::Default => {
                        if !self.enable_rip {
                            return self.ansi_parser.print_char(buf, caret, ch);
                        }

                        if let '!' = ch {
                            self.state = State::GotRipStart;
                            return Ok(CallbackAction::None);
                        }
                    }
                    _ => {}
                }
            }
        }

        self.ansi_parser.print_char(buf, caret, ch)
    }
}
