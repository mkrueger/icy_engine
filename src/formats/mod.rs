mod ansi;

use std::{error::Error, path::Path, thread, time::Duration};

pub use ansi::*;
pub use ansi::*;

mod pcboard;
use i18n_embed_fl::fl;
pub use pcboard::*;

mod avatar;
pub use avatar::*;

mod ascii;
pub use ascii::*;

mod bin;
pub use bin::*;

mod xbinary;
pub use xbinary::*;

mod artworx;
pub use artworx::*;

mod ice_draw;
pub use ice_draw::*;

mod tundra;
pub use tundra::*;

mod icy_draw;

use crate::{
    BitFont, Buffer, BufferFeatures, BufferParser, BufferType, Caret, EngineResult, Layer, Role,
    Size, TextPane, ANSI_FONTS, SAUCE_FONT_NAMES,
};

use super::{Position, TextAttribute};

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum ScreenPreperation {
    #[default]
    None,
    ClearScreen,
    Home,
}

#[derive(Clone, Debug)]
pub struct SaveOptions {
    pub screen_preparation: ScreenPreperation,
    pub buffer_type: BufferType,
    pub modern_terminal_output: bool,
    pub save_sauce: bool,
    pub compress: bool,
    pub output_line_length: Option<usize>,
    pub preserve_invisible_chars: bool,
}

impl SaveOptions {
    pub fn new() -> Self {
        SaveOptions {
            screen_preparation: ScreenPreperation::None,
            buffer_type: BufferType::CP437,
            modern_terminal_output: false,
            save_sauce: false,
            compress: true,
            output_line_length: None,
            preserve_invisible_chars: false,
        }
    }
}

impl Default for SaveOptions {
    fn default() -> Self {
        Self::new()
    }
}

pub trait OutputFormat: Send + Sync {
    fn get_file_extension(&self) -> &str;

    fn get_alt_extensions(&self) -> Vec<String> {
        Vec::new()
    }

    fn get_name(&self) -> &str;

