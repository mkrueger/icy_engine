use std::{cmp::min, path::Path};

use crate::{
    AttributedChar, BitFont, Buffer, BufferFeatures, BufferType, EngineResult, LoadingError,
    OutputFormat, Palette, Position, SavingError, TextPane,
};

use super::{CompressionLevel, SaveOptions, TextAttribute};

const XBIN_HEADER_SIZE: usize = 11;

const FLAG_PALETTE: u8 = 0b_0000_0001;
const FLAG_FONT: u8 = 0b_0000_0010;
const FLAG_COMPRESS: u8 = 0b_0000_0100;
const FLAG_NON_BLINK_MODE: u8 = 0b_0000_1000;
const FLAG_512CHAR_MODE: u8 = 0b_0001_0000;

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
enum Compression {
    Off = 0b0000_0000,
    Char = 0b0100_0000,
    Attr = 0b1000_0000,
    Full = 0b1100_0000,
}

#[derive(Default)]
pub(super) struct XBin {}

impl OutputFormat for XBin {
    fn get_file_extension(&self) -> &str {
        "xb"
    }

    fn get_name(&self) -> &str {
        "XBin"
    }

    fn analyze_features(&self, _features: &BufferFeatures) -> String {
        String::new()
    }

    fn to_bytes(&self, buf: &crate::Buffer, options: &SaveOptions) -> EngineResult<Vec<u8>> {
        let mut result = Vec::new();

        result.extend_from_slice(b"XBIN");
        result.push(0x1A); // CP/M EOF char (^Z) - used by DOS as well

        result.push(buf.get_width() as u8);
        result.push((buf.get_width() >> 8) as u8);
        result.push(buf.get_line_count() as u8);
        result.push((buf.get_line_count() >> 8) as u8);

        let mut flags = 0;
        let Some(font) = buf.get_font(0) else {
            return Err(Box::new(SavingError::NoFontFound));
        };

        if font.size.width != 8 || font.size.height < 1 || font.size.height > 32 {
            return Err(Box::new(SavingError::InvalidXBinFont));
        }

        result.push(font.size.height as u8);
        if !font.is_default() || buf.has_fonts() {
            flags |= FLAG_FONT;
        }

        if !buf.palette.is_default() {
            flags |= FLAG_PALETTE;
        }
        if options.compression_level != CompressionLevel::Off {
            flags |= FLAG_COMPRESS;
        }

        if buf.buffer_type.use_ice_colors() {
            flags |= FLAG_NON_BLINK_MODE;
        }

        if buf.buffer_type.use_extended_font() {
            flags |= FLAG_512CHAR_MODE;
        }

        result.push(flags);

        if (flags & FLAG_PALETTE) == FLAG_PALETTE {
            result.extend(buf.palette.to_16color_vec());
        }

        if flags & FLAG_FONT == FLAG_FONT {
            font.convert_to_u8_data(&mut result);
            if flags & FLAG_512CHAR_MODE == FLAG_512CHAR_MODE {
                if let Some(font) = buf.get_font(0) {
                    font.convert_to_u8_data(&mut result);
                }
            }
        }
        match options.compression_level {
            CompressionLevel::Medium => compress_greedy(&mut result, buf, buf.buffer_type),
            CompressionLevel::High => compress_backtrack(&mut result, buf, buf.buffer_type),
            CompressionLevel::Off => {
                for y in 0..buf.get_line_count() {
                    for x in 0..buf.get_width() {
                        let ch = buf.get_char((x, y));

                        result.push(ch.ch as u8);
                        result.push(encode_attr(ch.ch as u16, ch.attribute, buf.buffer_type));
                    }
                }
            }
        }

        if options.save_sauce {
            buf.write_sauce_info(crate::SauceFileType::XBin, &mut result)?;
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
        if let Some(sauce) = sauce_opt {
            result.set_sauce(sauce);
        }

        if data.len() < XBIN_HEADER_SIZE {
            return Err(Box::new(LoadingError::FileTooShort));
        }
        if b"XBIN" != &data[0..4] {
            return Err(Box::new(LoadingError::IDMismatch));
        }

        let mut o = 4;

        // let eof_char = bytes[o];
        o += 1;
        let width = data[o] as i32 + ((data[o + 1] as i32) << 8);
        result.set_width(width);
        o += 2;
        let height = data[o] as i32 + ((data[o + 1] as i32) << 8);
        result.set_height(height);
        result.layers[0].set_size((width, height));
        o += 2;
        let font_size = data[o];
        o += 1;
        let flags = data[o];
        o += 1;

        let has_custom_palette = (flags & FLAG_PALETTE) == FLAG_PALETTE;
        let has_custom_font = (flags & FLAG_FONT) == FLAG_FONT;
        let is_compressed = (flags & FLAG_COMPRESS) == FLAG_COMPRESS;
        let use_ice = (flags & FLAG_NON_BLINK_MODE) == FLAG_NON_BLINK_MODE;
        let extended_char_mode = (flags & FLAG_512CHAR_MODE) == FLAG_512CHAR_MODE;

        if extended_char_mode {
            result.buffer_type = if use_ice {
                BufferType::ExtendedFontAndIce
            } else {
                BufferType::ExtendedFont
            };
        } else {
            result.buffer_type = if use_ice {
                BufferType::LegacyIce
            } else {
                BufferType::LegacyDos
            };
        }

        if has_custom_palette {
            result.palette = Palette::from(&data[o..(o + 48)]);
            o += 48;
        }
        if has_custom_font {
            let font_length = font_size as usize * 256;
            result.clear_font_table();
            result.set_font(
                0,
                BitFont::create_8("", 8, font_size, &data[o..(o + font_length)]),
            );
            o += font_length;
            if extended_char_mode {
                result.set_font(
                    1,
                    BitFont::create_8("", 8, font_size, &data[o..(o + font_length)]),
                );
                o += font_length;
            }
        }
        if is_compressed {
            read_data_compressed(&mut result, &data[o..])?;
        } else {
            read_data_uncompressed(&mut result, &data[o..])?;
        }
        crate::crop_loaded_file(&mut result);

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

fn read_data_compressed(result: &mut Buffer, bytes: &[u8]) -> EngineResult<bool> {
    let mut pos = Position::default();
    let mut o = 0;
    while o < bytes.len() {
        let xbin_compression = bytes[o];

        o += 1;
        let compression = unsafe { std::mem::transmute(xbin_compression & 0b_1100_0000) };
        let repeat_counter = (xbin_compression & 0b_0011_1111) + 1;

        match compression {
            Compression::Off => {
                for _ in 0..repeat_counter {
                    if o + 2 > bytes.len() {
                        log::error!("Invalid XBin. Read char block beyond EOF.");
                        break;
                    }
                    let char_code = bytes[o];
                    let attribute = bytes[o + 1];
                    o += 2;
                    result.layers[0]
                        .set_char(pos, decode_char(char_code, attribute, result.buffer_type));

                    if !advance_pos(result, &mut pos) {
                        return Err(Box::new(LoadingError::OutOfBounds));
                    }
                }
            }
            Compression::Char => {
                let char_code = bytes[o];
                o += 1;
                for _ in 0..repeat_counter {
                    if o + 1 > bytes.len() {
                        log::error!("Invalid XBin. Read char compression block beyond EOF.");
                        break;
                    }

                    result.layers[0]
                        .set_char(pos, decode_char(char_code, bytes[o], result.buffer_type));
                    o += 1;
                    if !advance_pos(result, &mut pos) {
                        return Err(Box::new(LoadingError::OutOfBounds));
                    }
                }
            }
            Compression::Attr => {
                let attribute = bytes[o];
                o += 1;
                for _ in 0..repeat_counter {
                    if o + 1 > bytes.len() {
                        log::error!("Invalid XBin. Read attribute compression block beyond EOF.");
                        break;
                    }
                    result.layers[0]
                        .set_char(pos, decode_char(bytes[o], attribute, result.buffer_type));
                    o += 1;
                    if !advance_pos(result, &mut pos) {
                        return Err(Box::new(LoadingError::OutOfBounds));
                    }
                }
            }
            Compression::Full => {
                let char_code = bytes[o];
                o += 1;
                if o + 1 > bytes.len() {
                    log::error!("Invalid XBin. nRead compression block beyond EOF.");
                    break;
                }
                let attr = bytes[o];
                o += 1;
                let rep_ch = decode_char(char_code, attr, result.buffer_type);

                for _ in 0..repeat_counter {
                    result.layers[0].set_char(pos, rep_ch);
                    if !advance_pos(result, &mut pos) {
                        return Err(Box::new(LoadingError::OutOfBounds));
                    }
                }
            }
        }
    }

    Ok(true)
}

fn decode_char(char_code: u8, attr: u8, buffer_type: BufferType) -> AttributedChar {
    let mut attribute = TextAttribute::from_u8(attr, buffer_type);
    if buffer_type.use_extended_font() && (attr & 0b_1000) != 0 {
        attribute.set_foreground(attribute.get_foreground());
        AttributedChar::new(
            char::from_u32(char_code as u32 | 1 << 9).unwrap(),
            attribute,
        )
    } else {
        AttributedChar::new(char::from_u32(char_code as u32).unwrap(), attribute)
    }
}

fn encode_attr(char_code: u16, attr: TextAttribute, buffer_type: BufferType) -> u8 {
    if buffer_type.use_extended_font() {
        attr.as_u8(buffer_type) | if char_code > 255 { 0b1000 } else { 0 }
    } else {
        attr.as_u8(buffer_type)
    }
}

fn read_data_uncompressed(result: &mut Buffer, bytes: &[u8]) -> EngineResult<bool> {
    let mut pos = Position::default();
    let mut o = 0;
    while o < bytes.len() {
        if o + 1 >= bytes.len() {
            // last byte is not important enough to throw an error
            // there seem to be some invalid files out there.
            log::error!("Invalid XBin. Read char block beyond EOF.");
            return Ok(true);
        }
        result.layers[0].set_char(pos, decode_char(bytes[o], bytes[o + 1], result.buffer_type));
        o += 2;
        if !advance_pos(result, &mut pos) {
            return Err(Box::new(LoadingError::OutOfBounds));
        }
    }

    Ok(true)
}

fn compress_greedy(outputdata: &mut Vec<u8>, buffer: &Buffer, buffer_type: BufferType) {
    let mut run_mode = Compression::Off;
    let mut run_count = 0;
    let mut run_buf = Vec::new();
    let mut run_ch = AttributedChar::default();
    let len = buffer.get_line_count() * buffer.get_width();
    for x in 0..len {
        let cur = buffer.get_char(Position::from_index(buffer, x));

        let next = if x + 1 < len {
            buffer.get_char(Position::from_index(buffer, x + 1))
        } else {
            AttributedChar::default()
        };

        if run_count > 0 {
            let mut end_run = false;
            if run_count >= 64 {
                end_run = true;
            } else if run_count > 0 {
                match run_mode {
                    Compression::Off => {
                        if x + 2 < len && cur == next {
                            end_run = true;
                        } else if x + 2 < len {
                            let next2 = buffer.get_char(Position::from_index(buffer, x + 2));
                            end_run = cur.ch == next.ch && cur.ch == next2.ch
                                || cur.attribute == next.attribute
                                    && cur.attribute == next2.attribute;
                        }
                    }
                    Compression::Char => {
                        if cur.ch != run_ch.ch {
                            end_run = true;
                        } else if x + 3 < len {
                            let next2 = buffer.get_char(Position::from_index(buffer, x + 2));
                            let next3 = buffer.get_char(Position::from_index(buffer, x + 3));
                            end_run = cur == next && cur == next2 && cur == next3;
                        }
                    }
                    Compression::Attr => {
                        if cur.attribute != run_ch.attribute {
                            end_run = true;
                        } else if x + 3 < len {
                            let next2 = buffer.get_char(Position::from_index(buffer, x + 2));
                            let next3 = buffer.get_char(Position::from_index(buffer, x + 3));
                            end_run = cur == next && cur == next2 && cur == next3;
                        }
                    }
                    Compression::Full => {
                        end_run = cur != run_ch;
                    }
                }
            }

            if end_run {
                outputdata.push((run_mode as u8) | (run_count - 1));
                outputdata.extend(&run_buf);
                run_count = 0;
            }
        }

        if run_count > 0 {
            match run_mode {
                Compression::Off => {
                    run_buf.push(cur.ch as u8);
                    run_buf.push(encode_attr(cur.ch as u16, cur.attribute, buffer_type));
                }
                Compression::Char => {
                    run_buf.push(encode_attr(cur.ch as u16, cur.attribute, buffer_type));
                }
                Compression::Attr => {
                    run_buf.push(cur.ch as u8);
                }
                Compression::Full => {
                    // nothing
                }
            }
        } else {
            run_buf.clear();
            if x + 1 < len {
                if cur == next {
                    run_mode = Compression::Full;
                } else if cur.ch == next.ch {
                    run_mode = Compression::Char;
                } else if cur.attribute == next.attribute {
                    run_mode = Compression::Attr;
                } else {
                    run_mode = Compression::Off;
                }
            } else {
                run_mode = Compression::Off;
            }

            if let Compression::Attr = run_mode {
                run_buf.push(encode_attr(cur.ch as u16, cur.attribute, buffer_type));
                run_buf.push(cur.ch as u8);
            } else {
                run_buf.push(cur.ch as u8);
                run_buf.push(encode_attr(cur.ch as u16, cur.attribute, buffer_type));
            }

            run_ch = cur;
        }
        run_count += 1;
    }

    if run_count > 0 {
        outputdata.push((run_mode as u8) | (run_count - 1));
        outputdata.extend(run_buf);
    }
}

fn count_length(
    mut run_mode: Compression,
    mut run_ch: AttributedChar,
    mut end_run: Option<bool>,
    mut run_count: u8,
    buffer: &Buffer,
    mut x: i32,
) -> usize {
    let len = min(x + 256, (buffer.get_line_count() * buffer.get_width()) - 1);
    let mut count = 0;
    while x < len {
        let cur = buffer.get_char(Position::from_index(buffer, x));
        let next = buffer.get_char(Position::from_index(buffer, x + 1));

        if run_count > 0 {
            if end_run.is_none() {
                if run_count >= 64 {
                    end_run = Some(true);
                } else if run_count > 0 {
                    match run_mode {
                        Compression::Off => {
                            if x + 2 < len && cur == next {
                                end_run = Some(true);
                            } else if x + 2 < len {
                                let next2 = buffer.get_char(Position::from_index(buffer, x + 2));
                                end_run = Some(
                                    cur.ch == next.ch && cur.ch == next2.ch
                                        || cur.attribute == next.attribute
                                            && cur.attribute == next2.attribute,
                                );
                            }
                        }
                        Compression::Char => {
                            if cur.ch != run_ch.ch {
                                end_run = Some(true);
                            } else if x + 3 < len {
                                let next2 = buffer.get_char(Position::from_index(buffer, x + 2));
                                let next3 = buffer.get_char(Position::from_index(buffer, x + 3));
                                end_run = Some(cur == next && cur == next2 && cur == next3);
                            }
                        }
                        Compression::Attr => {
                            if cur.attribute != run_ch.attribute {
                                end_run = Some(true);
                            } else if x + 3 < len {
                                let next2 = buffer.get_char(Position::from_index(buffer, x + 2));
                                let next3 = buffer.get_char(Position::from_index(buffer, x + 3));
                                end_run = Some(cur == next && cur == next2 && cur == next3);
                            }
                        }
                        Compression::Full => {
                            end_run = Some(cur != run_ch);
                        }
                    }
                }
            }

            if let Some(true) = end_run {
                count += 1;
                run_count = 0;
            }
        }
        end_run = None;

        if run_count > 0 {
            match run_mode {
                Compression::Off => {
                    count += 2;
                }
                Compression::Char | Compression::Attr => {
                    count += 1;
                }
                Compression::Full => {
                    // nothing
                }
            }
        } else {
            if x + 1 < len {
                if cur == next {
                    run_mode = Compression::Full;
                } else if cur.ch == next.ch {
                    run_mode = Compression::Char;
                } else if cur.attribute == next.attribute {
                    run_mode = Compression::Attr;
                } else {
                    run_mode = Compression::Off;
                }
            } else {
                run_mode = Compression::Off;
            }
            count += 2;
            run_ch = cur;
            end_run = None;
        }
        run_count += 1;
        x += 1;
    }
    count
}

fn compress_backtrack(outputdata: &mut Vec<u8>, buffer: &Buffer, buffer_type: BufferType) {
    let mut run_mode = Compression::Off;
    let mut run_count = 0;
    let mut run_buf = Vec::new();
    let mut run_ch = AttributedChar::default();
    let len = buffer.get_line_count() * buffer.get_width();
    for x in 0..len {
        let cur = buffer.get_char(Position::from_index(buffer, x));

        let next = if x + 1 < len {
            buffer.get_char(Position::from_index(buffer, x + 1))
        } else {
            AttributedChar::default()
        };

        if run_count > 0 {
            let mut end_run = false;
            if run_count >= 64 {
                end_run = true;
            } else if run_count > 0 {
                match run_mode {
                    Compression::Off => {
                        if x + 2 < len && (cur.ch == next.ch || cur.attribute == next.attribute) {
                            let l1 =
                                count_length(run_mode, run_ch, Some(true), run_count, buffer, x);
                            let l2 =
                                count_length(run_mode, run_ch, Some(false), run_count, buffer, x);
                            end_run = l1 < l2;
                        }
                    }
                    Compression::Char => {
                        if cur.ch != run_ch.ch {
                            end_run = true;
                        } else if x + 4 < len {
                            let next2 = buffer.get_char(Position::from_index(buffer, x + 2));
                            if cur.attribute == next.attribute && cur.attribute == next2.attribute {
                                let l1 = count_length(
                                    run_mode,
                                    run_ch,
                                    Some(true),
                                    run_count,
                                    buffer,
                                    x,
                                );
                                let l2 = count_length(
                                    run_mode,
                                    run_ch,
                                    Some(false),
                                    run_count,
                                    buffer,
                                    x,
                                );
                                end_run = l1 < l2;
                            }
                        }
                    }
                    Compression::Attr => {
                        if cur.attribute != run_ch.attribute {
                            end_run = true;
                        } else if x + 3 < len {
                            let next2 = buffer.get_char(Position::from_index(buffer, x + 2));
                            if cur.ch == next.ch && cur.ch == next2.ch {
                                let l1 = count_length(
                                    run_mode,
                                    run_ch,
                                    Some(true),
                                    run_count,
                                    buffer,
                                    x,
                                );
                                let l2 = count_length(
                                    run_mode,
                                    run_ch,
                                    Some(false),
                                    run_count,
                                    buffer,
                                    x,
                                );
                                end_run = l1 < l2;
                            }
                        }
                    }
                    Compression::Full => {
                        end_run = cur != run_ch;
                    }
                }
            }

            if end_run {
                outputdata.push((run_mode as u8) | (run_count - 1));
                outputdata.extend(&run_buf);
                run_count = 0;
            }
        }

        if run_count > 0 {
            match run_mode {
                Compression::Off => {
                    run_buf.push(cur.ch as u8);
                    run_buf.push(encode_attr(cur.ch as u16, cur.attribute, buffer_type));
                }
                Compression::Char => {
                    run_buf.push(encode_attr(cur.ch as u16, cur.attribute, buffer_type));
                }
                Compression::Attr => {
                    run_buf.push(cur.ch as u8);
                }
                Compression::Full => {
                    // nothing
                }
            }
        } else {
            run_buf.clear();
            if x + 1 < len {
                if cur == next {
                    run_mode = Compression::Full;
                } else if cur.ch == next.ch {
                    run_mode = Compression::Char;
                } else if cur.attribute == next.attribute {
                    run_mode = Compression::Attr;
                } else {
                    run_mode = Compression::Off;
                }
            } else {
                run_mode = Compression::Off;
            }

            if let Compression::Attr = run_mode {
                run_buf.push(encode_attr(cur.ch as u16, cur.attribute, buffer_type));
                run_buf.push(cur.ch as u8);
            } else {
                run_buf.push(cur.ch as u8);
                run_buf.push(encode_attr(cur.ch as u16, cur.attribute, buffer_type));
            }

            run_ch = cur;
        }
        run_count += 1;
    }

    if run_count > 0 {
        outputdata.push((run_mode as u8) | (run_count - 1));
        outputdata.extend(run_buf);
    }
}

pub fn get_save_sauce_default_xb(buf: &Buffer) -> (bool, String) {
    (buf.sauce_data.is_some(), String::new())
}
