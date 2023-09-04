use std::io;

use super::{Position, SaveOptions, TextAttribute};
use crate::{AttributedChar, BitFont, Buffer, BufferType, Palette, Size};

// http://fileformats.archiveteam.org/wiki/ArtWorx_Data_Format

// u8                   Version
// 3 * 64 = 192 u8      Palette
// 256 * 16 = 4096 u8   Font Data (only 8x16 supported)
// [ch u8, attr u8]*    Screen data
//
// A very simple format with a weird palette storage. Only 16 colors got used but a full 64 color palette is stored.
// Maybe useful for DOS demos running in text mode.

/// .
///
/// # Panics
///
/// Panics if .
///
/// # Errors
///
/// This function will return an error if .
pub fn read_adf(result: &mut Buffer, bytes: &[u8], file_size: usize) -> io::Result<bool> {
    result.set_buffer_width(80);
    result.buffer_type = BufferType::LegacyIce;
    let mut o = 0;
    let mut pos = Position::default();
    if file_size < 1 + 3 * 64 + 4096 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid ADF - file too short",
        ));
    }

    let version = bytes[o];
    if version != 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Unsupported ADF version {version}"),
        ));
    }
    o += 1;

    // convert EGA -> VGA colors.
    let palette_size = 3 * 64;
    result.palette = Palette::from(&bytes[o..(o + palette_size)]).cycle_ega_colors();
    o += palette_size;

    let font_size = 4096;
    result.clear_font_table();
    result.set_font(0, BitFont::from_basic(8, 16, &bytes[o..(o + font_size)]));
    o += font_size;

    loop {
        for _ in 0..result.get_width() {
            if o + 2 > file_size {
                result.set_height_for_pos(pos);
                return Ok(true);
            }
            result.layers[0].set_height(pos.y + 1);
            result.layers[0].set_char(
                pos,
                AttributedChar::new(
                    char::from_u32(bytes[o] as u32).unwrap(),
                    TextAttribute::from_u8(bytes[o + 1], result.buffer_type),
                ),
            );
            pos.x += 1;
            o += 2;
        }
        pos.x = 0;
        pos.y += 1;
    }
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
pub fn convert_to_adf(buf: &Buffer, options: &SaveOptions) -> io::Result<Vec<u8>> {
    let mut result = vec![1]; // version

    result.extend(buf.palette.to_ega_palette());
    if buf.get_font_dimensions() != Size::new(8, 16) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Only 8x16 fonts are supported by adf.",
        ));
    }

    buf.get_font(0).unwrap().convert_to_u8_data(&mut result);

    for y in 0..buf.get_line_count() {
        for x in 0..buf.get_width() {
            let ch = buf.get_char((x, y));
            result.push(ch.ch as u8);
            result.push(ch.attribute.as_u8(BufferType::LegacyIce));
        }
    }
    if options.save_sauce {
        buf.write_sauce_info(crate::SauceFileType::Ansi, &mut result)?;
    }
    Ok(result)
}

pub fn get_save_sauce_default_adf(buf: &Buffer) -> (bool, String) {
    if buf.get_width() != 80 {
        return (true, "width != 80".to_string());
    }

    if buf.has_sauce_relevant_data() {
        return (true, String::new());
    }

    (false, String::new())
}
