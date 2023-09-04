use std::error::Error;

use crate::EngineResult;
use arboard::{Clipboard, ImageData};

pub const BUFFER_DATA: u16 = 0x0000;
pub const BITFONT_GLYPH: u16 = 0x0100;

/// .
///
/// # Errors
///
/// This function will return an error if .
pub fn push_data(data_type: u16, data: &[u8]) -> EngineResult<()> {
    match Clipboard::new() {
        Ok(mut clipboard) => {
            let mut clipboard_data: Vec<u8> = Vec::new();
            clipboard_data.extend(b"iced");
            clipboard_data.extend(u16::to_le_bytes(data_type));
            clipboard_data.extend(data);
            while clipboard_data.len() % 4 != 0 {
                clipboard_data.push(0);
            }

            let image = ImageData {
                width: clipboard_data.len() / 4,
                height: 1,
                bytes: clipboard_data.into(),
            };
            clipboard.set_image(image)?;
            Ok(())
        }
        Err(err) => Err(Box::new(ClipboardError::Error(format!("{err}")))),
    }
}

pub fn pop_data(data_type: u16) -> Option<Vec<u8>> {
    match Clipboard::new() {
        Ok(mut clipboard) => {
            if let Ok(img) = clipboard.get_image() {
                let data = img.bytes;
                if &data[0..4] == b"iced" && (data[4] as u16 | (data[5] as u16) << 8) == data_type {
                    let mut result = Vec::new();
                    result.extend(&data[6..]);
                    return Some(result);
                }
            }
        }
        Err(err) => {
            log::error!("Error creating clipboard: {err}");
        }
    }
    None
}

#[derive(Debug, Clone)]
enum ClipboardError {
    Error(String),
}

impl std::fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClipboardError::Error(err) => write!(f, "Error creating clipboard: {err}"),
        }
    }
}

impl Error for ClipboardError {
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
