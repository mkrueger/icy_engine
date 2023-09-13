mod ansi;

use std::{error::Error, path::Path};

pub use ansi::*;
pub use ansi::*;

mod pcboard;
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

use crate::{Buffer, BufferFeatures, BufferParser, Caret, EngineResult, TextPane};

use super::{Position, TextAttribute};

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum ScreenPreperation {
    #[default]
    None,
    ClearScreen,
    Home,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum CompressionLevel {
    #[default]
    Off,
    Medium,
    High,
}

#[derive(Clone, Debug)]
pub struct SaveOptions {
    pub screen_preparation: ScreenPreperation,
    pub modern_terminal_output: bool,
    pub save_sauce: bool,
    pub compression_level: CompressionLevel,
    pub output_line_length: Option<usize>,
}

impl SaveOptions {
    pub fn new() -> Self {
        SaveOptions {
            screen_preparation: ScreenPreperation::None,
            modern_terminal_output: false,
            save_sauce: false,
            compression_level: CompressionLevel::High,
            output_line_length: None,
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
    fn get_name(&self) -> &str;

    fn analyze_features(&self, _features: &BufferFeatures) -> String {
        String::new()
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    fn to_bytes(&self, buf: &crate::Buffer, options: &SaveOptions) -> EngineResult<Vec<u8>>;

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
    ) -> EngineResult<crate::Buffer>;
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
    bytes: &[u8],
    skip_errors: bool,
) -> EngineResult<()> {
    result.layers[0].lines.clear();
    let mut caret = Caret::default();
    if let Some(sauce) = &result.sauce_data {
        caret.set_ice_mode(sauce.use_ice);
    }

    for b in bytes {
        if let Some(ch) = char::from_u32(*b as u32) {
            let res = interpreter.print_char(result, 0, &mut caret, ch);
            if !skip_errors && res.is_err() {
                res?;
            }
        }
    }

    // crop last empty line (if any)
    while result.layers[0].lines.len() > 1 && result.layers[0].lines.last().unwrap().chars.is_empty() {
        result.layers[0].lines.pop();
    }
    result.set_height(result.get_line_count());
    Ok(())
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
