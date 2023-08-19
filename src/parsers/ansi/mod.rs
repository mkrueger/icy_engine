// Useful description: https://vt100.net/docs/vt510-rm/chapter4.html
//                     https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Normal-tracking-mode
use std::{
    cmp::{max, min},
    collections::HashMap,
    fmt::Display,
};

use super::{ascii, BufferParser};
use crate::{
    update_crc16, AnsiMusic, AttributedChar, AutoWrapMode, Buffer, CallbackAction, Caret,
    EngineResult, FontSelectionState, MouseMode, MusicAction, MusicStyle, OriginMode, ParserError,
    Position, TerminalScrolling, TextAttribute, BEL, BS, CR, FF, LF,
};

mod ansi_commands;
mod constants;
mod dcs;

#[cfg(test)]
mod sixel_tests;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy)]
pub enum MusicState {
    Default,
    ParseMusicStyle,
    SetTempo(u16),
    Pause(i32),
    SetOctave,
    Note(usize, u32),
    SetLength(i32),
}

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

    ParseAnsiMusic(MusicState),

    ReadAPS(ReadSTState),
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicOption {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaudOption {
    Off,
    Emulation(u32),
}

impl Display for BaudOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Off => write!(f, "Off"),
            Self::Emulation(v) => write!(f, "{v}"),
        }
    }
}

/*
Generated with:
for oct in range(1, 8):
    for i in range(16, 28):
        n = i + (28-16) * (oct - 1)
        freq = 440.0 * pow(2.0, (n - 49.0) / 12.0)
        print("{:.4f}".format(freq), end=", ")
    print()
*/
pub const FREQ: [f32; 12 * 7] = [
    //  C      C#       D        D#       E        F        F#       G         G#        A         A#        B
    65.4064, 69.2957, 73.4162, 77.7817, 82.4069, 87.3071, 92.4986, 97.9989, 103.8262, 110.0000,
    116.5409, 123.4708, 130.8128, 138.5913, 146.8324, 155.5635, 164.8138, 174.6141, 184.9972,
    195.9977, 207.6523, 220.0000, 233.0819, 246.9417, 261.6256, 277.1826, 293.6648, 311.127,
    329.6276, 349.2282, 369.9944, 391.9954, 415.3047, 440.0000, 466.1638, 493.8833, 523.2511,
    554.3653, 587.3295, 622.254, 659.2551, 698.4565, 739.9888, 783.9909, 830.6094, 880.0000,
    932.3275, 987.7666, 1046.5023, 1108.7305, 1_174.659, 1244.5079, 1318.5102, 1396.9129,
    1479.9777, 1567.9817, 1661.2188, 1760.0000, 1_864.655, 1975.5332, 2093.0045, 2217.461,
    2_349.318, 2489.0159, 2637.0205, 2_793.826, 2959.9554, 3135.9635, 3322.4376, 3520.0000,
    3_729.31, 3951.0664, 4_186.009, 4_434.922, 4_698.636, 4978.0317, 5_274.041, 5_587.652,
    5919.9108, 6_271.927, 6_644.875, 7040.0000, 7_458.62, 7_902.132,
];

pub struct Parser {
    ascii_parser: ascii::Parser,
    pub(crate) state: EngineState,
    saved_pos: Position,
    saved_cursor_opt: Option<Caret>,
    pub(crate) parsed_numbers: Vec<i32>,

    current_escape_sequence: String,

    /*     current_sixel_color: i32,
        sixel_cursor: Position,
        current_sixel_palette: Palette,
    */
    pub ansi_music: MusicOption,
    cur_music: Option<AnsiMusic>,
    cur_octave: usize,
    cur_length: u32,
    cur_tempo: u32,

    last_char: char,
    pub aps_string: String,
    pub(crate) macros: HashMap<usize, String>,
    pub dcs_string: String,
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
            aps_string: String::new(),
            macros: HashMap::new(),
            dcs_string: String::new(),
            last_char: '\0',
        }
    }
}

impl BufferParser for Parser {
    fn convert_from_unicode(&self, ch: char) -> char {
        self.ascii_parser.convert_from_unicode(ch)
    }

    fn convert_to_unicode(&self, ch: char) -> char {
        self.ascii_parser.convert_to_unicode(ch)
    }

