use std::path::Path;

use super::{Position, SaveOptions, TextAttribute};
use crate::{
    AttributedChar, Buffer, BufferFeatures, EngineResult, LoadingError, OutputFormat, TextPane,
};

#[derive(Default)]
pub(super) struct Bin {}

impl OutputFormat for Bin {
    fn get_file_extension(&self) -> &str {
        "bin"
    }

    fn get_name(&self) -> &str {
        "Bin"
    }

    fn analyze_features(&self, _features: &BufferFeatures) -> String {
        String::new()
    }

    fn to_bytes(&self, buf: &crate::Buffer, options: &SaveOptions) -> EngineResult<Vec<u8>> {
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

    fn load_buffer(
        &self,
        file_name: &Path,
        data: &[u8],
        sauce_opt: Option<crate::SauceData>,
    ) -> EngineResult<crate::Buffer> {
        let mut result = Buffer::new((160, 25));
        result.is_terminal_buffer = true;
        result.file_name = Some(file_name.into());
        if let Some(sauce) = sauce_opt {
            result.set_sauce(sauce);
        }
        let mut o = 0;
        let mut pos = Position::default();
        loop {
            for _ in 0..result.get_width() {
                if o >= data.len() {
                    result.set_height_for_pos(pos);
                    return Ok(result);
                }

                if o + 1 > data.len() {
                    return Err(Box::new(LoadingError::FileLengthNeedsToBeEven));
                }

                result.layers[0].set_height(pos.y + 1);
                result.layers[0].set_char(
                    pos,
                    AttributedChar::new(
                        char::from_u32(data[o] as u32).unwrap(),
                        TextAttribute::from_u8(data[o + 1], result.buffer_type),
                    ),
                );
                pos.x += 1;
                o += 2;
            }
            pos.x = 0;
            pos.y += 1;
        }
    }
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