    fn analyze_features(&self, _features: &BufferFeatures) -> String {
        String::new()
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    fn to_bytes(&self, buf: &crate::Buffer, options: &SaveOptions) -> anyhow::Result<Vec<u8>>;

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    fn load_buffer(
        &self,
        file_name: &Path,
        data: &[u8],
        sauce_opt: Option<crate::SauceData>,
    ) -> anyhow::Result<crate::Buffer>;
}

lazy_static::lazy_static! {
    pub static ref FORMATS: [Box<dyn OutputFormat>; 10] = [
        Box::<ansi::Ansi>::default(),
        Box::<icy_draw::IcyDraw>::default(),
        Box::<IceDraw>::default(),
        Box::<Bin>::default(),
        Box::<XBin>::default(),
        Box::<TundraDraw>::default(),
        Box::<PCBoard>::default(),
        Box::<Avatar>::default(),
        Box::<ascii::Ascii>::default(),
        Box::<artworx::Artworx>::default(),
    ];
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
pub fn parse_with_parser(
    result: &mut Buffer,
    interpreter: &mut dyn BufferParser,
    text: &str,
    skip_errors: bool,
) -> EngineResult<()> {
    result.layers[0].lines.clear();
    let mut caret = Caret::default();
    if let Some(sauce) = &result.get_sauce() {
        caret.set_ice_mode(sauce.use_ice);
    }

    for ch in text.chars() {
        let res = interpreter.print_char(result, 0, &mut caret, ch);
        if !skip_errors && res.is_err() {
            res?;
        }
    }

    // transform sixels to layers
    while !result.sixel_threads.is_empty() {
        thread::sleep(Duration::from_millis(50));
        result.update_sixel_threads()?;
    }
    let mut num = 0;
    while !result.layers[0].sixels.is_empty() {
        if let Some(mut sixel) = result.layers[0].sixels.pop() {
            let size = sixel.get_size();
            let font_size = result.get_font_dimensions();
            let size = Size::new(
                (size.width + font_size.width - 1) / font_size.width,
                (size.height + font_size.height - 1) / font_size.height,
            );
            num += 1;
            let mut layer = Layer::new(
                fl!(
                    crate::LANGUAGE_LOADER,
                    "layer-new-sixel_layer_name",
                    number = num
                ),
                size,
            );
            layer.role = Role::Image;
            layer.set_offset(sixel.position);
            sixel.position = Position::default();
            layer.sixels.push(sixel);
            result.layers.push(layer);
        }
    }

    // crop last empty line (if any)
    crop_loaded_file(result);
    Ok(())
}

pub(crate) fn crop_loaded_file(result: &mut Buffer) {
    while result.layers[0].lines.len() > 1
        && result.layers[0].lines.last().unwrap().chars.is_empty()
    {
        result.layers[0].lines.pop();
    }
    let height = result.get_line_count();
    result.layers[0].set_height(height);
    result.set_height(height);
}

#[derive(Debug, Clone)]
pub enum LoadingError {
    OpenFileError(String),
    Error(String),
    ReadFileError(String),
    FileTooShort,
    IcyDrawUnsupportedLayerMode(u8),
    InvalidPng(String),
    UnsupportedADFVersion(u8),
    FileLengthNeedsToBeEven,
    IDMismatch,
    OutOfBounds,
}

impl std::fmt::Display for LoadingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadingError::Error(err) => write!(f, "Error while loading: {err}"),
            LoadingError::OpenFileError(err) => write!(f, "Error while opening file: {err}"),
            LoadingError::ReadFileError(err) => write!(f, "Error while reading file: {err}"),
            LoadingError::FileTooShort => write!(f, "File too short"),
            LoadingError::UnsupportedADFVersion(version) => {
                write!(f, "Unsupported ADF version: {version}")
            }
            LoadingError::IcyDrawUnsupportedLayerMode(mode) => {
                write!(f, "Unsupported layer mode: {mode}")
            }
            LoadingError::InvalidPng(err) => write!(f, "Error decoding PNG: {err}"),
            LoadingError::FileLengthNeedsToBeEven => write!(f, "File length needs to be even"),
            LoadingError::IDMismatch => write!(f, "ID mismatch"),
            LoadingError::OutOfBounds => write!(f, "Out of bounds"),
        }
    }
}

impl Error for LoadingError {
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

#[derive(Debug, Clone)]
pub enum SavingError {
    NoFontFound,
    Only8x16FontsSupported,
    InvalidXBinFont,
}

impl std::fmt::Display for SavingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SavingError::NoFontFound => write!(f, "No font found"),
            SavingError::Only8x16FontsSupported => write!(f, "Only 8x16 fonts are supported by this format."),
            SavingError::InvalidXBinFont => write!(f, "font not supported by the .xb format only fonts with 8px width and a height from 1 to 32 are supported."),

        }
    }
}
impl Error for SavingError {
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

#[cfg(test)]
mod tests {
    use crate::{Buffer, OutputFormat, SaveOptions};
    use std::path::PathBuf;

    fn test_ansi(data: &[u8]) {
        let buf = Buffer::from_bytes(&PathBuf::from("test.ans"), false, data).unwrap();
        let converted = super::Ansi::default()
            .to_bytes(&buf, &SaveOptions::new())
            .unwrap();

        // more gentle output.
        let b: Vec<u8> = converted
            .iter()
            .map(|&x| if x == 27 { b'x' } else { x })
            .collect();
        let converted = String::from_utf8_lossy(b.as_slice());

        let b: Vec<u8> = data
            .iter()
            .map(|&x| if x == 27 { b'x' } else { x })
            .collect();
        let expected = String::from_utf8_lossy(b.as_slice());

        assert_eq!(expected, converted);
    }

    #[test]
    fn test_space_compression() {
        let data = b"\x1B[0mA A  A   A    A\x1B[5CA\x1B[6CA\x1B[8CA";
        test_ansi(data);
    }