    #[allow(clippy::single_match)]
    fn print_char(
        &mut self,
        buf: &mut Buffer,
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
                            caret.ff(buf);
                            buf.terminal_state.reset();
                            self.macros.clear();
                            Ok(CallbackAction::None)
                        }

                        'D' => {
                            // Index
                            caret.index(buf);
                            Ok(CallbackAction::None)
                        }
                        'M' => {
                            // Reverse Index
                            caret.reverse_index(buf);
                            Ok(CallbackAction::None)
                        }

                        'E' => {
                            // Next Line
                            caret.next_line(buf);
                            Ok(CallbackAction::None)
                        }

                        'P' => {
                            // DCS
                            self.state = EngineState::RecordDCS(ReadSTState::Default(0));
                            self.dcs_string.clear();
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
                            self.aps_string.clear();
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
                    self.aps_string.push(ch);
                }
                ReadSTState::GotEscape(nesting_level) => {
                    if ch == '\\' {
                        self.state = EngineState::Default;
                        self.execute_aps_command(buf, caret);
                        return Ok(CallbackAction::None);
                    }
                    self.state = EngineState::ReadAPS(ReadSTState::Default(*nesting_level));
                    self.aps_string.push('\x1B');
                    self.aps_string.push(ch);
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
                    self.parsed_numbers.push(d * 10 + ch as i32 - b'0' as i32);
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
                        self.dcs_string
                    ))));
                }
                ReadSTState::Default(nesting_level) => match ch {
                    '\x1B' => {
                        self.state = EngineState::RecordDCS(ReadSTState::GotEscape(*nesting_level));
                    }
                    _ => {
                        self.dcs_string.push(ch);
                    }
                },
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
                            Some(33) => buf.terminal_state.set_use_ice_colors(false),
                            Some(35) => caret.is_blinking = true,

                            Some(69) => {
                                buf.terminal_state.dec_margin_mode_left_right = false;
                                buf.terminal_state.margins_left_right = None;
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
                            Some(33) => buf.terminal_state.set_use_ice_colors(true),
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
                        self.parsed_numbers.push(d * 10 + ch as i32 - b'0' as i32);
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

                                if buf.terminal_state.use_ice_colors() {
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
                        self.parsed_numbers.push(d * 10 + ch as i32 - b'0' as i32);
                    }
                    ';' => {
                        self.parsed_numbers.push(0);
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
                    return self.print_char(buf, caret, ch);
                }
            }

            EngineState::EndCSI(func) => {
                self.current_escape_sequence.push(ch);
                match *func {
                    '*' => match ch {
                        'z' => return self.invoke_macro(buf, caret),
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
                                    if i < buf.terminal_state.tab_count() - 1 {
                                        str.push('/');
                                    }
                                });
                                str.push_str("\x1B\\");
                                return Ok(CallbackAction::SendString(str));
                            }
                        }
                        _ => {}
                    },

                    ' ' => {
                        self.state = EngineState::Default;

                        match ch {
                            'D' => return self.font_selection(buf, caret),
                            'A' => self.scroll_right(buf),
                            '@' => self.scroll_left(buf),
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
                                caret.pos.y = buf.get_first_visible_line()
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
                            caret.up(buf, 1);
                        } else {
                            caret.up(buf, self.parsed_numbers[0]);
                        }
                    }
                    'B' => {
                        // Cursor Down
                        self.state = EngineState::Default;
                        if self.parsed_numbers.is_empty() {
                            caret.down(buf, 1);
                        } else {
                            caret.down(buf, self.parsed_numbers[0]);
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
                        caret.pos.y = buf.get_first_visible_line() + num;
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
                        caret.pos.y = buf.get_first_visible_line() + caret.pos.y + num;
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
                                caret.pos.x = num.clamp(0, line.get_line_length() as i32 + 1);
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
                                    min(line.get_line_length() as i32 + 1, caret.pos.x + num);
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
                        caret.pos.y = buf.get_first_visible_line() + caret.pos.y + num;
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
                        caret.pos.y = buf.get_first_visible_line() + caret.pos.y - num;
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
                                    min(buf.get_buffer_height(), caret.pos.y + 1),
                                    min(buf.get_buffer_width(), caret.pos.x + 1)
                                );
                                return Ok(CallbackAction::SendString(s));
                            }
                            255 => {
                                // Current screen size
                                let s = format!(
                                    "\x1b[{};{}R",
                                    buf.terminal_state.height, buf.terminal_state.width
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
                    'X' => return self.erase_character(caret, buf),
                    '@' => {
                        // Insert character
                        self.state = EngineState::Default;

                        if let Some(number) = self.parsed_numbers.first() {
                            for _ in 0..*number {
                                caret.ins(buf);
                            }
                        } else {
                            caret.ins(buf);
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
                            self.state = EngineState::ParseAnsiMusic(MusicState::ParseMusicStyle);
                        } else if self.parsed_numbers.is_empty() {
                            if let Some(layer) = buf.layers.get(0) {
                                if caret.pos.y < layer.lines.len() as i32 {
                                    buf.remove_terminal_line(caret.pos.y);
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
                                    buf.remove_terminal_line(caret.pos.y);
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
                            self.state = EngineState::ParseAnsiMusic(MusicState::ParseMusicStyle);
                        }
                    }

                    '|' => {
                        if !matches!(self.ansi_music, MusicOption::Off) {
                            self.cur_music = Some(AnsiMusic::default());
                            self.state = EngineState::ParseAnsiMusic(MusicState::ParseMusicStyle);
                        }
                    }

                    'P' => {
                        // Delete character
                        self.state = EngineState::Default;
                        if self.parsed_numbers.is_empty() {
                            caret.del(buf);
                        } else {
                            if self.parsed_numbers.len() != 1 {
                                return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                    self.current_escape_sequence.clone(),
                                )));
                            }
                            if let Some(number) = self.parsed_numbers.first() {
                                for _ in 0..*number {
                                    caret.del(buf);
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
                            buf.insert_terminal_line(caret.pos.y);
                        } else {
                            if self.parsed_numbers.len() != 1 {
                                return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                    self.current_escape_sequence.clone(),
                                )));
                            }
                            if let Some(number) = self.parsed_numbers.first() {
                                for _ in 0..*number {
                                    buf.insert_terminal_line(caret.pos.y);
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
                            buf.clear_buffer_down(caret);
                        } else if let Some(number) = self.parsed_numbers.first() {
                            match *number {
                                0 => {
                                    buf.clear_buffer_down(caret);
                                }
                                1 => {
                                    buf.clear_buffer_up(caret);
                                }
                                2 |  // clear entire screen
                                3 => {
                                    buf.clear_screen(caret);
                                }
                                _ => {
                                    buf.clear_buffer_down(caret);
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
                            buf.clear_line_end(caret);
                        } else {
                            match self.parsed_numbers.first() {
                                Some(0) => {
                                    buf.clear_line_end(caret);
                                }
                                Some(1) => {
                                    buf.clear_line_start(caret);
                                }
                                Some(2) => {
                                    buf.clear_line(caret);
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
                    'r' => return self.set_top_and_bottom_margins(buf, caret),
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
                                caret.ins(buf);
                            } // home
                            Some(3) => {
                                caret.del(buf);
                            }
                            Some(4) => {
                                caret.eol(buf);
                            }
                            Some(5 | 6) => {} // pg up/down
                            _ => {
                                return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                    self.current_escape_sequence.clone(),
                                )))
                            }
                        }
                    }

                    't' => {
                        self.state = EngineState::Default;
                        if self.parsed_numbers.len() != 4 {
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                self.current_escape_sequence.clone(),
                            )));
                        }
                        let r = self.parsed_numbers[1];
                        let g = self.parsed_numbers[2];
                        let b = self.parsed_numbers[3];
                        let color = buf.palette.insert_color_rgb(r as u8, g as u8, b as u8);

                        match self.parsed_numbers.first() {
                            Some(0) => {
                                caret.attribute.set_background(color);
                            }
                            Some(1) => {
                                caret.attribute.set_foreground(color);
                            }
                            _ => {
                                return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                    self.current_escape_sequence.clone(),
                                )))
                            }
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
                        (0..num).for_each(|_| buf.scroll_up());
                    }
                    'T' => {
                        // Scroll Down
                        self.state = EngineState::Default;
                        let num = if let Some(number) = self.parsed_numbers.first() {
                            *number
                        } else {
                            1
                        };
                        (0..num).for_each(|_| buf.scroll_down());
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
                        (0..num).for_each(|_| buf.print_char(caret, ch));
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
                            self.parsed_numbers.push(d * 10 + ch as i32 - b'0' as i32);
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
                '\x00' | '\u{00FF}' => {
                    caret.reset_color_attribute();
                }
                LF => caret.lf(buf),
                FF => caret.ff(buf),
                CR => caret.cr(buf),
                BS => caret.bs(buf),
                BEL => return Ok(CallbackAction::Beep),
                '\x7F' => caret.del(buf),
                _ => {
                    self.last_char = unsafe { char::from_u32_unchecked(ch as u32) };
                    let ch = AttributedChar::new(self.last_char, caret.attribute);
                    buf.print_char(caret, ch);
                }
            },
        }

        Ok(CallbackAction::None)
    }
}

