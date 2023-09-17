use std::path::Path;

use crate::ascii::CP437_TO_UNICODE;
use crate::{
    parse_with_parser, parsers, Buffer, BufferFeatures, OutputFormat, TextPane, DOS_DEFAULT_PALETTE,
};
use crate::{Position, TextAttribute};

use super::SaveOptions;

const FG_TABLE: [&[u8; 2]; 8] = [b"30", b"34", b"32", b"36", b"31", b"35", b"33", b"37"];
const BG_TABLE: [&[u8; 2]; 8] = [b"40", b"44", b"42", b"46", b"41", b"45", b"43", b"47"];

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
        if let Some(sauce) = sauce_opt {
            result.set_sauce(sauce);
        }
        let mut parser = parsers::ansi::Parser::default();
        parser.bs_is_ctrl_char = false;

        parse_with_parser(&mut result, &mut parser, data, true)?;
        Ok(result)
    }
}

pub struct StringGenerator {
    output: Vec<u8>,
    options: SaveOptions,
    last_line_break: usize,
    max_output_line_length: usize,
}

impl StringGenerator {
    pub fn new(options: SaveOptions) -> Self {
        let max_output_line_length = options.output_line_length.unwrap_or(usize::MAX);
        Self {
            output: Vec::new(),
            options,
            last_line_break: 0,
            max_output_line_length,
        }
    }

