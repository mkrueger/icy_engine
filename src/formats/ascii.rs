use std::io;

use crate::{Buffer, Position};

use super::SaveOptions;

/// .
///
/// # Errors
///
/// This function will return an error if .
pub fn convert_to_asc(buf: &Buffer, options: &SaveOptions) -> io::Result<Vec<u8>> {
    let mut result = Vec::new();
    let mut pos = Position::default();
    let height = buf.get_line_count();

    while pos.y < height {
        let line_length = buf.get_line_length(pos.y);
        while pos.x < line_length {
            let ch = buf.get_char(pos);
            result.push(if ch.ch == '\0' { b' ' } else { ch.ch as u8 });
            pos.x += 1;
        }

        // do not end with eol
        if pos.x < buf.get_width() && pos.y + 1 < height {
            result.push(13);
            result.push(10);
        }

        pos.x = 0;
        pos.y += 1;
    }

    if options.save_sauce {
        buf.write_sauce_info(crate::SauceFileType::Ascii, &mut result)?;
    }
    Ok(result)
}

pub fn get_save_sauce_default_asc(buf: &Buffer) -> (bool, String) {
    if buf.get_width() != 80 {
        return (true, "width != 80".to_string());
    }

    if buf.has_sauce_relevant_data() {
        return (true, String::new());
    }

    (false, String::new())
}