impl Parser {
    fn parse_ansi_music(&mut self, ch: char) -> EngineResult<CallbackAction> {
        if let EngineState::ParseAnsiMusic(state) = self.state {
            match state {
                MusicState::ParseMusicStyle => {
                    self.state = EngineState::ParseAnsiMusic(MusicState::Default);
                    match ch {
                        'F' => self
                            .cur_music
                            .as_mut()
                            .unwrap()
                            .music_actions
                            .push(MusicAction::SetStyle(MusicStyle::Foreground)),
                        'B' => self
                            .cur_music
                            .as_mut()
                            .unwrap()
                            .music_actions
                            .push(MusicAction::SetStyle(MusicStyle::Background)),
                        'N' => self
                            .cur_music
                            .as_mut()
                            .unwrap()
                            .music_actions
                            .push(MusicAction::SetStyle(MusicStyle::Normal)),
                        'L' => self
                            .cur_music
                            .as_mut()
                            .unwrap()
                            .music_actions
                            .push(MusicAction::SetStyle(MusicStyle::Legato)),
                        'S' => self
                            .cur_music
                            .as_mut()
                            .unwrap()
                            .music_actions
                            .push(MusicAction::SetStyle(MusicStyle::Staccato)),
                        _ => return self.parse_ansi_music(ch),
                    }
                }
                MusicState::SetTempo(x) => {
                    let mut x = x;
                    if ch.is_ascii_digit() {
                        x = x * 10 + ch as u16 - b'0' as u16;
                        self.state = EngineState::ParseAnsiMusic(MusicState::SetTempo(x));
                    } else {
                        self.state = EngineState::ParseAnsiMusic(MusicState::Default);
                        self.cur_tempo = x.clamp(32, 255) as u32;
                        return Ok(self.parse_default_ansi_music(ch));
                    }
                }
                MusicState::SetOctave => {
                    if ('0'..='6').contains(&ch) {
                        self.cur_octave = ((ch as u8) - b'0') as usize;
                        self.state = EngineState::ParseAnsiMusic(MusicState::Default);
                    } else {
                        return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                            self.current_escape_sequence.clone(),
                        )));
                    }
                }
                MusicState::Note(n, len) => {
                    self.state = EngineState::ParseAnsiMusic(MusicState::Default);
                    match ch {
                        '+' | '#' => {
                            if n + 1 < FREQ.len() {
                                self.state =
                                    EngineState::ParseAnsiMusic(MusicState::Note(n + 1, len));
                            }
                        }
                        '-' => {
                            if n > 0 {
                                // B
                                self.state =
                                    EngineState::ParseAnsiMusic(MusicState::Note(n - 1, len));
                            }
                        }
                        '0'..='9' => {
                            let len = len * 10 + ch as u32 - b'0' as u32;
                            self.state = EngineState::ParseAnsiMusic(MusicState::Note(n, len));
                        }
                        '.' => {
                            let len = len * 3 / 2;
                            self.state = EngineState::ParseAnsiMusic(MusicState::Note(n, len));
                        }
                        _ => {
                            self.state = EngineState::ParseAnsiMusic(MusicState::Default);
                            let len = if len == 0 { self.cur_length } else { len };
                            self.cur_music.as_mut().unwrap().music_actions.push(
                                MusicAction::PlayNote(
                                    FREQ[n + (self.cur_octave * 12)],
                                    self.cur_tempo * len,
                                ),
                            );
                            return Ok(self.parse_default_ansi_music(ch));
                        }
                    }
                }
                MusicState::SetLength(x) => {
                    let mut x = x;
                    if ch.is_ascii_digit() {
                        x = x * 10 + ch as i32 - b'0' as i32;
                        self.state = EngineState::ParseAnsiMusic(MusicState::SetLength(x));
                    } else if ch == '.' {
                        x = x * 3 / 2;
                        self.state = EngineState::ParseAnsiMusic(MusicState::SetLength(x));
                    } else {
                        self.cur_length = (x as u32).clamp(1, 64);
                        return Ok(self.parse_default_ansi_music(ch));
                    }
                }
                MusicState::Pause(x) => {
                    let mut x = x;
                    if ch.is_ascii_digit() {
                        x = x * 10 + ch as i32 - b'0' as i32;
                        self.state = EngineState::ParseAnsiMusic(MusicState::Pause(x));
                    } else if ch == '.' {
                        x = x * 3 / 2;
                        self.state = EngineState::ParseAnsiMusic(MusicState::Pause(x));
                    } else {
                        let pause = (x as u32).clamp(1, 64);
                        self.cur_music
                            .as_mut()
                            .unwrap()
                            .music_actions
                            .push(MusicAction::Pause(self.cur_tempo * pause));
                        return Ok(self.parse_default_ansi_music(ch));
                    }
                }
                MusicState::Default => {
                    return Ok(self.parse_default_ansi_music(ch));
                }
            }
        }
        Ok(CallbackAction::None)
    }

    /// .
    ///
    /// # Panics
    ///
    /// Panics if .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    fn parse_default_ansi_music(&mut self, ch: char) -> CallbackAction {
        match ch {
            '\x0E' => {
                self.state = EngineState::Default;
                self.cur_octave = 3;
                return CallbackAction::PlayMusic(
                    self.cur_music.replace(AnsiMusic::default()).unwrap(),
                );
            }
            'T' => self.state = EngineState::ParseAnsiMusic(MusicState::SetTempo(0)),
            'L' => self.state = EngineState::ParseAnsiMusic(MusicState::SetLength(0)),
            'O' => self.state = EngineState::ParseAnsiMusic(MusicState::SetOctave),
            'C' => self.state = EngineState::ParseAnsiMusic(MusicState::Note(0, 0)),
            'D' => self.state = EngineState::ParseAnsiMusic(MusicState::Note(2, 0)),
            'E' => self.state = EngineState::ParseAnsiMusic(MusicState::Note(4, 0)),
            'F' => self.state = EngineState::ParseAnsiMusic(MusicState::Note(5, 0)),
            'G' => self.state = EngineState::ParseAnsiMusic(MusicState::Note(7, 0)),
            'A' => self.state = EngineState::ParseAnsiMusic(MusicState::Note(9, 0)),
            'B' => self.state = EngineState::ParseAnsiMusic(MusicState::Note(11, 0)),
            'M' => self.state = EngineState::ParseAnsiMusic(MusicState::ParseMusicStyle),
            '<' => {
                if self.cur_octave > 0 {
                    self.cur_octave -= 1;
                }
            }
            '>' => {
                if self.cur_octave < 6 {
                    self.cur_octave += 1;
                }
            }
            'P' => {
                self.state = EngineState::ParseAnsiMusic(MusicState::Pause(0));
            }
            _ => {}
        }
        CallbackAction::None
    }

    fn invoke_macro_by_id(
        &mut self,
        buf: &mut Buffer,
        caret: &mut Caret,
        id: i32,
    ) -> EngineResult<CallbackAction> {
        let m = if let Some(m) = self.macros.get(&(id as usize)) {
            m.clone()
        } else {
            return Ok(CallbackAction::None);
        };
        for ch in m.chars() {
            self.print_char(buf, caret, ch)?;
        }
        Ok(CallbackAction::None)
    }

    fn execute_aps_command(&self, _buf: &mut Buffer, _caret: &mut Caret) {
        println!("TODO execute APS command: {}", self.aps_string);
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