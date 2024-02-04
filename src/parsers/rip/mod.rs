use self::bgi::Bgi;

use super::{ansi, BufferParser};
use crate::{ansi::EngineState, Buffer, CallbackAction, Caret, EngineResult, ParserError, Rectangle};

pub mod bgi;
mod commands;

#[derive(Default, Debug)]
enum State {
    #[default]
    Default,
    GotRipStart,
    ReadCommand(usize),
    ReadParams,
    SkipEOL,
}

#[derive(Default)]
pub enum WriteMode {
    #[default]
    Normal,
    Xor,
}

pub trait Command {
    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    fn parse(&mut self, _state: &mut i32, _ch: char) -> EngineResult<bool> {
        Err(anyhow::Error::msg("Invalid state"))
    }

    fn to_rip_string(&self) -> String;

    fn run(&self, _bgi: &mut Bgi) -> EngineResult<()> {
        println!("not implemented RIP: {:?}", self.to_rip_string());
        Ok(())
    }
}

pub struct Parser {
    fallback_parser: Box<ansi::Parser>,
    enable_rip: bool,
    state: State,

    parameter_state: i32,
    _text_window: Option<Rectangle>,
    _viewport: Option<Rectangle>,
    _current_write_mode: WriteMode,

    rip_commands: Vec<Box<dyn Command>>,
    command: Option<Box<dyn Command>>,

    pub bgi: Bgi,
}

impl Parser {
    pub fn new(fallback_parser: Box<ansi::Parser>) -> Self {
        Self {
            fallback_parser,
            enable_rip: true,
            state: State::Default,
            parameter_state: 0,
            _text_window: None,
            _viewport: None,
            _current_write_mode: WriteMode::Normal,
            rip_commands: Vec::new(),
            command: None,
            bgi: Bgi::default(),
        }
    }

    pub fn clear(&mut self) {
        // clear viewport
    }

    pub fn run_commands(&mut self) -> EngineResult<()> {
        if let Some(cmd) = self.command.take() {
            self.rip_commands.push(cmd);
        }
        for cmd in &self.rip_commands {
            cmd.run(&mut self.bgi)?;
        }
        Ok(())
    }
}

static RIP_TERMINAL_ID: &str = "RIPSCRIP01540\0";

impl Parser {
    pub fn start_command(&mut self, cmd: Box<dyn Command>) {
        // println!("---- start_command: {:?}", cmd.to_rip_string());
        self.command = Some(cmd);
        self.parameter_state = 0;
        self.state = State::ReadParams;
    }

    pub fn push_command(&mut self, cmd: Box<dyn Command>) {
        self.state = State::GotRipStart;
        self.rip_commands.push(cmd);
    }
}

