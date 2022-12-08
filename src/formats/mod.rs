mod ansi;

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

mod mystic_draw;
pub use mystic_draw::*;

use super::{Position, TextAttribute};

#[derive(Clone, Copy, Debug)]
pub enum ScreenPreperation {
    None,
    ClearScreen,
    Home,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CompressionLevel {
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
}

impl SaveOptions {
    pub fn new() -> Self {
        SaveOptions {
            screen_preparation: ScreenPreperation::None,
            modern_terminal_output: false,
            save_sauce: false,
            compression_level: CompressionLevel::High,
        }
    }
}

impl Default for SaveOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Buffer, SaveOptions};
    use std::path::PathBuf;

    fn test_ansi(data: &[u8]) {
        let buf = Buffer::from_bytes(&PathBuf::from("test.ans"), data).unwrap();
        let converted = super::convert_to_ans(&buf, &SaveOptions::new()).unwrap();

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
