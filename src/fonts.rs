use crate::{EngineResult, ParserError};
use std::{
    collections::HashMap,
    error::Error,
    fmt::Display,
    io::{self},
    path::Path,
};

use super::Size;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BitFontType {
    BuiltIn,
    Library,
    Custom,
}

#[derive(Debug, Clone)]
pub struct Glyph {
    pub data: Vec<u8>,
}

impl Display for Glyph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        for b in &self.data {
            for i in 0..8 {
                if *b & (128 >> i) == 0 {
                    s.push('-');
                } else {
                    s.push('#');
                }
            }
            s.push('\n');
        }
        write!(f, "{s}---")
    }
}

impl Glyph {
    /// .
    ///
    /// # Panics
    ///
    /// Panics if .
    pub fn from_clipbard_data(data: &[u8]) -> (Size, Self) {
        let width = u16::from_le_bytes(data[0..2].try_into().unwrap());
        let height = u16::from_le_bytes(data[2..4].try_into().unwrap());
        let mut glyph = Glyph {
            data: vec![0; height as usize],
        };
        glyph.data = data[4..].to_vec();
        ((width, height).into(), glyph)
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BitFont {
    pub name: String,
    pub size: Size,
    pub length: i32,
    font_type: BitFontType,
    glyphs: HashMap<char, Glyph>,
}

impl Default for BitFont {
    fn default() -> Self {
        BitFont::from_ansi_font_page(0).unwrap()
    }
}

impl BitFont {
    /// .
    ///
    /// # Panics
    ///
    /// Panics if .
    pub fn get_clipboard_data(&self, ch: char) -> Option<Vec<u8>> {
        let Some(glyph) = self.get_glyph(ch) else {
            return None;
        };

        let mut data = Vec::new();
        data.extend_from_slice(&u16::to_le_bytes(self.size.width as u16));
        data.extend_from_slice(&u16::to_le_bytes(self.size.height as u16));
        data.extend_from_slice(&glyph.data);
        Some(data)
    }

    pub fn font_type(&self) -> BitFontType {
        self.font_type
    }

    pub fn is_default(&self) -> bool {
        self.name == DEFAULT_FONT_NAME
    }

    pub fn set_glyphs_from_u8_data(&mut self, data: &[u8]) {
        for ch in 0..self.length {
            let o = (ch * self.size.height) as usize;
            let glyph = Glyph {
                data: data[o..(o + self.size.height as usize)].into(),
            };
            self.glyphs
                .insert(unsafe { char::from_u32_unchecked(ch as u32) }, glyph);
        }
    }

    pub fn convert_to_u8_data(&self, data: &mut Vec<u8>) {
        for ch in 0..self.length {
            if let Some(glyph) = self.get_glyph(unsafe { char::from_u32_unchecked(ch as u32) }) {
                data.extend_from_slice(&glyph.data);
            }
        }
    }

    pub fn get_glyph(&self, ch: char) -> Option<&Glyph> {
        self.glyphs.get(&ch)
    }

    pub fn get_glyph_mut(&mut self, ch: char) -> Option<&mut Glyph> {
        self.glyphs.get_mut(&ch)
    }

    pub fn create_8(name: impl Into<String>, width: u8, height: u8, data: &[u8]) -> Self {
        let mut r = BitFont {
            name: name.into(),
            size: (width, height).into(),
            length: 256,
            font_type: BitFontType::Custom,
            glyphs: HashMap::new(),
        };
        r.set_glyphs_from_u8_data(data);

        r
    }

    pub fn from_basic(width: u8, height: u8, data: &[u8]) -> Self {
        let mut r = BitFont {
            name: String::new(),
            size: (width, height).into(),
            length: 256,
            font_type: BitFontType::Custom,
            glyphs: HashMap::new(),
        };
        r.set_glyphs_from_u8_data(data);
        r
    }

    const PSF1_MAGIC: u16 = 0x0436;
    const PSF1_MODE512: u8 = 0x01;
    // const PSF1_MODEHASTAB: u8 = 0x02;
    // const PSF1_MODEHASSEQ: u8 = 0x04;
    // const PSF1_MAXMODE: u8 = 0x05;

    fn load_psf1(font_name: impl Into<String>, data: &[u8]) -> Self {
        let mode = data[2];
        let charsize = data[3];
        let length = if mode & BitFont::PSF1_MODE512 == BitFont::PSF1_MODE512 {
            512
        } else {
            256
        };

        let mut r = BitFont {
            name: font_name.into(),
            size: (8, charsize).into(),
            length,
            font_type: BitFontType::BuiltIn,
            glyphs: HashMap::new(),
        };
        r.set_glyphs_from_u8_data(&data[4..]);
        r
    }

    fn load_plain_font(font_name: impl Into<String>, data: &[u8]) -> EngineResult<Self> {
        let size = match data.len() / 256 {
            8 => Size::new(8, 8),
            14 => Size::new(8, 14),
            16 => Size::new(8, 16),
            19 => Size::new(8, 19),
            _ => {
                return Err(Box::new(FontError::UnknownFontFormat(data.len())));
            }
        };

        let mut r: BitFont = BitFont {
            name: font_name.into(),
            size,
            length: 256,
            font_type: BitFontType::BuiltIn,
            glyphs: HashMap::new(),
        };
        r.set_glyphs_from_u8_data(data);
        Ok(r)
    }

    const PSF2_MAGIC: u32 = 0x864a_b572;
    // bits used in flags
    //const PSF2_HAS_UNICODE_TABLE: u8 = 0x01;
    // max version recognized so far
    const PSF2_MAXVERSION: u32 = 0x00;
    // UTF8 separators
    //const PSF2_SEPARATOR: u8 = 0xFF;
    //const PSF2_STARTSEQ: u8 = 0xFE;

    fn load_psf2(font_name: impl Into<String>, data: &[u8]) -> EngineResult<Self> {
        let version = u32::from_le_bytes(data[4..8].try_into().unwrap());
        if version > BitFont::PSF2_MAXVERSION {
            return Err(Box::new(FontError::UnsupportedVersion(version)));
        }
        let headersize = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;
        // let flags = u32::from_le_bytes(data[12..16].try_into().unwrap());
        let length = u32::from_le_bytes(data[16..20].try_into().unwrap()) as i32;
        let charsize = u32::from_le_bytes(data[20..24].try_into().unwrap()) as i32;
        if length * charsize + headersize as i32 != data.len() as i32 {
            return Err(Box::new(FontError::LengthMismatch(
                data.len(),
                (length * charsize) as usize + headersize,
            )));
        }
        let height = u32::from_le_bytes(data[24..28].try_into().unwrap()) as usize;
        let width = u32::from_le_bytes(data[28..32].try_into().unwrap()) as usize;

        let mut r = BitFont {
            name: font_name.into(),
            size: (width, height).into(),
            length,
            font_type: BitFontType::BuiltIn,
            glyphs: HashMap::new(),
        };
        r.set_glyphs_from_u8_data(&data[headersize..]);
        Ok(r)
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
    pub fn to_psf2_bytes(&self) -> EngineResult<Vec<u8>> {
        let mut data = Vec::new();
        // Write PSF2 header.
        data.extend(u32::to_le_bytes(BitFont::PSF2_MAGIC)); // magic
        data.extend(u32::to_le_bytes(0)); // version
        data.extend(u32::to_le_bytes(8 * 4)); // headersize
        data.extend(u32::to_le_bytes(0)); // flags
        data.extend(u32::to_le_bytes(self.length as u32)); // length
        data.extend(u32::to_le_bytes(self.size.height as u32)); // charsize
        data.extend(u32::to_le_bytes(self.size.height as u32)); // height
        data.extend(u32::to_le_bytes(self.size.width as u32)); // width

        // glyphs
        for i in 0..self.length {
            data.extend(
                &self
                    .get_glyph(unsafe { char::from_u32_unchecked(i as u32) })
                    .unwrap()
                    .data,
            );
        }

        Ok(data)
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
    pub fn from_bytes(font_name: impl Into<String>, data: &[u8]) -> EngineResult<Self> {
        let magic16 = u16::from_le_bytes(data[0..2].try_into().unwrap());
        if magic16 == BitFont::PSF1_MAGIC {
            return Ok(BitFont::load_psf1(font_name, data));
        }

        let magic32 = u32::from_le_bytes(data[0..4].try_into().unwrap());
        if magic32 == BitFont::PSF2_MAGIC {
            return BitFont::load_psf2(font_name, data);
        }

        BitFont::load_plain_font(font_name, data)
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
    pub fn import(path: &Path) -> io::Result<String> {
        let file_name = path.file_name();
        if file_name.is_none() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid file name",
            ));
        }
        let file_name = file_name.unwrap().to_str();
        if file_name.is_none() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid file name",
            ));
        }
        panic!("todo");
        /*
        let file_name = file_name.unwrap();

        if let Some(font_path) = unsafe { &WORKSPACE.settings.console_font_path } {
            let dest_file = font_path.join(file_name);

            if dest_file.exists() {
                return Err(io::Error::new(io::ErrorKind::InvalidData, format!("'{}' already exists", dest_file.to_str().unwrap())));
            }

            fs::copy(path, dest_file)?;

            let prefix  = path.file_stem().unwrap().to_str().unwrap().to_string();
            unsafe { ALL_FONTS.push(prefix.clone()); }
            Ok(prefix)
        } else {
            Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid font path"))
        } */
    }
}