    pub fn generate<T: TextPane>(&mut self, buf: &Buffer, layer: &T) {
        let mut result = Vec::new();
        let mut last_attr = TextAttribute::default();
        let mut last_fore_color = DOS_DEFAULT_PALETTE[7].clone();
        let mut last_back_color = DOS_DEFAULT_PALETTE[0].clone();

        let mut pos = Position::default();
        let height = layer.get_line_count();
        let mut first_char = true;
        let mut cur_font_page = 0;
        self.last_line_break = 0;

        while pos.y < height {
            let mut line_length =
                if self.options.modern_terminal_output || self.options.preserve_invisible_chars {
                    layer.get_width()
                } else {
                    layer.get_line_length(pos.y)
                };

            while line_length > 0
                && !layer
                    .get_char(Position::new(line_length - 1, pos.y))
                    .is_visible()
            {
                line_length -= 1;
            }

            while pos.x < line_length {
                let mut space_count = 0;
                let mut ch = layer.get_char(pos);
                // doesn't work well with unix terminal - background color needs to be painted.
                if !self.options.modern_terminal_output && !self.options.preserve_invisible_chars {
                    while (ch.ch == ' ' || ch.ch == '\0')
                        && ch.attribute.get_background() == 0
                        && pos.x < line_length
                    {
                        space_count += 1;
                        pos.x += 1;
                        ch = layer.get_char(pos);
                    }
                }
                let mut cur_attr = ch.attribute;
                // optimize color output for empty space lines.
                let cur_bg_color = buf.palette.get_color(cur_attr.get_background() as usize);
                if space_count > 0 && last_back_color == cur_bg_color {
                    cur_attr = ch.attribute;
                }

                if cur_font_page != cur_attr.get_font_page() {
                    cur_font_page = cur_attr.get_font_page();
                    result.extend_from_slice(b"\x1b[0;");
                    push_int(&mut result, cur_font_page);
                    result.extend_from_slice(b" D");
                    self.push_result(&mut result);
                }

                if last_attr != cur_attr || first_char {
                    if self.options.modern_terminal_output {
                        let cur_fore_color =
                            buf.palette.get_color(cur_attr.get_foreground() as usize);
                        if last_fore_color != cur_fore_color || first_char {
                            last_fore_color = cur_fore_color;
                            result.extend_from_slice(b"\x1b[38;2;");
                            let (r, g, b) = last_fore_color.clone().get_rgb();
                            result.extend_from_slice(r.to_string().as_bytes());
                            result.push(b';');
                            result.extend_from_slice(g.to_string().as_bytes());
                            result.push(b';');
                            result.extend_from_slice(b.to_string().as_bytes());
                            result.push(b'm');
                            self.push_result(&mut result);
                        }
                        let cur_back_color =
                            buf.palette.get_color(cur_attr.get_background() as usize);
                        if last_back_color != cur_back_color || first_char {
                            last_back_color = cur_back_color;
                            result.extend_from_slice(b"\x1b[48;2;");
                            let (r, g, b) = last_back_color.clone().get_rgb();
                            result.extend_from_slice(r.to_string().as_bytes());
                            result.push(b';');
                            result.extend_from_slice(g.to_string().as_bytes());
                            result.push(b';');
                            result.extend_from_slice(b.to_string().as_bytes());
                            result.push(b'm');
                            self.push_result(&mut result);
                        }
                    } else {
                        let mut write_24bit_fore_color = false;
                        let mut write_24bit_back_color = false;

                        let mut wrote_part = false;
                        let mut color_result = Vec::new();
                        // handle bold change
                        if (!last_attr.is_bold() || first_char) && cur_attr.is_bold() {
                            // if blinking is turned off "0;" will be written which would reset the bold state here
                            // bold state is set again after blink reset.
                            if (!last_attr.is_blinking() && !first_char) || cur_attr.is_blinking() {
                                color_result.push(b'1');
                                wrote_part = true;
                            }
                        } else if (last_attr.is_bold() || first_char) && !cur_attr.is_bold() {
                            color_result.push(b'0');
                            last_attr = TextAttribute::default();
                            last_fore_color = DOS_DEFAULT_PALETTE[7].clone();
                            last_back_color = DOS_DEFAULT_PALETTE[0].clone();

                            first_char = false; // attribute set.
                            wrote_part = true;
                        }

                        // handle blink change
                        if (!last_attr.is_blinking() || first_char) && cur_attr.is_blinking() {
                            if wrote_part {
                                color_result.push(b';');
                            }
                            color_result.push(b'5');
                            wrote_part = true;
                        } else if (last_attr.is_blinking() || first_char) && !cur_attr.is_blinking()
                        {
                            if wrote_part {
                                color_result.push(b';');
                            }
                            color_result.push(b'0');
                            if cur_attr.is_bold() || first_char {
                                color_result.extend_from_slice(b";1");
                            }
                            last_attr = TextAttribute::default();
                            last_fore_color = DOS_DEFAULT_PALETTE[7].clone();
                            last_back_color = DOS_DEFAULT_PALETTE[0].clone();

                            wrote_part = true;
                        }

                        if !last_attr.is_faint() && cur_attr.is_faint() {
                            if wrote_part {
                                color_result.push(b';');
                            }
                            color_result.push(b'2');
                            wrote_part = true;
                        } else if last_attr.is_faint() && !cur_attr.is_faint() {
                            if wrote_part {
                                color_result.push(b';');
                            }
                            color_result.extend_from_slice(b"22");
                            wrote_part = true;
                        }

                        if !last_attr.is_italic() && cur_attr.is_italic() {
                            if wrote_part {
                                color_result.push(b';');
                            }
                            color_result.push(b'3');
                            wrote_part = true;
                        } else if last_attr.is_italic() && !cur_attr.is_italic() {
                            if wrote_part {
                                color_result.push(b';');
                            }
                            color_result.extend_from_slice(b"23");
                            wrote_part = true;
                        }

                        if !last_attr.is_underlined() && cur_attr.is_underlined() {
                            if wrote_part {
                                color_result.push(b';');
                            }
                            color_result.push(b'4');
                            wrote_part = true;
                        } else if last_attr.is_underlined() && !cur_attr.is_underlined() {
                            if wrote_part {
                                color_result.push(b';');
                            }
                            color_result.extend_from_slice(b"24");
                            wrote_part = true;
                        }

                        if !last_attr.is_crossed_out() && cur_attr.is_crossed_out() {
                            if wrote_part {
                                color_result.push(b';');
                            }
                            color_result.push(b'9');
                            wrote_part = true;
                        } else if last_attr.is_crossed_out() && !cur_attr.is_crossed_out() {
                            if wrote_part {
                                color_result.push(b';');
                            }
                            color_result.extend_from_slice(b"29");
                            wrote_part = true;
                        }

                        if !last_attr.is_double_underlined() && cur_attr.is_double_underlined() {
                            if wrote_part {
                                color_result.push(b';');
                            }
                            color_result.extend_from_slice(b"21");
                            wrote_part = true;
                        } else if last_attr.is_double_underlined()
                            && !cur_attr.is_double_underlined()
                        {
                            if wrote_part {
                                color_result.push(b';');
                            }
                            color_result.push(b'0');
                            if cur_attr.is_bold() || first_char {
                                color_result.extend_from_slice(b";1");
                            }
                            //  last_attr = TextAttribute::default();
                            last_fore_color = DOS_DEFAULT_PALETTE[7].clone();
                            last_back_color = DOS_DEFAULT_PALETTE[0].clone();
                            wrote_part = true;
                        }

                        // color changes
                        let cur_fore_color =
                            buf.palette.get_color(cur_attr.get_foreground() as usize);
                        if last_fore_color != cur_fore_color {
                            last_fore_color = cur_fore_color.clone();
                            let fg = cur_attr.get_foreground() as usize;
                            if let Some(col) = get_standard_color(buf, fg) {
                                if wrote_part {
                                    color_result.push(b';');
                                }
                                color_result.extend_from_slice(FG_TABLE[col]);
                                wrote_part = true;
                            } else if let Some(col) = get_extended_color(buf, fg) {
                                if wrote_part {
                                    color_result.push(b';');
                                }
                                color_result.extend_from_slice(b"38;5;");
                                color_result.extend_from_slice(col.to_string().as_bytes());
                                wrote_part = true;
                            } else {
                                write_24bit_fore_color = true;
                            }
                        }
                        let cur_back_color =
                            buf.palette.get_color(cur_attr.get_background() as usize);
                        if last_back_color != cur_back_color {
                            last_back_color = cur_back_color.clone();
                            let bg = cur_attr.get_background() as usize;
                            if let Some(col) = get_standard_color(buf, bg) {
                                if wrote_part {
                                    color_result.push(b';');
                                }
                                color_result.extend_from_slice(BG_TABLE[col]);
                            } else if let Some(col) = get_extended_color(buf, bg) {
                                if wrote_part {
                                    color_result.push(b';');
                                }
                                color_result.extend_from_slice(b"48;5;");
                                color_result.extend_from_slice(col.to_string().as_bytes());
                            } else {
                                write_24bit_back_color = true;
                            }
                        }
                        if !color_result.is_empty() {
                            result.extend_from_slice(b"\x1b[");
                            result.extend_from_slice(&color_result);
                            result.push(b'm');
                        }
                        self.push_result(&mut result);

                        if write_24bit_fore_color {
                            result.extend_from_slice(b"\x1b[1;");
                            let (r, g, b) = cur_fore_color.get_rgb();
                            result.extend_from_slice(r.to_string().as_bytes());
                            result.push(b';');
                            result.extend_from_slice(g.to_string().as_bytes());
                            result.push(b';');
                            result.extend_from_slice(b.to_string().as_bytes());
                            result.push(b't');
                            self.push_result(&mut result);
                        }

                        if write_24bit_back_color {
                            result.extend_from_slice(b"\x1b[0;");
                            let (r, g, b) = cur_back_color.get_rgb();
                            result.extend_from_slice(r.to_string().as_bytes());
                            result.push(b';');
                            result.extend_from_slice(g.to_string().as_bytes());
                            result.push(b';');
                            result.extend_from_slice(b.to_string().as_bytes());
                            result.push(b't');
                            self.push_result(&mut result);
                        }
                    }
                    last_attr = cur_attr;
                }

                first_char = false;

                if space_count > 0 {
                    if space_count < 5 {
                        result.resize(result.len() + space_count, b' ');
                    } else {
                        result.extend_from_slice(b"\x1b[");
                        push_int(&mut result, space_count);
                        result.push(b'C');
                        self.push_result(&mut result);
                    }
                    continue;
                }
                if self.options.modern_terminal_output {
                    if ch.ch == '\0' {
                        result.push(b' ');
                    } else {
                        let uni_ch = CP437_TO_UNICODE[ch.ch as usize].to_string();
                        result.extend(uni_ch.as_bytes());
                    }
                } else if !ch.is_visible() && self.options.preserve_invisible_chars {
                    result.extend_from_slice(b"\x1b[C");
                } else if ch.ch == '\x1b' {
                    result.extend(b"\x1b\x1b");
                } else if ch.ch == '\x07' {
                    result.extend(b"\x1b\x07");
                } else if ch.ch == '\x08' {
                    result.extend(b"\x1b\x08");
                } else if ch.ch == '\x0C' {
                    result.extend(b"\x1b\x0C");
                } else if ch.ch == '\x7F' {
                    result.extend(b"\x1b\x7F");
                } else {
                    result.push(ch.ch as u8);
                }
                self.push_result(&mut result);
                pos.x += 1;
            }
            // do not end with eol except for terminal support.
            self.last_line_break = self.output.len();
            if self.options.modern_terminal_output {
                result.extend_from_slice(b"\x1b[0m");
                result.push(10);
                first_char = true;
            } else if pos.x < layer.get_width() && pos.y + 1 < height {
                result.push(13);
                result.push(10);
            }
            self.push_result(&mut result);
            self.last_line_break = self.output.len();

            pos.x = 0;
            pos.y += 1;
        }
        /*
        for u in &self.output {
            if *u == 27 {
                print!("\\x1B");
            } else {
                print!("{}", unsafe { char::from_u32_unchecked(*u as u32)});
            }
        }
        println!();*/
    }

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
                    Err(err) => eprintln!("{err}"),
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

fn get_extended_color(buf: &Buffer, color: usize) -> Option<usize> {
    let color = buf.palette.get_color(color);
    (0..crate::XTERM_256_PALETTE.len()).find(|&i| color == crate::XTERM_256_PALETTE[i].1)
}

fn get_standard_color(buf: &Buffer, color: usize) -> Option<usize> {
    let color = buf.palette.get_color(color);
    (0..8).find(|&i| color == crate::DOS_DEFAULT_PALETTE[i])
}

fn push_int(result: &mut Vec<u8>, number: usize) {
    result.extend_from_slice(number.to_string().as_bytes());
}

pub fn get_save_sauce_default_ans(buf: &Buffer) -> (bool, String) {
    if buf.get_width() != 80 {
        return (true, "width != 80".to_string());
    }

    if buf.sauce_data.is_some() {
        return (true, String::new());
    }

    (false, String::new())
}
