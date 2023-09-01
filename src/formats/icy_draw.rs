use std::io;

use crate::{
    convert_to_ansi_data, crc, parsers, BitFont, Buffer, BufferParser, Caret, Color, Layer,
    Position, SauceString,
};

mod constants {
    pub const ICD_HEADER: &[u8] = b"iced";
    pub const ICD_VERSION: u16 = 0;
    pub const ID_SIZE: usize = ICD_HEADER.len() + 1;
    pub const HEADER_SIZE: usize = 11;
    pub const CRC32_SIZE: usize = 4;

    pub mod block {
        pub const END: u8 = 0;
        pub const SAUCE: u8 = 1;
        pub const PALETTE: u8 = 2;
        pub const FONT: u8 = 3;
        pub const LAYER: u8 = 4;
    }

    pub mod layer {
        pub const IS_VISIBLE: u32 = 0b0000_0001;
        pub const POS_LOCK: u32 = 0b0000_0010;
        pub const EDIT_LOCK: u32 = 0b0000_0100;
        pub const HAS_ALPHA: u32 = 0b0000_1000;
        pub const ALPHA_LOCKED: u32 = 0b0001_0000;
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
pub fn read_icd(result: &mut Buffer, bytes: &[u8]) -> io::Result<bool> {
    if bytes.len() < constants::ID_SIZE + constants::CRC32_SIZE + constants::HEADER_SIZE {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "File too short"));
    }
    if &bytes[0..constants::ICD_HEADER.len()] != constants::ICD_HEADER {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid header"));
    }
    let crc32 = u32::from_be_bytes(
        bytes[constants::ID_SIZE..(constants::ID_SIZE + constants::CRC32_SIZE)]
            .try_into()
            .unwrap(),
    );
    let mut o = constants::ID_SIZE + constants::CRC32_SIZE;
    if crc32 != crc::get_crc32(&bytes[o..]) {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "CRC32 mismatch"));
    }
    result.layers.clear();
    o += 2; // skip version

    o += 1; // skip type

    let width: i32 = u32::from_be_bytes(bytes[o..(o + 4)].try_into().unwrap()) as i32;
    o += 4;
    let height: i32 = u32::from_be_bytes(bytes[o..(o + 4)].try_into().unwrap()) as i32;
    o += 4;
    result.set_buffer_size((width, height));

    while o < bytes.len() {
        let block_type = bytes[o];
        o += 1;
        match block_type {
            constants::block::END => {
                break;
            }
            constants::block::SAUCE => {
                o += result.title.read(&bytes[o..]);
                o += result.author.read(&bytes[o..]);
                o += result.group.read(&bytes[o..]);
                let mut comments = bytes[o];
                o += 1;
                while comments > 0 {
                    let mut comment: SauceString<64, 0> = SauceString::new();
                    o += comment.read(&bytes[o..]);
                    result.comments.push(comment);
                    comments -= 1;
                }
            }
            constants::block::PALETTE => {
                let mut colors = u32::from_be_bytes(bytes[o..(o + 4)].try_into().unwrap());
                result.palette.colors.clear();
                o += 4;
                while colors > 0 {
                    let r = bytes[o];
                    o += 1;
                    let g = bytes[o];
                    o += 1;
                    let b = bytes[o];
                    o += 2; // skip alpha

                    result.palette.colors.push(Color::new(r, g, b));
                    colors -= 1;
                }
            }
            constants::block::FONT => {
                let mut font_name: SauceString<22, 0> = SauceString::new();
                o += font_name.read(&bytes[o..]);

                let font_slot = u32::from_be_bytes(bytes[o..(o + 4)].try_into().unwrap()) as usize;
                o += 4;
                let (font_name, size) = read_utf8_encoded_string(&bytes[o..]);
                o += size;
                let data_length = u32::from_be_bytes(bytes[o..(o + 4)].try_into().unwrap());
                o += 4;
                let font =
                    BitFont::from_bytes(font_name, &bytes[o..(o + data_length as usize)]).unwrap();
                result.set_font(font_slot, font);
            }
            constants::block::LAYER => {
                let (title, size) = read_utf8_encoded_string(&bytes[o..]);
                o += size;

                let mut layer = Layer::new(title, (0, 0));

                let mode = bytes[o];

                layer.mode = match mode {
                    0 => crate::Mode::Normal,
                    1 => crate::Mode::Chars,
                    2 => crate::Mode::Attributes,
                    _ => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("Unsupported layer mode {mode}"),
                        ));
                    }
                };
                o += 1;

                // read layer color
                let red = bytes[o];
                o += 1;
                let green = bytes[o];
                o += 1;
                let blue = bytes[o];
                o += 1;
                let alpha = bytes[o];
                o += 1;
                if alpha != 0 {
                    layer.color = Some(Color::new(red, green, blue));
                }

                let flags = u32::from_be_bytes(bytes[o..(o + 4)].try_into().unwrap());
                o += 4;
                layer.is_visible =
                    (flags & constants::layer::IS_VISIBLE) == constants::layer::IS_VISIBLE;
                layer.is_locked =
                    (flags & constants::layer::EDIT_LOCK) == constants::layer::EDIT_LOCK;
                layer.is_position_locked =
                    (flags & constants::layer::POS_LOCK) == constants::layer::POS_LOCK;

                layer.has_alpha_channel =
                    (flags & constants::layer::HAS_ALPHA) == constants::layer::HAS_ALPHA;

                layer.is_alpha_channel_locked =
                    (flags & constants::layer::ALPHA_LOCKED) == constants::layer::ALPHA_LOCKED;

                let x_offset: i32 =
                    u32::from_be_bytes(bytes[o..(o + 4)].try_into().unwrap()) as i32;
                o += 4;
                let y_offset: i32 =
                    u32::from_be_bytes(bytes[o..(o + 4)].try_into().unwrap()) as i32;
                o += 4;
                layer.offset = Position::new(x_offset, y_offset);

                let width: i32 = u32::from_be_bytes(bytes[o..(o + 4)].try_into().unwrap()) as i32;
                o += 4;
                let height: i32 = u32::from_be_bytes(bytes[o..(o + 4)].try_into().unwrap()) as i32;
                o += 4;

                layer.size = (width, height).into();

                let length = u64::from_be_bytes(bytes[o..(o + 8)].try_into().unwrap()) as usize;
                o += 8;
                let mut p = parsers::ansi::Parser::default();
                let mut caret = Caret::default();
                result.layers.push(layer);
                if bytes.len() < o + length {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "data length out ouf bounds {} data lenth: {}",
                            o + length,
                            bytes.len()
                        ),
                    ));
                }
                (o..(o + length)).for_each(|i| {
                    let b = bytes[i];
                    let _ = p.print_char(
                        result,
                        result.layers.len().saturating_sub(1),
                        &mut caret,
                        char::from_u32(b as u32).unwrap(),
                    );
                });

                o += length;
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Unsupported block type {block_type}"),
                ));
            }
        }
    }
    Ok(true)
}

