use std::path::Path;

use crate::ansi::constants::COLOR_OFFSETS;
use crate::ascii::CP437_TO_UNICODE;
use crate::TextAttribute;
use crate::{
    analyze_font_usage, parse_with_parser, parsers, Buffer, BufferFeatures, OutputFormat,
    Rectangle, TextPane, DOS_DEFAULT_PALETTE,
};

use super::SaveOptions;

#[derive(Default)]
pub(crate) struct Ansi {}

impl OutputFormat for Ansi {
    fn get_file_extension(&self) -> &str {
        "ans"
    }

    fn get_alt_extensions(&self) -> Vec<String> {
        vec!["ice".to_string(), "diz".to_string()]
    }

    fn get_name(&self) -> &str {
        "Ansi"
    }

    fn analyze_features(&self, _features: &BufferFeatures) -> String {
        String::new()
    }

    fn to_bytes(&self, buf: &crate::Buffer, options: &SaveOptions) -> anyhow::Result<Vec<u8>> {
        let mut result = Vec::new();

        match options.screen_preparation {
            super::ScreenPreperation::None => {}
            super::ScreenPreperation::ClearScreen => {
                result.extend_from_slice(b"\x1b[2J");
            }
            super::ScreenPreperation::Home => {
                result.extend_from_slice(b"\x1b[1;1H");
            }
        }
        let mut gen = StringGenerator::new(options.clone());
        gen.generate(buf, buf);
        gen.add_sixels(buf);
        result.extend(gen.get_data());

        if options.save_sauce {
            buf.write_sauce_info(crate::SauceFileType::Ansi, &mut result)?;
        }
        Ok(result)
    }

    fn load_buffer(
        &self,
        file_name: &Path,
        data: &[u8],
        sauce_opt: Option<crate::SauceData>,
    ) -> anyhow::Result<crate::Buffer> {
        let mut result = Buffer::new((80, 25));
        result.is_terminal_buffer = true;
        result.file_name = Some(file_name.into());
        result.set_sauce(sauce_opt, true);
        let mut parser = parsers::ansi::Parser::default();
        parser.bs_is_ctrl_char = false;
        let (text, is_unicode) = crate::convert_ansi_to_utf8(data);
        if is_unicode {
            result.buffer_type = crate::BufferType::Unicode;
        }
        parse_with_parser(&mut result, &mut parser, &text, true)?;
        Ok(result)
    }
}

struct CharCell {
    ch: char,
    sgr: Vec<u8>,
    sgr_tc: Vec<u8>,
    font_page: usize,
}

pub struct StringGenerator {
    output: Vec<u8>,
    options: SaveOptions,
    last_line_break: usize,
    max_output_line_length: usize,
}

#[derive(Debug)]
struct AnsiState {
    pub is_bold: bool,
    pub is_blink: bool,

    pub fg: u32,
    pub bg: u32,
}

impl StringGenerator {
    pub fn new(options: SaveOptions) -> Self {
        let max_output_line_length = options.output_line_length.unwrap_or(usize::MAX);
        let mut output = Vec::new();

        if options.modern_terminal_output {
            // write UTF-8 BOM as unicode indicator.
            output.extend([0xEF, 0xBB, 0xBF]);
        }

        Self {
            output,
            options,
            last_line_break: 0,
            max_output_line_length,
        }
    }

    fn get_color(
        buf: &Buffer,
        attr: TextAttribute,
        mut state: AnsiState,
    ) -> (AnsiState, Vec<u8>, Vec<u8>) {
        let mut sgr = Vec::new();
        let mut sgr_tc = Vec::new();

        let fg = attr.get_foreground();
        let cur_fore_color = buf.palette.get_color(fg as usize);
        let cur_fore_rgb = cur_fore_color.get_rgb();

        let bg = attr.get_background();
        let cur_back_color = buf.palette.get_color(bg as usize);
        let cur_back_rgb = cur_back_color.get_rgb();

        let mut fore_idx = DOS_DEFAULT_PALETTE
            .iter()
            .position(|c| c.get_rgb() == cur_fore_rgb);
        let mut back_idx = DOS_DEFAULT_PALETTE
            .iter()
            .position(|c| c.get_rgb() == cur_back_rgb);

        let mut is_bold = state.is_bold;
        let mut is_blink = state.is_blink;

        if let Some(idx) = fore_idx {
            if idx < 8 {
                is_bold = false;
            } else if idx > 7 && idx < 16 {
                is_bold = true;
                fore_idx = Some(idx - 8);
            }
        }

        if let Some(idx) = back_idx {
            if idx < 8 {
                is_blink = false;
            } else if idx > 7 && idx < 16 {
                is_blink = true;
                back_idx = Some(idx - 8);
            }
        } else {
            is_blink = attr.is_blinking();
        }

        if fore_idx.is_some() && !is_bold && state.is_bold
            || back_idx.is_some() && !is_blink && state.is_blink
        {
            sgr.push(0);
            state.fg = 7;
            state.is_bold = false;
            state.bg = 0;
            state.is_blink = false;
        }

        if is_bold && !state.is_bold {
            sgr.push(1);
        }
        if fg != state.fg {
            if let Some(fg_idx) = fore_idx {
                sgr.push(COLOR_OFFSETS[fg_idx] + 30);
            } else {
                sgr_tc.push(1);
                sgr_tc.push(cur_fore_rgb.0);
                sgr_tc.push(cur_fore_rgb.1);
                sgr_tc.push(cur_fore_rgb.2);
            }
        }

        if is_blink && !state.is_blink {
            sgr.push(5);
        }
        if bg != state.bg {
            if let Some(bg_idx) = back_idx {
                sgr.push(COLOR_OFFSETS[bg_idx] + 40);
            } else {
                sgr_tc.push(0);
                sgr_tc.push(cur_back_rgb.0);
                sgr_tc.push(cur_back_rgb.1);
                sgr_tc.push(cur_back_rgb.2);
            }
        }

        (
            AnsiState {
                is_bold,
                is_blink,
                fg,
                bg,
            },
            sgr,
            sgr_tc,
        )
    }