    #[test]
    fn test_fg_color_change() {
        let data = b"\x1B[0ma\x1B[32ma\x1B[33ma\x1B[1ma\x1B[35ma\x1B[0;35ma\x1B[1;32ma\x1B[0;36ma";
        test_ansi(data);
    }

    #[test]
    fn test_bg_color_change() {
        let data = b"\x1B[0mA\x1B[44mA\x1B[45mA\x1B[31;40mA\x1B[42mA\x1B[40mA\x1B[1;46mA\x1B[0mA\x1B[1;47mA\x1B[0;47mA";
        test_ansi(data);
    }

    #[test]
    fn test_blink_change() {
        let data = b"\x1B[0mA\x1B[5mA\x1B[0mA\x1B[1;5;42mA\x1B[0;1;42mA\x1B[0;5mA\x1B[0;36mA\x1B[5;33mA\x1B[0;1mA";
        test_ansi(data);
    }

    #[test]
    fn test_eol_skip() {
        let data = b"\x1B[0;1m\x1B[79Cdd";
        test_ansi(data);
    }

    #[test]
    fn test_23bit() {
        let data = b"\x1B[0m\x1B[1;24;12;200t#";
        test_ansi(data);
        let data = b"\x1B[0m\x1B[0;44;2;120t#";
        test_ansi(data);
    }

    #[test]
    fn test_extended_color() {
        let data = b"\x1B[0;38;5;42m#";
        test_ansi(data);
        let data = b"\x1B[0;48;5;100m#";
        test_ansi(data);
    }

