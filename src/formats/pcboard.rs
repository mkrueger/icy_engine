use std::io;

use crate::{Buffer, Position, TextAttribute};

use super::{ SaveOptions};


const HEX_TABLE: &[u8;16] = b"0123456789ABCDEF";

pub fn convert_to_pcb(buf: &Buffer, options: &SaveOptions) -> io::Result<Vec<u8>>
{
    let mut result = Vec::new();
    let mut last_attr = TextAttribute::DEFAULT;
    let mut pos = Position::new();
    let height = buf.get_buffer_height() as i32;
    let mut first_char = true;

    match options.screen_preparation {
        super::ScreenPreperation::None | super::ScreenPreperation::Home => {}, // home not supported
        super::ScreenPreperation::ClearScreen => { result.extend(b"@CLS@"); },
    }

    while pos.y < height {
        let line_length = buf.get_line_length(pos.y);
        
        while pos.x < line_length {
            let ch = buf.get_char(pos).unwrap_or_default();

            if first_char || ch.attribute != last_attr {
                result.extend_from_slice(b"@X");
                result.push(HEX_TABLE[ch.attribute.get_background() as usize]);
                result.push(HEX_TABLE[ch.attribute.get_foreground() as usize]);
                last_attr = ch.attribute;
            }

            result.push(if ch.char_code == 0 { b' ' } else { ch.char_code as u8 });
            first_char = false;
            pos.x += 1;
        }

        // do not end with eol
        if pos.x < buf.get_buffer_width() as i32 && pos.y + 1 < height {
            result.push(13);
            result.push(10);
        }

        pos.x = 0;
        pos.y += 1;
    }
    if options.save_sauce {
        buf.write_sauce_info(&crate::SauceFileType::PCBoard, &mut result)?;
    }
    Ok(result)
}

pub fn get_save_sauce_default_pcb(buf: &Buffer) -> (bool, String)
{
    if buf.get_buffer_width() != 80 {
        return (true, "width != 80".to_string() );
    }

    if buf.has_sauce_relevant_data() { return (true, String::new()); }

    ( false, String::new() )
}

