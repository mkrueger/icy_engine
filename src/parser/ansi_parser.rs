// Useful description: https://vt100.net/docs/vt510-rm/chapter4.html

use std::{cmp::{max, min}};

use crate::{Position, Buffer, TextAttribute, Caret, TerminalScrolling, OriginMode, AutoWrapMode, EngineResult, ParserError, BitFont, LF, FF, CR, BS, AttributedChar, MouseMode, Palette, Sixel, SixelReadStatus, XTERM_256_PALETTE};

use super::{BufferParser, AsciiParser};

pub enum SixelState {
    Read,
    ReadColor,
    ReadSize,
    Repeat,
    EndSequence
}

pub enum AnsiState {
    Default,
    GotEscape,
    ReadSequence,
    ReadCustomCommand,
    StartFontSelector,
    GotDCS,
    ReadSixel(SixelState)
}

pub struct AnsiParser {
    ascii_parser: AsciiParser,
    pub (crate) state: AnsiState,
    current_font_page: usize,
    saved_pos: Position,
    saved_cursor_opt: Option<Caret>,
    pub (crate) parsed_numbers: Vec<i32>,

    current_sixel: usize,
    sixel_cursor: Position,
    current_sixel_palette: Palette,

    current_sequence: String,
    current_sixel_color: i32,
}

const ANSI_CSI: char = '[';
const ANSI_ESC: char = '\x1B';

const COLOR_OFFSETS : [u8; 8] = [ 0, 4, 2, 6, 1, 5, 3, 7 ];

// TODO: Get missing fonts https://github.com/lattera/freebsd/tree/master/share/syscons/fonts
pub static ANSI_FONT_NAMES: [&str;43] = [
    "IBM VGA", // Codepage 437 English
    "IBM VGA 855", // Codepage 1251 Cyrillic
    "IBM VGA 866", // Maybe wrong: Russian koi8-r
    "IBM VGA 850", // ISO-8859-2 Central European
    "IBM VGA 775", // ISO-8859-4 Baltic wide
    "IBM VGA 866", // Codepage 866 (c) Russian
    "IBM VGA 857", // ISO-8859-9 Turkish
    "IBM VGA", // Unsupported:  haik8 codepage
    "IBM VGA 862", // ISO-8859-8 Hebrew
    "IBM VGA", // Unsupported: Ukrainian font koi8-u
    "IBM VGA", // Unsupported: ISO-8859-15 West European, (thin)
    "IBM VGA", // Unsupported: ISO-8859-4 Baltic (VGA 9bit mapped)
    "IBM VGA", // Unsupported: Russian koi8-r (b)
    "IBM VGA", // Unsupported: ISO-8859-4 Baltic wide
    "IBM VGA", // Unsupported: ISO-8859-5 Cyrillic
    "IBM VGA", // Unsupported: ARMSCII-8 Character set
    "IBM VGA", // Unsupported: ISO-8859-15 West European
    "IBM VGA 850", // Codepage 850 Multilingual Latin I, (thin)
    "IBM VGA 850", // Codepage 850 Multilingual Latin I
    "IBM VGA", // Unsupported: Codepage 885 Norwegian, (thin)
    "IBM VGA", // Unsupported: Codepage 1251 Cyrillic
    "IBM VGA", // Unsupported: ISO-8859-7 Greek
    "IBM VGA", // Unsupported: Russian koi8-r (c)
    "IBM VGA", // Unsupported: ISO-8859-4 Baltic
    "IBM VGA", // Unsupported: ISO-8859-1 West European
    "IBM VGA 866", // Codepage 866 Russian
    "IBM VGA", // Unsupported: Codepage 437 English, (thin)
    "IBM VGA", // Unsupported: Codepage 866 (b) Russian
    "IBM VGA", // Unsupported: Codepage 885 Norwegian
    "IBM VGA", // Unsupported: Ukrainian font cp866u
    "IBM VGA", // Unsupported: ISO-8859-1 West European, (thin)
    "IBM VGA", // Unsupported: Codepage 1131 Belarusian, (swiss)
    "C64 PETSCII shifted", // Commodore 64 (UPPER)
    "C64 PETSCII unshifted", // Commodore 64 (Lower)
    "C64 PETSCII shifted", // Commodore 128 (UPPER)
    "C64 PETSCII unshifted", // Commodore 128 (Lower)
    "Atari ATASCII", // Atari
    "Amiga P0T-NOoDLE", // P0T NOoDLE (Amiga)
    "Amiga mOsOul", // mO'sOul (Amiga)
    "Amiga MicroKnight+", // MicroKnight Plus (Amiga)
    "Amiga Topaz 1+", // Topaz Plus (Amiga)
    "Amiga MicroKnight", // MicroKnight (Amiga)
    "Amiga Topaz 1", // Topaz (Amiga)
];