    #[test]
    fn test_first_char_color() {
        let data = b"\x1B[0;1;36mA";
        test_ansi(data);
        let data = b"\x1B[0;31mA";
        test_ansi(data);
        let data = b"\x1B[0;33;45mA";
        test_ansi(data);
        let data = b"\x1B[0;1;33;45mA";
        test_ansi(data);
    }
}

#[cfg(test)]
fn crop2_loaded_file(result: &mut Buffer) {
    for l in 0..result.layers.len() {
        if let Some(line) = result.layers[l].lines.last_mut() {
            while !line.chars.is_empty() && !line.chars.last().unwrap().is_visible() {
                line.chars.pop();
            }
        }

        if !result.layers[l].lines.is_empty()
            && result.layers[l].lines.last().unwrap().chars.is_empty()
        {
            result.layers[l].lines.pop();
            crop2_loaded_file(result);
        }
    }
}

#[cfg(test)]
#[derive(Clone, Copy)]
pub struct CompareOptions {
    pub compare_palette: bool,
}

#[cfg(test)]
impl CompareOptions {
    pub const ALL: CompareOptions = CompareOptions {
        compare_palette: true,
    };
}

#[cfg(test)]
pub(crate) fn compare_buffers(
    buf_old: &mut Buffer,
    buf_new: &mut Buffer,
    compare_options: CompareOptions,
) {
    use crate::AttributedChar;

    assert_eq!(buf_old.layers.len(), buf_new.layers.len());
    let ic = AttributedChar::invisible();
    assert_eq!(
        buf_old.get_size(),
        buf_new.get_size(),
        "size differs: {} != {}",
        buf_old.get_size(),
        buf_new.get_size()
    );

    crop2_loaded_file(buf_old);
    crop2_loaded_file(buf_new);

    if compare_options.compare_palette {
        assert_eq!(
            buf_old.palette.len(),
            buf_new.palette.len(),
            "palette color count differs"
        );

        assert_eq!(
            buf_old.palette.export_palette(&crate::PaletteFormat::Hex),
            buf_new.palette.export_palette(&crate::PaletteFormat::Hex)
        );
    }

    assert_eq!(buf_old.font_count(), buf_new.font_count());

    for (i, old_fnt) in buf_old.font_iter() {
        let new_fnt = buf_new.get_font(*i).unwrap();

        for (ch, glyph) in &old_fnt.glyphs {
            let new_glyph = new_fnt.glyphs.get(ch).unwrap();
            assert_eq!(
                glyph, new_glyph,
                "glyphs differ font: {i}, char: {ch} (0x{:02X})",
                *ch as u32
            );
        }
    }

    for layer in 0..buf_old.layers.len() {
        /*      assert_eq!(
            buf_old.layers[layer].lines.len(),
            buf_new.layers[layer].lines.len(),
            "layer {layer} line count differs"
        );*/
        assert_eq!(
            buf_old.layers[layer].get_offset(),
            buf_new.layers[layer].get_offset(),
            "layer {layer} offset differs"
        );
        assert_eq!(
            buf_old.layers[layer].get_size(),
            buf_new.layers[layer].get_size(),
            "layer {layer} size differs"
        );
        assert_eq!(
            buf_old.layers[layer].is_visible, buf_new.layers[layer].is_visible,
            "layer {layer} is_visible differs"
        );
        assert_eq!(
            buf_old.layers[layer].has_alpha_channel, buf_new.layers[layer].has_alpha_channel,
            "layer {layer} has_alpha_channel differs"
        );

        assert_eq!(
            buf_old.layers[layer].default_font_page, buf_new.layers[layer].default_font_page,
            "layer {layer} default_font_page differs"
        );

        for line in 0..buf_old.layers[layer].lines.len() {
            for i in 0..buf_old.layers[layer].get_width() as usize {
                let mut ch = *buf_old.layers[layer].lines[line]
                    .chars
                    .get(i)
                    .unwrap_or(&ic);
                let mut ch2 = *buf_new.layers[layer].lines[line]
                    .chars
                    .get(i)
                    .unwrap_or(&ic);
                assert_eq!(
                    buf_old
                        .palette
                        .get_color(ch.attribute.get_foreground() as usize),
                    buf_new
                        .palette
                        .get_color(ch2.attribute.get_foreground() as usize),
                    "fg differs at layer: {layer}, line: {line}, char: {i} (old:{}={}, new:{}={})",
                    ch.attribute.get_foreground(),
                    buf_old
                        .palette
                        .get_color(ch.attribute.get_foreground() as usize),
                    ch2.attribute.get_foreground(),
                    buf_new
                        .palette
                        .get_color(ch2.attribute.get_foreground() as usize)
                );
                assert_eq!(
                    buf_old
                        .palette
                        .get_color(ch.attribute.get_background() as usize),
                    buf_new
                        .palette
                        .get_color(ch2.attribute.get_background() as usize),
                    "bg differs at layer: {layer}, line: {line}, char: {i} (old:{}={}, new:{}={})",
                    ch.attribute.get_background(),
                    buf_old
                        .palette
                        .get_color(ch.attribute.get_background() as usize),
                    ch2.attribute.get_background(),
                    buf_new
                        .palette
                        .get_color(ch2.attribute.get_background() as usize)
                );

                ch.attribute.set_foreground(0);
                ch.attribute.set_background(0);

                ch2.attribute.set_foreground(0);
                ch2.attribute.set_background(0);

                assert_eq!(ch, ch2, "layer: {layer}, line: {line}, char: {i}");
            }
        }
    }
}

pub fn convert_ansi_to_utf8(data: &[u8]) -> (String, bool) {
    if data.starts_with(&[0xEF, 0xBB, 0xBF]) {
        if let Ok(result) = String::from_utf8(data.to_vec()) {
            return (result, true);
        }
    }

    // interpret CP437
    let mut result = String::new();
    for ch in data {
        let ch = *ch as char;
        result.push(ch);
    }
    (result, false)
}

pub fn guess_font_name(font: &BitFont) -> String {
    for i in 0..ANSI_FONTS {
        if let Ok(ansi_font) = BitFont::from_ansi_font_page(i) {
            if ansi_font.glyphs == font.glyphs {
                return ansi_font.name.clone();
            }
        }
    }

    for name in SAUCE_FONT_NAMES {
        if let Ok(sauce_font) = BitFont::from_sauce_name(name) {
            if sauce_font.glyphs == font.glyphs {
                return sauce_font.name.clone();
            }
        }
    }

    format!("Unknown {}x{} font", font.size.width, font.size.height)
}