fn read_utf8_encoded_string(data: &[u8]) -> (String, usize) {
    let size = u32::from_be_bytes(data[0..4].try_into().unwrap()) as usize;
    (
        unsafe { String::from_utf8_unchecked(data[4..(4 + size)].to_vec()) },
        size + 4,
    )
}

fn write_utf8_encoded_string(data: &mut Vec<u8>, s: &str) {
    data.extend(u32::to_be_bytes(s.len() as u32));
    data.extend(s.as_bytes());
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
pub fn convert_to_icd(buf: &Buffer) -> io::Result<Vec<u8>> {
    let mut result = constants::ICD_HEADER.to_vec();
    result.push(0x1A); // CP/M EOF char (^Z) - used by DOS as well

    result.push(0); // CRC32 will be calculated at the end
    result.push(0);
    result.push(0);
    result.push(0);

    result.push(constants::ICD_VERSION as u8);
    result.push((constants::ICD_VERSION >> 8) as u8);
    result.push(0);
    result.extend(u32::to_be_bytes(buf.get_width() as u32));
    result.extend(u32::to_be_bytes(buf.get_line_count() as u32));

    if buf.has_sauce_relevant_data() {
        result.push(constants::block::SAUCE);
        buf.title.append_to(&mut result);
        buf.author.append_to(&mut result);
        buf.group.append_to(&mut result);
        if buf.comments.len() > 255 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "too many comments. Maximum of 255 are supported",
            ));
        }
        result.push(buf.comments.len() as u8);
        for cmt in &buf.comments {
            cmt.append_to(&mut result);
        }
    }

    if !buf.palette.is_default() {
        result.push(constants::block::PALETTE);
        result.extend(u32::to_be_bytes(buf.palette.len()));
        for col in &buf.palette.colors {
            let rgb = col.get_rgb();
            result.push(rgb.0);
            result.push(rgb.1);
            result.push(rgb.2);
            result.push(0xFF); // so far only solid colors are supported
        }
    }

    for (k, v) in buf.font_iter() {
        if k >= &100 {
            result.push(constants::block::FONT);
            v.name.append_to(&mut result);
            result.extend(v.to_psf2_bytes().unwrap());
        }
    }
    for (i, layer) in buf.layers.iter().enumerate() {
        result.push(constants::block::LAYER);
        write_utf8_encoded_string(&mut result, &layer.title);

        let mode = match layer.mode {
            crate::Mode::Normal => 0,
            crate::Mode::Chars => 1,
            crate::Mode::Attributes => 2,
        };
        result.push(mode);

        if let Some(color) = &layer.color {
            let (r, g, b) = color.get_rgb();
            result.push(r);
            result.push(g);
            result.push(b);
            result.push(0xFF);
        } else {
            result.extend([0, 0, 0, 0]);
        }

        let mut flags = 0;
        if layer.is_visible {
            flags |= constants::layer::IS_VISIBLE;
        }
        if layer.is_locked {
            flags |= constants::layer::EDIT_LOCK;
        }
        if layer.is_position_locked {
            flags |= constants::layer::POS_LOCK;
        }
        if layer.has_alpha_channel {
            flags |= constants::layer::HAS_ALPHA;
        }
        if layer.is_alpha_channel_locked {
            flags |= constants::layer::ALPHA_LOCKED;
        }
        result.extend(u32::to_be_bytes(flags));

        result.extend(i32::to_be_bytes(layer.get_offset().x));
        result.extend(i32::to_be_bytes(layer.get_offset().y));

        result.extend(i32::to_be_bytes(layer.size.width as i32));
        result.extend(i32::to_be_bytes(layer.size.height as i32));

        let data = convert_to_ansi_data(buf, i, false);
        result.extend(u64::to_be_bytes(data.len() as u64));
        result.extend(data);
    }

    result.push(constants::block::END);

    let crc = u32::to_be_bytes(crc::get_crc32(
        &result[(constants::ID_SIZE + constants::CRC32_SIZE)..],
    ));
    result[constants::ID_SIZE..(constants::ID_SIZE + crc.len())].clone_from_slice(&crc[..]);
    Ok(result)
}