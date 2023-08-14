// Useful description: https://vt100.net/docs/vt510-rm/chapter4.html
//                     https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Normal-tracking-mode
use std::{
    cmp::{max, min},
    collections::HashMap,
    fmt::Display,
};

use self::constants::{ANSI_FONT_NAMES, COLOR_OFFSETS};

use super::{ascii, BufferParser};
use crate::{
    update_crc16, AnsiMusic, AttributedChar, AutoWrapMode, BitFont, Buffer, CallbackAction, Caret,
    EngineResult, FontSelectionState, MouseMode, MusicAction, MusicStyle, OriginMode, ParserError,
    Position, TerminalScrolling, TextAttribute, BEL, BS, CR, FF, LF, XTERM_256_PALETTE,
};

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
    ReadCSICommand, // CSI ?
    ReadCSIRequest, // CSI =
    EndCSI(char),

    RecordDCS(ReadSTState),
    ReadPossibleMacroInDCS(u8),

    ParseAnsiMusic(MusicState),

    ReadAPS(ReadSTState),
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
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
    pub(crate) current_font_page: usize,
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
            current_font_page: 0,
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
                    return self.invoke_macro(buf, caret, *self.parsed_numbers.first().unwrap());
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
                    'p' => {
                        // [!p Soft Teminal Reset
                        self.state = EngineState::Default;
                        buf.terminal_state.reset();
                    }
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

            EngineState::EndCSI(func) => {
                self.current_escape_sequence.push(ch);
                match *func {
                    '*' => {
                        match ch {
                            'z' => {
                                // DECINVM invoke macro
                                self.state = EngineState::Default;
                                if let Some(id) = self.parsed_numbers.first() {
                                    return self.invoke_macro(buf, caret, *id);
                                }
                                return Ok(CallbackAction::None);
                            }
                            'r' => {
                                // DECSCS—Select Communication Speed https://vt100.net/docs/vt510-rm/DECSCS.html
                                self.state = EngineState::Default;
                                let ps1 = self.parsed_numbers.first().unwrap_or(&0);
                                if *ps1 != 0 && *ps1 != 1 {
                                    // silently ignore all other options
                                    // 2 	Host Receive
                                    // 3 	Printer
                                    // 4 	Modem Hi
                                    // 5 	Modem Lo
                                    return Ok(CallbackAction::None);
                                }

                                if let Some(ps2) = self.parsed_numbers.get(1) {
                                    match ps2 {
                                        1 => buf.terminal_state.set_baud_rate(300),
                                        2 => buf.terminal_state.set_baud_rate(600),
                                        3 => buf.terminal_state.set_baud_rate(1200),
                                        4 => buf.terminal_state.set_baud_rate(2400),
                                        5 => buf.terminal_state.set_baud_rate(4800),
                                        6 => buf.terminal_state.set_baud_rate(9600),
                                        7 => buf.terminal_state.set_baud_rate(19200),
                                        8 => buf.terminal_state.set_baud_rate(38400),
                                        9 => buf.terminal_state.set_baud_rate(57600),
                                        10 => buf.terminal_state.set_baud_rate(76800),
                                        11 => buf.terminal_state.set_baud_rate(115_200),
                                        _ => buf.terminal_state.set_baud_rate(0),
                                    }
                                }
                                return Ok(CallbackAction::None);
                            }

                            'y' => {
                                // DECRQCRA—Request Checksum of Rectangular Area
                                // <https://vt100.net/docs/vt510-rm/DECRQCRA.html>
                                self.state = EngineState::Default;

                                if self.parsed_numbers.len() != 6 {
                                    return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                        self.current_escape_sequence.clone(),
                                    )));
                                }

                                let pt = self.parsed_numbers[2];
                                let pl = self.parsed_numbers[3];
                                let pb = self.parsed_numbers[4];
                                let pr = self.parsed_numbers[5];

                                if pt > pb
                                    || pl > pr
                                    || pr > buf.get_buffer_width()
                                    || pb > buf.get_buffer_height()
                                    || pl < 0
                                    || pt < 0
                                {
                                    return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                        format!("invalid area for requesting checksum pt:{pt} pl:{pl} pb:{pb} pr:{pr}"),
                                    )));
                                }

                                let mut crc16 = 0;
                                for y in pt..pb {
                                    for x in pl..pr {
                                        if let Some(ch) = buf.get_char_xy(x, y) {
                                            crc16 = update_crc16(crc16, ch.ch as u8);
                                            for b in ch.attribute.attr.to_be_bytes() {
                                                crc16 = update_crc16(crc16, b);
                                            }
                                            for b in ch.attribute.get_foreground().to_be_bytes() {
                                                crc16 = update_crc16(crc16, b);
                                            }
                                            for b in ch.attribute.get_background().to_be_bytes() {
                                                crc16 = update_crc16(crc16, b);
                                            }
                                        }
                                    }
                                }
                                return Ok(CallbackAction::SendString(format!(
                                    "\x1BP{}!~{crc16:04X}\x1B\\",
                                    self.parsed_numbers[0]
                                )));
                            }
                            _ => {}
                        }
                    }

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
                            'D' => {
                                if self.parsed_numbers.len() != 2 {
                                    self.current_escape_sequence.push('D');
                                    return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                        self.current_escape_sequence.clone(),
                                    )));
                                }

                                if let Some(nr) = self.parsed_numbers.get(1) {
                                    let nr = *nr as usize;
                                    if buf.get_font(nr).is_some() {
                                        self.current_font_page = nr;
                                        self.set_font_selection_success(buf, caret, nr);
                                        return Ok(CallbackAction::None);
                                    }
                                    if let Some(font_name) = ANSI_FONT_NAMES.get(nr) {
                                        match BitFont::from_name(font_name) {
                                            Ok(font) => {
                                                self.set_font_selection_success(buf, caret, nr);
                                                if let Some(font_number) =
                                                    buf.search_font_by_name(font.name.to_string())
                                                {
                                                    self.current_font_page = font_number;
                                                    return Ok(CallbackAction::None);
                                                }
                                                self.current_font_page = nr;
                                                buf.set_font(nr, font);
                                            }
                                            Err(err) => {
                                                buf.terminal_state.font_selection_state =
                                                    FontSelectionState::Failure;
                                                return Err(err);
                                            }
                                        }
                                    } else {
                                        buf.terminal_state.font_selection_state =
                                            FontSelectionState::Failure;
                                        return Err(Box::new(ParserError::UnsupportedFont(nr)));
                                    }
                                }
                            }
                            'A' => {
                                // Scroll Right
                                let num = if let Some(number) = self.parsed_numbers.first() {
                                    *number
                                } else {
                                    1
                                };
                                (0..num).for_each(|_| buf.scroll_right());
                            }
                            '@' => {
                                // Scroll Left
                                let num = if let Some(number) = self.parsed_numbers.first() {
                                    *number
                                } else {
                                    1
                                };
                                (0..num).for_each(|_| buf.scroll_left());
                            }
                            'd' => {
                                // tab stop remove
                                if self.parsed_numbers.len() != 1 {
                                    return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                        format!(
                                            "Invalid parameter number in remove tab stops: {}",
                                            self.parsed_numbers.len()
                                        ),
                                    )));
                                }
                                if let Some(num) = self.parsed_numbers.first() {
                                    buf.terminal_state.remove_tab_stop(*num - 1);
                                }
                            }
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
                    'm' => {
                        // Select Graphic Rendition
                        self.state = EngineState::Default;
                        if self.parsed_numbers.is_empty() {
                            caret.attr = TextAttribute::default(); // Reset or normal
                        }
                        let mut i = 0;
                        while i < self.parsed_numbers.len() {
                            let n = self.parsed_numbers[i];
                            match n {
                                0 => caret.attr = TextAttribute::default(), // Reset or normal
                                1 => caret.attr.set_is_bold(true),
                                2 => {
                                    caret.attr.set_is_faint(true);
                                }
                                3 => {
                                    caret.attr.set_is_italic(true);
                                }
                                4 => caret.attr.set_is_underlined(true),
                                5 | 6 => caret.attr.set_is_blinking(true),
                                7 => {
                                    let fg = caret.attr.get_foreground();
                                    caret.attr.set_foreground(caret.attr.get_background());
                                    caret.attr.set_background(fg);
                                }
                                8 => {
                                    caret.attr.set_is_concealed(true);
                                }
                                9 => caret.attr.set_is_crossed_out(true),
                                10 => self.current_font_page = 0, // Primary (default) font
                                11..=19 => { /* ignore alternate fonts for now */ } //return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone()))),
                                21 => caret.attr.set_is_double_underlined(true),
                                22 => {
                                    caret.attr.set_is_bold(false);
                                    caret.attr.set_is_faint(false);
                                }
                                23 => caret.attr.set_is_italic(false),
                                24 => caret.attr.set_is_underlined(false),
                                25 => caret.attr.set_is_blinking(false),
                                27 => {
                                    return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                        self.current_escape_sequence.clone(),
                                    )))
                                }
                                28 => caret.attr.set_is_concealed(false),
                                29 => caret.attr.set_is_crossed_out(false),
                                // set foreaground color
                                30..=37 => caret
                                    .attr
                                    .set_foreground(COLOR_OFFSETS[n as usize - 30] as u32),
                                38 => {
                                    caret
                                        .attr
                                        .set_foreground(self.parse_extended_colors(buf, &mut i)?);
                                    continue;
                                }
                                39 => caret.attr.set_foreground(7), // Set foreground color to default, ECMA-48 3rd
                                // set background color
                                40..=47 => caret
                                    .attr
                                    .set_background(COLOR_OFFSETS[n as usize - 40] as u32),
                                48 => {
                                    caret
                                        .attr
                                        .set_background(self.parse_extended_colors(buf, &mut i)?);
                                    continue;
                                }
                                49 => caret.attr.set_background(0), // Set background color to default, ECMA-48 3rd

                                // high intensity colors
                                90..=97 => caret
                                    .attr
                                    .set_foreground(8 + COLOR_OFFSETS[n as usize - 90] as u32),
                                100..=107 => caret
                                    .attr
                                    .set_background(8 + COLOR_OFFSETS[n as usize - 100] as u32),

                                _ => {
                                    return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                        self.current_escape_sequence.clone(),
                                    )));
                                }
                            }
                            i += 1;
                        }
                    }
                    'H' |    // Cursor Position
                    'f' => { // Character and Line Position (HVP)
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
                    'j' | // Character Position Backward
                    'D' => {
                        // Cursor Back
                        self.state = EngineState::Default;
                        if self.parsed_numbers.is_empty() {
                            caret.left(buf, 1);
                        } else {
                            caret.left(buf, self.parsed_numbers[0]);
                        }
                    }
                    'k' | // Line Position Backward
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
                            // Set Left and Right Margins
                            self.state = EngineState::Default;
                            let (start, end) = match self.parsed_numbers.len() {
                                2 => (self.parsed_numbers[0] - 1, self.parsed_numbers[1] - 1),
                                1 => (0, self.parsed_numbers[0] - 1),
                                0 => (0, buf.terminal_state.height),
                                _ => {
                                    return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                        self.current_escape_sequence.clone(),
                                    )));
                                }
                            };
                            if start > end {
                                // undocumented behavior but CSI 1; 0 s seems to turn off on some terminals.
                                buf.terminal_state.margins_left_right = None;
                            } else {
                                buf.terminal_state.margins_left_right = Some((start, end));
                            }
                        } else {
                            // Save Current Cursor Position
                            self.state = EngineState::Default;
                            self.saved_pos = caret.pos;
                        }
                    }
                    'u' => {
                        // Restore Saved Cursor Position
                        self.state = EngineState::Default;
                        caret.pos = self.saved_pos;
                    }

                    'd' => {
                        // Vertical Line Position Absolute
                        self.state = EngineState::Default;
                        let num = match self.parsed_numbers.first() {
                            Some(n) => n - 1,
                            _ => 0,
                        };
                        caret.pos.y = buf.get_first_visible_line() + num;
                        buf.terminal_state.limit_caret_pos(buf, caret);
                    }
                    'e' => {
                        // Vertical Line Position Relative
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
                        // Horizontal Line Position Relative
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
                        // Device Status Report
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
                    'X' => {
                        // Erase character
                        self.state = EngineState::Default;

                        if let Some(number) = self.parsed_numbers.first() {
                            caret.erase_charcter(buf, *number);
                        } else {
                            caret.erase_charcter(buf, 1);
                            if self.parsed_numbers.len() != 1 {
                                return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                    self.current_escape_sequence.clone(),
                                )));
                            }
                        }
                    }
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

                    'c' => {
                        // device attributes
                        self.state = EngineState::Default;
                        // respond with IcyTerm as ASCII followed by the package version.
                        return Ok(CallbackAction::SendString(format!(
                            "\x1b[=73;99;121;84;101;114;109;{};{};{}c",
                            env!("CARGO_PKG_VERSION_MAJOR"),
                            env!("CARGO_PKG_VERSION_MINOR"),
                            env!("CARGO_PKG_VERSION_PATCH")
                        )));
                    }
                    'r' => {
                        // Set Top and Bottom Margins
                        self.state = EngineState::Default;
                        let (start, end) = match self.parsed_numbers.len() {
                            2 => (self.parsed_numbers[0] - 1, self.parsed_numbers[1] - 1),
                            1 => (0, self.parsed_numbers[0] - 1),
                            0 => (0, buf.terminal_state.height),
                            _ => {
                                return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                                    self.current_escape_sequence.clone(),
                                )));
                            }
                        };

                        if start > end {
                            // undocumented behavior but CSI 1; 0 r seems to turn off on some terminals.
                            buf.terminal_state.margins_up_down = None;
                        } else {
                            caret.pos = buf.upper_left_position();
                            buf.terminal_state.margins_up_down = Some((start, end));
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
                                caret.attr.set_background(color);
                            }
                            Some(1) => {
                                caret.attr.set_foreground(color);
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
                        // repeat last char
                        self.state = EngineState::Default;
                        let num: i32 = if let Some(number) = self.parsed_numbers.first() {
                            *number
                        } else {
                            1
                        };
                        let mut ch = AttributedChar::new(self.last_char, caret.attr);
                        ch.set_font_page(self.current_font_page);
                        (0..num).for_each(|_| buf.print_char(caret, ch));
                    }
                    'g' => {
                        // clear tab stops
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
                        // next tab stop
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
                        // prev tab stop
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
                    caret.attr = TextAttribute::default();
                }
                LF => caret.lf(buf),
                FF => caret.ff(buf),
                CR => caret.cr(buf),
                BS => caret.bs(buf),
                BEL => return Ok(CallbackAction::Beep),
                '\x7F' => caret.del(buf),
                _ => {
                    self.last_char = unsafe { char::from_u32_unchecked(ch as u32) };
                    let mut ch = AttributedChar::new(self.last_char, caret.attr);
                    ch.set_font_page(self.current_font_page);
                    buf.print_char(caret, ch);
                }
            },
        }

        Ok(CallbackAction::None)
    }
}

