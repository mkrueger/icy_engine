use std::{any, path::Path};

use super::{Position, SaveOptions, TextAttribute};
use crate::{
    petscii, AttributedChar, BitFont, Buffer, BufferFeatures, BufferParser, Caret, EngineResult, OutputFormat, Palette, TextPane, C64_DEFAULT_PALETTE,
    C64_LOWER, C64_UPPER,
};

#[derive(Default)]
pub(super) struct Seq {}

impl OutputFormat for Seq {
    fn get_file_extension(&self) -> &str {
        "seq"
    }

    fn get_name(&self) -> &str {
        "Seq"
    }

    fn analyze_features(&self, _features: &BufferFeatures) -> String {
        String::new()
    }

    fn to_bytes(&self, buf: &crate::Buffer, _options: &SaveOptions) -> EngineResult<Vec<u8>> {
        if buf.buffer_type != crate::BufferType::Petscii {
            return Err(anyhow::anyhow!("Buffer is not a Petscii buffer!"));
        }

        Err(anyhow::anyhow!("not implemented!"))
    }

    fn load_buffer(&self, file_name: &Path, data: &[u8], sauce_opt: Option<crate::SauceData>) -> EngineResult<crate::Buffer> {
        let mut result = Buffer::new((40, 25));
        result.clear_font_table();
        result.set_font(0, BitFont::from_bytes("", C64_LOWER).unwrap());
        result.set_font(1, BitFont::from_bytes("", C64_UPPER).unwrap());
        result.palette = Palette::from_slice(&C64_DEFAULT_PALETTE);
        result.buffer_type = crate::BufferType::Petscii;
        result.is_terminal_buffer = false;
        result.file_name = Some(file_name.into());
        result.set_sauce(sauce_opt, true);

        let mut p = petscii::Parser::default();
        let mut caret = Caret::default();
        for ch in data.iter() {
            let res = p.print_char(&mut result, 0, &mut caret, *ch as char);
        }
        Ok(result)
    }
}
