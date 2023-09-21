use std::{io, path::Path};

use super::{SaveOptions, TextAttribute};
use crate::{
    AttributedChar, Buffer, BufferFeatures, BufferType, EngineResult, LoadingError, OutputFormat,
    Position, TextPane,
};

// http://fileformats.archiveteam.org/wiki/TUNDRA
// ANSI code for 24 bit: ESC[(0|1);R;G;Bt
// 0 for background
// 1 for foreground

const TUNDRA_VER: u8 = 24;
const TUNDRA_HEADER: &[u8] = b"TUNDRA24";

const TUNDRA_POSITION: u8 = 1;
const TUNDRA_COLOR_FOREGROUND: u8 = 2;
const TUNDRA_COLOR_BACKGROUND: u8 = 4;

#[derive(Default)]
pub(super) struct TundraDraw {}

impl OutputFormat for TundraDraw {
    fn get_file_extension(&self) -> &str {
        "tnd"
    }

    fn get_name(&self) -> &str {
        "Tundra Draw"
    }

    fn analyze_features(&self, _features: &BufferFeatures) -> String {
        String::new()
    }

    fn to_bytes(&self, buf: &crate::Buffer, options: &SaveOptions) -> EngineResult<Vec<u8>> {
        let mut result: Vec<u8> = vec![TUNDRA_VER]; // version
        result.extend(TUNDRA_HEADER);
        let mut attr = TextAttribute::from_u8(0, buf.ice_mode);
        let mut skip_pos = None;
        for y in 0..buf.get_line_count() {
            for x in 0..buf.get_width() {
                let pos = Position::new(x, y);
                let ch = buf.get_char(pos);
                if !ch.is_visible() {
                    if skip_pos.is_none() {
                        skip_pos = Some(pos);
                    }
                    continue;
                }
                if ch.is_transparent() && attr.get_background() == 0 {
                    if skip_pos.is_none() {
                        skip_pos = Some(pos);
                    }
                    continue;
                }

                if let Some(pos2) = skip_pos {
                    let skip_len =
                        (pos.x + pos.y * buf.get_width()) - (pos2.x + pos2.y * buf.get_width());
                    if skip_len <= TND_GOTO_BLOCK_LEN {
                        result.resize(result.len() + skip_len as usize, 0);
                    } else {
                        result.push(1);
                        result.extend(i32::to_be_bytes(pos.y));
                        result.extend(i32::to_be_bytes(pos.x));
                    }
                    skip_pos = None;
                }
                if attr != ch.attribute {
                    let mut cmd = 0;
                    if attr.get_foreground() != ch.attribute.get_foreground() {
                        cmd |= TUNDRA_COLOR_FOREGROUND;
                    }
                    if attr.get_background() != ch.attribute.get_background() {
                        cmd |= TUNDRA_COLOR_BACKGROUND;
                    }

                    result.push(cmd);
                    result.push(ch.ch as u8);
                    if attr.get_foreground() != ch.attribute.get_foreground() {
                        let rgb = buf.palette.get_rgb(ch.attribute.get_foreground() as usize);
                        result.push(0);
                        result.push(rgb.0);
                        result.push(rgb.1);
                        result.push(rgb.2);
                    }
                    if attr.get_background() != ch.attribute.get_background() {
                        let rgb = buf.palette.get_rgb(ch.attribute.get_background() as usize);
                        result.push(0);
                        result.push(rgb.0);
                        result.push(rgb.1);
                        result.push(rgb.2);
                    }
                    attr = ch.attribute;
                    continue;
                }
                if ch.ch as u16 >= 1 && ch.ch as u16 <= 6 {
                    // fake color change
                    result.push(2);
                    result.push(ch.ch as u8);

                    let rgb = buf.palette.get_rgb(attr.get_foreground() as usize);
                    result.push(0);
                    result.push(rgb.0);
                    result.push(rgb.1);
                    result.push(rgb.2);
                    continue;
                }
                result.push(ch.ch as u8);
            }
        }
        if let Some(pos2) = skip_pos {
            let pos = Position::new(
                buf.get_width().saturating_sub(1),
                buf.get_line_count().saturating_sub(1),
            );

            let skip_len =
                (pos.x + pos.y * buf.get_width()) - (pos2.x + pos2.y * buf.get_width()) + 1;
            result.resize(result.len() + skip_len as usize, 0);
        }

        if options.save_sauce {
            buf.write_sauce_info(crate::SauceFileType::TundraDraw, &mut result)?;
        }
        Ok(result)
    }

