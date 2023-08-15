use crate::EngineResult;
use std::{
    collections::HashMap,
    error::Error,
    fmt::Display,
    io::{self},
    path::Path,
};

use super::{SauceString, Size};

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

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BitFont {
    pub name: SauceString<22, 0>,
    pub size: Size<u8>,
    pub length: usize,
    font_type: BitFontType,
    glyphs: HashMap<char, Glyph>,
}

impl Default for BitFont {
    fn default() -> Self {
        BitFont::from_name(DEFAULT_FONT_NAME).unwrap()
    }
}

static mut ALL_FONTS: Vec<String> = Vec::new();
/*
fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map_or(false, |s| s.starts_with('.'))
}

fn load_fonts()
{
    if let Some(path) = unsafe { &WORKSPACE.settings.console_font_path } {
        let walker = WalkDir::new(path).into_iter();
        for entry in walker.filter_entry(|e| !is_hidden(e)) {
            if let Err(e) = entry {
                eprintln!("Can't load tdf font library: {}", e);
                break;
            }
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                continue;
            }
            let prefix  = path.file_stem().unwrap().to_str().unwrap().to_string();
            unsafe {
                ALL_FONTS.push(prefix.to_string());
            }
        }
    }
}*/

impl BitFont {
    pub fn get_font_list() -> &'static Vec<String> {
        unsafe {
            if ALL_FONTS.is_empty() {
                for s in SUPPORTED_FONTS {
                    ALL_FONTS.push(s.to_string());
                }
                // load_fonts();
            }
            &ALL_FONTS
        }
    }

    pub fn font_type(&self) -> BitFontType {
        self.font_type
    }

    pub fn is_default(&self) -> bool {
        self.name.to_string() == DEFAULT_FONT_NAME || self.name.to_string() == ALT_DEFAULT_FONT_NAME
    }

    pub fn set_glyphs_from_u8_data(&mut self, data: &[u8]) {
        for ch in 0..self.length {
            let o = ch * self.size.height as usize;
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

    pub fn create_8(name: SauceString<22, 0>, width: u8, height: u8, data: &[u8]) -> Self {
        let mut r = BitFont {
            name,
            size: Size::new(width, height),
            length: 256,
            font_type: BitFontType::Custom,
            glyphs: HashMap::new(),
        };
        r.set_glyphs_from_u8_data(data);

        r
    }

    pub fn from_basic(width: u8, height: u8, data: &[u8]) -> Self {
        let mut r = BitFont {
            name: SauceString::EMPTY,
            size: Size::new(width, height),
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
            name: SauceString::from(font_name),
            size: Size::new(8, charsize),
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
                return Err(Box::new(FontError::MagicNumberMismatch));
            }
        };

        let mut r = BitFont {
            name: SauceString::from(font_name),
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
        let length = u32::from_le_bytes(data[16..20].try_into().unwrap()) as usize;
        let charsize = u32::from_le_bytes(data[20..24].try_into().unwrap()) as usize;
        if length * charsize + headersize != data.len() {
            return Err(Box::new(FontError::LengthMismatch(
                data.len(),
                length * charsize + headersize,
            )));
        }
        let height = u32::from_le_bytes(data[24..28].try_into().unwrap());
        let width = u32::from_le_bytes(data[28..32].try_into().unwrap());

        let mut r = BitFont {
            name: SauceString::from(font_name),
            size: Size::new(width as u8, height as u8),
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
    pub fn to_bytes(&self) -> EngineResult<Vec<u8>> {
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
    pub fn from_name(font_name: &str) -> EngineResult<Self> {
        if let Some(data) = get_font_data(font_name) {
            BitFont::from_bytes(font_name, data)
        } else {
            Err(Box::new(FontError::FontNotFound))
        }

        /* else {
            if let Some(path) = unsafe { &WORKSPACE.settings.console_font_path } {
                let walker = WalkDir::new(path).into_iter();
                for entry in walker.filter_entry(|e| !is_hidden(e)) {
                    if let Err(e) = entry {
                        eprintln!("Can't load tdf font library: {}", e);
                        break;
                    }
                    let entry = entry.unwrap();
                    let path = entry.path();

                    if path.is_dir() {
                        continue;
                    }
                    let prefix  = path.file_stem().unwrap().to_str().unwrap().to_string();
                    if prefix == font_name {
                        let mut f = File::open(path).unwrap();
                        let mut bytes = Vec::new();
                        f.read_to_end(&mut bytes).expect("error while reading file");

                        return Some(BitFont {
                            name: SauceString::from(&prefix),
                            size: len_to_size(bytes.len()),
                            font_type: BitFontType::Library,
                            data_32: None,
                            data_8: bytes
                        });
                    }
                }
            }
            None
        }*/
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

const CODEPAGE_437_ENGLISH_16: &[u8] =
    include_bytes!("../data/fonts/Codepage_437_English_8x16.f16");
const CODEPAGE_437_ENGLISH_8: &[u8] = include_bytes!("../data/fonts/Codepage_437_English_8x8.f8");
const CODEPAGE_437_ENGLISH_14: &[u8] =
    include_bytes!("../data/fonts/Codepage_437_English_8x14.f14");
const CODEPAGE_1251_CYRILLIC_SWISS_16: &[u8] =
    include_bytes!("../data/fonts/Codepage_1251_Cyrillic_swiss_8x16.f16");
const RUSSIAN_KOI8_R_16: &[u8] = include_bytes!("../data/fonts/Russian_koi8-r_8x16.f16");
const RUSSIAN_KOI8_R_8: &[u8] = include_bytes!("../data/fonts/Russian_koi8-r_8x8.f8");
const RUSSIAN_KOI8_R_14: &[u8] = include_bytes!("../data/fonts/Russian_koi8-r_8x14.f14");
const ISO_8859_2_CENTRAL_EUROPEAN_16: &[u8] =
    include_bytes!("../data/fonts/ISO-8859-2_Central_European_8x16.f16");
const ISO_8859_2_CENTRAL_EUROPEAN_8: &[u8] =
    include_bytes!("../data/fonts/ISO-8859-2_Central_European_8x8.f8");
const ISO_8859_2_CENTRAL_EUROPEAN_14: &[u8] =
    include_bytes!("../data/fonts/ISO-8859-2_Central_European_8x14.f14");
const ISO_8859_4_BALTIC_WIDE_VGA_9BIT_MAPPED_16: &[u8] =
    include_bytes!("../data/fonts/ISO-8859-4_Baltic_wide_VGA_9bit_mapped_8x16.f16");
const CODEPAGE_866_C_RUSSIAN_16: &[u8] =
    include_bytes!("../data/fonts/Codepage_866_c_Russian_8x16.f16");
const ISO_8859_9_TURKISH_16: &[u8] = include_bytes!("../data/fonts/ISO-8859-9_Turkish_8x16.f16");
const HAIK8_CODEPAGE_USE_ONLY_WITH_ARMSCII8_SCREENMAP_16: &[u8] =
    include_bytes!("../data/fonts/haik8_codepage_use_only_with_armscii8_screenmap_8x16.f16");
const HAIK8_CODEPAGE_USE_ONLY_WITH_ARMSCII8_SCREENMAP_8: &[u8] =
    include_bytes!("../data/fonts/haik8_codepage_use_only_with_armscii8_screenmap_8x8.f8");
const HAIK8_CODEPAGE_USE_ONLY_WITH_ARMSCII8_SCREENMAP_14: &[u8] =
    include_bytes!("../data/fonts/haik8_codepage_use_only_with_armscii8_screenmap_8x14.f14");
const ISO_8859_8_HEBREW_16: &[u8] = include_bytes!("../data/fonts/ISO-8859-8_Hebrew_8x16.f16");
const ISO_8859_8_HEBREW_8: &[u8] = include_bytes!("../data/fonts/ISO-8859-8_Hebrew_8x8.f8");
const ISO_8859_8_HEBREW_14: &[u8] = include_bytes!("../data/fonts/ISO-8859-8_Hebrew_8x14.f14");
const UKRAINIAN_FONT_KOI8_U_16: &[u8] =
    include_bytes!("../data/fonts/Ukrainian_font_koi8-u_8x16.f16");
const UKRAINIAN_FONT_KOI8_U_8: &[u8] = include_bytes!("../data/fonts/Ukrainian_font_koi8-u_8x8.f8");
const UKRAINIAN_FONT_KOI8_U_14: &[u8] =
    include_bytes!("../data/fonts/Ukrainian_font_koi8-u_8x14.f14");
const ISO_8859_15_WEST_EUROPEAN_THIN_16: &[u8] =
    include_bytes!("../data/fonts/ISO-8859-15_West_European_thin_8x16.f16");
const ISO_8859_4_BALTIC_VGA_9BIT_MAPPED_16: &[u8] =
    include_bytes!("../data/fonts/ISO-8859-4_Baltic_VGA_9bit_mapped_8x16.f16");
const ISO_8859_4_BALTIC_VGA_9BIT_MAPPED_8: &[u8] =
    include_bytes!("../data/fonts/ISO-8859-4_Baltic_VGA_9bit_mapped_8x8.f8");
const ISO_8859_4_BALTIC_VGA_9BIT_MAPPED_14: &[u8] =
    include_bytes!("../data/fonts/ISO-8859-4_Baltic_VGA_9bit_mapped_8x14.f14");
const RUSSIAN_KOI8_R_B_16: &[u8] = include_bytes!("../data/fonts/Russian_koi8-r_b_8x16.f16");
const ISO_8859_4_BALTIC_WIDE_16: &[u8] =
    include_bytes!("../data/fonts/ISO-8859-4_Baltic_wide_8x16.f16");
const ISO_8859_5_CYRILLIC_16: &[u8] = include_bytes!("../data/fonts/ISO-8859-5_Cyrillic_8x16.f16");
const ISO_8859_5_CYRILLIC_8: &[u8] = include_bytes!("../data/fonts/ISO-8859-5_Cyrillic_8x8.f8");
const ISO_8859_5_CYRILLIC_14: &[u8] = include_bytes!("../data/fonts/ISO-8859-5_Cyrillic_8x14.f14");
const ARMSCII_8_CHARACTER_SET_16: &[u8] =
    include_bytes!("../data/fonts/ARMSCII-8_Character_set_8x16.f16");
const ARMSCII_8_CHARACTER_SET_8: &[u8] =
    include_bytes!("../data/fonts/ARMSCII-8_Character_set_8x8.f8");
const ARMSCII_8_CHARACTER_SET_14: &[u8] =
    include_bytes!("../data/fonts/ARMSCII-8_Character_set_8x14.f14");
const ISO_8859_15_WEST_EUROPEAN_16: &[u8] =
    include_bytes!("../data/fonts/ISO-8859-15_West_European_8x16.f16");
const ISO_8859_15_WEST_EUROPEAN_8: &[u8] =
    include_bytes!("../data/fonts/ISO-8859-15_West_European_8x8.f8");
const ISO_8859_15_WEST_EUROPEAN_14: &[u8] =
    include_bytes!("../data/fonts/ISO-8859-15_West_European_8x14.f14");
const CODEPAGE_850_MULTILINGUAL_LATIN_I_THIN_16: &[u8] =
    include_bytes!("../data/fonts/Codepage_850_Multilingual_Latin_I_thin_8x16.f16");
const CODEPAGE_850_MULTILINGUAL_LATIN_I_THIN_8: &[u8] =
    include_bytes!("../data/fonts/Codepage_850_Multilingual_Latin_I_thin_8x8.f8");
const CODEPAGE_850_MULTILINGUAL_LATIN_I_16: &[u8] =
    include_bytes!("../data/fonts/Codepage_850_Multilingual_Latin_I_8x16.f16");
const CODEPAGE_850_MULTILINGUAL_LATIN_I_8: &[u8] =
    include_bytes!("../data/fonts/Codepage_850_Multilingual_Latin_I_8x8.f8");
const CODEPAGE_850_MULTILINGUAL_LATIN_I_14: &[u8] =
    include_bytes!("../data/fonts/Codepage_850_Multilingual_Latin_I_8x14.f14");
const CODEPAGE_865_NORWEGIAN_THIN_16: &[u8] =
    include_bytes!("../data/fonts/Codepage_865_Norwegian_thin_8x16.f16");
const CODEPAGE_865_NORWEGIAN_THIN_8: &[u8] =
    include_bytes!("../data/fonts/Codepage_865_Norwegian_thin_8x8.f8");
const CODEPAGE_1251_CYRILLIC_16: &[u8] =
    include_bytes!("../data/fonts/Codepage_1251_Cyrillic_8x16.f16");
const CODEPAGE_1251_CYRILLIC_8: &[u8] =
    include_bytes!("../data/fonts/Codepage_1251_Cyrillic_8x8.f8");
const CODEPAGE_1251_CYRILLIC_14: &[u8] =
    include_bytes!("../data/fonts/Codepage_1251_Cyrillic_8x14.f14");
const ISO_8859_7_GREEK_16: &[u8] = include_bytes!("../data/fonts/ISO-8859-7_Greek_8x16.f16");
const ISO_8859_7_GREEK_8: &[u8] = include_bytes!("../data/fonts/ISO-8859-7_Greek_8x8.f8");
const ISO_8859_7_GREEK_14: &[u8] = include_bytes!("../data/fonts/ISO-8859-7_Greek_8x14.f14");
const RUSSIAN_KOI8_R_C_16: &[u8] = include_bytes!("../data/fonts/Russian_koi8-r_c_8x16.f16");
const ISO_8859_4_BALTIC_16: &[u8] = include_bytes!("../data/fonts/ISO-8859-4_Baltic_8x16.f16");
const ISO_8859_4_BALTIC_8: &[u8] = include_bytes!("../data/fonts/ISO-8859-4_Baltic_8x8.f8");
const ISO_8859_4_BALTIC_14: &[u8] = include_bytes!("../data/fonts/ISO-8859-4_Baltic_8x14.f14");
const ISO_8859_1_WEST_EUROPEAN_16: &[u8] =
    include_bytes!("../data/fonts/ISO-8859-1_West_European_8x16.f16");
const ISO_8859_1_WEST_EUROPEAN_8: &[u8] =
    include_bytes!("../data/fonts/ISO-8859-1_West_European_8x8.f8");
const ISO_8859_1_WEST_EUROPEAN_14: &[u8] =
    include_bytes!("../data/fonts/ISO-8859-1_West_European_8x14.f14");
const CODEPAGE_866_RUSSIAN_16: &[u8] =
    include_bytes!("../data/fonts/Codepage_866_Russian_8x16.f16");
const CODEPAGE_866_RUSSIAN_8: &[u8] = include_bytes!("../data/fonts/Codepage_866_Russian_8x8.f8");
const CODEPAGE_866_RUSSIAN_14: &[u8] =
    include_bytes!("../data/fonts/Codepage_866_Russian_8x14.f14");
const CODEPAGE_437_ENGLISH_THIN_16: &[u8] =
    include_bytes!("../data/fonts/Codepage_437_English_thin_8x16.f16");
const CODEPAGE_437_ENGLISH_THIN_8: &[u8] =
    include_bytes!("../data/fonts/Codepage_437_English_thin_8x8.f8");
const CODEPAGE_866_B_RUSSIAN_16: &[u8] =
    include_bytes!("../data/fonts/Codepage_866_b_Russian_8x16.f16");
const CODEPAGE_865_NORWEGIAN_16: &[u8] =
    include_bytes!("../data/fonts/Codepage_865_Norwegian_8x16.f16");
const CODEPAGE_865_NORWEGIAN_8: &[u8] =
    include_bytes!("../data/fonts/Codepage_865_Norwegian_8x8.f8");
const CODEPAGE_865_NORWEGIAN_14: &[u8] =
    include_bytes!("../data/fonts/Codepage_865_Norwegian_8x14.f14");
const UKRAINIAN_FONT_CP866U_16: &[u8] =
    include_bytes!("../data/fonts/Ukrainian_font_cp866u_8x16.f16");
const UKRAINIAN_FONT_CP866U_8: &[u8] = include_bytes!("../data/fonts/Ukrainian_font_cp866u_8x8.f8");
const UKRAINIAN_FONT_CP866U_14: &[u8] =
    include_bytes!("../data/fonts/Ukrainian_font_cp866u_8x14.f14");
const ISO_8859_1_WEST_EUROPEAN_THIN_16: &[u8] =
    include_bytes!("../data/fonts/ISO-8859-1_West_European_thin_8x16.f16");
const CODEPAGE_1131_BELARUSIAN_SWISS_16: &[u8] =
    include_bytes!("../data/fonts/Codepage_1131_Belarusian_swiss_8x16.f16");
const COMMODORE_64_UPPER_16: &[u8] = include_bytes!("../data/fonts/Commodore_64_UPPER_8x16.f16");
const COMMODORE_64_UPPER_8: &[u8] = include_bytes!("../data/fonts/Commodore_64_UPPER_8x8.f8");
const COMMODORE_64_LOWER_16: &[u8] = include_bytes!("../data/fonts/Commodore_64_Lower_8x16.f16");
const COMMODORE_64_LOWER_8: &[u8] = include_bytes!("../data/fonts/Commodore_64_Lower_8x8.f8");
const COMMODORE_128_UPPER_16: &[u8] = include_bytes!("../data/fonts/Commodore_128_UPPER_8x16.f16");
const COMMODORE_128_UPPER_8: &[u8] = include_bytes!("../data/fonts/Commodore_128_UPPER_8x8.f8");
const COMMODORE_128_LOWER_16: &[u8] = include_bytes!("../data/fonts/Commodore_128_Lower_8x16.f16");
const COMMODORE_128_LOWER_8: &[u8] = include_bytes!("../data/fonts/Commodore_128_Lower_8x8.f8");
const ATARI_16: &[u8] = include_bytes!("../data/fonts/Atari_8x16.f16");
const ATARI_8: &[u8] = include_bytes!("../data/fonts/Atari_8x8.f8");
const P0T_NOODLE_AMIGA_16: &[u8] = include_bytes!("../data/fonts/P0T_NOoDLE_Amiga_8x16.f16");
const P0T_NOODLE_AMIGA_14: &[u8] = include_bytes!("../data/fonts/P0T_NOoDLE_Amiga_8x14.f14");
const MOSOUL_AMIGA_16: &[u8] = include_bytes!("../data/fonts/mOsOul_Amiga_8x16.f16");
const MOSOUL_AMIGA_8: &[u8] = include_bytes!("../data/fonts/mOsOul_Amiga_8x8.f8");
const MICROKNIGHT_PLUS_AMIGA_16: &[u8] =
    include_bytes!("../data/fonts/MicroKnight_Plus_Amiga_8x16.f16");
const TOPAZ_PLUS_AMIGA_16: &[u8] = include_bytes!("../data/fonts/Topaz_Plus_Amiga_8x16.f16");
const MICROKNIGHT_AMIGA_16: &[u8] = include_bytes!("../data/fonts/MicroKnight_Amiga_8x16.f16");
const MICROKNIGHT_AMIGA_8: &[u8] = include_bytes!("../data/fonts/MicroKnight_Amiga_8x8.f8");
// const TOPAZ_AMIGA_16: &[u8] = include_bytes!("../data/fonts/Topaz_Amiga_8x16.f16");
// const TOPAZ_AMIGA_14: &[u8] = include_bytes!("../data/fonts/Topaz_Amiga_8x14.f14");

// const IBM_CP437_VGA50: &[u8] = include_bytes!("../data/fonts/IBM/cp437/VGA50.psf");
// const IBM_CP437_EGA: &[u8] = include_bytes!("../data/fonts/IBM/cp437/EGA.psf");
// const IBM_CP437_VGA: &[u8] = include_bytes!("../data/fonts/IBM/cp437/VGA.psf");
const IBM_CP437_VGA25G: &[u8] = include_bytes!("../data/fonts/IBM/cp437/VGA25G.psf");

const IBM_CP737_VGA50: &[u8] = include_bytes!("../data/fonts/IBM/cp737/VGA50.psf");
const IBM_CP737_EGA: &[u8] = include_bytes!("../data/fonts/IBM/cp737/EGA.psf");
const IBM_CP737_VGA: &[u8] = include_bytes!("../data/fonts/IBM/cp737/VGA.psf");

const IBM_CP775_VGA50: &[u8] = include_bytes!("../data/fonts/IBM/cp775/VGA50.psf");
const IBM_CP775_EGA: &[u8] = include_bytes!("../data/fonts/IBM/cp775/EGA.psf");
const IBM_CP775_VGA: &[u8] = include_bytes!("../data/fonts/IBM/cp775/VGA.psf");

// const IBM_CP850_VGA50: &[u8] = include_bytes!("../data/fonts/IBM/cp850/VGA50.psf");
// const IBM_CP850_EGA: &[u8] = include_bytes!("../data/fonts/IBM/cp850/EGA.psf");
// const IBM_CP850_VGA: &[u8] = include_bytes!("../data/fonts/IBM/cp850/VGA.psf");
// const IBM_CP850_VGA25G: &[u8] = include_bytes!("../data/fonts/IBM/cp850/VGA25G.psf");

const IBM_CP852_VGA50: &[u8] = include_bytes!("../data/fonts/IBM/cp852/VGA50.psf");
const IBM_CP852_EGA: &[u8] = include_bytes!("../data/fonts/IBM/cp852/EGA.psf");
const IBM_CP852_VGA: &[u8] = include_bytes!("../data/fonts/IBM/cp852/VGA.psf");
const IBM_CP852_VGA25G: &[u8] = include_bytes!("../data/fonts/IBM/cp852/VGA25G.psf");

const IBM_CP855_VGA50: &[u8] = include_bytes!("../data/fonts/IBM/cp855/VGA50.psf");
const IBM_CP855_EGA: &[u8] = include_bytes!("../data/fonts/IBM/cp855/EGA.psf");
const IBM_CP855_VGA: &[u8] = include_bytes!("../data/fonts/IBM/cp855/VGA.psf");

const IBM_CP857_VGA50: &[u8] = include_bytes!("../data/fonts/IBM/cp857/VGA50.psf");
const IBM_CP857_EGA: &[u8] = include_bytes!("../data/fonts/IBM/cp857/EGA.psf");
const IBM_CP857_VGA: &[u8] = include_bytes!("../data/fonts/IBM/cp857/VGA.psf");

const IBM_CP860_VGA50: &[u8] = include_bytes!("../data/fonts/IBM/cp860/VGA50.psf");
const IBM_CP860_EGA: &[u8] = include_bytes!("../data/fonts/IBM/cp860/EGA.psf");
const IBM_CP860_VGA: &[u8] = include_bytes!("../data/fonts/IBM/cp860/VGA.psf");
const IBM_CP860_VGA25G: &[u8] = include_bytes!("../data/fonts/IBM/cp860/VGA25G.psf");

const IBM_CP861_VGA50: &[u8] = include_bytes!("../data/fonts/IBM/cp861/VGA50.psf");
const IBM_CP861_EGA: &[u8] = include_bytes!("../data/fonts/IBM/cp861/EGA.psf");
const IBM_CP861_VGA: &[u8] = include_bytes!("../data/fonts/IBM/cp861/VGA.psf");
const IBM_CP861_VGA25G: &[u8] = include_bytes!("../data/fonts/IBM/cp861/VGA25G.psf");

const IBM_CP862_VGA50: &[u8] = include_bytes!("../data/fonts/IBM/cp862/VGA50.psf");
const IBM_CP862_EGA: &[u8] = include_bytes!("../data/fonts/IBM/cp862/EGA.psf");
const IBM_CP862_VGA: &[u8] = include_bytes!("../data/fonts/IBM/cp862/VGA.psf");

const IBM_CP863_VGA50: &[u8] = include_bytes!("../data/fonts/IBM/cp863/VGA50.psf");
const IBM_CP863_EGA: &[u8] = include_bytes!("../data/fonts/IBM/cp863/EGA.psf");
const IBM_CP863_VGA: &[u8] = include_bytes!("../data/fonts/IBM/cp863/VGA.psf");
const IBM_CP863_VGA25G: &[u8] = include_bytes!("../data/fonts/IBM/cp863/VGA25G.psf");
const IBM_CP864_VGA50: &[u8] = include_bytes!("../data/fonts/IBM/cp864/VGA50.psf");
const IBM_CP864_EGA: &[u8] = include_bytes!("../data/fonts/IBM/cp864/EGA.psf");
const IBM_CP864_VGA: &[u8] = include_bytes!("../data/fonts/IBM/cp864/VGA.psf");

const IBM_CP865_VGA50: &[u8] = include_bytes!("../data/fonts/IBM/cp865/VGA50.psf");
const IBM_CP865_EGA: &[u8] = include_bytes!("../data/fonts/IBM/cp865/EGA.psf");
const IBM_CP865_VGA: &[u8] = include_bytes!("../data/fonts/IBM/cp865/VGA.psf");
const IBM_CP865_VGA25G: &[u8] = include_bytes!("../data/fonts/IBM/cp865/VGA25G.psf");

const IBM_CP866_VGA50: &[u8] = include_bytes!("../data/fonts/IBM/cp866/VGA50.psf");
const IBM_CP866_EGA: &[u8] = include_bytes!("../data/fonts/IBM/cp866/EGA.psf");
const IBM_CP866_VGA: &[u8] = include_bytes!("../data/fonts/IBM/cp866/VGA.psf");

const IBM_CP869_VGA50: &[u8] = include_bytes!("../data/fonts/IBM/cp869/VGA50.psf");
const IBM_CP869_EGA: &[u8] = include_bytes!("../data/fonts/IBM/cp869/EGA.psf");
const IBM_CP869_VGA: &[u8] = include_bytes!("../data/fonts/IBM/cp869/VGA.psf");

const AMIGA_TOPAZ_1: &[u8] = include_bytes!("../data/fonts/Amiga/Topaz1.psf");
const AMIGA_TOPAZ_1P: &[u8] = include_bytes!("../data/fonts/Amiga/Topaz1+.psf");
const AMIGA_TOPAZ_2: &[u8] = include_bytes!("../data/fonts/Amiga/Topaz2.psf");
const AMIGA_TOPAZ_2P: &[u8] = include_bytes!("../data/fonts/Amiga/Topaz2+.psf");
const AMIGA_P0T_NOODLE: &[u8] = include_bytes!("../data/fonts/Amiga/P0T-NOoDLE.psf");
const AMIGA_MICROKNIGHT: &[u8] = include_bytes!("../data/fonts/Amiga/MicroKnight.psf");
const AMIGA_MICROKNIGHTP: &[u8] = include_bytes!("../data/fonts/Amiga/MicroKnight+.psf");
const AMIGA_MOSOUL: &[u8] = include_bytes!("../data/fonts/Amiga/mOsOul.psf");

const C64_PETSCII_UNSHIFTED: &[u8] = include_bytes!("../data/fonts/C64_PETSCII_unshifted.psf");
const C64_PETSCII_SHIFTED: &[u8] = include_bytes!("../data/fonts/C64_PETSCII_shifted.psf");
const ATARI_ATASCII: &[u8] = include_bytes!("../data/fonts/Atari_ATASCII.psf");
const VIEWDATA: &[u8] = include_bytes!("../data/fonts/saa5050.psf");

pub const DEFAULT_FONT_NAME: &str = "IBM VGA";
pub const ALT_DEFAULT_FONT_NAME: &str = "IBM VGA 437";

pub struct FontDescription {
    pub name: &'static str,
    pub variant_8x8: Option<&'static [u8]>,
    pub variant_8x14: Option<&'static [u8]>,
    pub variant_8x16: Option<&'static [u8]>,
}

pub const FONT_TABLE: [FontDescription; 42] = [
    FontDescription {
        name: "Codepage 437 English",
        variant_8x16: Some(CODEPAGE_437_ENGLISH_16),
        variant_8x8: Some(CODEPAGE_437_ENGLISH_8),
        variant_8x14: Some(CODEPAGE_437_ENGLISH_14),
    },
    FontDescription {
        name: "Codepage 1251 Cyrillic, (swiss)",
        variant_8x16: Some(CODEPAGE_1251_CYRILLIC_SWISS_16),
        variant_8x8: None,
        variant_8x14: None,
    },
    FontDescription {
        name: "Russian koi8-r",
        variant_8x16: Some(RUSSIAN_KOI8_R_16),
        variant_8x8: Some(RUSSIAN_KOI8_R_8),
        variant_8x14: Some(RUSSIAN_KOI8_R_14),
    },
    FontDescription {
        name: "ISO-8859-2 Central European",
        variant_8x16: Some(ISO_8859_2_CENTRAL_EUROPEAN_16),
        variant_8x8: Some(ISO_8859_2_CENTRAL_EUROPEAN_8),
        variant_8x14: Some(ISO_8859_2_CENTRAL_EUROPEAN_14),
    },
    FontDescription {
        name: "ISO-8859-4 Baltic wide (VGA 9bit mapped)",
        variant_8x16: Some(ISO_8859_4_BALTIC_WIDE_VGA_9BIT_MAPPED_16),
        variant_8x8: None,
        variant_8x14: None,
    },
    FontDescription {
        name: "Codepage 866 (c) Russian",
        variant_8x16: Some(CODEPAGE_866_C_RUSSIAN_16),
        variant_8x8: None,
        variant_8x14: None,
    },
    FontDescription {
        name: "ISO-8859-9 Turkish",
        variant_8x16: Some(ISO_8859_9_TURKISH_16),
        variant_8x8: None,
        variant_8x14: None,
    },
    FontDescription {
        name: "haik8 codepage (use only with armscii8 screenmap)",
        variant_8x16: Some(HAIK8_CODEPAGE_USE_ONLY_WITH_ARMSCII8_SCREENMAP_16),
        variant_8x8: Some(HAIK8_CODEPAGE_USE_ONLY_WITH_ARMSCII8_SCREENMAP_8),
        variant_8x14: Some(HAIK8_CODEPAGE_USE_ONLY_WITH_ARMSCII8_SCREENMAP_14),
    },
    FontDescription {
        name: "ISO-8859-8 Hebrew",
        variant_8x16: Some(ISO_8859_8_HEBREW_16),
        variant_8x8: Some(ISO_8859_8_HEBREW_8),
        variant_8x14: Some(ISO_8859_8_HEBREW_14),
    },
    FontDescription {
        name: "Ukrainian font koi8-u",
        variant_8x16: Some(UKRAINIAN_FONT_KOI8_U_16),
        variant_8x8: Some(UKRAINIAN_FONT_KOI8_U_8),
        variant_8x14: Some(UKRAINIAN_FONT_KOI8_U_14),
    },
    FontDescription {
        name: "ISO-8859-15 West European, (thin)",
        variant_8x16: Some(ISO_8859_15_WEST_EUROPEAN_THIN_16),
        variant_8x8: None,
        variant_8x14: None,
    },
    FontDescription {
        name: "ISO-8859-4 Baltic (VGA 9bit mapped)",
        variant_8x16: Some(ISO_8859_4_BALTIC_VGA_9BIT_MAPPED_16),
        variant_8x8: Some(ISO_8859_4_BALTIC_VGA_9BIT_MAPPED_8),
        variant_8x14: Some(ISO_8859_4_BALTIC_VGA_9BIT_MAPPED_14),
    },
    FontDescription {
        name: "Russian koi8-r (b)",
        variant_8x16: Some(RUSSIAN_KOI8_R_B_16),
        variant_8x8: None,
        variant_8x14: None,
    },
    FontDescription {
        name: "ISO-8859-4 Baltic wide",
        variant_8x16: Some(ISO_8859_4_BALTIC_WIDE_16),
        variant_8x8: None,
        variant_8x14: None,
    },
    FontDescription {
        name: "ISO-8859-5 Cyrillic",
        variant_8x16: Some(ISO_8859_5_CYRILLIC_16),
        variant_8x8: Some(ISO_8859_5_CYRILLIC_8),
        variant_8x14: Some(ISO_8859_5_CYRILLIC_14),
    },
    FontDescription {
        name: "ARMSCII-8 Character set",
        variant_8x16: Some(ARMSCII_8_CHARACTER_SET_16),
        variant_8x8: Some(ARMSCII_8_CHARACTER_SET_8),
        variant_8x14: Some(ARMSCII_8_CHARACTER_SET_14),
    },
    FontDescription {
        name: "ISO-8859-15 West European",
        variant_8x16: Some(ISO_8859_15_WEST_EUROPEAN_16),
        variant_8x8: Some(ISO_8859_15_WEST_EUROPEAN_8),
        variant_8x14: Some(ISO_8859_15_WEST_EUROPEAN_14),
    },
    FontDescription {
        name: "Codepage 850 Multilingual Latin I, (thin)",
        variant_8x16: Some(CODEPAGE_850_MULTILINGUAL_LATIN_I_THIN_16),
        variant_8x8: Some(CODEPAGE_850_MULTILINGUAL_LATIN_I_THIN_8),
        variant_8x14: None,
    },
    FontDescription {
        name: "Codepage 850 Multilingual Latin I",
        variant_8x16: Some(CODEPAGE_850_MULTILINGUAL_LATIN_I_16),
        variant_8x8: Some(CODEPAGE_850_MULTILINGUAL_LATIN_I_8),
        variant_8x14: Some(CODEPAGE_850_MULTILINGUAL_LATIN_I_14),
    },
    FontDescription {
        name: "Codepage 865 Norwegian, (thin)",
        variant_8x16: Some(CODEPAGE_865_NORWEGIAN_THIN_16),
        variant_8x8: Some(CODEPAGE_865_NORWEGIAN_THIN_8),
        variant_8x14: None,
    },
    FontDescription {
        name: "Codepage 1251 Cyrillic",
        variant_8x16: Some(CODEPAGE_1251_CYRILLIC_16),
        variant_8x8: Some(CODEPAGE_1251_CYRILLIC_8),
        variant_8x14: Some(CODEPAGE_1251_CYRILLIC_14),
    },
    FontDescription {
        name: "ISO-8859-7 Greek",
        variant_8x16: Some(ISO_8859_7_GREEK_16),
        variant_8x8: Some(ISO_8859_7_GREEK_8),
        variant_8x14: Some(ISO_8859_7_GREEK_14),
    },
    FontDescription {
        name: "Russian koi8-r (c)",
        variant_8x16: Some(RUSSIAN_KOI8_R_C_16),
        variant_8x8: None,
        variant_8x14: None,
    },
    FontDescription {
        name: "ISO-8859-4 Baltic",
        variant_8x16: Some(ISO_8859_4_BALTIC_16),
        variant_8x8: Some(ISO_8859_4_BALTIC_8),
        variant_8x14: Some(ISO_8859_4_BALTIC_14),
    },
    FontDescription {
        name: "ISO-8859-1 West European",
        variant_8x16: Some(ISO_8859_1_WEST_EUROPEAN_16),
        variant_8x8: Some(ISO_8859_1_WEST_EUROPEAN_8),
        variant_8x14: Some(ISO_8859_1_WEST_EUROPEAN_14),
    },
    FontDescription {
        name: "Codepage 866 Russian",
        variant_8x16: Some(CODEPAGE_866_RUSSIAN_16),
        variant_8x8: Some(CODEPAGE_866_RUSSIAN_8),
        variant_8x14: Some(CODEPAGE_866_RUSSIAN_14),
    },
    FontDescription {
        name: "Codepage 437 English, (thin)",
        variant_8x16: Some(CODEPAGE_437_ENGLISH_THIN_16),
        variant_8x8: Some(CODEPAGE_437_ENGLISH_THIN_8),
        variant_8x14: None,
    },
    FontDescription {
        name: "Codepage 866 (b) Russian",
        variant_8x16: Some(CODEPAGE_866_B_RUSSIAN_16),
        variant_8x8: None,
        variant_8x14: None,
    },
    FontDescription {
        name: "Codepage 865 Norwegian",
        variant_8x16: Some(CODEPAGE_865_NORWEGIAN_16),
        variant_8x8: Some(CODEPAGE_865_NORWEGIAN_8),
        variant_8x14: Some(CODEPAGE_865_NORWEGIAN_14),
    },
    FontDescription {
        name: "Ukrainian font cp866u",
        variant_8x16: Some(UKRAINIAN_FONT_CP866U_16),
        variant_8x8: Some(UKRAINIAN_FONT_CP866U_8),
        variant_8x14: Some(UKRAINIAN_FONT_CP866U_14),
    },
    FontDescription {
        name: "ISO-8859-1 West European, (thin)",
        variant_8x16: Some(ISO_8859_1_WEST_EUROPEAN_THIN_16),
        variant_8x8: None,
        variant_8x14: None,
    },
    FontDescription {
        name: "Codepage 1131 Belarusian, (swiss)",
        variant_8x16: Some(CODEPAGE_1131_BELARUSIAN_SWISS_16),
        variant_8x8: None,
        variant_8x14: None,
    },
    FontDescription {
        name: "Commodore 64 (UPPER)",
        variant_8x16: Some(COMMODORE_64_UPPER_16),
        variant_8x8: Some(COMMODORE_64_UPPER_8),
        variant_8x14: None,
    },
    FontDescription {
        name: "Commodore 64 (Lower)",
        variant_8x16: Some(COMMODORE_64_LOWER_16),
        variant_8x8: Some(COMMODORE_64_LOWER_8),
        variant_8x14: None,
    },
    FontDescription {
        name: "Commodore 128 (UPPER)",
        variant_8x16: Some(COMMODORE_128_UPPER_16),
        variant_8x8: Some(COMMODORE_128_UPPER_8),
        variant_8x14: None,
    },
    FontDescription {
        name: "Commodore 128 (Lower)",
        variant_8x16: Some(COMMODORE_128_LOWER_16),
        variant_8x8: Some(COMMODORE_128_LOWER_8),
        variant_8x14: None,
    },
    FontDescription {
        name: "Atari",
        variant_8x16: Some(ATARI_16),
        variant_8x8: Some(ATARI_8),
        variant_8x14: None,
    },
    FontDescription {
        name: "P0T NOoDLE (Amiga)",
        variant_8x16: Some(P0T_NOODLE_AMIGA_16),
        variant_8x8: None,
        variant_8x14: Some(P0T_NOODLE_AMIGA_14),
    },
    FontDescription {
        name: "mO'sOul (Amiga)",
        variant_8x16: Some(MOSOUL_AMIGA_16),
        variant_8x8: Some(MOSOUL_AMIGA_8),
        variant_8x14: None,
    },
    FontDescription {
        name: "MicroKnight Plus (Amiga)",
        variant_8x16: Some(MICROKNIGHT_PLUS_AMIGA_16),
        variant_8x8: None,
        variant_8x14: None,
    },
    FontDescription {
        name: "Topaz Plus (Amiga)",
        variant_8x16: Some(TOPAZ_PLUS_AMIGA_16),
        variant_8x8: None,
        variant_8x14: None,
    },
    FontDescription {
        name: "MicroKnight (Amiga)",
        variant_8x16: Some(MICROKNIGHT_AMIGA_16),
        variant_8x8: Some(MICROKNIGHT_AMIGA_8),
        variant_8x14: None,
    },
];

pub const SUPPORTED_FONTS: [&str; 91] = [
    "IBM VGA",
    "IBM VGA50",
    "IBM VGA25G",
    "IBM EGA",
    "IBM EGA43",
    "IBM VGA 437",
    "IBM VGA50 437",
    "IBM VGA25G 437",
    "IBM EGA 437",
    "IBM EGA43 437",
    /*
    "IBM VGA 720",
    "IBM VGA50 720",
    "IBM VGA25G 720",
    "IBM EGA 720",
    "IBM EGA43 720",*/
    "IBM VGA 737",
    "IBM VGA50 737",
    //"IBM VGA25G 737",
    "IBM EGA 737",
    "IBM EGA43 737",
    "IBM VGA 775",
    "IBM VGA50 775",
    //"IBM VGA25G 775",
    "IBM EGA 775",
    "IBM EGA43 775",
    /* "IBM VGA 819",
    "IBM VGA50 819",
    "IBM VGA25G 819",
    "IBM EGA 819",
    "IBM EGA43 819",*/
    "IBM VGA 850",
    "IBM VGA50 850",
    "IBM VGA25G 850",
    "IBM EGA 850",
    "IBM EGA43 850",
    "IBM VGA 852",
    "IBM VGA50 852",
    "IBM VGA25G 852",
    "IBM EGA 852",
    "IBM EGA43 852",
    "IBM VGA 855",
    "IBM VGA50 855",
    //"IBM VGA25G 855",
    "IBM EGA 855",
    "IBM EGA43 855",
    "IBM VGA 857",
    "IBM VGA50 857",
    //"IBM VGA25G 857",
    "IBM EGA 857",
    "IBM EGA43 857", /*

                     "IBM VGA 858",
                     "IBM VGA50 858",
                     "IBM VGA25G 858",
                     "IBM EGA 858",
                     "IBM EGA43 858",*/
    "IBM VGA 860",
    "IBM VGA50 860",
    "IBM VGA25G 860",
    "IBM EGA 860",
    "IBM EGA43 860",
    "IBM VGA 861",
    "IBM VGA50 861",
    "IBM VGA25G 861",
    "IBM EGA 861",
    "IBM EGA43 861",
    "IBM VGA 862",
    "IBM VGA50 862",
    //"IBM VGA25G 862",
    "IBM EGA 862",
    "IBM EGA43 862",
    "IBM VGA 863",
    "IBM VGA50 863",
    "IBM VGA25G 863",
    "IBM EGA 863",
    "IBM EGA43 863",
    "IBM VGA 864",
    "IBM VGA50 864",
    //"IBM VGA25G 864",
    "IBM EGA 864",
    "IBM EGA43 864",
    "IBM VGA 865",
    "IBM VGA50 865",
    "IBM VGA25G 865",
    "IBM EGA 865",
    "IBM EGA43 865",
    "IBM VGA 866",
    "IBM VGA50 866",
    //"IBM VGA25G 866",
    "IBM EGA 866",
    "IBM EGA43 866",
    "IBM VGA 869",
    "IBM VGA50 869",
    //"IBM VGA25G 869",
    "IBM EGA 869",
    "IBM EGA43 869",
    /*"IBM VGA 872",
    "IBM VGA50 872",
    "IBM VGA25G 872",
    "IBM EGA 872",
    "IBM EGA43 872",

    "IBM VGA KAM",
    "IBM VGA50 KAM",
    "IBM VGA25G KAM",
    "IBM EGA KAM",
    "IBM EGA43 KAM",

    "IBM VGA MAZ",
    "IBM VGA50 MAZ",
    "IBM VGA25G MAZ",
    "IBM EGA MAZ",
    "IBM EGA43 MAZ",*/
    "IBM VGA MIK",
    "IBM VGA50 MIK",
    //"IBM VGA25G MIK",
    "IBM EGA MIK",
    "IBM EGA43 MIK",
    /* "IBM VGA 667",
    "IBM VGA50 667",
    "IBM VGA25G 667",
    "IBM EGA 667",
    "IBM EGA43 667",

    "IBM VGA 790",
    "IBM VGA50 790",
    "IBM VGA25G 790",
    "IBM EGA 790",
    "IBM EGA43 790",*/
    "IBM VGA 866",
    "IBM VGA50 866",
    //"IBM VGA25G 866",
    "IBM EGA 866",
    "IBM EGA43 866",
    /*
    "IBM VGA 867",
    "IBM VGA50 867",
    "IBM VGA25G 867",
    "IBM EGA 867",
    "IBM EGA43 867",

    "IBM VGA 895",
    "IBM VGA50 895",
    "IBM VGA25G 895",
    "IBM EGA 895",
    "IBM EGA43 895",

    "IBM VGA 991",
    "IBM VGA50 991",
    "IBM VGA25G 991",
    "IBM EGA 991",
    "IBM EGA43 991",*/
    "Amiga Topaz 1",
    "Amiga Topaz 1+",
    "Amiga Topaz 2",
    "Amiga Topaz 2+",
    "Amiga P0T-NOoDLE",
    "Amiga MicroKnight",
    "Amiga MicroKnight+",
    "Amiga mOsOul",
    "C64 PETSCII unshifted",
    "C64 PETSCII shifted",
    "Atari ATASCII",
];

#[allow(clippy::match_same_arms)]
pub fn get_font_data(font_name: &str) -> Option<&[u8]> {
    match font_name {
        "IBM VGA" | "IBM VGA 437" => Some(CODEPAGE_437_ENGLISH_16),
        "IBM VGA50" | "IBM VGA50 437" => Some(CODEPAGE_437_ENGLISH_8),
        "IBM VGA25G" | "IBM VGA25G 437" => Some(IBM_CP437_VGA25G),
        "IBM EGA" | "IBM EGA 437" => Some(CODEPAGE_437_ENGLISH_14),
        "IBM EGA43" | "IBM EGA43 437" => Some(CODEPAGE_437_ENGLISH_8),

        /*

        "IBM VGA 720" => Some(IBM_CP720_VGA),
        "IBM VGA50 720" => Some(IBM_CP720_VGA50),
        "IBM VGA25G 720" => Some(IBM_CP720_VGA25G),
        "IBM EGA 720" => Some(IBM_CP720_EGA),
        "IBM EGA43 720" => Some(IBM_CP720_VGA50),*/
        "IBM VGA 737" => Some(IBM_CP737_VGA),
        "IBM VGA50 737" => Some(IBM_CP737_VGA50),
        //        "IBM VGA25G 737" => Some(IBM_CP737_VGA25G),
        "IBM EGA 737" => Some(IBM_CP737_EGA),
        "IBM EGA43 737" => Some(IBM_CP737_VGA50),

        "IBM VGA 775" => Some(IBM_CP775_VGA),
        "IBM VGA50 775" => Some(IBM_CP775_VGA50),
        //        "IBM VGA25G 775" => Some(IBM_CP775_VGA25G),
        "IBM EGA 775" => Some(IBM_CP775_EGA),
        "IBM EGA43 775" => Some(IBM_CP775_VGA50),

        /*         "IBM VGA 819" => Some(IBM_CP819_VGA),
        "IBM VGA50 819" => Some(IBM_CP819_VGA50),
        "IBM VGA25G 819" => Some(IBM_CP819_VGA25G),
        "IBM EGA 819" => Some(IBM_CP819_EGA),
        "IBM EGA43 819" => Some(IBM_CP819_VGA50),*/
        "IBM VGA 850" => Some(CODEPAGE_850_MULTILINGUAL_LATIN_I_16),
        "IBM VGA50 850" => Some(CODEPAGE_850_MULTILINGUAL_LATIN_I_8),
        "IBM VGA25G 850" => Some(CODEPAGE_850_MULTILINGUAL_LATIN_I_16),
        "IBM EGA 850" => Some(CODEPAGE_850_MULTILINGUAL_LATIN_I_14),
        "IBM EGA43 850" => Some(CODEPAGE_850_MULTILINGUAL_LATIN_I_16),

        "IBM VGA 852" => Some(IBM_CP852_VGA),
        "IBM VGA50 852" => Some(IBM_CP852_VGA50),
        "IBM VGA25G 852" => Some(IBM_CP852_VGA25G),
        "IBM EGA 852" => Some(IBM_CP852_EGA),
        "IBM EGA43 852" => Some(IBM_CP852_VGA50),

        "IBM VGA 855" => Some(IBM_CP855_VGA),
        "IBM VGA50 855" => Some(IBM_CP855_VGA50),
        //        "IBM VGA25G 855" => Some(IBM_CP855_VGA25G),
        "IBM EGA 855" => Some(IBM_CP855_EGA),
        "IBM EGA43 855" => Some(IBM_CP855_VGA50),

        "IBM VGA 857" => Some(IBM_CP857_VGA),
        "IBM VGA50 857" => Some(IBM_CP857_VGA50),
        //        "IBM VGA25G 857" => Some(IBM_CP857_VGA25G),
        "IBM EGA 857" => Some(IBM_CP857_EGA),
        "IBM EGA43 857" => Some(IBM_CP857_VGA50), /*

        "IBM VGA 858" => Some(IBM_CP858_VGA),
        "IBM VGA50 858" => Some(IBM_CP858_VGA50),
        "IBM VGA25G 858" => Some(IBM_CP858_VGA25G),
        "IBM EGA 858" => Some(IBM_CP858_EGA),
        "IBM EGA43 858" => Some(IBM_CP858_VGA50),*/
        "IBM VGA 860" => Some(IBM_CP860_VGA),
        "IBM VGA50 860" => Some(IBM_CP860_VGA50),
        "IBM VGA25G 860" => Some(IBM_CP860_VGA25G),
        "IBM EGA 860" => Some(IBM_CP860_EGA),
        "IBM EGA43 860" => Some(IBM_CP860_VGA50),

        "IBM VGA 861" => Some(IBM_CP861_VGA),
        "IBM VGA50 861" => Some(IBM_CP861_VGA50),
        "IBM VGA25G 861" => Some(IBM_CP861_VGA25G),
        "IBM EGA 861" => Some(IBM_CP861_EGA),
        "IBM EGA43 861" => Some(IBM_CP861_VGA50),

        "IBM VGA 862" => Some(IBM_CP862_VGA),
        "IBM VGA50 862" => Some(IBM_CP862_VGA50),
        //        "IBM VGA25G 862" => Some(IBM_CP862_VGA25G),
        "IBM EGA 862" => Some(IBM_CP862_EGA),
        "IBM EGA43 862" => Some(IBM_CP862_VGA50),

        "IBM VGA 863" => Some(IBM_CP863_VGA),
        "IBM VGA50 863" => Some(IBM_CP863_VGA50),
        "IBM VGA25G 863" => Some(IBM_CP863_VGA25G),
        "IBM EGA 863" => Some(IBM_CP863_EGA),
        "IBM EGA43 863" => Some(IBM_CP863_VGA50),

        "IBM VGA 864" => Some(IBM_CP864_VGA),
        "IBM VGA50 864" => Some(IBM_CP864_VGA50),
        //        "IBM VGA25G 864" => Some(IBM_CP864_VGA25G),
        "IBM EGA 864" => Some(IBM_CP864_EGA),
        "IBM EGA43 864" => Some(IBM_CP864_VGA50),

        "IBM VGA 865" => Some(IBM_CP865_VGA),
        "IBM VGA50 865" => Some(IBM_CP865_VGA50),
        "IBM VGA25G 865" => Some(IBM_CP865_VGA25G),
        "IBM EGA 865" => Some(IBM_CP865_EGA),
        "IBM EGA43 865" => Some(IBM_CP865_VGA50),

        "IBM VGA 866" => Some(IBM_CP866_VGA),
        "IBM VGA50 866" => Some(IBM_CP866_VGA50),
        //        "IBM VGA25G 866" => Some(IBM_CP866_VGA25G),
        "IBM EGA 866" => Some(IBM_CP866_EGA),
        "IBM EGA43 866" => Some(IBM_CP866_VGA50),

        "IBM VGA 869" => Some(IBM_CP869_VGA),
        "IBM VGA50 869" => Some(IBM_CP869_VGA50),
        //        "IBM VGA25G 869" => Some(IBM_CP869_VGA25G),
        "IBM EGA 869" => Some(IBM_CP869_EGA),
        "IBM EGA43 869" => Some(IBM_CP869_VGA50),

        /*"IBM VGA 872" => Some(IBM_CP872_VGA),
                "IBM VGA50 872" => Some(IBM_CP872_VGA50),
                "IBM VGA25G 872" => Some(IBM_CP872_VGA25G),
                "IBM EGA 872" => Some(IBM_CP872_EGA),
                "IBM EGA43 872" => Some(IBM_CP872_VGA50),

                "IBM VGA KAM" => Some(IBM_CP867_VGA),
                "IBM VGA50 KAM" => Some(IBM_CP867_VGA50),
                "IBM VGA25G KAM" => Some(IBM_CP867_VGA25G),
                "IBM EGA KAM" => Some(IBM_CP867_EGA),
                "IBM EGA43 KAM" => Some(IBM_CP867_VGA50),

                "IBM VGA MAZ" => Some(IBM_CP667_VGA),
                "IBM VGA50 MAZ" => Some(IBM_CP667_VGA50),
                "IBM VGA25G MAZ" => Some(IBM_CP667_VGA25G),
                "IBM EGA MAZ" => Some(IBM_CP667_EGA),
                "IBM EGA43 MAZ" => Some(IBM_CP667_VGA50),

                "IBM VGA MIK" => Some(IBM_CP866_VGA),
                "IBM VGA50 MIK" => Some(IBM_CP866_VGA50),
        //        "IBM VGA25G MIK" => Some(IBM_CP866_VGA25G),
                "IBM EGA MIK" => Some(IBM_CP866_EGA),
                "IBM EGA43 MIK" => Some(IBM_CP866_VGA50),

        /*         "IBM VGA 667" => Some(IBM_CP667_VGA),
                "IBM VGA50 667" => Some(IBM_CP667_VGA50),
                "IBM VGA25G 667" => Some(IBM_CP667_VGA25G),
                "IBM EGA 667" => Some(IBM_CP667_EGA),
                "IBM EGA43 667" => Some(IBM_CP667_VGA50),

                "IBM VGA 790" => Some(IBM_CP790_VGA),
                "IBM VGA50 790" => Some(IBM_CP790_VGA50),
                "IBM VGA25G 790" => Some(IBM_CP790_VGA25G),
                "IBM EGA 790" => Some(IBM_CP790_EGA),
                "IBM EGA43 790" => Some(IBM_CP790_VGA50),*/


                "IBM VGA 867" => Some(IBM_CP867_VGA),
                "IBM VGA50 867" => Some(IBM_CP867_VGA50),
                "IBM VGA25G 867" => Some(IBM_CP867_VGA25G),
                "IBM EGA 867" => Some(IBM_CP867_EGA),
                "IBM EGA43 867" => Some(IBM_CP867_VGA50),

                "IBM VGA 895" => Some(IBM_CP895_VGA),
                "IBM VGA50 895" => Some(IBM_CP895_VGA50),
                "IBM VGA25G 895" => Some(IBM_CP895_VGA25G),
                "IBM EGA 895" => Some(IBM_CP895_EGA),
                "IBM EGA43 895" => Some(IBM_CP895_VGA50),

                "IBM VGA 991" => Some(IBM_CP991_VGA),
                "IBM VGA50 991" => Some(IBM_CP991_VGA50),
                "IBM VGA25G 991" => Some(IBM_CP991_VGA25G),
                "IBM EGA 991" => Some(IBM_CP991_EGA),
                "IBM EGA43 991" => Some(IBM_CP991_VGA50),*/
        "Amiga Topaz 1" => Some(AMIGA_TOPAZ_1),
        "Amiga Topaz 1+" => Some(AMIGA_TOPAZ_1P),
        "Amiga Topaz 2" => Some(AMIGA_TOPAZ_2),
        "Amiga Topaz 2+" => Some(AMIGA_TOPAZ_2P),
        "Amiga P0T-NOoDLE" => Some(AMIGA_P0T_NOODLE),
        "Amiga MicroKnight" => Some(AMIGA_MICROKNIGHT),
        "Amiga MicroKnight+" => Some(AMIGA_MICROKNIGHTP),
        "Amiga mOsOul" => Some(AMIGA_MOSOUL),

        "C64 PETSCII unshifted" => Some(C64_PETSCII_UNSHIFTED),
        "C64 PETSCII shifted" => Some(C64_PETSCII_SHIFTED),

        "Atari ATASCII" => Some(ATARI_ATASCII),
        "Viewdata" => Some(VIEWDATA),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub enum FontError {
    FontNotFound,
    MagicNumberMismatch,
    UnsupportedVersion(u32),
    LengthMismatch(usize, usize),
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
