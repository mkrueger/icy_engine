use std::{fs::File, io::Read, path::Path, error::Error};

use crate::{AttributedChar, Buffer, BufferType, Size, TextAttribute, UPosition, EngineResult};

#[derive(Copy, Clone, Debug)]
pub enum FontType {
    Outline,
    Block,
    Color,
}

struct FontGlyph {
    pub size: Size,
    pub data: Vec<u8>,
}

#[allow(dead_code)]
pub struct TheDrawFont {
    pub name: String,
    pub font_type: FontType,
    pub spaces: i32,
    char_table: Vec<Option<FontGlyph>>
}

static THE_DRAW_FONT_ID: &[u8; 18] = b"TheDraw FONTS file";
const THE_DRAW_FONT_HEADER_SIZE: usize = 233;

pub const MAX_WIDTH: usize = 30;
pub const MAX_HEIGHT: usize = 12;
pub const FONT_NAME_LEN: usize = 12;
pub const MAX_LETTER_SPACE: usize = 40;

pub const CHAR_TABLE_SIZE: usize = 94;

const CTRL_Z: u8 = 0x1A; // indicates end of file

const FONT_INDICATOR: u32 = 0xFF00_AA55;

impl TheDrawFont {
    /// .
    ///
    /// # Panics
    ///
    /// Panics if .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn load(file_name: &Path) -> EngineResult<Vec<TheDrawFont>> {
        let mut f = File::open(file_name).expect("error while opening file");
        let mut bytes = Vec::new();
        f.read_to_end(&mut bytes).expect("error while reading file");
        TheDrawFont::from_tdf_bytes(&bytes)
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
    pub fn from_tdf_bytes(bytes: &[u8]) -> EngineResult<Vec<TheDrawFont>> {
        let mut result = Vec::new();

        if bytes.len() < THE_DRAW_FONT_HEADER_SIZE {
            return Err(Box::new(TdfError::FileTooShort));
        }

        if bytes[0] as usize != THE_DRAW_FONT_ID.len() + 1 {
            return Err(Box::new(TdfError::IdLengthMismatch(bytes[0])));
        }

        if THE_DRAW_FONT_ID != &bytes[1..19] {
            return Err(Box::new(TdfError::IdMismatch));
        }
        let mut o = THE_DRAW_FONT_ID.len() + 1;
        
        let magic_byte = bytes[o];
        if magic_byte != CTRL_Z {
            return Err(Box::new(TdfError::IdMismatch));
        }
        o += 1;

        while o < bytes.len() {
            let indicator = u32::from_le_bytes(bytes[o..(o + 4)].try_into().unwrap());
            if indicator != FONT_INDICATOR {
                return Err(Box::new(TdfError::FontIndicatorMismatch));
            }
            o += 4; // FONT_INDICATOR bytes!

            let font_name_len = bytes[o] as usize;
            o += 1;
            if font_name_len > FONT_NAME_LEN {
                return Err(Box::new(TdfError::NameTooLong(font_name_len)));
            }

            let name = String::from_utf8_lossy(&bytes[o..(o + font_name_len)]).to_string();
            o += FONT_NAME_LEN;
           
            o += 4; // 4 magic bytes!

            let font_type = match bytes[o] {
                0 => FontType::Outline,
                1 => FontType::Block,
                2 => FontType::Color,
                unsupported => {
                    return Err(Box::new(TdfError::UnsupportedTtfType(unsupported)));
                }
            };
            o += 1;
            
            let spaces: i32 = bytes[o] as i32;
            if spaces > MAX_LETTER_SPACE as i32 {
                return Err(Box::new(TdfError::LetterSpaceTooMuch(spaces)));
            }
            o += 1;

            let block_size = (bytes[o] as u16 | ((bytes[o + 1] as u16) << 8)) as usize;
            o += 2;

            let mut char_lookup_table = Vec::new();
            for _ in 0..CHAR_TABLE_SIZE {
                let cur_char = bytes[o] as u16 | ((bytes[o + 1] as u16) << 8);
                o += 2;
                char_lookup_table.push(cur_char);
            }

            let mut char_table = Vec::new();

            for char_offset in char_lookup_table {
                let mut char_offset = char_offset as usize;

                let mut char_data = Vec::new();

                if char_offset == 0xFFFF {
                    char_table.push(None);
                    continue;
                }          
                
                if char_offset >= block_size {
                    return Err(Box::new(TdfError::GlyphOutsideFontDataSize(char_offset)));
                }
    
                char_offset += o;

                let width = bytes[char_offset] as usize;
                char_offset += 1;
                let height = bytes[char_offset] as usize;
                char_offset += 1;

                loop {
                    if char_offset >= bytes.len() {
                        return Err(Box::new(TdfError::DataOverflow(char_offset)));
                    }

                    let ch = bytes[char_offset];
                    if ch == 0 {
                        break;
                    }
                    if ch == 13 { 
                        continue;
                    }
                    if matches!(font_type, FontType::Color) {
                        char_data.push(ch);
                        char_offset += 1;
                        char_data.push(ch);
                        char_offset += 1;
                    } else {
                        char_data.push(ch);
                        char_offset += 1;
                    }
                }
                char_table.push(Some(FontGlyph {
                    size: Size::new(width, height),
                    data: char_data,
                }));
            }
            o += block_size;

            result.push( TheDrawFont {
                name,
                font_type,
                spaces,
                char_table
            });
        }

        Ok(result)
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn as_tdf_bytes(&self) -> EngineResult<Vec<u8>> {
        let mut result = Vec::new();
        result.push(THE_DRAW_FONT_ID.len() as u8 + 1);
        result.extend(THE_DRAW_FONT_ID);
        result.push(CTRL_Z);
        result.extend(u32::to_le_bytes(FONT_INDICATOR));
        if self.name.len() > FONT_NAME_LEN {
            return Err(Box::new(TdfError::NameTooLong(self.name.len())));
        }
        result.push(FONT_NAME_LEN as u8); // font name length is always 11
        result.extend(self.name.as_bytes());
        result.extend(vec![0; FONT_NAME_LEN - self.name.len()]);

        result.extend([0, 0, 0, 0]); // unused bytes? 
        let type_byte = match self.font_type {
            FontType::Outline => 0,
            FontType::Block => 1,
            FontType::Color => 2,
        };
        result.push(type_byte);
        if self.spaces > MAX_LETTER_SPACE as i32 {
            return Err(Box::new(TdfError::LetterSpaceTooMuch(self.spaces)));
        }
        result.push(self.spaces as u8);

        let mut char_lookup_table = Vec::new();
        let mut font_data = Vec::new();

        for glyph in &self.char_table {
            match glyph {
                Some(glyph) =>  {
                    char_lookup_table.extend(u16::to_le_bytes(font_data.len() as u16));
                    font_data.push(glyph.size.width as u8);
                    font_data.push(glyph.size.height as u8);
                    font_data.extend(&glyph.data);
                    font_data.push(0);
                }
                None => char_lookup_table.extend(u16::to_le_bytes(0xFFFF))
            }
        }

        result.extend(u16::to_le_bytes(font_data.len() as u16));
        result.extend(char_lookup_table);
  
        result.extend(font_data);

        Ok(result)
    }

    pub fn transform_outline(outline: usize, ch: u8) -> u8 {
        if ch > 64 && ch - 64 <= 17 {
            TheDrawFont::OUTLINE_CHAR_SET[outline][(ch - 65) as usize]
        } else {
            b' '
        }
    }

    pub fn get_font_height(&self) -> usize {
        for glyph_opt in &self.char_table {
            if let Some(glyph) = glyph_opt{
                return glyph.size.height;
            } 
        }
        0
    }

    pub fn has_char(&self, char_code: u8) -> bool {
        let char_offset = (char_code as i32) - b' ' as i32 - 1;
        if char_offset < 0 || char_offset > self.char_table.len() as i32 {
            return false;
        }
        self.char_table[char_offset as usize].is_some()
    }

    pub fn render(
        &self,
        buffer: &mut Buffer,
        layer: usize,
        pos: UPosition,
        color: TextAttribute,
        outline_style: usize,
        char_code: u8,
    ) -> Option<Size> {
        let char_index = (char_code as i32) - b' ' as i32 - 1;
        if char_index < 0 || char_index > self.char_table.len() as i32 {
            return None;
        }
        let table_entry = &self.char_table[char_index as usize] ;
        let Some(glyph) = table_entry else {
            return None;
        };

        let mut cur = pos;
        let mut char_offset = 0;
        while char_offset < glyph.data.len() {
            let ch = glyph.data[char_offset];
            if ch == 13 {
                cur.x = pos.x;
                cur.y += 1;
            } else {
                let attributed_char = match self.font_type {
                    FontType::Outline => AttributedChar::new(
                        unsafe {
                            char::from_u32_unchecked(TheDrawFont::transform_outline(
                                outline_style,
                                ch,
                            ) as u32)
                        },
                        color,
                    ),
                    FontType::Block => {
                        AttributedChar::new(unsafe { char::from_u32_unchecked(ch as u32) }, color)
                    }
                    FontType::Color => {
                        let ch_attr = TextAttribute::from_u8(
                            ch,
                            BufferType::LegacyIce,
                        ); // tdf fonts don't support ice mode by default
                        char_offset += 1;
                        AttributedChar::new(unsafe { char::from_u32_unchecked(glyph.data[char_offset] as u32) }, ch_attr)
                    }
                };
                if cur.x < buffer.get_width() && cur.y < buffer.get_line_count() {
                    buffer.layers[layer].set_char(cur, attributed_char);
                }
                cur.x += 1;
            }
            char_offset += 1;
        }

        Some(Size::new(glyph.size.width, cur.y - pos.y + 1))
    }

    pub const OUTLINE_STYLES: usize = 19;
    const OUTLINE_CHAR_SET: [[u8; 17]; TheDrawFont::OUTLINE_STYLES] = [
        [
            0xC4, 0xC4, 0xB3, 0xB3, 0xDA, 0xBF, 0xDA, 0xBF, 0xC0, 0xD9, 0xC0, 0xD9, 0xB4, 0xC3,
            0x20, 0x20, 0x20,
        ],
        [
            0xCD, 0xC4, 0xB3, 0xB3, 0xD5, 0xB8, 0xDA, 0xBF, 0xD4, 0xBE, 0xC0, 0xD9, 0xB5, 0xC3,
            0x20, 0x20, 0x20,
        ],
        [
            0xC4, 0xCD, 0xB3, 0xB3, 0xDA, 0xBF, 0xD5, 0xB8, 0xC0, 0xD9, 0xD4, 0xBE, 0xB4, 0xC6,
            0x20, 0x20, 0x20,
        ],
        [
            0xCD, 0xCD, 0xB3, 0xB3, 0xD5, 0xB8, 0xD5, 0xB8, 0xD4, 0xBE, 0xD4, 0xBE, 0xB5, 0xC6,
            0x20, 0x20, 0x20,
        ],
        [
            0xC4, 0xC4, 0xBA, 0xB3, 0xD6, 0xBF, 0xDA, 0xB7, 0xC0, 0xBD, 0xD3, 0xD9, 0xB6, 0xC3,
            0x20, 0x20, 0x20,
        ],
        [
            0xCD, 0xC4, 0xBA, 0xB3, 0xC9, 0xB8, 0xDA, 0xB7, 0xD4, 0xBC, 0xD3, 0xD9, 0xB9, 0xC3,
            0x20, 0x20, 0x20,
        ],
        [
            0xC4, 0xCD, 0xBA, 0xB3, 0xD6, 0xBF, 0xD5, 0xBB, 0xC0, 0xBD, 0xC8, 0xBE, 0xB6, 0xC6,
            0x20, 0x20, 0x20,
        ],
        [
            0xCD, 0xCD, 0xBA, 0xB3, 0xC9, 0xB8, 0xD5, 0xBB, 0xD4, 0xBC, 0xC8, 0xBE, 0xB9, 0xC6,
            0x20, 0x20, 0x20,
        ],
        [
            0xC4, 0xC4, 0xB3, 0xBA, 0xDA, 0xB7, 0xD6, 0xBF, 0xD3, 0xD9, 0xC0, 0xBD, 0xB4, 0xC7,
            0x20, 0x20, 0x20,
        ],
        [
            0xCD, 0xC4, 0xB3, 0xBA, 0xD5, 0xBB, 0xD6, 0xBF, 0xC8, 0xBE, 0xC0, 0xBD, 0xB5, 0xC7,
            0x20, 0x20, 0x20,
        ],
        [
            0xC4, 0xCD, 0xB3, 0xBA, 0xDA, 0xB7, 0xC9, 0xB8, 0xD3, 0xD9, 0xD4, 0xBC, 0xB4, 0xCC,
            0x20, 0x20, 0x20,
        ],
        [
            0xCD, 0xCD, 0xB3, 0xBA, 0xD5, 0xBB, 0xC9, 0xB8, 0xC8, 0xBE, 0xD4, 0xBC, 0xB5, 0xCC,
            0x20, 0x20, 0x20,
        ],
        [
            0xC4, 0xC4, 0xBA, 0xBA, 0xD6, 0xB7, 0xD6, 0xB7, 0xD3, 0xBD, 0xD3, 0xBD, 0xB6, 0xC7,
            0x20, 0x20, 0x20,
        ],
        [
            0xCD, 0xC4, 0xBA, 0xBA, 0xC9, 0xBB, 0xD6, 0xB7, 0xC8, 0xBC, 0xD3, 0xBD, 0xB9, 0xC7,
            0x20, 0x20, 0x20,
        ],
        [
            0xC4, 0xCD, 0xBA, 0xBA, 0xD6, 0xB7, 0xC9, 0xBB, 0xD3, 0xBD, 0xC8, 0xBC, 0xB6, 0xCC,
            0x20, 0x20, 0x20,
        ],
        [
            0xCD, 0xCD, 0xBA, 0xBA, 0xC9, 0xBB, 0xC9, 0xBB, 0xC8, 0xBC, 0xC8, 0xBC, 0xB9, 0xCC,
            0x20, 0x20, 0x20,
        ],
        [
            0xDC, 0xDC, 0xDB, 0xDB, 0xDC, 0xDC, 0xDC, 0xDC, 0xDB, 0xDB, 0xDB, 0xDB, 0xDB, 0xDB,
            0x20, 0x20, 0x20,
        ],
        [
            0xDF, 0xDF, 0xDB, 0xDB, 0xDB, 0xDB, 0xDB, 0xDB, 0xDF, 0xDF, 0xDF, 0xDF, 0xDB, 0xDB,
            0x20, 0x20, 0x20,
        ],
        [
            0xDF, 0xDC, 0xDE, 0xDD, 0xDE, 0xDD, 0xDC, 0xDC, 0xDF, 0xDF, 0xDE, 0xDD, 0xDB, 0xDB,
            0x20, 0x20, 0x20,
        ],
    ];
}

#[derive(Debug, Clone)]
pub enum TdfError {
    FileTooShort,
    IdMismatch,
    NameTooLong(usize),
    UnsupportedTtfType(u8),
    DataOverflow(usize),
    GlyphOutsideFontDataSize(usize),
    LetterSpaceTooMuch(i32),
    IdLengthMismatch(u8),
    FontIndicatorMismatch,
}

impl std::fmt::Display for TdfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TdfError::FileTooShort => write!(f, "file too short."),
            TdfError::IdMismatch => write!(f, "id mismatch."),
            TdfError::NameTooLong(len) => write!(f, "name too long was {len} max is {FONT_NAME_LEN}"),
            TdfError::UnsupportedTtfType(t) => write!(f, "unsupported ttf type {t}, only 0, 1, 2 are valid."),
            TdfError::DataOverflow(offset) => write!(f, "data overflow at offset {offset}"),
            TdfError::GlyphOutsideFontDataSize(i) => write!(f, "glyph {i} outside font data size"),
            TdfError::LetterSpaceTooMuch(spaces) => write!(f, "letter space is max {MAX_LETTER_SPACE} was {spaces}"),
            TdfError::IdLengthMismatch(len) => write!(f, "id length mismatch {len} should be 19."),
            TdfError::FontIndicatorMismatch => write!(f, "font indicator mismatch should be 0x55AA00FF."),
        }
    }
}

impl Error for TdfError {
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
    use crate::TheDrawFont;
    const TEST_FONT: &[u8] = include_bytes!("CODERX.TDF");
    #[test]
    fn test_load() {
        let result = TheDrawFont::from_tdf_bytes(TEST_FONT).unwrap();
        assert_eq!(6, result.len());
    }

    const TEST_FONT2: &[u8] = include_bytes!("CODERBLU.TDF");

    #[test]
    fn test_load_save() {
       let font = TheDrawFont::from_tdf_bytes(TEST_FONT2).unwrap();
       assert_eq!(1, font.len());

        let res = font[0].as_tdf_bytes().unwrap();
        
        assert_eq!(res.len(), TEST_FONT2.len());
    }
}