    fn generate_cells<T: TextPane>(buf: &Buffer, layer: &T, area: Rectangle) -> Vec<Vec<CharCell>> {
        let mut result = Vec::new();
        let mut state = AnsiState {
            is_bold: false,
            is_blink: false,
            fg: 7,
            bg: 0,
        };
        for y in area.y_range() {
            let mut line = Vec::new();
            for x in area.x_range() {
                let ch = layer.get_char((x, y));

                let (new_state, sgr, sgr_tc) = StringGenerator::get_color(buf, ch.attribute, state);
                state = new_state;

                line.push(CharCell {
                    ch: ch.ch,
                    sgr,
                    sgr_tc,
                    font_page: ch.get_font_page(),
                });
            }

            result.push(line);
        }
        result
    }

    pub fn screen_prep(&mut self, buf: &Buffer) {
        if buf.ice_mode.has_high_bg_colors() {
            self.push_result(&mut b"\x1b[?33h".to_vec());
        }
    }

    pub fn screen_end(&mut self, buf: &Buffer) {
        if buf.ice_mode.has_high_bg_colors() {
            self.push_result(&mut b"\x1b[?33l".to_vec());
        }
    }

    /// .
    ///
    /// # Panics
    ///
    /// Panics if .
    pub fn generate<T: TextPane>(&mut self, buf: &Buffer, layer: &T) {
        let mut result = Vec::new();

        let used_fonts = analyze_font_usage(buf);
        for font_slot in used_fonts {
            if font_slot >= 100 {
                if let Some(font) = buf.get_font(font_slot) {
                    result.extend_from_slice(font.encode_as_ansi(font_slot).as_bytes());
                }
            }
        }
        let cells = StringGenerator::generate_cells(buf, layer, layer.get_rectangle());
        let mut cur_font_page = 0;

        for (y, line) in cells.iter().enumerate() {
            let mut x = 0;
            if self.options.longer_terminal_output && y > 0 {
                result.extend_from_slice(b"\x1b[");
                result.extend_from_slice(y.to_string().as_bytes());
                result.push(b'H');
            }

            let len = if self.options.compress {
                let mut last = line.len() as i32 - 1;
                while last > 0 {
                    if line[last as usize].ch != ' '
                        && line[last as usize].ch != 0xFF as char
                        && line[last as usize].ch != 0 as char
                    {
                        break;
                    }
                    if !line[last as usize].sgr.is_empty() || !line[last as usize].sgr_tc.is_empty()
                    {
                        break;
                    }
                    last -= 1;
                }
                let cur_len = last as usize + 1;
                if cur_len >= line.len() - 1 {
                    // don't compress if we have only one char, since eol are 2 chars
                    line.len()
                } else {
                    cur_len
                }
            } else {
                line.len()
            };

            while x < len {
                let cell = &line[x];
                if cur_font_page != cell.font_page {
                    cur_font_page = cell.font_page;
                    result.extend_from_slice(b"\x1b[0;");
                    result.extend_from_slice(cur_font_page.to_string().as_bytes());
                    result.extend_from_slice(b" D");
                    self.push_result(&mut result);
                }

                if !cell.sgr.is_empty() {
                    result.extend_from_slice(b"\x1b[");
                    for i in 0..cell.sgr.len() - 1 {
                        result.extend_from_slice(cell.sgr[i].to_string().as_bytes());
                        result.push(b';');
                    }
                    result.extend_from_slice(cell.sgr.last().unwrap().to_string().as_bytes());
                    result.push(b'm');
                    self.push_result(&mut result);
                }
                let mut idx = 0;
                while idx < cell.sgr_tc.len() {
                    result.extend_from_slice(b"\x1b[");
                    for i in 0..3 {
                        result.extend_from_slice(cell.sgr_tc[idx + i].to_string().as_bytes());
                        result.push(b';');
                    }
                    result.extend_from_slice(cell.sgr_tc[idx + 3].to_string().as_bytes());
                    result.push(b't');
                    self.push_result(&mut result);
                    idx += 4;
                }

                let cell_char = if self.options.modern_terminal_output {
                    if cell.ch == '\0' {
                        vec![b' ']
                    } else {
                        let uni_ch = CP437_TO_UNICODE[cell.ch as usize].to_string();
                        uni_ch.as_bytes().to_vec()
                    }
                } else if StringGenerator::CONTROL_CHARS.contains(cell.ch) {
                    match self.options.control_char_handling {
                        crate::ControlCharHandling::Ignore => {
                            vec![cell.ch as u8]
                        }
                        crate::ControlCharHandling::IcyTerm => {
                            vec![b'\x1B', cell.ch as u8]
                        }
                        crate::ControlCharHandling::FilterOut => {
                            vec![b' ']
                        }
                    }
                } else {
                    vec![cell.ch as u8]
                };

                if self.options.compress {
                    let mut rle = x + 1;
                    while rle < len {
                        if line[rle].ch != line[x].ch
                            || !line[rle].sgr.is_empty()
                            || !line[rle].sgr_tc.is_empty()
                            || line[rle].font_page != line[x].font_page
                        {
                            break;
                        }
                        rle += 1;
                    }
                    // rle is always >= x + 1 but "x - 1" may overflow.
                    rle -= 1;
                    rle -= x;
                    if rle > 4 {
                        if line[x].ch == ' ' {
                            let fmt = &format!("\x1B[{}C", rle + 1);
                            let output = fmt.as_bytes();
                            if output.len() < rle {
                                self.push_result(&mut result);
                                result.extend_from_slice(output);
                                self.push_result(&mut result);
                                x += rle + 1;
                                continue;
                            }
                        }
                        let fmt = &format!("\x1B[{rle}b");
                        let output = fmt.as_bytes();
                        if output.len() < rle {
                            self.push_result(&mut result);
                            result.extend_from_slice(&cell_char);
                            result.extend_from_slice(output);
                            self.push_result(&mut result);
                            x += rle + 1;
                            continue;
                        }
                    }
                }

                result.extend_from_slice(&cell_char);
                self.push_result(&mut result);

                x += 1;
            }

            if !self.options.longer_terminal_output {
                if self.options.modern_terminal_output {
                    result.extend_from_slice(b"\x1b[0m");
                    result.push(10);
                    self.last_line_break = result.len();
                } else if x < layer.get_width() as usize && y + 1 < layer.get_height() as usize {
                    result.push(13);
                    result.push(10);
                    self.last_line_break = result.len();
                }
            }
        }
    }

