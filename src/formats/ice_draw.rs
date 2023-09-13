use std::{io, path::Path};

use super::{SaveOptions, TextAttribute};
use crate::{
    AttributedChar, BitFont, Buffer, BufferType, EngineResult, LoadingError, OutputFormat, Palette,
    Position, SavingError, Size, TextPane,
};

// http://fileformats.archiveteam.org/wiki/ICEDraw

const HEADER_SIZE: usize = 4 + 4 * 2;

const IDF_V1_3_HEADER: &[u8] = b"\x041.3";
const IDF_V1_4_HEADER: &[u8] = b"\x041.4";

const FONT_SIZE: usize = 4096;
const PALETTE_SIZE: usize = 3 * 16;

#[derive(Default)]
pub(super) struct IceDraw {}

impl OutputFormat for IceDraw {
    fn get_file_extension(&self) -> &str {
        "idf"
    }

    fn get_name(&self) -> &str {
        "IceDraw"
    }

    fn to_bytes(&self, buf: &crate::Buffer, options: &SaveOptions) -> EngineResult<Vec<u8>> {
        let mut result = IDF_V1_4_HEADER.to_vec();

        // x1
        result.push(0);
        result.push(0);

        // y1
        result.push(0);
        result.push(0);

        let w = buf.get_width().saturating_sub(1);
        result.push(w as u8);
        result.push((w >> 8) as u8);

        let h = buf.get_line_count().saturating_sub(1);
        result.push(h as u8);
        result.push((h >> 8) as u8);

        let len = buf.get_line_count() * buf.get_width();
        let mut x = 0;
        while x < len {
            let ch = buf.get_char(Position::from_index(buf, x));
            let mut rle_count = 1;
            while x + rle_count < len && rle_count < (u16::MAX) as i32 {
                if ch != buf.get_char(Position::from_index(buf, x + rle_count)) {
                    break;
                }
                rle_count += 1;
            }
            if rle_count > 3 || ch.ch == '\x01' {
                result.push(1);
                result.push(0);

                result.push(rle_count as u8);
                result.push((rle_count >> 8) as u8);
            } else {
                rle_count = 1;
            }
            result.push(ch.ch as u8);
            result.push(ch.attribute.as_u8(BufferType::LegacyIce));

            x += rle_count;
        }

        // font
        if buf.get_font_dimensions() != Size::new(8, 16) {
            return Err(Box::new(SavingError::Only8x16FontsSupported));
        }
        if let Some(font) = buf.get_font(0) {
            font.convert_to_u8_data(&mut result);
        } else {
            return Err(Box::new(SavingError::NoFontFound));
        }

        // palette
        result.extend(buf.palette.to_16color_vec());
        if options.save_sauce {
            buf.write_sauce_info(crate::SauceFileType::Bin, &mut result)?;
        }
        Ok(result)
    }

    fn load_buffer(
        &self,
        file_name: &Path,
        data: &[u8],
        _sauce_opt: Option<crate::SauceData>,
    ) -> EngineResult<crate::Buffer> {
        let mut result = Buffer::new((80, 25));
        result.is_terminal_buffer = true;
        result.file_name = Some(file_name.into());

        if data.len() < HEADER_SIZE + FONT_SIZE + PALETTE_SIZE {
            return Err(Box::new(LoadingError::FileTooShort));
        }
        let version = &data[0..4];

        if version != IDF_V1_3_HEADER && version != IDF_V1_4_HEADER {
            return Err(Box::new(LoadingError::IDMismatch));
        }

        let mut o = 4;
        let x1 = (data[o] as u16 + ((data[o + 1] as u16) << 8)) as i32;
        o += 2;
        let y1 = (data[o] as u16 + ((data[o + 1] as u16) << 8)) as i32;
        o += 2;
        let x2 = (data[o] as u16 + ((data[o + 1] as u16) << 8)) as i32;
        o += 2;
        // skip y2
        o += 2;

        if x2 < x1 {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid bounds for idf width needs to be >=0.",
            )));
        }

        result.set_width(x2 + 1);
        result.buffer_type = BufferType::LegacyIce;
        let data_size = data.len() - FONT_SIZE - PALETTE_SIZE;
        let mut pos = Position::new(x1, y1);

        while o + 1 < data_size {
            let mut rle_count = 1;
            let mut char_code = data[o];
            o += 1;
            let mut attr = data[o];
            o += 1;

            if char_code == 1 && attr == 0 {
                rle_count = data[o] as i32 + ((data[o + 1] as i32) << 8);

                if o + 3 >= data_size {
                    break;
                }
                o += 2;
                char_code = data[o];
                o += 1;
                attr = data[o];
                o += 1;
            }
            while rle_count > 0 {
                result.layers[0].set_height(pos.y + 1);
                result.layers[0].set_char(
                    pos,
                    AttributedChar::new(
                        char::from_u32(char_code as u32).unwrap(),
                        TextAttribute::from_u8(attr, result.buffer_type),
                    ),
                );
                advance_pos(x1, x2, &mut pos);
                rle_count -= 1;
            }
        }
        result.layers[0].clear();
        result.set_font(0, BitFont::from_basic(8, 16, &data[o..(o + FONT_SIZE)]));
        o += FONT_SIZE;

        result.palette = Palette::from(&data[o..(o + PALETTE_SIZE)]);

        crate::crop_loaded_file(&mut result);

        Ok(result)
    }
}

pub fn get_save_sauce_default_idf(buf: &Buffer) -> (bool, String) {
    if buf.get_width() != 80 {
        return (true, "width != 80".to_string());
    }

    if buf.sauce_data.is_some() {
        return (true, String::new());
    }

    (false, String::new())
}

fn advance_pos(x1: i32, x2: i32, pos: &mut Position) -> bool {
    pos.x += 1;
    if pos.x > x2 {
        pos.x = x1;
        pos.y += 1;
    }
    true
}
