use std::path::Path;

use crate::{
    parse_with_parser, parsers, Buffer, BufferFeatures, EngineResult, OutputFormat, Position,
    TextAttribute, TextPane,
};

use super::SaveOptions;

pub(crate) const HEX_TABLE: &[u8; 16] = b"0123456789ABCDEF";

#[derive(Default)]
pub(super) struct PCBoard {}

impl OutputFormat for PCBoard {
    fn get_file_extension(&self) -> &str {
        "pcb"
    }

    fn get_name(&self) -> &str {
        "PCBoard"
    }

    fn analyze_features(&self, _features: &BufferFeatures) -> String {
        String::new()
    }

    fn to_bytes(&self, buf: &crate::Buffer, options: &SaveOptions) -> EngineResult<Vec<u8>> {
        let mut result = Vec::new();
        let mut last_attr = TextAttribute::default();
        let mut pos = Position::default();
        let height = buf.get_line_count();
        let mut first_char = true;

        match options.screen_preparation {
            super::ScreenPreperation::None | super::ScreenPreperation::Home => {} // home not supported
            super::ScreenPreperation::ClearScreen => {
                result.extend(b"@CLS@");
            }
        }

        while pos.y < height {
            let line_length = buf.get_line_length(pos.y);

            while pos.x < line_length {
                let ch = buf.get_char(pos);

                if first_char || ch.attribute != last_attr {
                    result.extend_from_slice(b"@X");
                    result.push(HEX_TABLE[ch.attribute.get_background() as usize]);
                    result.push(HEX_TABLE[ch.attribute.get_foreground() as usize]);
                    last_attr = ch.attribute;
                }

                result.push(if ch.ch == '\0' { b' ' } else { ch.ch as u8 });
                first_char = false;
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
            buf.write_sauce_info(crate::SauceFileType::PCBoard, &mut result)?;
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
        result.set_sauce(sauce_opt);

        /*
                let mut interpreter: Box<dyn BufferParser> = match interpreter {
            CharInterpreter::Ansi => {
                let mut parser = Box::<parsers::ansi::Parser>::default();
                parser.bs_is_ctrl_char = false;
                parser
            }
        };
         */
        let (text, is_unicode) = crate::convert_ansi_to_utf8(data);
        if is_unicode {
            result.buffer_type = crate::BufferType::Unicode;
        }
        parse_with_parser(
            &mut result,
            &mut parsers::pcboard::Parser::default(),
            &text,
            true,
        )?;
        Ok(result)
    }
}

pub fn get_save_sauce_default_pcb(buf: &Buffer) -> (bool, String) {
    if buf.get_width() != 80 {
        return (true, "width != 80".to_string());
    }

    if buf.has_sauce() {
        return (true, String::new());
    }

    (false, String::new())
}
