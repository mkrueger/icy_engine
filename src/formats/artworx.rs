use std::path::Path;

use super::{Position, SaveOptions, TextAttribute};
use crate::{
    guess_font_name, AttributedChar, BitFont, Buffer, BufferFeatures, BufferType, EngineResult,
    FontMode, IceMode, LoadingError, OutputFormat, Palette, SavingError, Size, TextPane,
};

// http://fileformats.archiveteam.org/wiki/ArtWorx_Data_Format

// u8                   Version
// 3 * 64 = 192 u8      Palette
// 256 * 16 = 4096 u8   Font Data (only 8x16 supported)
// [ch u8, attr u8]*    Screen data
//
// A very simple format with a weird palette storage. Only 16 colors got used but a full 64 color palette is stored.
// Maybe useful for DOS demos running in text mode.

#[derive(Default)]
pub(crate) struct Artworx {}

const HEADER_LENGTH: usize = 1 + 3 * 64 + 4096;
const VERSION: u8 = 1;

impl OutputFormat for Artworx {
    fn get_file_extension(&self) -> &str {
        "adf"
    }

    fn get_name(&self) -> &str {
        "Artworx"
    }

    fn analyze_features(&self, _features: &BufferFeatures) -> String {
        String::new()
    }

    fn to_bytes(&self, buf: &crate::Buffer, options: &SaveOptions) -> EngineResult<Vec<u8>> {
        if buf.ice_mode != IceMode::Ice {
            return Err(anyhow::anyhow!(
                "Only ice mode files are supported by this format."
            ));
        }
        if buf.get_width() != 80 {
            return Err(anyhow::anyhow!(
                "Only width==80 files are supported by this format."
            ));
        }
        let mut result = vec![1]; // version
        result.extend(buf.palette.to_ega_palette());
        if buf.get_font_dimensions().height != 16 {
            return Err(SavingError::Only8x16FontsSupported.into());
        }

        if let Some(font) = buf.get_font(0) {
            result.extend(font.convert_to_u8_data());
        } else {
            return Err(SavingError::NoFontFound.into());
        }

        for y in 0..buf.get_height() {
            for x in 0..buf.get_width() {
                let ch = buf.get_char((x, y));
                result.push(ch.ch as u8);
                result.push(ch.attribute.as_u8(IceMode::Ice));
            }
        }
        if options.save_sauce {
            buf.write_sauce_info(crate::SauceFileType::Ansi, &mut result)?;
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
        result.set_width(80);
        result.buffer_type = BufferType::CP437;
        result.palette_mode = crate::PaletteMode::Free16;
        result.ice_mode = IceMode::Ice;
        result.font_mode = FontMode::Single;
        let file_size = data.len();
        let mut o = 0;
        let mut pos = Position::default();
        if file_size < HEADER_LENGTH {
            return Err(LoadingError::FileTooShort.into());
        }

        let version = data[o];
        if version != VERSION {
            return Err(LoadingError::UnsupportedADFVersion(version).into());
        }
        o += 1;

        // convert EGA -> VGA colors.
        let palette_size = 3 * 64;
        result.palette = Palette::from(&data[o..(o + palette_size)]).cycle_ega_colors();
        o += palette_size;

        let font_size = 4096;
        result.clear_font_table();
        let mut font = BitFont::from_basic(8, 16, &data[o..(o + font_size)]);
        font.name = guess_font_name(&font);
        result.set_font(0, font);
        o += font_size;

        loop {
            for _ in 0..result.get_width() {
                if o + 2 > file_size {
                    crate::crop_loaded_file(&mut result);
                    return Ok(result);
                }
                result.layers[0].set_height(pos.y + 1);
                let attribute = TextAttribute::from_u8(data[o + 1], result.ice_mode);
                result.layers[0].set_char(
                    pos,
                    AttributedChar::new(char::from_u32(data[o] as u32).unwrap(), attribute),
                );
                pos.x += 1;
                o += 2;
            }
            pos.x = 0;
            pos.y += 1;
        }
    }
}

pub fn get_save_sauce_default_adf(buf: &Buffer) -> (bool, String) {
    if buf.get_width() != 80 {
        return (true, "width != 80".to_string());
    }

    if buf.has_sauce() {
        return (true, String::new());
    }

    (false, String::new())
}

#[cfg(test)]
mod tests {
    use crate::{compare_buffers, AttributedChar, Buffer, OutputFormat, TextAttribute, TextPane};

    #[test]
    pub fn test_ice() {
        let mut buffer = create_buffer();
        buffer.ice_mode = crate::IceMode::Ice;
        buffer.layers[0].set_char(
            (0, 0),
            AttributedChar::new(
                'A',
                TextAttribute::from_u8(0b0000_1000, crate::IceMode::Ice),
            ),
        );
        buffer.layers[0].set_char(
            (1, 0),
            AttributedChar::new(
                'B',
                TextAttribute::from_u8(0b1100_1111, crate::IceMode::Ice),
            ),
        );
        test_artworx(buffer);
    }

    fn create_buffer() -> Buffer {
        let mut buffer = Buffer::new((80, 25));
        for y in 0..buffer.get_height() {
            for x in 0..buffer.get_width() {
                buffer.layers[0]
                    .set_char((x, y), AttributedChar::new(' ', TextAttribute::default()));
            }
        }
        buffer
    }

    fn test_artworx(mut buffer: Buffer) -> Buffer {
        let xb = super::Artworx::default();
        let mut opt = crate::SaveOptions::default();
        opt.compress = false;
        let bytes = xb.to_bytes(&buffer, &opt).unwrap();
        let mut buffer2 = xb
            .load_buffer(std::path::Path::new("test.adf"), &bytes, None)
            .unwrap();
        compare_buffers(
            &mut buffer,
            &mut buffer2,
            crate::CompareOptions {
                compare_palette: false,
            },
        );
        buffer2
    }
}
