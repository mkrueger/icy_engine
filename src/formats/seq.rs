use std::path::Path;

use super::SaveOptions;
use crate::{petscii, BitFont, Buffer, BufferFeatures, BufferParser, Caret, EngineResult, OutputFormat, Palette, C64_DEFAULT_PALETTE, C64_LOWER, C64_UPPER, TextPane, AttributedChar};

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
        result.set_font(0, BitFont::from_bytes("", C64_UPPER).unwrap());
        result.set_font(1, BitFont::from_bytes("", C64_LOWER).unwrap());

        for y in 0..result.get_height() {
            for x in 0..result.get_width() {
                let mut ch = AttributedChar::default();
                ch.attribute.set_foreground(14);
                ch.attribute.set_background(6);
                result.layers[0].set_char((x, y), ch);
            }
        }
        result.palette = Palette::from_slice(&C64_DEFAULT_PALETTE);
        result.buffer_type = crate::BufferType::Petscii;
        result.is_terminal_buffer = false;
        result.file_name = Some(file_name.into());
        result.set_sauce(sauce_opt, true);

        let mut p = petscii::Parser::default();
        let mut caret = Caret::default();
        caret.set_foreground(14);
        caret.set_background(6);
        for ch in data.iter() {
            let _ = p.print_char(&mut result, 0, &mut caret, *ch as char);
        }
        Ok(result)
    }
}