impl AnsiParser {
    pub fn new() -> Self {
        AnsiParser {
            ascii_parser: AsciiParser::new(),
            current_font_page: 0,
            state: AnsiState::Default,
            saved_pos: Position::default(),
            parsed_numbers: Vec::new(),
            current_sequence: String::new(),
            saved_cursor_opt: None,
            current_sixel_palette: Palette::default(),
            current_sixel_color: 0,
            sixel_cursor: Position::default(),
            current_sixel: 0
        }
    }

    fn start_sequence(&mut self, buf: &mut Buffer, caret: &mut Caret, ch: char) -> EngineResult<Option<String>> {
        self.state = AnsiState::Default;
        match ch {
            ANSI_CSI => {
                self.current_sequence.push('[');
                self.state = AnsiState::ReadSequence;
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

            'P' => { // DCS
                self.state = AnsiState::GotDCS;
                Ok(None)
            }
            
            _ => {
                Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())))
            }
        }
    }

    fn parse_sixel(&mut self, buf: &mut Buffer, ch: char) -> EngineResult<()> {
        let sixel = &mut buf.layers[0].sixels[self.current_sixel];
        if ch < '?'  {
            sixel.read_status = SixelReadStatus::Error;
            return Err(Box::new(ParserError::InvalidSixelChar(ch)));
        }
        let mask = ch as u8 - b'?';

        sixel.len += 1;
        let fg_color = Some(self.current_sixel_palette.colors[(self.current_sixel_color as usize) % self.current_sixel_palette.colors.len()]);
        let offset = self.sixel_cursor.x as usize;
        if sixel.picture.len() <= offset {
            sixel.picture.resize(offset + 1, Vec::new());
        }
        let cur_line = &mut sixel.picture[offset];
        let line_offset = self.sixel_cursor.y as usize * 6;
        if cur_line.len() < line_offset + 6 {
            cur_line.resize(line_offset + 6, None);
        }
        
        for i in 0..6 {
            if mask & (1 << i) != 0 {  
                cur_line[line_offset + i] = fg_color;
            }
        }
        self.sixel_cursor.x += 1;
        sixel.read_status = SixelReadStatus::Position(self.sixel_cursor.x, self.sixel_cursor.y);
        Ok(())
    }

    fn parse_extended_colors(&mut self, buf: &mut Buffer, i: &mut usize) -> EngineResult<u32> {
        if *i + 1 >= self.parsed_numbers.len() {
            return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
        }
        match self.parsed_numbers[*i + 1] {
            5 => { // ESC[38/48;5;⟨n⟩m Select fg/bg color from 256 color lookup
                if *i + 3 > self.parsed_numbers.len() {
                    return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                }
                let color = self.parsed_numbers[*i + 2];
                *i += 3;
                if color >= 0 && color <= 255 {
                    let color = buf.palette.insert_color(XTERM_256_PALETTE[color as usize]);
                    Ok(color)
                } else {
                    Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())))
                }
            }
            2 => { // ESC[38/48;2;⟨r⟩;⟨g⟩;⟨b⟩ m Select RGB fg/bg color
                if *i + 5 > self.parsed_numbers.len() {
                    return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                }
                let r = self.parsed_numbers[*i + 2];
                let g = self.parsed_numbers[*i + 3];
                let b = self.parsed_numbers[*i + 4];
                *i += 5;
                if r >= 0 && r <= 255 && g >= 0 && g <= 255 && b >= 0 && b <= 255 {
                    let color = buf.palette.insert_color_rgb(r as u8, g as u8, b as u8);
                    Ok(color)
                } else { 
                    Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())))
                }
            }
            _ => Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())))
        }
    }
}

