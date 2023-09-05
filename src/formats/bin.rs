use std::io;

use super::{Position, SaveOptions, TextAttribute};
use crate::{AttributedChar, Buffer};

/// .
///
/// # Panics
///
/// Panics if .
///
/// # Errors
///
/// This function will return an error if .
pub fn read_binary(result: &mut Buffer, bytes: &[u8], file_size: usize) -> io::Result<bool> {
    let mut o = 0;
    let mut pos = Position::default();
    loop {
        for _ in 0..result.get_width() {
            if o >= file_size {
                result.set_height_for_pos(pos);
                return Ok(true);
            }

            if o + 1 > file_size {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "invalid file - needs to be % 2 == 0",
                ));
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
/// # Errors
///
/// This function will return an error if .
pub fn convert_to_binary(buf: &Buffer, options: &SaveOptions) -> io::Result<Vec<u8>> {
    let mut result = Vec::new();

    for y in 0..buf.get_height() {
        for x in 0..buf.get_width() {
            let ch = buf.get_char((x, y));
            result.push(ch.ch as u8);
            result.push(ch.attribute.as_u8(buf.buffer_type));
        }
    }
    if options.save_sauce {
        buf.write_sauce_info(crate::SauceFileType::Bin, &mut result)?;
    }
    Ok(result)
}

pub fn get_save_sauce_default_binary(buf: &Buffer) -> (bool, String) {
    if buf.get_width() != 160 {
        return (true, "width != 160".to_string());
    }

    if buf.sauce_data.is_some() {
        return (true, String::new());
    }

    (false, String::new())
}
