// Useful description: https://vt100.net/docs/vt510-rm/chapter4.html
//                     https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Normal-tracking-mode
use std::{
    cmp::{max, min},
    collections::HashMap,
    fmt::Display,
};

use self::sound::{AnsiMusic, MusicState};

use super::{ascii, BufferParser};
use crate::{
    update_crc16, AttributedChar, AutoWrapMode, Buffer, CallbackAction, Caret, EngineResult,
    FontSelectionState, HyperLink, MouseMode, OriginMode, ParserError, Position, TerminalScrolling,
    BEL, CR, FF, LF,
};

mod ansi_commands;
pub mod constants;
mod dcs;
mod osc;
pub mod sound;

#[cfg(test)]
mod sixel_tests;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub enum EngineState {
    Default,
    ReadEscapeSequence,

    ReadCSISequence(bool),
    ReadCSICommand,        // CSI ?
    ReadCSIRequest,        // CSI =
    ReadRIPSupportRequest, // CSI !
    EndCSI(char),

    RecordDCS(ReadSTState),
    ReadPossibleMacroInDCS(u8),

    ParseAnsiMusic(sound::MusicState),

    ReadAPS(ReadSTState),

    ReadOSCSequence(ReadSTState),
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MusicOption {
    #[default]
    Off,
    Conflicting,
    Banana,
    Both,
}

#[derive(Debug, Clone, Copy)]
pub enum ReadSTState {
    Default(usize),
    GotEscape(usize),
}

impl Display for MusicOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl From<String> for MusicOption {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Conflicting" => MusicOption::Conflicting,
            "Banana" => MusicOption::Banana,
            "Both" => MusicOption::Both,
            _ => MusicOption::Off,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum BaudEmulation {
    #[default]
    Off,
    Rate(u32),
}

impl Display for BaudEmulation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Off => write!(f, "Off"),
            Self::Rate(v) => write!(f, "{v}"),
        }
    }
}

impl BaudEmulation {
    pub const OPTIONS: [BaudEmulation; 12] = [
        BaudEmulation::Off,
        BaudEmulation::Rate(300),
        BaudEmulation::Rate(600),
        BaudEmulation::Rate(1200),
        BaudEmulation::Rate(2400),
        BaudEmulation::Rate(4800),
        BaudEmulation::Rate(9600),
        BaudEmulation::Rate(19200),
        BaudEmulation::Rate(38400),
        BaudEmulation::Rate(57600),
        BaudEmulation::Rate(76800),
        BaudEmulation::Rate(115_200),
    ];

    pub fn get_baud_rate(&self) -> u32 {
        match self {
            BaudEmulation::Off => 0,
            BaudEmulation::Rate(baud) => *baud,
        }
    }
}

pub struct Parser {
    ascii_parser: ascii::Parser,
    pub(crate) state: EngineState,
    saved_pos: Position,
    saved_cursor_opt: Option<Caret>,
    pub(crate) parsed_numbers: Vec<i32>,

    pub hyper_links: Vec<HyperLink>,

    current_escape_sequence: String,

    /*     current_sixel_color: i32,
        sixel_cursor: Position,
        current_sixel_palette: Palette,
    */
    pub ansi_music: MusicOption,
    cur_music: Option<AnsiMusic>,
    cur_octave: usize,
    cur_length: i32,
    cur_tempo: i32,
    dotted_note: bool,

    last_char: char,
    pub(crate) macros: HashMap<usize, String>,
    pub parse_string: String,
    pub bs_is_ctrl_char: bool,
}

impl Default for Parser {
    fn default() -> Self {
        Parser {
            ascii_parser: ascii::Parser::default(),
            state: EngineState::Default,
            saved_pos: Position::default(),
            parsed_numbers: Vec::new(),
            current_escape_sequence: String::new(),
            saved_cursor_opt: None,
            ansi_music: MusicOption::Off,
            cur_music: None,
            cur_octave: 3,
            cur_length: 4,
            cur_tempo: 120,
            dotted_note: false,
            parse_string: String::new(),
            macros: HashMap::new(),
            last_char: '\0',
            hyper_links: Vec::new(),
            bs_is_ctrl_char: false,
        }
    }
}

impl BufferParser for Parser {
    fn convert_from_unicode(&self, ch: char, font_page: usize) -> char {
        self.ascii_parser.convert_from_unicode(ch, font_page)
    }

    fn convert_to_unicode(&self, ch: AttributedChar) -> char {
        self.ascii_parser.convert_to_unicode(ch)
    }