impl BufferParser for AnsiParser {
    fn from_unicode(&self, ch: char) -> char {
        self.ascii_parser.from_unicode(ch)
    }
    
    fn to_unicode(&self, ch: char) -> char {
        self.ascii_parser.to_unicode(ch)
    }

    fn print_char(&mut self, buf: &mut Buffer, caret: &mut Caret, ch: char) -> EngineResult<Option<String>> {
        match &self.state {
            AnsiState::GotEscape => return self.start_sequence(buf, caret, ch),
            AnsiState::StartFontSelector => {
                self.state = AnsiState::Default;
                match ch {
                    'D' => {
                        if self.parsed_numbers.len() != 2 {
                            self.current_sequence.push('D');
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                        }

                        if let Some(nr) = self.parsed_numbers.get(1) {
                            if *nr >= (ANSI_FONT_NAMES.len() as i32) {
                                return Err(Box::new(ParserError::UnsupportedFont(*nr)));
                            }
                            match BitFont::from_name(&ANSI_FONT_NAMES[*nr as usize]) {
                                Ok(font) => {
                                    if buf.font.name == font.name {
                                        self.current_font_page = 0;
                                    } else {
                                        for i in 0..buf.extended_fonts.len() {
                                            if buf.extended_fonts[i].name == font.name {
                                                self.current_font_page = i + 1;
                                                return Ok(None);
                                            }
                                        }
                                        buf.extended_fonts.push(font);
                                        self.current_font_page = buf.extended_fonts.len();
                                    }
                                } 
                                Err(err) => {
                                    return Err(err);
                                }
                            }
                        }
                    }
                    _ => { 
                        self.current_sequence.push(ch);
                        return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                    }
                }
            }
            AnsiState::GotDCS => {
                match ch {
                    'q' => {
                        let aspect_ratio = 
                            match self.parsed_numbers.get(0) {
                                Some(0) | Some(1) => 5,
                                Some(2) => 3,
                                Some(7) | Some(8) | Some(9) => 1,
                                _ => 2
                            };

                        let mut sixel = Sixel::new(caret.pos, aspect_ratio);
                        match self.parsed_numbers.get(0) {
                            Some(1) => sixel.background_color = Some(buf.palette.colors[caret.attr.get_background() as usize]),
                            _ =>  sixel.background_color = None
                        };

                        self.current_sixel = buf.layers[0].sixels.len();
                        buf.layers[0].sixels.push(sixel);
                        self.sixel_cursor = Position::default();
                        self.current_sixel_palette.clear();
                        self.state = AnsiState::ReadSixel(SixelState::Read);
                    }
                    _ => { 
                        if ('0'..='9').contains(&ch) {
                            let d = match self.parsed_numbers.pop() {
                                Some(number) => number,
                                _ => 0
                            };
                            self.parsed_numbers.push(d * 10 + ch as i32 - b'0' as i32);
                        } else if ch == ';' {
                            self.parsed_numbers.push(0);
                        } else {
                            self.state = AnsiState::Default;
                            self.current_sequence.push(ch);
                            buf.layers[0].sixels[self.current_sixel].read_status = SixelReadStatus::Error;
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                        }
                    }
                }
            }
            AnsiState::ReadSixel(state) => {
                match state {
                    SixelState::EndSequence => {
                        let layer = &mut buf.layers[0];
                        let sixel = &mut layer.sixels[self.current_sixel];
                        sixel.read_status = SixelReadStatus::Finished;

                        // remove sixels that are shadowed by the new one 
                        /*if self.current_sixel > 0 {
                            let cur_rect = sixel.get_rect();
                            let mut i = self.current_sixel - 1;
                            loop {
                                let other_rect = layer.sixels[i].get_rect();
                                if cur_rect.contains(other_rect) {
                                    layer.sixels.remove(i);
                                    println!("remove sixel! {} {:?}", layer.sixels.len(), layer.sixels[0].read_status);
                                }
                                if i == 0 {
                                    break;
                                }
                                i -= 1;
                            }
                        }*/
                        
                        if ch == '\\' {
                            self.state = AnsiState::Default;
                        } else {
                            return Err(Box::new(ParserError::UnexpectedSixelEnd(ch)));
                        }
                    }
                    SixelState::ReadColor => {
                        if ('0'..='9').contains(&ch) {
                            let d = match self.parsed_numbers.pop() {
                                Some(number) => number,
                                _ => 0
                            };
                            self.parsed_numbers.push(d * 10 + ch as i32 - b'0' as i32);
                        } else if ch == ';' {
                            self.parsed_numbers.push(0);
                        } else {
                            if let Some(color) = self.parsed_numbers.get(0) {
                                self.current_sixel_color = *color;
                            }
                            if self.parsed_numbers.len() > 1 {
                                if self.parsed_numbers.len() != 5 {
                                    self.state = AnsiState::Default;
                                    return Err(Box::new(ParserError::InvalidColorInSixelSequence));
                                }

                                match self.parsed_numbers[1] {
                                    2 => {
                                        self.current_sixel_palette.set_color_rgb(self.current_sixel_color as usize,
                                            (self.parsed_numbers[2] * 255 / 100) as u8,
                                            (self.parsed_numbers[3] * 255 / 100) as u8,
                                            (self.parsed_numbers[4] * 255 / 100) as u8,
                                        );
                                    }
                                    1 => {
                                        self.current_sixel_palette.set_color_hsl(self.current_sixel_color as usize,
                                            self.parsed_numbers[2] as f32 * 360.0 / (2.0 * std::f32::consts::PI),
                                            self.parsed_numbers[4] as f32 / 100.0, // sixel is hls
                                            self.parsed_numbers[3] as f32 / 100.0,
                                        );
                                    }
                                    n => {
                                        let sixel = &mut buf.layers[0].sixels[self.current_sixel];
                                        sixel.read_status = SixelReadStatus::Error;
                                        return Err(Box::new(ParserError::UnsupportedSixelColorformat(n)));
                                    }
                                }
                            }
                            self.read_sixel_data(buf, ch)?;
                        }
                    }
                    SixelState::ReadSize => {
                        if ('0'..='9').contains(&ch) {
                            let d = match self.parsed_numbers.pop() {
                                Some(number) => number,
                                _ => 0
                            };
                            self.parsed_numbers.push(d * 10 + ch as i32 - b'0' as i32);
                        } else if ch == ';' {
                            self.parsed_numbers.push(0);
                        } else {
                            let sixel = &mut buf.layers[0].sixels[self.current_sixel];
                            if self.parsed_numbers.len() != 4 {
                                self.state = AnsiState::ReadSixel(SixelState::Read);
                                sixel.read_status = SixelReadStatus::Error;
                                return Err(Box::new(ParserError::InvalidPictureSize));
                            }
                            unsafe {
                                let width = *self.parsed_numbers.get_unchecked(2);
                                let height = *self.parsed_numbers.get_unchecked(3);

                                if (sixel.picture.len() as i32) < width {
                                    sixel.picture.resize(width as usize, Vec::new())
                                }
                                for row in &mut sixel.picture {
                                    if (row.len() as i32) < height {
                                        row.resize(height as usize, None);
                                    }
                                }
                            }
                            self.read_sixel_data(buf, ch)?;
                        }
                    }

                    SixelState::Repeat => {
                        if ('0'..='9').contains(&ch) {
                            let d = match self.parsed_numbers.pop() {
                                Some(number) => number,
                                _ => 0
                            };
                            self.parsed_numbers.push(d * 10 + ch as i32 - '0' as i32);
                        } else {
                            if let Some(i) = self.parsed_numbers.get(0) {
                                for _ in 0..*i {
                                    self.parse_sixel(buf, ch)?;
                                }
                            } else {
                                self.state = AnsiState::Default;
                                let sixel = &mut buf.layers[0].sixels[self.current_sixel];
                                sixel.read_status = SixelReadStatus::Error;
                                return Err(Box::new(ParserError::NumberMissingInSixelRepeat));
                            }
                            self.state = AnsiState::ReadSixel(SixelState::Read);
                        }
                    }
                    SixelState::Read => {
                        self.read_sixel_data(buf, ch)?;
                    }
                }
            }
            AnsiState::ReadCustomCommand => {
                match ch {
                    'p' => { // [!p Soft Teminal Reset
                        self.state = AnsiState::Default;
                        buf.terminal_state.reset();
                    }
                    'l' => {
                        self.state = AnsiState::Default;
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
                            Some(33) => buf.terminal_state.set_use_ice_colors(false),
                            _ => { 
                                return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                            }
                        } 
                    }
                    'h' => {
                        self.state = AnsiState::Default;
                        if self.parsed_numbers.len() != 1 {
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                        }
                        match self.parsed_numbers.get(0) {
                            Some(4) => buf.terminal_state.scroll_state = TerminalScrolling::Smooth,
                            Some(6) => buf.terminal_state.origin_mode = OriginMode::UpperLeftCorner,
                            Some(7) => buf.terminal_state.auto_wrap_mode = AutoWrapMode::AutoWrap,
                            Some(25) => caret.is_visible = true,
                            Some(33) => buf.terminal_state.set_use_ice_colors(true),

                            // Mouse tracking see https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Normal-tracking-mode
                            Some(9) => buf.terminal_state.mouse_mode = MouseMode::X10,
                            Some(1000) => buf.terminal_state.mouse_mode = MouseMode::Default,
                            Some(1001) => buf.terminal_state.mouse_mode = MouseMode::Highlight,
                            Some(1002) => buf.terminal_state.mouse_mode = MouseMode::ButtonEvents,
                            Some(1003) => buf.terminal_state.mouse_mode = MouseMode::AnyEvents,

                            Some(1004) => buf.terminal_state.mouse_mode = MouseMode::FocusEvent,
                            Some(1007) => buf.terminal_state.mouse_mode = MouseMode::AlternateScroll,
                            Some(1005) => buf.terminal_state.mouse_mode = MouseMode::ExtendedMode,
                            Some(1006) => buf.terminal_state.mouse_mode = MouseMode::SGRExtendedMode,
                            Some(1015) => buf.terminal_state.mouse_mode = MouseMode::URXVTExtendedMode,
                            Some(1016) => buf.terminal_state.mouse_mode = MouseMode::PixelPosition,

                            Some(cmd) => { 
                                return Err(Box::new(ParserError::UnsupportedCustomCommand(*cmd)));
                            }
                            None => return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())))
                        } 
                    }
                    '0'..='9' => {
                        let d = match self.parsed_numbers.pop() {
                            Some(number) => number,
                            _ => 0
                        };
                        self.parsed_numbers.push(d * 10 + ch as i32 - b'0' as i32);
                    }
                    _ => {
                        self.state = AnsiState::Default;
                        // error in control sequence, terminate reading
                        return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                    }
                }
            }
            AnsiState::ReadSequence => {
                if let Some(ch) = char::from_u32(ch as u32) {
                    self.current_sequence.push(ch);
                } else {
                    return Err(Box::new(ParserError::InvalidChar('\0')));
                }
                
                match ch {
                    'm' => { // Select Graphic Rendition 
                        self.state = AnsiState::Default;
                        if self.parsed_numbers.len() == 0 {
                            caret.attr = TextAttribute::default(); // Reset or normal 
                        } 
                        let mut i = 0;
                        while i < self.parsed_numbers.len() {
                            let n = self.parsed_numbers[i];
                            match n {
                                0 => caret.attr = TextAttribute::default(), // Reset or normal 
                                1 => caret.attr.set_is_bold(true),
                                2 => { caret.attr.set_is_faint(true); },  
                                3 => { caret.attr.set_is_italic(true); },  
                                4 => caret.attr.set_is_underlined(true), 
                                5 | 6 => caret.attr.set_is_blinking(true),
                                7 => {
                                    let fg = caret.attr.get_foreground();
                                    caret.attr.set_foreground(caret.attr.get_background());
                                    caret.attr.set_background(fg);
                                }
                                8 => { caret.attr.set_is_concealed(true); },
                                9 => caret.attr.set_is_crossed_out(true),
                                10 => self.current_font_page = 0, // Primary (default) font 
                                11..=19 => { /* ignore alternate fonts for now */  } //return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone()))),
                                21 => { caret.attr.set_is_double_underlined(true) },
                                22 => { caret.attr.set_is_bold(false); caret.attr.set_is_faint(false) },
                                23 => caret.attr.set_is_italic(false),
                                24 => caret.attr.set_is_underlined(false),
                                25 => caret.attr.set_is_blinking(false),
                                27 => return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone()))),
                                28 => caret.attr.set_is_concealed(false),
                                29 => caret.attr.set_is_crossed_out(false),
                                // set foreaground color
                                30..=37 => caret.attr.set_foreground(COLOR_OFFSETS[n as usize - 30] as u32),
                                38 => {
                                    caret.attr.set_foreground(self.parse_extended_colors(buf, &mut i)?);
                                }
                                39 => caret.attr.set_foreground(7), // Set foreground color to default, ECMA-48 3rd 
                                // set background color
                                40..=47 => caret.attr.set_background(COLOR_OFFSETS[n as usize - 40] as u32),
                                48 => {
                                    caret.attr.set_background(self.parse_extended_colors(buf, &mut i)?);
                                }
                                49 => caret.attr.set_background(0), // Set background color to default, ECMA-48 3rd
                                _ => { 
                                    return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                                }
                            }
                            i += 1;
                        }
                    }
                    'H' | 'f' => { // Cursor Position + Horizontal Vertical Position ('f')
                        self.state = AnsiState::Default;
                        if !self.parsed_numbers.is_empty() {
                            if self.parsed_numbers[0] >= 0 { 
                                // always be in terminal mode for gotoxy
                                caret.pos.y =  buf.get_first_visible_line() + max(0, self.parsed_numbers[0] - 1);
                            }
                            if self.parsed_numbers.len() > 1 {
                                if self.parsed_numbers[1] >= 0 {
                                    caret.pos.x = max(0, self.parsed_numbers[1] - 1);
                                }
                            } else {
                                caret.pos.x = 0;
                            }
                        } else {
                            caret.pos = buf.upper_left_position();
                        }
                        buf.terminal_state.limit_caret_pos(buf, caret);
                    }
                    'C' => { // Cursor Forward 
                        self.state = AnsiState::Default;
                        if self.parsed_numbers.is_empty() {
                            caret.right(buf, 1);
                        } else {
                            caret.right(buf, self.parsed_numbers[0]);
                        }
                    }
                    'D' => { // Cursor Back 
                        self.state = AnsiState::Default;
                        if self.parsed_numbers.is_empty() {
                            caret.left(buf, 1);
                        } else {
                            caret.left(buf, self.parsed_numbers[0]);
                        }
                    }
                    'A' => { // Cursor Up 
                        self.state = AnsiState::Default;
                        if self.parsed_numbers.is_empty() {
                            caret.up(buf, 1);
                        } else {
                            caret.up(buf, self.parsed_numbers[0]);
                        }
                    }
                    'B' => { // Cursor Down 
                        self.state = AnsiState::Default;
                        if self.parsed_numbers.is_empty() {
                            caret.down(buf, 1);
                        } else {
                            caret.down(buf, self.parsed_numbers[0]);
                        }
                    }
                    's' => { // Save Current Cursor Position
                        self.state = AnsiState::Default;
                        self.saved_pos = caret.pos;
                    }
                    'u' => { // Restore Saved Cursor Position 
                        self.state = AnsiState::Default;
                        caret.pos = self.saved_pos;
                    }
                    
                    'd' => { // Vertical Line Position Absolute
                        self.state = AnsiState::Default;
                        let num  = match self.parsed_numbers.get(0) { 
                            Some(n) => n - 1,
                            _ => 0
                        };
                        caret.pos.y =  buf.get_first_visible_line() + num;
                        buf.terminal_state.limit_caret_pos(buf, caret);
                    }
                    'e' => { // Vertical Line Position Relative
                        self.state = AnsiState::Default;
                        let num  = match self.parsed_numbers.get(0) { 
                            Some(n) => *n,
                            _ => 1
                        };
                        caret.pos.y = buf.get_first_visible_line() + caret.pos.y + num;
                        buf.terminal_state.limit_caret_pos(buf, caret);
                    }
                    '\'' => { // Horizontal Line Position Absolute
                        self.state = AnsiState::Default;
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
                    }
                    'a' => { // Horizontal Line Position Relative
                        self.state = AnsiState::Default;
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
                    }
                    
                    'G' => { // Cursor Horizontal Absolute
                        self.state = AnsiState::Default;
                        let num = match self.parsed_numbers.get(0) { 
                            Some(n) => n - 1,
                            _ => 0
                        };
                        caret.pos.x = num;
                        buf.terminal_state.limit_caret_pos(buf, caret);
                    }
                    'E' => { // Cursor Next Line
                        self.state = AnsiState::Default;
                        let num  = match self.parsed_numbers.get(0) { 
                            Some(n) => *n,
                            _ => 1
                        };
                        caret.pos.y = buf.get_first_visible_line() + caret.pos.y + num;
                        caret.pos.x = 0;
                        buf.terminal_state.limit_caret_pos(buf, caret);
                    }
                    'F' => { // Cursor Previous Line
                        self.state = AnsiState::Default;
                        let num  = match self.parsed_numbers.get(0) { 
                            Some(n) => *n,
                            _ => 1
                        };
                        caret.pos.y = buf.get_first_visible_line() + caret.pos.y - num;
                        caret.pos.x = 0;
                        buf.terminal_state.limit_caret_pos(buf, caret);
                    }
                            
                    'n' => { // Device Status Report 
                        self.state = AnsiState::Default;
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
                        Insert Column 	  CSI Pn ' }
                        Delete Column 	  CSI Pn ' ~ 
                    */
                    
                    
                    'X' => { // Erase character
                        self.state = AnsiState::Default;

                        if let Some(number) = self.parsed_numbers.get(0) {
                            caret.erase_charcter(buf, *number);
                        } else {
                            caret.erase_charcter(buf, 1);
                            if self.parsed_numbers.len() != 1 {
                                return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                            }
                        }
                    }
                    '@' => { // Insert character
                        self.state = AnsiState::Default;

                        if let Some(number) = self.parsed_numbers.get(0) {
                            for _ in 0..*number {
                                caret.ins(buf);
                            }
                        } else {
                            caret.ins(buf);
                            if self.parsed_numbers.len() != 1 {
                                return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                            }
                        }
                    }
                    'M' => { // Delete line
                        self.state = AnsiState::Default;
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
                    }
                    
                    'P' => { // Delete character
                        self.state = AnsiState::Default;
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
                    }
    
                    'L' => { // Insert line 
                        self.state = AnsiState::Default;
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
                    }
                    
                    'J' => { // Erase in Display 
                        self.state = AnsiState::Default;
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
                    }
                    
                    '?' => { // read custom command
                        self.state = AnsiState::ReadCustomCommand;
                    }
    
                    'K' => { // Erase in line
                        self.state = AnsiState::Default;
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
                    }
                    
                    'c' => { // device attributes
                        self.state = AnsiState::Default;
                        return Ok(Some("\x1b[?1;0c".to_string()));
                    }
    
                    'r' => { // Set Top and Bottom Margins
                        self.state = AnsiState::Default;
                        if self.parsed_numbers.len() != 2 {
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                        }
                        let start = self.parsed_numbers[0] - 1;
                        let end = self.parsed_numbers[1] - 1;
    
                        if start > end {
                            // undocumented behavior but CSI 1; 0 r seems to turn off on some terminals.
                            buf.terminal_state.margins  = None;
                        } else {
                            caret.pos = buf.upper_left_position();
                            buf.terminal_state.margins  = Some((start, end));
                        }
                    }
                    'h' => {
                        self.state = AnsiState::Default;
                        if self.parsed_numbers.len() != 1 {
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                        }
                        match self.parsed_numbers.get(0) {
                            Some(4) => { caret.insert_mode = true; }
                            _ => return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())))
                        }
                    }
    
                    'l' => {
                        self.state = AnsiState::Default;
                        if self.parsed_numbers.len() != 1 {
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                        }
                        match self.parsed_numbers.get(0)  {
                            Some(4) => { caret.insert_mode = false; }
                            _ => return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())))
                        }
                    }
                    '~' => {
                        self.state = AnsiState::Default;
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
                    }
                    ' ' => {
                        self.state = AnsiState::StartFontSelector;
                    }
                    _ => {
                        if ('\x40'..='\x7E').contains(&ch) {
                            // unknown control sequence, terminate reading
                            self.state = AnsiState::Default;
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
                        } else {
                            self.state = AnsiState::Default;
                            // error in control sequence, terminate reading
                            return Err(Box::new(ParserError::UnsupportedEscapeSequence(self.current_sequence.clone())));
                        }
                    }
                }
            }
            AnsiState::Default => {
                match ch {
                    ANSI_ESC => {
                        self.current_sequence.clear();
                        self.current_sequence.push_str("<ESC>");
                        self.state = AnsiState::Default;
                        self.state = AnsiState::GotEscape;
                    }
                    '\x00' | '\u{00FF}' => {
                        caret.attr = TextAttribute::default();
                    }
                    LF => caret.lf(buf),
                    FF => caret.ff(buf),
                    CR => caret.cr(buf),
                    BS => caret.bs(buf),
                    '\x7F' => caret.del(buf),
                    _ => {
                        let mut ch = AttributedChar::new(char::from_u32(ch as u32).unwrap(), caret.attr);
                        ch.set_font_page(self.current_font_page);
                        buf.print_char(caret, ch);
                    }
                }
            }
        }

        Ok(None)
    }

}


impl AnsiParser {
    fn read_sixel_data(&mut self, buf: &mut Buffer, ch: char) -> EngineResult<()> {
        match ch {
            ANSI_ESC => {
                self.state = AnsiState::ReadSixel(SixelState::EndSequence)
            },

            '#' =>  {
                self.parsed_numbers.clear();
                self.state = AnsiState::ReadSixel(SixelState::ReadColor);
            },
            '!' =>  {
                self.parsed_numbers.clear();
                self.state = AnsiState::ReadSixel(SixelState::Repeat);
            },
            '-' =>  {
                self.sixel_cursor.x = 0;
                self.sixel_cursor.y += 1;
            },
            '$' =>  {
                self.sixel_cursor.x = 0;
            },
            '"' => {
                self.parsed_numbers.clear();
                self.state = AnsiState::ReadSixel(SixelState::ReadSize);
            }
            _ => {
                if ch > '\x7F' {
                    self.state = AnsiState::Default;
                }
                self.parse_sixel(buf, ch)?;
            }
        }
        Ok(())
    }
}