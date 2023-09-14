use std::{error::Error, io, path::Path};

use base64::{engine::general_purpose, Engine};

use crate::{
    parsers, BitFont, Buffer, BufferParser, Caret, Color, EngineResult, Layer, LoadingError,
    OutputFormat, Position, SauceData, SauceFileType, SaveOptions, Sixel, Size, StringGenerator,
    TextPane,
};

mod constants {
    pub const ICD_VERSION: u16 = 0;

    pub mod layer {
        pub const IS_VISIBLE: u32 = 0b0000_0001;
        pub const POS_LOCK: u32 = 0b0000_0010;
        pub const EDIT_LOCK: u32 = 0b0000_0100;
        pub const HAS_ALPHA: u32 = 0b0000_1000;
        pub const ALPHA_LOCKED: u32 = 0b0001_0000;
    }
}

#[derive(Default)]
pub struct IcyDraw {}

impl OutputFormat for IcyDraw {
    fn get_file_extension(&self) -> &str {
        "icy"
    }

    fn get_name(&self) -> &str {
        "Iced"
    }

    fn to_bytes(&self, buf: &crate::Buffer, _options: &SaveOptions) -> EngineResult<Vec<u8>> {
        let mut result = Vec::new();

        let font_dims = buf.get_font_dimensions();
        let mut width = buf.get_width() * font_dims.width;

        let mut first_line = 0;
        while first_line < buf.get_height() {
            if !buf.is_line_empty(first_line) {
                break;
            }
            first_line += 1;
        }

        let last_line = (first_line + MAX_LINES).min(buf.get_line_count().max(buf.get_height()));
        let mut height = (last_line - first_line) * font_dims.height;

        let image_empty = if width == 0 || height == 0 {
            width = 1;
            height = 1;
            true
        } else {
            false
        };

        let mut encoder: png::Encoder<'_, &mut Vec<u8>> =
            png::Encoder::new(&mut result, width as u32, height as u32); // Width is 2 pixels and height is 1.
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(png::Compression::Best);

        {
            let mut result = vec![
                constants::ICD_VERSION as u8,
                (constants::ICD_VERSION >> 8) as u8,
            ];
            result.extend(u32::to_le_bytes(0)); // Type
            result.extend(u16::to_le_bytes(0)); // Mode

            result.extend(u32::to_le_bytes(buf.get_width() as u32));
            result.extend(u32::to_le_bytes(buf.get_line_count() as u32));
            let sauce_data = general_purpose::STANDARD.encode(&result);
            if let Err(err) = encoder.add_ztxt_chunk("ICED".to_string(), sauce_data) {
                return Err(Box::new(IcedError::ErrorEncodingZText(format!("{err}"))));
            }
        }

        if buf.sauce_data.is_some() {
            let mut sauce_vec: Vec<u8> = Vec::new();
            buf.write_sauce_info(SauceFileType::Ansi, &mut sauce_vec)?;
            let sauce_data = general_purpose::STANDARD.encode(&sauce_vec);
            if let Err(err) = encoder.add_ztxt_chunk("SAUCE".to_string(), sauce_data) {
                return Err(Box::new(IcedError::ErrorEncodingZText(format!("{err}"))));
            }
        }

        for (k, v) in buf.font_iter() {
            if k >= &100 {
                let mut font_data: Vec<u8> = Vec::new();
                write_utf8_encoded_string(&mut font_data, &v.name);
                font_data.extend(v.to_psf2_bytes().unwrap());

                if let Err(err) = encoder.add_ztxt_chunk(
                    format!("FONT_{k}"),
                    general_purpose::STANDARD.encode(&font_data),
                ) {
                    return Err(Box::new(IcedError::ErrorEncodingZText(format!("{err}"))));
                }
            }
        }

        for (i, layer) in buf.layers.iter().enumerate() {
            let mut result: Vec<u8> = Vec::new();
            write_utf8_encoded_string(&mut result, &layer.title);

            match layer.role {
                crate::Role::Image => result.push(1),
                _ => result.push(0),
            }

            // Some extra bytes not yet used
            result.extend([0, 0, 0, 0]);

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
            result.extend(u32::to_le_bytes(flags));

            result.extend(i32::to_le_bytes(layer.get_offset().x));
            result.extend(i32::to_le_bytes(layer.get_offset().y));

            result.extend(i32::to_le_bytes(layer.get_width()));
            result.extend(i32::to_le_bytes(layer.get_height()));

            if matches!(layer.role, crate::Role::Image) {
                let sixel = &layer.sixels[0];
                result.extend(u64::to_le_bytes(16 + sixel.picture_data.len() as u64));

                result.extend(i32::to_le_bytes(sixel.get_width()));
                result.extend(i32::to_le_bytes(sixel.get_height()));
                result.extend(i32::to_le_bytes(sixel.vertical_scale));
                result.extend(i32::to_le_bytes(sixel.horizontal_scale));
                result.extend(&sixel.picture_data);
            } else {
                let mut gen = StringGenerator::new(SaveOptions::default());
                gen.generate(buf, layer);
                result.extend(u64::to_le_bytes(gen.get_data().len() as u64));
                result.extend(gen.get_data());
            }
            let layer_data = general_purpose::STANDARD.encode(&result);
            if let Err(err) = encoder.add_ztxt_chunk(format!("LAYER_{i}"), layer_data) {
                return Err(Box::new(IcedError::ErrorEncodingZText(format!("{err}"))));
            }
        }

        if let Err(err) = encoder.add_ztxt_chunk("END".to_string(), String::new()) {
            return Err(Box::new(IcedError::ErrorEncodingZText(format!("{err}"))));
        }

        let mut writer = encoder.write_header().unwrap();

        if image_empty {
            writer.write_image_data(&[0, 0, 0, 0]).unwrap();
        } else {
            let (_, data) = buf.render_to_rgba(crate::Rectangle {
                start: Position::new(0, first_line),
                size: Size::new(buf.get_width(), last_line - first_line),
            });
            writer.write_image_data(&data).unwrap();
        }
        writer.finish().unwrap();

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
        result.layers.clear();

        let mut decoder = png::StreamingDecoder::new();
        let mut len = 0;
        let mut last_info = 0;
        let mut is_running = true;
        while is_running {
            match decoder.update(&data[len..], &mut Vec::new()) {
                Ok((b, _)) => {
                    len += b;
                    if data.len() <= len {
                        break;
                    }
                    if let Some(info) = decoder.info() {
                        for i in last_info..info.compressed_latin1_text.len() {
                            let chunk = &info.compressed_latin1_text[i];
                            let Ok(text) = chunk.get_text() else {
                                log::error!("error decoding iced chunk: {}", chunk.keyword);
                                continue;
                            };

                            let bytes = match general_purpose::STANDARD.decode(text) {
                                Ok(data) => data,
                                Err(e) => {
                                    log::warn!("error decoding iced chunk: {e}");
                                    continue;
                                }
                            };
                            match chunk.keyword.as_str() {
                                "END" => {
                                    is_running = false;
                                    break;
                                }
                                "ICED" => {
                                    let mut o: usize = 0;

                                    o += 2; // skip version

                                    // TODO: read type ATM only 1 type is generated.
                                    o += 4; // skip type

                                    o += 2; // skip mode

                                    let width: i32 =
                                        u32::from_le_bytes(bytes[o..(o + 4)].try_into().unwrap())
                                            as i32;
                                    o += 4;
                                    let height: i32 =
                                        u32::from_le_bytes(bytes[o..(o + 4)].try_into().unwrap())
                                            as i32;
                                    result.set_size((width, height));
                                }
                                "SAUCE" => {
                                    let sauce = SauceData::extract(&bytes).unwrap();
                                    result.set_sauce(sauce);
                                }
                                text => {
                                    if let Some(font_slot) = text.strip_prefix("FONT_") {
                                        match font_slot.parse() {
                                            Ok(font_slot) => {
                                                let mut o: usize = 0;
                                                let (font_name, size) =
                                                    read_utf8_encoded_string(&bytes[o..]);
                                                o += size;
                                                let font =
                                                    BitFont::from_bytes(font_name, &bytes[o..])
                                                        .unwrap();
                                                result.set_font(font_slot, font);
                                                continue;
                                            }
                                            Err(err) => {
                                                return Err(Box::new(
                                                    IcedError::ErrorParsingFontSlot(format!(
                                                        "{err}"
                                                    )),
                                                ));
                                            }
                                        }
                                    }
                                    if !text.starts_with("LAYER_") {
                                        log::warn!("unsupported chunk {text}");
                                        continue;
                                    }
                                    let mut o: usize = 0;

                                    let (title, size) = read_utf8_encoded_string(&bytes[o..]);
                                    let mut layer = Layer::new(title, (0, 0));

                                    o += size;
                                    let role = bytes[o];
                                    o += 1;
                                    if role == 1 {
                                        layer.role = crate::Role::Image;
                                    } else {
                                        layer.role = crate::Role::Normal;
                                    }

                                    o += 4; // skip unused

                                    let mode = bytes[o];

                                    layer.mode = match mode {
                                        0 => crate::Mode::Normal,
                                        1 => crate::Mode::Chars,
                                        2 => crate::Mode::Attributes,
                                        _ => {
                                            return Err(Box::new(
                                                LoadingError::IcyDrawUnsupportedLayerMode(mode),
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

                                    let flags =
                                        u32::from_le_bytes(bytes[o..(o + 4)].try_into().unwrap());
                                    o += 4;
                                    layer.is_visible = (flags & constants::layer::IS_VISIBLE)
                                        == constants::layer::IS_VISIBLE;
                                    layer.is_locked = (flags & constants::layer::EDIT_LOCK)
                                        == constants::layer::EDIT_LOCK;
                                    layer.is_position_locked = (flags & constants::layer::POS_LOCK)
                                        == constants::layer::POS_LOCK;

                                    layer.has_alpha_channel = (flags & constants::layer::HAS_ALPHA)
                                        == constants::layer::HAS_ALPHA;

                                    layer.is_alpha_channel_locked = (flags
                                        & constants::layer::ALPHA_LOCKED)
                                        == constants::layer::ALPHA_LOCKED;

                                    let x_offset: i32 =
                                        u32::from_le_bytes(bytes[o..(o + 4)].try_into().unwrap())
                                            as i32;
                                    o += 4;
                                    let y_offset: i32 =
                                        u32::from_le_bytes(bytes[o..(o + 4)].try_into().unwrap())
                                            as i32;
                                    o += 4;
                                    layer.set_offset((x_offset, y_offset));

                                    let width: i32 =
                                        u32::from_le_bytes(bytes[o..(o + 4)].try_into().unwrap())
                                            as i32;
                                    o += 4;
                                    let height: i32 =
                                        u32::from_le_bytes(bytes[o..(o + 4)].try_into().unwrap())
                                            as i32;
                                    o += 4;

                                    layer.set_size((width, height));

                                    let length =
                                        u64::from_le_bytes(bytes[o..(o + 8)].try_into().unwrap())
                                            as usize;
                                    o += 8;

                                    if role == 1 {
                                        let width: i32 = u32::from_le_bytes(
                                            bytes[o..(o + 4)].try_into().unwrap(),
                                        )
                                            as i32;
                                        o += 4;
                                        let height: i32 = u32::from_le_bytes(
                                            bytes[o..(o + 4)].try_into().unwrap(),
                                        )
                                            as i32;
                                        o += 4;

                                        let vert_scale: i32 = u32::from_le_bytes(
                                            bytes[o..(o + 4)].try_into().unwrap(),
                                        )
                                            as i32;
                                        o += 4;
                                        let horiz_scale: i32 = u32::from_le_bytes(
                                            bytes[o..(o + 4)].try_into().unwrap(),
                                        )
                                            as i32;
                                        o += 4;
                                        layer.sixels.push(Sixel::from_data(
                                            (width, height),
                                            vert_scale as i32,
                                            horiz_scale as i32,
                                            bytes[o..].to_vec(),
                                        ));
                                        result.layers.push(layer);
                                    } else {
                                        let mut p = parsers::ansi::Parser::default();
                                        let mut caret = Caret::default();
                                        result.layers.push(layer);
                                        if bytes.len() < o + length {
                                            return Err(Box::new(io::Error::new(
                                                io::ErrorKind::InvalidData,
                                                format!(
                                                    "data length out ouf bounds {} data lenth: {}",
                                                    o + length,
                                                    bytes.len()
                                                ),
                                            )));
                                        }
                                        (o..(o + length)).for_each(|i| {
                                            let b = bytes[i];
                                            let current_layer =
                                                result.layers.len().saturating_sub(1);
                                            let _ = p.print_char(
                                                &mut result,
                                                current_layer,
                                                &mut caret,
                                                char::from_u32(b as u32).unwrap(),
                                            );
                                        });
                                    }
                                }
                            }
                        }
                        last_info = info.compressed_latin1_text.len();
                    }
                }
                Err(err) => {
                    return Err(Box::new(LoadingError::InvalidPng(format!("{err}"))));
                }
            }
        }

        Ok(result)
    }
}

fn read_utf8_encoded_string(data: &[u8]) -> (String, usize) {
    let size = u32::from_le_bytes(data[0..4].try_into().unwrap()) as usize;
    (
        unsafe { String::from_utf8_unchecked(data[4..(4 + size)].to_vec()) },
        size + 4,
    )
}

fn write_utf8_encoded_string(data: &mut Vec<u8>, s: &str) {
    data.extend(u32::to_le_bytes(s.len() as u32));
    data.extend(s.as_bytes());
}

const MAX_LINES: i32 = 80;

impl Buffer {
    pub fn is_line_empty(&self, line: i32) -> bool {
        for i in 0..self.get_width() {
            if !self.get_char((i, line)).is_transparent() {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Clone)]
pub enum IcedError {
    ErrorEncodingZText(String),
    ErrorParsingFontSlot(String),
}

impl std::fmt::Display for IcedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IcedError::ErrorEncodingZText(err) => {
                write!(f, "Error while encoding ztext chunk: {err}")
            }
            IcedError::ErrorParsingFontSlot(err) => {
                write!(f, "Error while parsing font slot: {err}")
            }
        }
    }
}

impl Error for IcedError {
    fn description(&self) -> &str {
        "use std::display"
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}