impl BufferParser for Parser {
    fn print_char(&mut self, buf: &mut Buffer, current_layer: usize, caret: &mut Caret, ch: char) -> EngineResult<CallbackAction> {
        // println!("state: {:?}, ch: {}, ch#:{}", self.state, ch, ch as u32);
        match self.state {
            State::ReadParams => {
                if ch == '\\' {
                    self.state = State::SkipEOL;
                    return Ok(CallbackAction::NoUpdate);
                }
                if ch == '|' {
                    if let Some(t) = self.command.take() {
                        // println!("push: {:?}", t.to_rip_string());
                        self.rip_commands.push(t);
                    }
                    self.state = State::ReadCommand(0);
                    return Ok(CallbackAction::NoUpdate);
                }
                match self.command.as_mut().unwrap().parse(&mut self.parameter_state, ch) {
                    Ok(true) => {
                        self.parameter_state += 1;
                    }
                    Ok(false) => {
                        if let Some(t) = self.command.take() {
                            self.state = State::GotRipStart;
                            self.rip_commands.push(t);
                        }
                    }
                    Err(e) => {
                        log::error!("Error in RipScript: {}", e);
                        self.state = State::Default;
                        return Ok(CallbackAction::NoUpdate);
                    }
                }
            }
            State::SkipEOL => {
                if ch == '\r' {
                    return Ok(CallbackAction::NoUpdate);
                }
                if ch == '\n' {
                    self.state = State::ReadParams;
                    return Ok(CallbackAction::NoUpdate);
                }
                self.state = State::ReadParams;
                return Ok(CallbackAction::NoUpdate);
            }
            State::ReadCommand(level) => {
                if level == 1 {
                    match ch {
                        'M' => self.start_command(Box::<commands::Mouse>::default()),
                        'K' => self.push_command(Box::<commands::MouseFields>::default()),
                        'T' => self.start_command(Box::<commands::BeginText>::default()),
                        't' => self.start_command(Box::<commands::RegionText>::default()),
                        'E' => self.push_command(Box::<commands::EndText>::default()),
                        'C' => self.start_command(Box::<commands::GetImage>::default()),
                        'P' => self.start_command(Box::<commands::PutImage>::default()),
                        'W' => self.start_command(Box::<commands::WriteIcon>::default()),
                        'I' => self.start_command(Box::<commands::LoadIcon>::default()),
                        'B' => self.start_command(Box::<commands::ButtonStyle>::default()),
                        'U' => self.start_command(Box::<commands::Button>::default()),
                        'D' => self.start_command(Box::<commands::Define>::default()),
                        '\x1B' => self.start_command(Box::<commands::Query>::default()),
                        'G' => self.start_command(Box::<commands::CopyRegion>::default()),
                        'R' => self.start_command(Box::<commands::ReadScene>::default()),
                        'F' => self.start_command(Box::<commands::FileQuery>::default()),

                        _ => {
                            log::error!("Error in RipScript: Unknown level 1 command: {}", ch);
                            self.state = State::Default;
                            return Ok(CallbackAction::NoUpdate);
                        }
                    }
                    return Ok(CallbackAction::NoUpdate);
                }
                if level == 9 {
                    match ch {
                        '\x1B' => self.start_command(Box::<commands::EnterBlockMode>::default()),

                        _ => {
                            log::error!("Error in RipScript: Unknown level 1 command: {}", ch);
                            self.state = State::Default;
                            return Ok(CallbackAction::NoUpdate);
                        }
                    }
                    return Ok(CallbackAction::NoUpdate);
                }

                match ch {
                    'w' => self.start_command(Box::<commands::TextWindow>::default()),
                    'v' => self.start_command(Box::<commands::ViewPort>::default()),
                    '*' => self.push_command(Box::<commands::ResetWindows>::default()),
                    'e' => self.push_command(Box::<commands::EraseWindow>::default()),
                    'E' => self.push_command(Box::<commands::EraseView>::default()),
                    'g' => self.start_command(Box::<commands::GotoXY>::default()),
                    'H' => self.push_command(Box::<commands::Home>::default()),
                    '>' => self.push_command(Box::<commands::EraseEOL>::default()),
                    'c' => self.start_command(Box::<commands::Color>::default()),
                    'Q' => self.start_command(Box::<commands::SetPalette>::default()),
                    'a' => self.start_command(Box::<commands::OnePalette>::default()),
                    'W' => self.start_command(Box::<commands::WriteMode>::default()),
                    'm' => self.start_command(Box::<commands::Move>::default()),
                    'T' => self.start_command(Box::<commands::Text>::default()),
                    '@' => self.start_command(Box::<commands::TextXY>::default()),
                    'Y' => self.start_command(Box::<commands::FontStyle>::default()),
                    'X' => self.start_command(Box::<commands::Pixel>::default()),
                    'L' => self.start_command(Box::<commands::Line>::default()),
                    'R' => self.start_command(Box::<commands::Rectangle>::default()),
                    'B' => self.start_command(Box::<commands::Bar>::default()),
                    'C' => self.start_command(Box::<commands::Circle>::default()),
                    'O' => self.start_command(Box::<commands::Oval>::default()),
                    'o' => self.start_command(Box::<commands::FilledOval>::default()),
                    'A' => self.start_command(Box::<commands::Arc>::default()),
                    'V' => self.start_command(Box::<commands::OvalArc>::default()),
                    'I' => self.start_command(Box::<commands::PieSlice>::default()),
                    'i' => self.start_command(Box::<commands::OvalPieSlice>::default()),
                    'Z' => self.start_command(Box::<commands::Bezier>::default()),
                    'P' => self.start_command(Box::<commands::Polygon>::default()),
                    'p' => self.start_command(Box::<commands::FilledPolygon>::default()),
                    'l' => self.start_command(Box::<commands::PolyLine>::default()),
                    'F' => self.start_command(Box::<commands::Fill>::default()),
                    '=' => self.start_command(Box::<commands::LineStyle>::default()),
                    'S' => self.start_command(Box::<commands::FillStyle>::default()),
                    's' => self.start_command(Box::<commands::FillPattern>::default()),
                    '1' => {
                        self.state = State::ReadCommand(1);
                        return Ok(CallbackAction::NoUpdate);
                    }
                    '9' => {
                        self.state = State::ReadCommand(9);
                        return Ok(CallbackAction::NoUpdate);
                    }
                    '#' => {
                        // RIP_NO_MORE
                        self.state = State::Default;
                        return Ok(CallbackAction::NoUpdate);
                    }
                    _ => {
                        self.state = State::Default;
                        self.fallback_parser.print_char(buf, current_layer, caret, '!')?;
                        self.fallback_parser.print_char(buf, current_layer, caret, '|')?;
                        return self.fallback_parser.print_char(buf, current_layer, caret, ch);
                    }
                }
                return Ok(CallbackAction::NoUpdate);
            }
            State::GotRipStart => {
                // got !
                if ch != '|' {
                    self.state = State::Default;
                    self.fallback_parser.print_char(buf, current_layer, caret, '!')?;
                    return self.fallback_parser.print_char(buf, current_layer, caret, ch);
                }
                self.state = State::ReadCommand(0);
                return Ok(CallbackAction::NoUpdate);
            }
            State::Default => {
                match self.fallback_parser.state {
                    EngineState::ReadCSISequence(_) => {
                        if let '!' = ch {
                            // Select Graphic Rendition
                            self.fallback_parser.state = EngineState::Default;
                            if self.fallback_parser.parsed_numbers.is_empty() {
                                return Ok(CallbackAction::SendString(RIP_TERMINAL_ID.to_string()));
                            }

                            match self.fallback_parser.parsed_numbers.first() {
                                Some(0) => {
                                    return Ok(CallbackAction::SendString(RIP_TERMINAL_ID.to_string()));
                                }
                                Some(1) => {
                                    self.enable_rip = false;
                                }
                                Some(2) => {
                                    self.enable_rip = true;
                                }
                                _ => {
                                    return Err(ParserError::InvalidRipAnsiQuery(self.fallback_parser.parsed_numbers[0]).into());
                                }
                            }
                            return Ok(CallbackAction::NoUpdate);
                        }
                    }
                    EngineState::Default => {
                        if !self.enable_rip {
                            return self.fallback_parser.print_char(buf, current_layer, caret, ch);
                        }

                        if let '!' = ch {
                            self.state = State::GotRipStart;
                            return Ok(CallbackAction::NoUpdate);
                        }
                    }
                    _ => {}
                }
            }
        }

        self.fallback_parser.print_char(buf, current_layer, caret, ch)
    }
}