impl Parser {
    fn parse_extended_colors(&mut self, buf: &mut Buffer, i: &mut usize) -> EngineResult<u32> {
        if *i + 1 >= self.parsed_numbers.len() {
            return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                self.current_escape_sequence.clone(),
            )));
        }
        match self.parsed_numbers.get(*i + 1).unwrap() {
            5 => {
                // ESC[38/48;5;⟨n⟩m Select fg/bg color from 256 color lookup
                if *i + 3 > self.parsed_numbers.len() {
                    return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                        self.current_escape_sequence.clone(),
                    )));
                }
                let color = self.parsed_numbers[*i + 2];
                *i += 3;
                if (0..=255).contains(&color) {
                    let color = buf.palette.insert_color(XTERM_256_PALETTE[color as usize]);
                    Ok(color)
                } else {
                    Err(Box::new(ParserError::UnsupportedEscapeSequence(
                        self.current_escape_sequence.clone(),
                    )))
                }
            }
            2 => {
                // ESC[38/48;2;⟨r⟩;⟨g⟩;⟨b⟩ m Select RGB fg/bg color
                if *i + 5 > self.parsed_numbers.len() {
                    return Err(Box::new(ParserError::UnsupportedEscapeSequence(
                        self.current_escape_sequence.clone(),
                    )));
                }
                let r = self.parsed_numbers[*i + 2];
                let g = self.parsed_numbers[*i + 3];
                let b = self.parsed_numbers[*i + 4];
                *i += 5;
                if (0..=255).contains(&r) && (0..=255).contains(&g) && (0..=255).contains(&b) {
                    let color = buf.palette.insert_color_rgb(r as u8, g as u8, b as u8);
                    Ok(color)
                } else {
                    Err(Box::new(ParserError::UnsupportedEscapeSequence(
                        self.current_escape_sequence.clone(),
                    )))
                }
            }
            _ => Err(Box::new(ParserError::UnsupportedEscapeSequence(
                self.current_escape_sequence.clone(),
            ))),
        }
    }

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

    fn invoke_macro(
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

    fn set_font_selection_success(&self, buf: &mut Buffer, caret: &Caret, slot: usize) {
        buf.terminal_state.font_selection_state = FontSelectionState::Success;

        if caret.attr.is_blinking() && caret.attr.is_bold() {
            buf.terminal_state.high_intensity_blink_attribute_font_slot = slot;
        } else if caret.attr.is_blinking() {
            buf.terminal_state.blink_attribute_font_slot = slot;
        } else if caret.attr.is_bold() {
            buf.terminal_state.high_intensity_attribute_font_slot = slot;
        } else {
            buf.terminal_state.normal_attribute_font_slot = slot;
        }
    }
}