    const CONTROL_CHARS: &str = "\x1b\x07\x08\x09\x0C\x7F";

    pub fn add_sixels(&mut self, buf: &Buffer) {
        for layer in &buf.layers {
            for sixel in &layer.sixels {
                match icy_sixel::sixel_string(
                    &sixel.picture_data,
                    sixel.get_width(),
                    sixel.get_height(),
                    icy_sixel::PixelFormat::RGBA8888,
                    icy_sixel::DiffusionMethod::None,
                    icy_sixel::MethodForLargest::Auto,
                    icy_sixel::MethodForRep::Auto,
                    icy_sixel::Quality::AUTO,
                ) {
                    Err(err) => log::error!("{err}"),
                    Ok(data) => {
                        let p = layer.get_offset() + sixel.position;
                        self.output
                            .extend(format!("\x1b[{};{}H", p.y + 1, p.x + 1).as_bytes());
                        self.output.extend(data.as_bytes());
                    }
                }
            }
        }
    }

    pub fn get_data(&self) -> &[u8] {
        &self.output
    }

    fn push_result(&mut self, result: &mut Vec<u8>) {
        if self.output.len() + result.len() - self.last_line_break > self.max_output_line_length {
            self.output.extend_from_slice(b"\x1b[s");
            self.output.push(13);
            self.output.push(10);
            self.last_line_break = self.output.len();
            self.output.extend_from_slice(b"\x1b[u");
        }
        self.output.append(result);
        result.clear();
    }
}

pub fn get_save_sauce_default_ans(buf: &Buffer) -> (bool, String) {
    if buf.get_width() != 80 {
        return (true, "width != 80".to_string());
    }

    if buf.has_sauce() {
        return (true, String::new());
    }

    (false, String::new())
}