macro_rules! fonts {
    ($( ($i:ident, $file:expr, $name: expr, $width:expr, $height:expr $(, $font_slot:expr)? ) ),* $(,)? ) => {

        $(
            pub const $i: &[u8] = include_bytes!(concat!("../data/fonts/", $file));
        )*

        impl BitFont {
            /// .
            ///
            /// # Panics
            ///
            /// Panics if .
            ///
            /// # Errors
            ///
            /// This function will return an error if .
            pub fn from_ansi_font_page(font_page: usize) -> EngineResult<Self> {
                match font_page {
                    $(
                        $( $font_slot => {BitFont::from_bytes($name, $i)}  )?
                    )*
                    _ => Err(Box::new(ParserError::UnsupportedFont(font_page))),
                }
            }
        }

        pub const FONT_NAMES: &[&str] = &[
            $(
                $name,
            )*
        ];
    };
}

const DEFAULT_FONT_NAME: &str = "Codepage 437 English";
pub const ANSI_FONTS: usize = 42;

fonts![
    (CP437, "Ansi/cp437_8x16.psf", DEFAULT_FONT_NAME, 8, 16, 0),
    (
        CP1251,
        "Ansi/cp1251_swiss.f16",
        "Codepage 1251 Cyrillic, (swiss)",
        8,
        16,
        1
    ),
    (KOI8_R, "Ansi/KOI8-R.F16", "Russian koi8-r", 8, 16, 2),
    (
        ISO8859,
        "Ansi/ISO-8859-2_Central_European_8x16.f16",
        "ISO-8859-2 Central European",
        8,
        16,
        3
    ),
    (
        ISO8859_BALTIC_9BIT,
        "Ansi/ISO-8859-4_Baltic_wide_VGA_9bit_mapped_8x16.f16",
        "ISO-8859-4 Baltic wide (VGA 9bit mapped)",
        8,
        16,
        4
    ),
    (
        CP866,
        "Ansi/cp866_russian.psf",
        "Codepage 866 (c) Russian",
        8,
        16,
        5
    ),
    (
        CP8859_T,
        "Ansi/ISO-8859-9_Turkish_8x16.f16",
        "ISO-8859-9 Turkish",
        8,
        16,
        6
    ),
    (HAIK8, "Ansi/HAIK8.F16", "haik8 codepage", 8, 16, 7),
    (
        ISO8859_HEB,
        "Ansi/ISO-8859-8_Hebrew_8x16.f16",
        "ISO-8859-8 Hebrew",
        8,
        16,
        8
    ),
    (
        KOI8_U,
        "Ansi/Ukrainian_font_koi8-u_8x16.f16",
        "Ukrainian font koi8-u",
        8,
        16,
        9
    ),
    (
        ISO8859_WE,
        "Ansi/ISO-8859-15_West_European_thin_8x16.f16",
        "ISO-8859-15 West European, (thin)",
        8,
        16,
        10
    ),
    (
        ISO8859_4_BALTIC,
        "Ansi/ISO-8859-4_Baltic_VGA_9bit_mapped_8x16.f16",
        "ISO-8859-4 Baltic (VGA 9bit mapped)",
        8,
        16,
        11
    ),
    (
        KOI8_R_B,
        "Ansi/Russian_koi8-r_b_8x16.f16",
        "Russian koi8-r (b)",
        8,
        16,
        12
    ),
    (
        ISO8859_BW,
        "Ansi/ISO-8859-4_Baltic_wide_8x16.f16",
        "ISO-8859-4 Baltic wide",
        8,
        16,
        13
    ),
    (
        ISO8859_5,
        "Ansi/ISO-8859-5_Cyrillic_8x16.f16",
        "ISO-8859-5 Cyrillic",
        8,
        16,
        14
    ),
    (
        ARMSCII_8,
        "Ansi/ARMSCII-8_Character_set_8x16.f16",
        "ARMSCII-8 Character set",
        8,
        16,
        15
    ),
    (
        ISO8859_15,
        "Ansi/ISO-8859-15_West_European_8x16.f16",
        "ISO-8859-15 West European",
        8,
        16,
        16
    ),
    (
        CP850_LI,
        "Ansi/Codepage_850_Multilingual_Latin_I_thin_8x16.f16",
        "Codepage 850 Multilingual Latin I, (thin)",
        8,
        16,
        17
    ),
    (
        CP850_ML,
        "Ansi/Codepage_850_Multilingual_Latin_I_8x16.f16",
        "Codepage 850 Multilingual Latin I",
        8,
        16,
        18
    ),
    (
        CP865,
        "Ansi/Codepage_865_Norwegian_thin_8x16.f16",
        "Codepage 865 Norwegian, (thin)",
        8,
        16,
        19
    ),
    (
        CP1251_CYR,
        "Ansi/Codepage_1251_Cyrillic_8x16.f16",
        "Codepage 1251 Cyrillic",
        8,
        16,
        20
    ),
    (
        ISO8859_7,
        "Ansi/ISO-8859-7_Greek_8x16.f16",
        "ISO-8859-7 Greek",
        8,
        16,
        21
    ),
    (
        KOI8_RC,
        "Ansi/Russian_koi8-r_c_8x16.f16",
        "Russian koi8-r (c)",
        8,
        16,
        22
    ),
    (
        ISO8859_4_BALTIC2,
        "Ansi/ISO-8859-4_Baltic_8x16.f16",
        "ISO-8859-4 Baltic",
        8,
        16,
        23
    ),
    (
        ISO8859_1_WE,
        "Ansi/ISO-8859-1_West_European_8x16.f16",
        "ISO-8859-1 West European",
        8,
        16,
        24
    ),
    (
        CP886_RUS,
        "Ansi/Codepage_866_Russian_8x16.f16",
        "Codepage 866 Russian",
        8,
        16,
        25
    ),
    (
        CP437_THIN,
        "Ansi/Codepage_437_English_thin_8x16.f16",
        "Codepage 437 English, (thin)",
        8,
        16,
        26
    ),
    (
        CP866_R,
        "Ansi/Codepage_866_b_Russian_8x16.f16",
        "Codepage 866 (b) Russian",
        8,
        16,
        27
    ),
    (
        CP865_NOR,
        "Ansi/Codepage_865_Norwegian_8x16.f16",
        "Codepage 865 Norwegian",
        8,
        16,
        28
    ),
    (
        CP866U,
        "Ansi/Ukrainian_font_cp866u_8x16.f16",
        "Ukrainian font cp866u",
        8,
        16,
        29
    ),
    (
        ISO8859_1_WE_T,
        "Ansi/ISO-8859-1_West_European_thin_8x16.f16",
        "ISO-8859-1 West European, (thin)",
        8,
        16,
        30
    ),
    (
        CP1131_BEL,
        "Ansi/Codepage_1131_Belarusian_swiss_8x16.f16",
        "Codepage 1131 Belarusian, (swiss)",
        8,
        16,
        31
    ),
    (
        C64_UPPER,
        "Commodore/C64_PETSCII_shifted.psf",
        "Commodore 64 (UPPER)",
        8,
        8,
        32
    ),
    (
        C64_LOWER,
        "Commodore/C64_PETSCII_unshifted.psf",
        "Commodore 64 (Lower)",
        8,
        8,
        33
    ),
    (
        C128_UPPER,
        "Commodore/Commodore_128_UPPER_8x16.f16",
        "Commodore 128 (UPPER)",
        8,
        8,
        34
    ),
    (
        C128_LOWER,
        "Commodore/Commodore_128_Lower_8x16.f16",
        "Commodore 128 (Lower)",
        8,
        8,
        35
    ),
    (ATARI, "Atari/Atari_ATASCII.psf", "Atari", 8, 8, 36),
    (
        AMIGA_P0T_NOODLE,
        "Amiga/P0T-NOoDLE.psf",
        "P0T NOoDLE (Amiga)",
        8,
        16,
        37
    ),
    (
        AMIGA_MOSOUL,
        "Amiga/mOsOul.psf",
        "mO'sOul (Amiga)",
        8,
        16,
        38
    ),
    (
        AMIGA_MICROKNIGHTP,
        "Amiga/MicroKnight+.psf",
        "MicroKnight Plus (Amiga)",
        8,
        16,
        39
    ),
    (
        AMIGA_TOPAZ_1P,
        "Amiga/Topaz1+.psf",
        "Topaz Plus (Amiga)",
        8,
        16,
        40
    ),
    (
        AMIGA_MICROKNIGHT,
        "Amiga/MicroKnight.psf",
        "MicroKnight (Amiga)",
        8,
        16,
        41
    ),
    (
        AMIGA_TOPAZ_1,
        "Amiga/Topaz1.psf",
        "Topaz (Amiga)",
        8,
        16,
        42
    ),
    (VIEWDATA, "Viewdata/saa5050.psf", "Viewdata", 6, 16),
];

#[derive(Debug, Clone)]
pub enum FontError {
    FontNotFound,
    MagicNumberMismatch,
    UnsupportedVersion(u32),
    LengthMismatch(usize, usize),
    UnknownFontFormat(usize),
}
impl std::fmt::Display for FontError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FontError::FontNotFound => write!(f, "font not found."),
            FontError::MagicNumberMismatch => write!(f, "not a valid .psf file."),
            FontError::UnsupportedVersion(ver) => write!(f, "version {ver} not supported"),
            FontError::LengthMismatch(actual, calculated) => {
                write!(f, "length should be {calculated} was {actual}")
            }
            FontError::UnknownFontFormat(size) => {
                let sizes = [8, 14, 16, 19];
                let list = sizes.iter().fold(String::new(), |a, &b| {
                    let empty = a.is_empty();
                    a + &format!(
                        "{}{} height ({} bytes)",
                        if empty { "" } else { ", " },
                        b,
                        &(b * 256)
                    )
                });

                write!(
                    f,
                    "Unknown binary font format {size} bytes not supported. Valid format heights are: {list}"
                )
            }
        }
    }
}

impl Error for FontError {
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
