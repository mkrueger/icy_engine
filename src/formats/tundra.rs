use std::io;

use super::{SaveOptions, TextAttribute};
use crate::{AttributedChar, Buffer, BufferType, Position};

// http://fileformats.archiveteam.org/wiki/TUNDRA
// ANSI code for 24 bit: ESC[(0|1);R;G;Bt
// 0 for background
// 1 for foreground

const TUNDRA_VER: u8 = 24;
const TUNDRA_HEADER: &[u8] = b"TUNDRA24";

const TUNDRA_POSITION: u8 = 1;
const TUNDRA_COLOR_FOREGROUND: u8 = 2;
const TUNDRA_COLOR_BACKGROUND: u8 = 4;

/// .
///
/// # Panics
///
/// Panics if .
///
/// # Errors
///
/// This function will return an error if .
pub fn read_tnd(result: &mut Buffer, bytes: &[u8], file_size: usize) -> io::Result<bool> {
    if file_size < 1 + TUNDRA_HEADER.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid Tundra Draw file.\nFile too short",
        ));
    }
    let mut o = 1;

    let header = &bytes[1..=TUNDRA_HEADER.len()];

    if header != TUNDRA_HEADER {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid Tundra Draw file.\nWrong ID",
        ));
    }
    o += TUNDRA_HEADER.len();

    result.palette.clear();
    result.palette.insert_color_rgb(0, 0, 0);
    result.buffer_type = BufferType::NoLimits;

    let mut pos = Position::default();
    let mut attr = TextAttribute::from_u8(0, result.buffer_type);

    while o < file_size {
        let mut cmd = bytes[o];
        o += 1;
        if cmd == TUNDRA_POSITION {
            pos.y = to_u32(&bytes[o..]);
            if pos.y >= (u16::MAX) as i32 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid Tundra Draw file.\nJump y position {} out of bounds (height is {})", pos.y, result.get_line_count())));
            }
            o += 4;
            pos.x = to_u32(&bytes[o..]);
            if pos.x >= result.get_width() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Invalid Tundra Draw file.\nJump x position {} out of bounds (width is {})",
                        pos.x,
                        result.get_width()
                    ),
                ));
            }

            o += 4;
            continue;
        }

        if cmd > 1 && cmd <= 6 {
            let ch = bytes[o];
            o += 1;

            if cmd & TUNDRA_COLOR_FOREGROUND != 0 {
                o += 1;
                let r = bytes[o];
                o += 1;
                let g = bytes[o];
                o += 1;
                let b = bytes[o];
                o += 1;
                attr.set_foreground(result.palette.insert_color_rgb(r, g, b));
            }
            if cmd & TUNDRA_COLOR_BACKGROUND != 0 {
                o += 1;
                let r = bytes[o];
                o += 1;
                let g = bytes[o];
                o += 1;
                let b = bytes[o];
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
        advance_pos(result, &mut pos);
    }
    result.set_height_for_pos(pos);
    result.palette.fill_to_16();

    Ok(true)
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

/// .
///
/// # Panics
///
/// Panics if .
///
/// # Errors
///
/// This function will return an error if .
pub fn convert_to_tnd(buf: &Buffer, options: &SaveOptions) -> io::Result<Vec<u8>> {
    let mut result = vec![TUNDRA_VER]; // version
    result.extend(TUNDRA_HEADER);
    let mut attr = TextAttribute::from_u8(0, buf.buffer_type);
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
                    let rgb = buf.palette.colors[ch.attribute.get_foreground() as usize].get_rgb();
                    result.push(0);
                    result.push(rgb.0);
                    result.push(rgb.1);
                    result.push(rgb.2);
                }
                if attr.get_background() != ch.attribute.get_background() {
                    let rgb = buf.palette.colors[ch.attribute.get_background() as usize].get_rgb();
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

                let rgb = buf.palette.colors[attr.get_foreground() as usize].get_rgb();
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

        let skip_len = (pos.x + pos.y * buf.get_width()) - (pos2.x + pos2.y * buf.get_width()) + 1;
        result.resize(result.len() + skip_len as usize, 0);
    }

    if options.save_sauce {
        buf.write_sauce_info(crate::SauceFileType::TundraDraw, &mut result)?;
    }
    Ok(result)
}

pub fn get_save_sauce_default_tnd(buf: &Buffer) -> (bool, String) {
    if buf.get_width() != 80 {
        return (true, "width != 80".to_string());
    }

    if buf.sauce_data.is_some() {
        return (true, String::new());
    }

    (false, String::new())
}