fn to_base_36(len: usize, number: i32) -> String {
    let mut res = String::new();
    let mut number = number;
    for _ in 0..len {
        let num2 = (number % 36) as u8;
        let ch2 = if num2 < 10 { (num2 + b'0') as char } else { (num2 - 10 + b'A') as char };

        res = ch2.to_string() + res.as_str();
        number /= 36;
    }

    res
}

fn parse_base_36(number: &mut i32, ch: char) -> EngineResult<()> {
    if let Some(digit) = ch.to_digit(36) {
        *number = *number * 36 + digit as i32;
        Ok(())
    } else {
        Err(anyhow::Error::msg("Invalid base 36 digit"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::create_buffer;

    #[test]
    fn test_rip_text_window() {
        test_roundtrip("|w00001B0M10");
    }

    #[test]
    fn test_rip_viewport() {
        test_roundtrip("|v00002E1M");
    }

    #[test]
    fn test_reset_windows() {
        test_roundtrip("|*");
    }

    #[test]
    fn test_erase_window() {
        test_roundtrip("|e");
    }

    #[test]
    fn test_erase_view() {
        test_roundtrip("|E");
    }

    #[test]
    fn test_gotoxy() {
        test_roundtrip("|g0509");
    }

    #[test]
    fn test_home() {
        test_roundtrip("|H");
    }

    #[test]
    fn test_erase_eol() {
        test_roundtrip("|>");
    }

    #[test]
    fn test_color() {
        test_roundtrip("|c0A");
    }

    #[test]
    fn test_set_palette() {
        test_roundtrip("|Q000102030405060708090A0B0C0D0E0F");
    }

    #[test]
    fn test_one_palette() {
        test_roundtrip("|a051B");
    }

    #[test]
    fn test_write_mode() {
        test_roundtrip("|W00");
    }

    #[test]
    fn test_move() {
        test_roundtrip("|m0509");
    }

    #[test]
    fn test_text() {
        test_roundtrip("|Thello world");
    }

    #[test]
    fn test_text_xy() {
        test_roundtrip("|@0011hello world");
    }

    #[test]
    fn test_font_style() {
        test_roundtrip("|Y01000400");
    }

    #[test]
    fn test_pixel() {
        test_roundtrip("|X1122");
    }

    #[test]
    fn test_line() {
        test_roundtrip("|L00010A0E");
    }

    #[test]
    fn test_rectangle() {
        test_roundtrip("|R00010A0E");
    }

    #[test]
    fn test_bar() {
        test_roundtrip("|B00010A0E");
    }

    #[test]
    fn test_circle() {
        test_roundtrip("|C1E180M");
    }

    #[test]
    fn test_oval() {
        test_roundtrip("|O1E1A18003G15");
    }

    #[test]
    fn test_filled_oval() {
        test_roundtrip("|o1G2B0M0G");
    }

    #[test]
    fn test_arc() {
        test_roundtrip("|A1E18003G15");
    }

    #[test]
    fn test_oval_arc() {
        test_roundtrip("|V1E18003G151Q");
    }

    #[test]
    fn test_pie_slice() {
        test_roundtrip("|I1E18003G15");
    }

    #[test]
    fn test_oval_pie_slice() {
        test_roundtrip("|i1E18003G151Q");
    }

    #[test]
    fn test_bezier() {
        test_roundtrip("|Z0A0B0C0D0E0F0G0H1G");
    }

    #[test]
    fn test_polygon() {
        test_roundtrip("|P03010105090905");
    }

    #[test]
    fn test_fill_polygon() {
        test_roundtrip("|p03010105050909");
    }

    #[test]
    fn test_polyline() {
        test_roundtrip("|l03010105050909");
    }

    #[test]
    fn test_fill() {
        test_roundtrip("|F25090F");
    }

    #[test]
    fn test_line_style() {
        test_roundtrip("|=01000001");
    }

    #[test]
    fn test_fill_style() {
        test_roundtrip("|S050F");
    }

    #[test]
    fn test_fill_pattern() {
        test_roundtrip("|s11223344556677880F");
    }

    #[test]
    fn test_mouse() {
        test_roundtrip("|1M00001122331100000host command^M");
    }

    #[test]
    fn test_kill_mouse_fields() {
        test_roundtrip("|1K");
    }

    #[test]
    fn test_begin_text() {
        test_roundtrip("|1T0011001100");
    }

    #[test]
    fn test_region_text() {
        test_roundtrip("|1t1This is a text line to be justified");
    }

    #[test]
    fn test_end_text() {
        test_roundtrip("|1K");
    }

    #[test]
    fn test_get_image() {
        test_roundtrip("|1C001122330");
    }

    #[test]
    fn test_put_image() {
        test_roundtrip("|1P0011010");
    }

    #[test]
    fn test_write_icon() {
        test_roundtrip("|1W0filename.icn");
    }

    #[test]
    fn test_load_icon() {
        test_roundtrip("|1I001101010button.icn");
    }

    #[test]
    fn test_button_style() {
        test_roundtrip("|1B0A0A010274030F080F080700010E07000000");
    }

    #[test]
    fn test_button() {
        test_roundtrip("|1U010100003200iconfile<>Label<>HostCmd^m");
    }

    #[test]
    fn test_define() {
        test_roundtrip("|1D00700text_var,60:?question?default data");
    }

    #[test]
    fn test_query() {
        test_roundtrip("|1\x1B0000this is a query $COMMAND$^m");
    }

    #[test]
    fn test_copy_region() {
        test_roundtrip("|1G080G140M0005");
    }

    #[test]
    fn test_read_scene() {
        test_roundtrip("|1R00000000testfile.rip");
    }

    #[test]
    fn test_enter_block_mode() {
        test_roundtrip("|9\x1B00010000ICONFILE.ICN<>");
    }

    fn test_roundtrip(arg: &str) {
        let mut parser = Parser::new(Box::default());
        create_buffer(&mut parser, ("!".to_string() + arg + "|").as_bytes());

        assert!(parser.command.is_none());
        assert_eq!(parser.rip_commands.len(), 1);
        assert_eq!(parser.rip_commands[0].to_rip_string(), arg);
    }
}