    fn load_buffer(
        &self,
        file_name: &Path,
        data: &[u8],
        sauce_opt: Option<crate::SauceData>,
    ) -> EngineResult<crate::Buffer> {
        let mut result = Buffer::new((80, 25));
        result.is_terminal_buffer = true;
        result.file_name = Some(file_name.into());
        result.set_sauce(sauce_opt, true);
        if data.len() < 1 + TUNDRA_HEADER.len() {
            return Err(LoadingError::FileTooShort.into());
        }
        let mut o = 1;

        let header = &data[1..=TUNDRA_HEADER.len()];

        if header != TUNDRA_HEADER {
            return Err(LoadingError::IDMismatch.into());
        }
        o += TUNDRA_HEADER.len();

        result.palette.clear();
        result.palette.insert_color_rgb(0, 0, 0);
        result.buffer_type = BufferType::CP437;

        let mut pos = Position::default();
        let mut attr = TextAttribute::from_u8(0, result.ice_mode);

        while o < data.len() {
            let mut cmd = data[o];
            o += 1;
            if cmd == TUNDRA_POSITION {
                pos.y = to_u32(&data[o..]);
                if pos.y >= (u16::MAX) as i32 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid Tundra Draw file.\nJump y position {} out of bounds (height is {})", pos.y, result.get_line_count())).into());
                }
                o += 4;
                pos.x = to_u32(&data[o..]);
                if pos.x >= result.get_width() {
                    return Err(anyhow::anyhow!(
                        "Invalid Tundra Draw file.\nJump x position {} out of bounds (width is {})",
                        pos.x,
                        result.get_width()
                    ));
                }
                o += 4;
                continue;
            }

            if cmd > 1 && cmd <= 6 {
                let ch = data[o];
                o += 1;
                if cmd & TUNDRA_COLOR_FOREGROUND != 0 {
                    o += 1;
                    let r = data[o];
                    o += 1;
                    let g = data[o];
                    o += 1;
                    let b = data[o];
                    o += 1;
                    attr.set_foreground(result.palette.insert_color_rgb(r, g, b));
                }
                if cmd & TUNDRA_COLOR_BACKGROUND != 0 {
                    o += 1;
                    let r = data[o];
                    o += 1;
                    let g = data[o];
                    o += 1;
                    let b = data[o];
                    o += 1;
                    attr.set_background(result.palette.insert_color_rgb(r, g, b));
                }
                cmd = ch;
            }
            result.layers[0].set_height(pos.y + 1);
            result.layers[0].set_char(
                pos,
                AttributedChar::new(char::from_u32(cmd as u32).unwrap(), attr),
            );
            advance_pos(&result, &mut pos);
        }
        crate::crop_loaded_file(&mut result);
        result.palette.fill_to_16();

        Ok(result)
    }
}

fn advance_pos(result: &Buffer, pos: &mut Position) -> bool {
    pos.x += 1;
    if pos.x >= result.get_width() {
        pos.x = 0;
        pos.y += 1;
    }
    true
}

fn to_u32(bytes: &[u8]) -> i32 {
    bytes[3] as i32 | (bytes[2] as i32) << 8 | (bytes[1] as i32) << 16 | (bytes[0] as i32) << 24
}

const TND_GOTO_BLOCK_LEN: i32 = 1 + 2 * 4;

pub fn get_save_sauce_default_tnd(buf: &Buffer) -> (bool, String) {
    if buf.get_width() != 80 {
        return (true, "width != 80".to_string());
    }

    if buf.has_sauce() {
        return (true, String::new());
    }

    (false, String::new())
}