    #[allow(clippy::single_match)]
    fn print_char(
        &mut self,
        buf: &mut Buffer,
        current_layer: usize,
        caret: &mut Caret,
        ch: char,
    ) -> EngineResult<CallbackAction> {
        match &self.state {
            EngineState::ParseAnsiMusic(_) => {
                return self.parse_ansi_music(ch);
            }
            EngineState::ReadEscapeSequence => {
                return {
                    self.state = EngineState::Default;
                    self.current_escape_sequence.push(ch);

                    match ch {
                        '[' => {
                            self.state = EngineState::ReadCSISequence(true);
                            self.parsed_numbers.clear();
                            Ok(CallbackAction::None)
                        }
                        ']' => {
                            self.state = EngineState::ReadOSCSequence(ReadSTState::Default(0));
                            self.parsed_numbers.clear();
                            self.parse_string.clear();
                            Ok(CallbackAction::None)
                        }
                        '7' => {
                            self.saved_cursor_opt = Some(caret.clone());
                            Ok(CallbackAction::None)
                        }
                        '8' => {
                            if let Some(saved_caret) = &self.saved_cursor_opt {
                                *caret = saved_caret.clone();
                            }
                            Ok(CallbackAction::None)
                        }

                        'c' => {
                            // RIS—Reset to Initial State see https://vt100.net/docs/vt510-rm/RIS.html
                            caret.ff(buf, current_layer);
                            buf.reset_terminal();
                            self.macros.clear();
                            Ok(CallbackAction::None)
                        }

                        'D' => {
                            // Index
                            caret.index(buf, current_layer);
                            Ok(CallbackAction::None)
                        }
                        'M' => {
                            // Reverse Index
                            caret.reverse_index(buf, current_layer);
                            Ok(CallbackAction::None)
                        }

                        'E' => {
                            // Next Line
                            caret.next_line(buf, current_layer);
                            Ok(CallbackAction::None)
                        }

                        'P' => {
                            // DCS
                            self.state = EngineState::RecordDCS(ReadSTState::Default(0));
                            self.parse_string.clear();
                            self.parsed_numbers.clear();
                            Ok(CallbackAction::None)
                        }
                        'H' => {
                            // set tab at current column
                            self.state = EngineState::Default;
                            buf.terminal_state.set_tab_at(caret.get_position().x);
                            Ok(CallbackAction::None)
                        }

                        '_' => {
                            // Application Program String
                            self.state = EngineState::ReadAPS(ReadSTState::Default(0));
                            self.parse_string.clear();
                            Ok(CallbackAction::None)
                        }

                        '0'..='~' => {
                            // Silently drop unsupported sequences
                            self.state = EngineState::Default;
                            Ok(CallbackAction::None)
                        }
                        _ => Err(Box::new(ParserError::UnsupportedEscapeSequence(
                            self.current_escape_sequence.clone(),
                        ))),
                    }
                };
            }

            EngineState::ReadAPS(st_state) => match st_state {
                ReadSTState::Default(nesting_level) => {
                    if ch == '\x1B' {
                        self.state = EngineState::ReadAPS(ReadSTState::GotEscape(*nesting_level));
                        return Ok(CallbackAction::None);
                    }
                    self.parse_string.push(ch);
                }
                ReadSTState::GotEscape(nesting_level) => {
                    if ch == '\\' {
                        self.state = EngineState::Default;
                        self.execute_aps_command(buf, caret);
                        return Ok(CallbackAction::None);
                    }
                    self.state = EngineState::ReadAPS(ReadSTState::Default(*nesting_level));
                    self.parse_string.push('\x1B');
                    self.parse_string.push(ch);
                }
            },
            EngineState::ReadPossibleMacroInDCS(i) => {
                // \x1B[<num>*z
                // read macro inside dcs sequence, 3 states:´
                // 0: [
                // 1: <num>
                // 2: *
                // z

                if ch.is_ascii_digit() {
                    if *i != 1 {
                        self.state = EngineState::Default;
                        return Err(Box::new(ParserError::UnsupportedDCSSequence(format!(
                            "Error in macro inside dcs, expected number got '{ch}'"
                        ))));
                    }
                    let d = match self.parsed_numbers.pop() {
                        Some(number) => number,
                        _ => 0,
                    };
                    self.parsed_numbers.push(parse_next_number(d, ch as u8));
                    return Ok(CallbackAction::None);
                }
                if ch == '[' {
                    if *i != 0 {
                        self.state = EngineState::Default;
                        return Err(Box::new(ParserError::UnsupportedDCSSequence(format!(
                            "Error in macro inside dcs, expected '[' got '{ch}'"
                        ))));
                    }
                    self.state = EngineState::ReadPossibleMacroInDCS(1);
                    return Ok(CallbackAction::None);
                }
                if ch == '*' {
                    if *i != 1 {
                        self.state = EngineState::Default;
                        return Err(Box::new(ParserError::UnsupportedDCSSequence(format!(
                            "Error in macro inside dcs, expected '*' got '{ch}'"
                        ))));
                    }
                    self.state = EngineState::ReadPossibleMacroInDCS(2);
                    return Ok(CallbackAction::None);
                }
                if ch == 'z' {
                    if *i != 2 {
                        self.state = EngineState::Default;
                        return Err(Box::new(ParserError::UnsupportedDCSSequence(format!(
                            "Error in macro inside dcs, expected 'z' got '{ch}'"
                        ))));
                    }
                    if self.parsed_numbers.len() != 1 {
                        self.state = EngineState::Default;
                        return Err(Box::new(ParserError::UnsupportedDCSSequence(format!(
                            "Macro hasn't one number defined got '{}'",
                            self.parsed_numbers.len()
                        ))));
                    }
                    self.state = EngineState::RecordDCS(ReadSTState::Default(0));
                    return self.invoke_macro_by_id(
                        buf,
                        current_layer,
                        caret,
                        *self.parsed_numbers.first().unwrap(),
                    );
                }

                self.state = EngineState::Default;
                return Err(Box::new(ParserError::UnsupportedDCSSequence(format!(
                    "Invalid macro inside dcs '{ch}'"
                ))));
            }
            EngineState::RecordDCS(dcs_state) => match dcs_state {
                ReadSTState::GotEscape(_nesting_level) => {
                    self.state = EngineState::Default;
                    if ch == '\\' {
                        return self.execute_dcs(buf, caret);
                    }
                    if ch == '[' {
                        //
                        self.state = EngineState::ReadPossibleMacroInDCS(1);
                        return Ok(CallbackAction::None);
                    }
                    return Err(Box::new(ParserError::UnsupportedDCSSequence(format!(
                        "sequence: {} end char <ESC>{ch}",
                        self.parse_string
                    ))));
                }
                ReadSTState::Default(nesting_level) => match ch {
                    '\x1B' => {
                        self.state = EngineState::RecordDCS(ReadSTState::GotEscape(*nesting_level));
                    }
                    _ => {
                        self.parse_string.push(ch);
                    }
                },
            },

            EngineState::ReadOSCSequence(dcs_state) => match dcs_state {
                ReadSTState::Default(nesting_level) => {
                    if ch == '\x1B' {
                        self.state =
                            EngineState::ReadOSCSequence(ReadSTState::GotEscape(*nesting_level));
                        return Ok(CallbackAction::None);
                    }
                    self.parse_string.push(ch);
                }
                ReadSTState::GotEscape(nesting_level) => {
                    if ch == '\\' {
                        self.state = EngineState::Default;
                        return self.parse_osc(buf, caret);
                    }
                    self.state = EngineState::ReadOSCSequence(ReadSTState::Default(*nesting_level));
                    self.parse_string.push('\x1B');
                    self.parse_string.push(ch);
                }
            },

            EngineState::ReadCSICommand => {
                self.current_escape_sequence.push(ch);
                match ch {
                    'l' => {
                        self.state = EngineState::Default;
                        if self.parsed_numbers.len() != 1 {
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                self.current_escape_sequence.clone(),
                            )));
                        }
                        match self.parsed_numbers.first() {
                            Some(4) => buf.terminal_state.scroll_state = TerminalScrolling::Fast,
                            Some(6) => {
                                //  buf.terminal_state.origin_mode = OriginMode::WithinMargins;
                            }
                            Some(7) => buf.terminal_state.auto_wrap_mode = AutoWrapMode::NoWrap,
                            Some(25) => caret.is_visible = false,
                            Some(33) => caret.set_ice_mode(false),
                            Some(35) => caret.is_blinking = true,

                            Some(69) => {
                                buf.terminal_state.dec_margin_mode_left_right = false;
                                buf.terminal_state.clear_margins_left_right();
                            }

                            Some(9 | 1000..=1007 | 1015 | 1016) => {
                                buf.terminal_state.mouse_mode = MouseMode::Default;
                            }
                            _ => {
                                return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                    self.current_escape_sequence.clone(),
                                )));
                            }
                        }
                    }
                    'h' => {
                        self.state = EngineState::Default;
                        if self.parsed_numbers.len() != 1 {
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                self.current_escape_sequence.clone(),
                            )));
                        }
                        match self.parsed_numbers.first() {
                            Some(4) => buf.terminal_state.scroll_state = TerminalScrolling::Smooth,
                            Some(6) => buf.terminal_state.origin_mode = OriginMode::UpperLeftCorner,
                            Some(7) => buf.terminal_state.auto_wrap_mode = AutoWrapMode::AutoWrap,
                            Some(25) => caret.is_visible = true,
                            Some(33) => caret.set_ice_mode(true),
                            Some(35) => caret.is_blinking = false,

                            Some(69) => buf.terminal_state.dec_margin_mode_left_right = true,

                            // Mouse tracking see https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Normal-tracking-mode
                            Some(9) => buf.terminal_state.mouse_mode = MouseMode::X10,
                            Some(1000) => buf.terminal_state.mouse_mode = MouseMode::VT200,
                            Some(1001) => {
                                buf.terminal_state.mouse_mode = MouseMode::VT200_Highlight;
                            }
                            Some(1002) => buf.terminal_state.mouse_mode = MouseMode::ButtonEvents,
                            Some(1003) => buf.terminal_state.mouse_mode = MouseMode::AnyEvents,

                            Some(1004) => buf.terminal_state.mouse_mode = MouseMode::FocusEvent,
                            Some(1007) => {
                                buf.terminal_state.mouse_mode = MouseMode::AlternateScroll;
                            }
                            Some(1005) => buf.terminal_state.mouse_mode = MouseMode::ExtendedMode,
                            Some(1006) => {
                                buf.terminal_state.mouse_mode = MouseMode::SGRExtendedMode;
                            }
                            Some(1015) => {
                                buf.terminal_state.mouse_mode = MouseMode::URXVTExtendedMode;
                            }
                            Some(1016) => buf.terminal_state.mouse_mode = MouseMode::PixelPosition,

                            Some(cmd) => {
                                return Err(Box::new(ParserError::UnsupportedCustomCommand(*cmd)));
                            }
                            None => {
                                return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                    self.current_escape_sequence.clone(),
                                )))
                            }
                        }
                    }
                    '0'..='9' => {
                        let d = match self.parsed_numbers.pop() {
                            Some(number) => number,
                            _ => 0,
                        };
                        self.parsed_numbers.push(parse_next_number(d, ch as u8));
                    }
                    ';' => {
                        self.parsed_numbers.push(0);
                    }
                    'n' => {
                        self.state = EngineState::Default;
                        match self.parsed_numbers.first() {
                            Some(62) => {
                                // DSR—Macro Space Report
                                return Ok(CallbackAction::SendString("\x1B[32767*{".to_string()));
                            }
                            Some(63) => {
                                // Memory Checksum Report (DECCKSR)
                                if self.parsed_numbers.len() != 2 {
                                    return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                        "Memory Checksum Report (DECCKSR) requires 2 parameters."
                                            .to_string(),
                                    )));
                                }
                                let mut crc16 = 0;
                                for i in 0..64 {
                                    if let Some(m) = self.macros.get(&i) {
                                        for b in m.as_bytes() {
                                            crc16 = update_crc16(crc16, *b);
                                        }
                                        crc16 = update_crc16(crc16, 0);
                                    } else {
                                        crc16 = update_crc16(crc16, 0);
                                    }
                                }
                                return Ok(CallbackAction::SendString(format!(
                                    "\x1BP{}!~{crc16:04X}\x1B\\",
                                    self.parsed_numbers[1]
                                )));
                            }
                            _ => {
                                return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                    self.current_escape_sequence.clone(),
                                )));
                            }
                        }
                    }
                    _ => {
                        self.state = EngineState::Default;
                        // error in control sequence, terminate reading
                        return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                            self.current_escape_sequence.clone(),
                        )));
                    }
                }
            }

            EngineState::ReadCSIRequest => {
                self.current_escape_sequence.push(ch);
                match ch {
                    'n' => {
                        self.state = EngineState::Default;
                        if self.parsed_numbers.len() != 1 {
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                self.current_escape_sequence.clone(),
                            )));
                        }
                        match self.parsed_numbers.first() {
                            Some(1) => {
                                // font state report
                                let font_selection_result =
                                    match buf.terminal_state.font_selection_state {
                                        FontSelectionState::NoRequest => 99,
                                        FontSelectionState::Success => 0,
                                        FontSelectionState::Failure => 1,
                                    };

                                return Ok(CallbackAction::SendString(format!(
                                    "\x1B[=1;{font_selection_result};{};{};{};{}n",
                                    buf.terminal_state.normal_attribute_font_slot,
                                    buf.terminal_state.high_intensity_attribute_font_slot,
                                    buf.terminal_state.blink_attribute_font_slot,
                                    buf.terminal_state.high_intensity_blink_attribute_font_slot
                                )));
                            }
                            Some(2) => {
                                // font mode report
                                let mut mode_report = "\x1B[=2".to_string();
                                if buf.terminal_state.origin_mode == OriginMode::WithinMargins {
                                    mode_report.push_str(";6");
                                }
                                if buf.terminal_state.auto_wrap_mode == AutoWrapMode::AutoWrap {
                                    mode_report.push_str(";7");
                                }
                                if caret.is_visible {
                                    mode_report.push_str(";25");
                                }

                                if caret.ice_mode() {
                                    mode_report.push_str(";33");
                                }

                                if caret.is_blinking {
                                    mode_report.push_str(";35");
                                }
                                match buf.terminal_state.mouse_mode {
                                    MouseMode::Default => {}
                                    MouseMode::X10 => mode_report.push_str(";9"),
                                    MouseMode::VT200 => mode_report.push_str(";1000"),
                                    MouseMode::VT200_Highlight => mode_report.push_str(";1001"),
                                    MouseMode::ButtonEvents => mode_report.push_str(";1002"),
                                    MouseMode::AnyEvents => mode_report.push_str(";1003"),
                                    MouseMode::FocusEvent => mode_report.push_str(";1004"),
                                    MouseMode::AlternateScroll => mode_report.push_str(";1007"),
                                    MouseMode::ExtendedMode => mode_report.push_str(";1005"),
                                    MouseMode::SGRExtendedMode => mode_report.push_str(";1006"),
                                    MouseMode::URXVTExtendedMode => mode_report.push_str(";1015"),
                                    MouseMode::PixelPosition => mode_report.push_str(";1016"),
                                }

                                if mode_report.len() == "\x1B[=2".len() {
                                    mode_report.push(';');
                                }
                                mode_report.push('n');

                                return Ok(CallbackAction::SendString(mode_report));
                            }
                            Some(3) => {
                                // font dimension request
                                let dim = buf.get_font_dimensions();
                                return Ok(CallbackAction::SendString(format!(
                                    "\x1B[=3;{};{}n",
                                    dim.height, dim.width
                                )));
                            }
                            _ => {
                                return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                    self.current_escape_sequence.clone(),
                                )));
                            }
                        }
                    }
                    '0'..='9' => {
                        let d = match self.parsed_numbers.pop() {
                            Some(number) => number,
                            _ => 0,
                        };
                        self.parsed_numbers.push(parse_next_number(d, ch as u8));
                    }
                    ';' => {
                        self.parsed_numbers.push(0);
                    }
                    'r' => return self.reset_margins(buf),
                    'm' => {
                        if self.parsed_numbers.len() != 2 {
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                self.current_escape_sequence.clone(),
                            )));
                        }
                        return self.set_specific_margin(buf);
                    }
                    _ => {
                        self.state = EngineState::Default;
                        // error in control sequence, terminate reading
                        return Err(Box::new(ParserError::UnsupportedEscapeSequence(format!(
                            "Error in CSI request: {}",
                            self.current_escape_sequence
                        ))));
                    }
                }
            }

            EngineState::ReadRIPSupportRequest => {
                self.current_escape_sequence.push(ch);
                if let 'p' = ch {
                    self.soft_terminal_reset(buf, caret);
                } else {
                    // potential rip support request
                    // ignore that for now and continue parsing
                    self.state = EngineState::Default;
                    return self.print_char(buf, current_layer, caret, ch);
                }
            }

            EngineState::EndCSI(func) => {
                self.current_escape_sequence.push(ch);
                match *func {
                    '*' => match ch {
                        'z' => return self.invoke_macro(buf, current_layer, caret),
                        'r' => return self.select_communication_speed(buf),
                        'y' => return self.request_checksum_of_rectangular_area(buf),
                        _ => {}
                    },

                    '$' => match ch {
                        'w' => {
                            self.state = EngineState::Default;
                            if let Some(2) = self.parsed_numbers.first() {
                                let mut str = "\x1BP2$u".to_string();
                                (0..buf.terminal_state.tab_count()).for_each(|i| {
                                    let tab = buf.terminal_state.get_tabs()[i];
                                    str.push_str(&(tab + 1).to_string());
                                    if i < buf.terminal_state.tab_count().saturating_sub(1) {
                                        str.push('/');
                                    }
                                });
                                str.push_str("\x1B\\");
                                return Ok(CallbackAction::SendString(str));
                            }
                        }
                        'x' => return self.fill_rectangular_area(buf, caret),
                        'z' => return self.erase_rectangular_area(buf),
                        '{' => return self.selective_erase_rectangular_area(buf),

                        _ => {}
                    },

                    ' ' => {
                        self.state = EngineState::Default;

                        match ch {
                            'D' => return self.font_selection(buf, caret),
                            'A' => self.scroll_right(buf, current_layer),
                            '@' => self.scroll_left(buf, current_layer),
                            'd' => return self.tabulation_stop_remove(buf),
                            _ => {
                                self.current_escape_sequence.push(ch);
                                return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                    self.current_escape_sequence.clone(),
                                )));
                            }
                        }
                    }
                    _ => {
                        self.state = EngineState::Default;
                        return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                            self.current_escape_sequence.clone(),
                        )));
                    }
                }
            }
            EngineState::ReadCSISequence(is_start) => {
                if let Some(ch) = char::from_u32(ch as u32) {
                    self.current_escape_sequence.push(ch);
                } else {
                    return Err(Box::new(ParserError::InvalidChar('\0')));
                }
                match ch {
                    'm' => return self.select_graphic_rendition(caret, buf),
                    'H' |    // Cursor Position
                    'f' // CSI Pn1 ; Pn2 f 
                        // HVP - Character and line position
                    => {
                        self.state = EngineState::Default;
                        if self.parsed_numbers.is_empty() {
                            caret.pos = buf.upper_left_position();
                        } else {
                            if self.parsed_numbers[0] >= 0 {
                                // always be in terminal mode for gotoxy
                                caret.pos.y = buf.get_first_visible_line() as i32
                                    + max(0, self.parsed_numbers[0] - 1);
                            }
                            if self.parsed_numbers.len() > 1 {
                                if self.parsed_numbers[1] >= 0 {
                                    caret.pos.x = max(0, self.parsed_numbers[1] - 1);
                                }
                            } else {
                                caret.pos.x = 0;
                            }
                        }
                        buf.terminal_state.limit_caret_pos(buf, caret);
                    }
                    'C' => {
                        // Cursor Forward
                        self.state = EngineState::Default;
                        if self.parsed_numbers.is_empty() {
                            caret.right(buf, 1);
                        } else {
                            caret.right(buf, self.parsed_numbers[0]);
                        }
                    }
                    'j' | // CSI Pn j
                          // HPB - Character position backward
                    'D' => {
                        // Cursor Back
                        self.state = EngineState::Default;
                        if self.parsed_numbers.is_empty() {
                            caret.left(buf, 1);
                        } else {
                            caret.left(buf, self.parsed_numbers[0]);
                        }
                    }
                    'k' | // CSI Pn k
                          // VPB - Line position backward
                    'A' => {
                        // Cursor Up
                        self.state = EngineState::Default;
                        if self.parsed_numbers.is_empty() {
                            caret.up(buf, current_layer, 1);
                        } else {
                            caret.up(buf, current_layer, self.parsed_numbers[0]);
                        }
                    }
                    'B' => {
                        // Cursor Down
                        self.state = EngineState::Default;
                        if self.parsed_numbers.is_empty() {
                            caret.down(buf, current_layer, 1);
                        } else {
                            caret.down(buf, current_layer,self.parsed_numbers[0]);
                        }
                    }
                    's' => {
                        if buf.terminal_state.dec_margin_mode_left_right {
                            return self.set_left_and_right_margins(buf);
                        }
                        self.save_cursor_position(caret);
                    }
                    'u' => self.restore_cursor_position(caret),
                    'd' => {
                        // CSI Pn d
                        // VPA - Line position absolute
                        self.state = EngineState::Default;
                        let num = match self.parsed_numbers.first() {
                            Some(n) => n - 1,
                            _ => 0,
                        };
                        caret.pos.y = buf.get_first_visible_line()as i32 + num;
                        buf.terminal_state.limit_caret_pos(buf, caret);
                    }
                    'e' => {
                        // CSI Pn e
                        // VPR - Line position forward
                        self.state = EngineState::Default;
                        let num = match self.parsed_numbers.first() {
                            Some(n) => *n,
                            _ => 1,
                        };
                        caret.pos.y = buf.get_first_visible_line() as i32 + caret.pos.y + num;
                        buf.terminal_state.limit_caret_pos(buf, caret);
                    }
                    '\'' => {
                        // Horizontal Line Position Absolute
                        self.state = EngineState::Default;
                        let num = match self.parsed_numbers.first() {
                            Some(n) => n - 1,
                            _ => 0,
                        };
                        if let Some(layer) = &buf.layers.get(0) {
                            if let Some(line) = layer.lines.get(caret.pos.y as usize) {
                                caret.pos.x = num.clamp(0, line.get_line_length() as i32);
                                buf.terminal_state.limit_caret_pos(buf, caret);
                            }
                        } else {
                            return Err(Box::new(ParserError::InvalidBuffer));
                        }
                    }
                    'a' => {
                        // CSI Pn a
                        // HPR - Character position forward
                        self.state = EngineState::Default;
                        let num = match self.parsed_numbers.first() {
                            Some(n) => *n,
                            _ => 1,
                        };
                        if let Some(layer) = &buf.layers.get(0) {
                            if let Some(line) = layer.lines.get(caret.pos.y as usize) {
                                caret.pos.x =
                                    min(line.get_line_length() as i32, caret.pos.x + num);
                                buf.terminal_state.limit_caret_pos(buf, caret);
                            }
                        } else {
                            return Err(Box::new(ParserError::InvalidBuffer));
                        }
                    }

                    'G' => {
                        // Cursor Horizontal Absolute
                        self.state = EngineState::Default;
                        let num = match self.parsed_numbers.first() {
                            Some(n) => n - 1,
                            _ => 0,
                        };
                        caret.pos.x = num;
                        buf.terminal_state.limit_caret_pos(buf, caret);
                    }
                    'E' => {
                        // Cursor Next Line
                        self.state = EngineState::Default;
                        let num = match self.parsed_numbers.first() {
                            Some(n) => *n,
                            _ => 1,
                        };
                        caret.pos.y = buf.get_first_visible_line()as i32 + caret.pos.y + num;
                        caret.pos.x = 0;
                        buf.terminal_state.limit_caret_pos(buf, caret);
                    }
                    'F' => {
                        // Cursor Previous Line
                        self.state = EngineState::Default;
                        let num = match self.parsed_numbers.first() {
                            Some(n) => *n,
                            _ => 1,
                        };
                        caret.pos.y = buf.get_first_visible_line()as i32 + caret.pos.y - num;
                        caret.pos.x = 0;
                        buf.terminal_state.limit_caret_pos(buf, caret);
                    }

                    'n' => {
                        // CSI Ps n
                        // DSR - Device Status Report
                        self.state = EngineState::Default;
                        if self.parsed_numbers.is_empty() {
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                self.current_escape_sequence.clone(),
                            )));
                        }
                        if self.parsed_numbers.len() != 1 {
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                self.current_escape_sequence.clone(),
                            )));
                        }
                        match self.parsed_numbers.first().unwrap() {
                            5 => {
                                // Device status report
                                return Ok(CallbackAction::SendString("\x1b[0n".to_string()));
                            }
                            6 => {
                                // Get cursor position
                                let s = format!(
                                    "\x1b[{};{}R",
                                    min(buf.get_height(), caret.pos.y as usize + 1),
                                    min(buf.get_width(), caret.pos.x as usize + 1)
                                );
                                return Ok(CallbackAction::SendString(s));
                            }
                            255 => {
                                // Current screen size
                                let s = format!(
                                    "\x1b[{};{}R",
                                    buf.get_height(), buf.get_width()
                                );
                                return Ok(CallbackAction::SendString(s));
                            }
                            _ => {
                                return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                    self.current_escape_sequence.clone(),
                                )));
                            }
                        }
                    }

                    /*  TODO:
                        Insert Column 	  CSI Pn ' }
                        Delete Column 	  CSI Pn ' ~
                    */
                    'X' => return self.erase_character(caret, buf, current_layer),
                    '@' => {
                        // Insert character
                        self.state = EngineState::Default;

                        if let Some(number) = self.parsed_numbers.first() {
                            for _ in 0..*number {
                                caret.ins(buf, current_layer);
                            }
                        } else {
                            caret.ins(buf, current_layer);
                            if self.parsed_numbers.len() != 1 {
                                return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                    self.current_escape_sequence.clone(),
                                )));
                            }
                        }
                    }
                    'M' => {
                        // Delete line
                        self.state = EngineState::Default;
                        if matches!(self.ansi_music, MusicOption::Conflicting)
                            || matches!(self.ansi_music, MusicOption::Both)
                        {
                            self.cur_music = Some(AnsiMusic::default());
                            self.dotted_note = false;
                            self.state = EngineState::ParseAnsiMusic(MusicState::ParseMusicStyle);
                        } else if self.parsed_numbers.is_empty() {
                            if let Some(layer) = buf.layers.get(0) {
                                if caret.pos.y < layer.lines.len() as i32 {
                                    buf.remove_terminal_line(current_layer, caret.pos.y);
                                }
                            } else {
                                return Err(Box::new(ParserError::InvalidBuffer));
                            }
                        } else {
                            if self.parsed_numbers.len() != 1 {
                                return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                    self.current_escape_sequence.clone(),
                                )));
                            }
                            if let Some(number) = self.parsed_numbers.first() {
                                let mut number = *number;
                                if let Some(layer) = buf.layers.get(0) {
                                    number = min(number, layer.lines.len() as i32 - caret.pos.y);
                                } else {
                                    return Err(Box::new(ParserError::InvalidBuffer));
                                }
                                for _ in 0..number {
                                    buf.remove_terminal_line(current_layer, caret.pos.y);
                                }
                            } else {
                                return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                    self.current_escape_sequence.clone(),
                                )));
                            }
                        }
                    }
                    'N' => {
                        if matches!(self.ansi_music, MusicOption::Banana)
                            || matches!(self.ansi_music, MusicOption::Both)
                        {
                            self.cur_music = Some(AnsiMusic::default());
                            self.dotted_note = false;
                            self.state = EngineState::ParseAnsiMusic(MusicState::ParseMusicStyle);
                        }
                    }

                    '|' => {
                        if !matches!(self.ansi_music, MusicOption::Off) {
                            self.cur_music = Some(AnsiMusic::default());
                            self.dotted_note = false;
                            self.state = EngineState::ParseAnsiMusic(MusicState::ParseMusicStyle);
                        }
                    }

                    'P' => {
                        // Delete character
                        self.state = EngineState::Default;
                        if self.parsed_numbers.is_empty() {
                            caret.del(buf, current_layer);
                        } else {
                            if self.parsed_numbers.len() != 1 {
                                return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                    self.current_escape_sequence.clone(),
                                )));
                            }
                            if let Some(number) = self.parsed_numbers.first() {
                                for _ in 0..*number {
                                    caret.del(buf,current_layer);
                                }
                            } else {
                                return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                    self.current_escape_sequence.clone(),
                                )));
                            }
                        }
                    }

                    'L' => {
                        // Insert line
                        self.state = EngineState::Default;
                        if self.parsed_numbers.is_empty() {
                            buf.insert_terminal_line( current_layer, caret.pos.y);
                        } else {
                            if self.parsed_numbers.len() != 1 {
                                return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                    self.current_escape_sequence.clone(),
                                )));
                            }
                            if let Some(number) = self.parsed_numbers.first() {
                                for _ in 0..*number {
                                    buf.insert_terminal_line(current_layer,caret.pos.y);
                                }
                            } else {
                                return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                    self.current_escape_sequence.clone(),
                                )));
                            }
                        }
                    }

                    'J' => {
                        // Erase in Display
                        self.state = EngineState::Default;
                        if self.parsed_numbers.is_empty() {
                            buf.clear_buffer_down(current_layer,caret);
                        } else if let Some(number) = self.parsed_numbers.first() {
                            match *number {
                                0 => {
                                    buf.clear_buffer_down(current_layer,caret);
                                }
                                1 => {
                                    buf.clear_buffer_up(current_layer,caret);
                                }
                                2 |  // clear entire screen
                                3 => {
                                    buf.clear_screen(current_layer,caret);
                                }
                                _ => {
                                    buf.clear_buffer_down(current_layer,caret);
                                    return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_escape_sequence.clone())));
                                }
                            }
                        } else {
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                self.current_escape_sequence.clone(),
                            )));
                        }
                    }

                    '?' => {
                        if !is_start {
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                self.current_escape_sequence.clone(),
                            )));
                        }
                        // read custom command
                        self.state = EngineState::ReadCSICommand;
                    }
                    '=' => {
                        if !is_start {
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                self.current_escape_sequence.clone(),
                            )));
                        }
                        // read custom command
                        self.state = EngineState::ReadCSIRequest;
                        return Ok(CallbackAction::None);
                    }
                    '!' => {
                        if !is_start {
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                self.current_escape_sequence.clone(),
                            )));
                        }
                        // read custom command
                        self.state = EngineState::ReadRIPSupportRequest;
                    }

                    '*' => {
                        self.state = EngineState::EndCSI('*');
                    }
                    '$' => {
                        self.state = EngineState::EndCSI('$');
                    }
                    ' ' => {
                        self.state = EngineState::EndCSI(' ');
                    }

                    'K' => {
                        // Erase in line
                        self.state = EngineState::Default;
                        if self.parsed_numbers.is_empty() {
                            buf.clear_line_end(current_layer,caret);
                        } else {
                            match self.parsed_numbers.first() {
                                Some(0) => {
                                    buf.clear_line_end(current_layer,caret);
                                }
                                Some(1) => {
                                    buf.clear_line_start(current_layer,caret);
                                }
                                Some(2) => {
                                    buf.clear_line(current_layer,caret);
                                }
                                _ => {
                                    return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                        self.current_escape_sequence.clone(),
                                    )));
                                }
                            }
                        }
                    }
                    'c' => return self.device_attributes(),
                    'r' => return if self.parsed_numbers.len() > 2 {
                        self.change_scrolling_region(buf, caret)
                    } else {
                        self.set_top_and_bottom_margins(buf, caret)
                    },
                    'h' => {
                        self.state = EngineState::Default;
                        if self.parsed_numbers.len() != 1 {
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                self.current_escape_sequence.clone(),
                            )));
                        }
                        match self.parsed_numbers.first() {
                            Some(4) => {
                                caret.insert_mode = true;
                            }
                            _ => {
                                return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                    self.current_escape_sequence.clone(),
                                )))
                            }
                        }
                    }

                    'l' => {
                        self.state = EngineState::Default;
                        if self.parsed_numbers.len() != 1 {
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                self.current_escape_sequence.clone(),
                            )));
                        }
                        match self.parsed_numbers.first() {
                            Some(4) => {
                                caret.insert_mode = false;
                            }
                            _ => {
                                return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                    self.current_escape_sequence.clone(),
                                )))
                            }
                        }
                    }
                    '~' => {
                        self.state = EngineState::Default;
                        if self.parsed_numbers.len() != 1 {
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                self.current_escape_sequence.clone(),
                            )));
                        }
                        match self.parsed_numbers.first() {
                            Some(1) => {
                                caret.pos.x = 0;
                            } // home
                            Some(2) => {
                                caret.ins(buf, current_layer);
                            } // home
                            Some(3) => {
                                caret.del(buf, current_layer);
                            }
                            Some(4) => {
                                caret.eol(buf);
                            }
                            Some(5 | 6) => {} // pg up/downf
                            _ => {
                                return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                    self.current_escape_sequence.clone(),
                                )))
                            }
                        }
                    }

                    't' => {
                        self.state = EngineState::Default;
                        match self.parsed_numbers.len() {
                            3 => return self.window_manipulation(buf),
                            4 => return self.select_24bit_color(buf, caret),
                            _ => return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                self.current_escape_sequence.clone(),
                            )))
                        }
                    }
                    'S' => {
                        // Scroll Up
                        self.state = EngineState::Default;
                        let num = if let Some(number) = self.parsed_numbers.first() {
                            *number
                        } else {
                            1
                        };
                        (0..num).for_each(|_| buf.scroll_up(current_layer));
                    }
                    'T' => {
                        // Scroll Down
                        self.state = EngineState::Default;
                        let num = if let Some(number) = self.parsed_numbers.first() {
                            *number
                        } else {
                            1
                        };
                        (0..num).for_each(|_| buf.scroll_down(current_layer));
                    }
                    'b' => {
                        // CSI Pn b
                        // REP - Repeat the preceding graphic character Pn times (REP).
                        self.state = EngineState::Default;
                        let num: i32 = if let Some(number) = self.parsed_numbers.first() {
                            *number
                        } else {
                            1
                        };
                        let ch = AttributedChar::new(self.last_char, caret.attribute);
                        (0..num).for_each(|_| buf.print_char(current_layer, caret, ch));
                    }
                    'g' => {
                        // CSI Ps g
                        // TBC - Tabulation clear
                        self.state = EngineState::Default;
                        if self.parsed_numbers.len() > 1 {
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                format!("Invalid parameter number in clear tab stops: {}", self.parsed_numbers.len()),
                            )));
                        }

                        let num: i32 = if let Some(number) = self.parsed_numbers.first() {
                            *number
                        } else {
                            0
                        };

                        match num {
                            0 => { buf.terminal_state.remove_tab_stop(caret.get_position().x) }
                            3 | 5 => {
                                buf.terminal_state.clear_tab_stops();
                            }
                            _ => {
                                return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                    format!("Unsupported option in clear tab stops sequence: {num}"),
                                )));
                            }
                        }
                    }
                    'Y' => {
                        // CVT - Cursor line tabulation
                        self.state = EngineState::Default;
                        if self.parsed_numbers.len() > 1 {
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                format!("Invalid parameter number in goto next tab stop: {}", self.parsed_numbers.len()),
                            )));
                        }

                        let num: i32 = if let Some(number) = self.parsed_numbers.first() {
                            *number
                        } else {
                            1
                        };
                        (0..num).for_each(|_| caret.set_x_position(buf.terminal_state.next_tab_stop(caret.get_position().x)));
                    }
                    'Z' => {
                        // CBT - Cursor backward tabulation
                        self.state = EngineState::Default;
                        if self.parsed_numbers.len() > 1 {
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                format!("Invalid parameter number in goto next tab stop: {}", self.parsed_numbers.len()),
                            )));
                        }

                        let num: i32 = if let Some(number) = self.parsed_numbers.first() {
                            *number
                        } else {
                            1
                        };
                        (0..num).for_each(|_| caret.set_x_position(buf.terminal_state.prev_tab_stop(caret.get_position().x)));
                    }
                    _ => {
                        self.state = EngineState::ReadCSISequence(false);
                        if ('\x40'..='\x7E').contains(&ch) {
                            // unknown control sequence, terminate reading
                            self.state = EngineState::Default;
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                self.current_escape_sequence.clone(),
                            )));
                        }

                        if ch.is_ascii_digit() {
                            let d = match self.parsed_numbers.pop() {
                                Some(number) => number,
                                _ => 0,
                            };
                            self.parsed_numbers.push(parse_next_number(d, ch as u8));
                        } else if ch == ';' {
                            self.parsed_numbers.push(0);
                        } else {
                            self.state = EngineState::Default;
                            // error in control sequence, terminate reading
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                self.current_escape_sequence.clone(),
                            )));
                        }
                    }
                }
            }

            EngineState::Default => match ch {
                '\x1B' => {
                    self.current_escape_sequence.clear();
                    self.current_escape_sequence.push_str("<ESC>");
                    self.state = EngineState::Default;
                    self.state = EngineState::ReadEscapeSequence;
                }
                LF => caret.lf(buf, current_layer),
                FF => caret.ff(buf, current_layer),
                CR => caret.cr(buf),
                BEL => return Ok(CallbackAction::Beep),
                '\x7F' => caret.del(buf, current_layer),
                _ => {
                    if ch == crate::BS && self.bs_is_ctrl_char {
                        caret.bs(buf, current_layer);
                    } else if (ch == '\x00' || ch == '\u{00FF}') && self.bs_is_ctrl_char {
                        caret.reset_color_attribute();
                    } else {
                        self.last_char = unsafe { char::from_u32_unchecked(ch as u32) };
                        let ch = AttributedChar::new(self.last_char, caret.get_attribute());
                        buf.print_char(current_layer, caret, ch);
                    }
                }
            },
        }

        Ok(CallbackAction::None)
    }
}

impl Parser {
    fn invoke_macro_by_id(
        &mut self,
        buf: &mut Buffer,
        current_layer: usize,
        caret: &mut Caret,
        id: i32,
    ) -> EngineResult<CallbackAction> {
        let m = if let Some(m) = self.macros.get(&(id as usize)) {
            m.clone()
        } else {
            return Ok(CallbackAction::None);
        };
        for ch in m.chars() {
            self.print_char(buf, current_layer, caret, ch)?;
        }
        Ok(CallbackAction::None)
    }

    fn execute_aps_command(&self, _buf: &mut Buffer, _caret: &mut Caret) {
        println!("TODO execute APS command: {}", self.parse_string);
    }
}

fn set_font_selection_success(buf: &mut Buffer, caret: &mut Caret, slot: usize) {
    buf.terminal_state.font_selection_state = FontSelectionState::Success;
    caret.set_font_page(slot);
    log::info!("Set Font to {slot}");

    if caret.attribute.is_blinking() && caret.attribute.is_bold() {
        buf.terminal_state.high_intensity_blink_attribute_font_slot = slot;
    } else if caret.attribute.is_blinking() {
        buf.terminal_state.blink_attribute_font_slot = slot;
    } else if caret.attribute.is_bold() {
        buf.terminal_state.high_intensity_attribute_font_slot = slot;
    } else {
        buf.terminal_state.normal_attribute_font_slot = slot;
    }
}

pub fn parse_next_number(x: i32, ch: u8) -> i32 {
    x.saturating_mul(10)
        .saturating_add(ch as i32)
        .saturating_sub(b'0' as i32)
}
