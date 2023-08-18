#![allow(dead_code)]

use std::io;

use crate::{ascii::CP437_TO_UNICODE, BufferType, EngineResult, Size};

use super::Buffer;

use sauce_errors::SauceError;
mod sauce_errors;

#[repr(u8)]
#[derive(Clone, Debug, Default)]
pub enum SauceDataType {
    /// Undefined filetype.
    /// You could use this to add SAUCE to a custom or proprietary file, without giving it any particular meaning or interpretation.
    #[default]
    Undefined = 0,

    /// A character based file.
    /// These files are typically interpreted sequentially. Also known as streams.  
    Character = 1,

    /// Bitmap graphic and animation files.
    Bitmap = 2,

    /// A vector graphic file.
    Vector = 3,

    /// An audio file.
    Audio = 4,

    /// This is a raw memory copy of a text mode screen. Also known as a .BIN file.
    /// This is essentially a collection of character and attribute pairs.
    BinaryText = 5,

    /// An XBin or eXtended BIN file.
    XBin = 6,

    /// An archive file.
    Archive = 7,

    ///  A executable file.
    Executable = 8,
}

impl SauceDataType {
    pub fn from(b: u8) -> SauceDataType {
        match b {
            0 => SauceDataType::Undefined,
            1 => SauceDataType::Character,
            2 => SauceDataType::Bitmap,
            3 => SauceDataType::Vector,
            4 => SauceDataType::Audio,
            5 => SauceDataType::BinaryText,
            6 => SauceDataType::XBin,
            7 => SauceDataType::Archive,
            8 => SauceDataType::Executable,
            _ => {
                log::error!("unknown sauce data type {b}");
                SauceDataType::Undefined
            }
        }
    }
}

const SAUCE_SIZE: i32 = 128;

#[derive(Debug, Clone, Copy)]
pub enum SauceFileType {
    Undefined,
    Ascii,
    Ansi,
    ANSiMation,
    PCBoard,
    Avatar,
    TundraDraw,
    Bin,
    XBin,
}

pub struct SauceData {
    pub title: SauceString<35, b' '>,
    pub author: SauceString<20, b' '>,
    pub group: SauceString<20, b' '>,
    pub comments: Vec<SauceString<64, 0>>,

    pub data_type: SauceDataType,
    pub buffer_size: Size<u16>,

    pub font_opt: Option<String>,
    pub use_ice: bool,
    pub sauce_header_len: usize,

    pub sauce_file_type: SauceFileType,
}

impl SauceData {
    /// .
    ///
    /// # Panics
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn extract(data: &[u8]) -> EngineResult<SauceData> {
        if data.len() < SAUCE_LEN {
            return Err(Box::new(SauceError::FileTooShort));
        }

        let mut o = data.len() - SAUCE_LEN;
        if SAUCE_ID != data[o..(o + 5)] {
            return Err(Box::new(SauceError::NoSauce));
        }
        o += 5;

        if b"00" != &data[o..(o + 2)] {
            return Err(Box::new(SauceError::UnsupportedSauceVersion(
                String::from_utf8_lossy(&data[o..(o + 2)]).to_string(),
            )));
        }

        let mut title = SauceString::<35, b' '>::new();
        let mut author = SauceString::<20, b' '>::new();
        let mut group = SauceString::<20, b' '>::new();
        let mut comments: Vec<SauceString<64, 0>> = Vec::new();
        let mut buffer_size = Size::<u16>::new(80, 25);
        let mut font_opt = None;
        let mut use_ice = false;

        o += 2;
        o += title.read(&data[o..]);
        o += author.read(&data[o..]);
        o += group.read(&data[o..]);

        // skip date
        o += 8;

        // skip file_size - we can calculate it, better than to rely on random 3rd party software.
        // Question: are there files where that is important?
        o += 4;

        let data_type = SauceDataType::from(data[o]);
        o += 1;
        let file_type = data[o];
        o += 1;
        let t_info1 = data[o] as i32 + ((data[o + 1] as i32) << 8);
        o += 2;
        let t_info2 = data[o] as i32 + ((data[o + 1] as i32) << 8);
        o += 2;
        // let t_info3 = data[o] as u16 + ((data[o + 1] as u16) << 8);
        o += 2;
        // let t_info4 = data[o] as u16 + ((data[o + 1] as u16) << 8);
        o += 2;
        let num_comments: u8 = data[o];
        o += 1;
        let t_flags: u8 = data[o];
        o += 1;
        let mut t_info_str: SauceString<22, 0> = SauceString::new();
        o += t_info_str.read(&data[o..]);
        assert_eq!(data.len(), o);

        let mut sauce_file_type = SauceFileType::Undefined;

        match data_type {
            SauceDataType::BinaryText => {
                buffer_size.width = (file_type as u16) << 1;
                sauce_file_type = SauceFileType::Bin;
                use_ice = (t_flags & ANSI_FLAG_NON_BLINK_MODE) == ANSI_FLAG_NON_BLINK_MODE;
                font_opt = Some(t_info_str.to_string());
            }
            SauceDataType::XBin => {
                buffer_size = Size::new(t_info1 as u16, t_info2 as u16);
                sauce_file_type = SauceFileType::XBin;
                // no flags according to spec
            }
            SauceDataType::Character => {
                match file_type {
                    SAUCE_FILE_TYPE_ASCII => {
                        buffer_size = Size::new(t_info1 as u16, t_info2 as u16);
                        sauce_file_type = SauceFileType::Ascii;
                        use_ice = (t_flags & ANSI_FLAG_NON_BLINK_MODE) == ANSI_FLAG_NON_BLINK_MODE;
                        font_opt = Some(t_info_str.to_string());
                    }
                    SAUCE_FILE_TYPE_ANSI => {
                        buffer_size = Size::new(t_info1 as u16, t_info2 as u16);
                        sauce_file_type = SauceFileType::Ansi;
                        use_ice = (t_flags & ANSI_FLAG_NON_BLINK_MODE) == ANSI_FLAG_NON_BLINK_MODE;
                        font_opt = Some(t_info_str.to_string());
                    }
                    SAUCE_FILE_TYPE_ANSIMATION => {
                        buffer_size = Size::new(t_info1 as u16, t_info2 as u16);
                        sauce_file_type = SauceFileType::ANSiMation;
                        use_ice = (t_flags & ANSI_FLAG_NON_BLINK_MODE) == ANSI_FLAG_NON_BLINK_MODE;
                        font_opt = Some(t_info_str.to_string());
                    }

                    SAUCE_FILE_TYPE_PCBOARD => {
                        buffer_size = Size::new(t_info1 as u16, t_info2 as u16);
                        sauce_file_type = SauceFileType::PCBoard;
                        // no flags according to spec
                    }
                    SAUCE_FILE_TYPE_AVATAR => {
                        buffer_size = Size::new(t_info1 as u16, t_info2 as u16);
                        sauce_file_type = SauceFileType::Avatar;
                        // no flags according to spec
                    }
                    SAUCE_FILE_TYPE_TUNDRA_DRAW => {
                        buffer_size = Size::new(t_info1 as u16, t_info2 as u16);
                        sauce_file_type = SauceFileType::TundraDraw;
                        // no flags according to spec
                    }
                    _ => {}
                }
            }
            _ => {
                log::error!(
                    "useless/invalid sauce info data type: {data_type:?} file type: {file_type}."
                );
            }
        }
        let len = if num_comments > 0 {
            if (data.len() - SAUCE_LEN) as i32 - num_comments as i32 * 64 - 5 < 0 {
                return Err(Box::new(SauceError::InvalidCommentBlock));
            }
            let comment_start = (data.len() - SAUCE_LEN) - num_comments as usize * 64 - 5;
            o = comment_start;
            if SAUCE_COMMENT_ID != data[o..(o + 5)] {
                return Err(Box::new(SauceError::InvalidCommentId(
                    String::from_utf8_lossy(&data[o..(o + 5)]).to_string(),
                )));
            }
            o += 5;
            for _ in 0..num_comments {
                let mut comment: SauceString<64, 0> = SauceString::new();
                o += comment.read(&data[o..]);
                comments.push(comment);
            }
            comment_start // -1 is from the EOF char
        } else {
            data.len() - SAUCE_LEN
        };

        let offset = len - 1; // -1 is from the EOF char

        Ok(SauceData {
            title,
            author,
            group,
            comments,
            data_type,
            buffer_size,
            font_opt,
            use_ice,
            sauce_header_len: data.len() - offset,
            sauce_file_type,
        })
    }
}

const SAUCE_FILE_TYPE_ASCII: u8 = 0;
const SAUCE_FILE_TYPE_ANSI: u8 = 1;
const SAUCE_FILE_TYPE_ANSIMATION: u8 = 2;
const SAUCE_FILE_TYPE_PCBOARD: u8 = 4;
const SAUCE_FILE_TYPE_AVATAR: u8 = 5;
const SAUCE_FILE_TYPE_TUNDRA_DRAW: u8 = 8;

#[derive(Clone, Default)]
pub struct SauceString<const LEN: usize, const EMPTY: u8>(Vec<u8>);

impl<const LEN: usize, const EMPTY: u8> std::fmt::Display for SauceString<LEN, EMPTY> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str = String::new();
        let len = self.len();
        for i in 0..len {
            let b = self.0[i];
            str.push(CP437_TO_UNICODE[b as usize]);
        }
        write!(f, "{str}")
    }
}

impl<const LEN: usize, const EMPTY: u8> PartialEq for SauceString<LEN, EMPTY> {
    fn eq(&self, other: &Self) -> bool {
        let l1 = self.len();
        let l2 = other.len();

        if l1 != l2 {
            return false;
        }

        self.0[0..l1] == other.0[0..l2]
    }
}

impl<const LEN: usize, const EMPTY: u8> std::fmt::Debug for SauceString<LEN, EMPTY> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(SauceString<{}> {})",
            LEN,
            String::from_utf8_lossy(&self.0)
        )
    }
}

impl<const LEN: usize, const EMPTY: u8> SauceString<LEN, EMPTY> {
    pub const EMPTY: SauceString<LEN, EMPTY> = SauceString(Vec::new());

    pub fn new() -> Self {
        SauceString(Vec::new())
    }

    pub fn from(str: impl Into<String>) -> Self {
        let mut data = Vec::new();
        for ch in str.into().chars() {
            if data.len() >= LEN {
                break;
            }
            let mut found = false;
            #[allow(clippy::needless_range_loop)]
            for i in 0..CP437_TO_UNICODE.len() {
                if ch == CP437_TO_UNICODE[i] {
                    data.push(i as u8);
                    found = true;
                    break;
                }
            }
            if !found {
                data.push(b'?');
            }
        }
        SauceString(data)
    }

    pub fn len(&self) -> usize {
        let mut len = self.0.len();
        while len > 0 {
            let ch = self.0[len - 1];
            if ch != 0 && ch != b' ' {
                break;
            }
            len -= 1;
        }
        len
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[allow(clippy::unused_self)]
    pub fn max_len(&self) -> usize {
        LEN
    }

    pub fn read(&mut self, data: &[u8]) -> usize {
        let mut last_non_empty = LEN;
        #[allow(clippy::needless_range_loop)]
        for i in 0..LEN {
            if EMPTY == 0 && data[i] == 0 {
                break;
            }
            if data[i] != EMPTY {
                last_non_empty = i + 1;
            }
            self.0.push(data[i]);
        }
        if last_non_empty < LEN {
            self.0.truncate(last_non_empty);
        }
        LEN
    }

    pub fn append_to(&self, vec: &mut Vec<u8>) {
        vec.extend(&self.0);
        if self.0.len() < LEN {
            vec.resize(vec.len() + LEN - self.0.len(), EMPTY);
        }
    }
}

/// | Field    | Type | Size | Descritption
/// |----------|------|------|-------------
/// | ID       | char | 5    | SAUCE comment block ID. This should be equal to "COMNT".
/// | Line 1   | char | 64   | Line of text.
/// | ...      |      |      |
/// | Line n   | char | 64   | Last line of text
const SAUCE_COMMENT_ID: [u8; 5] = *b"COMNT";
const SAUCE_ID: [u8; 5] = *b"SAUCE";
const SAUCE_LEN: usize = 128;
const ANSI_FLAG_NON_BLINK_MODE: u8 = 1;
static EMPTY_TINFO: SauceString<22, 0> = SauceString(Vec::new());

impl Buffer {
    /// .
    ///
    /// # Panics
    ///
    /// Panics if .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn read_sauce_info(&mut self, data: &[u8]) -> io::Result<(SauceFileType, usize)> {
        if data.len() < SAUCE_LEN {
            return Ok((SauceFileType::Undefined, data.len()));
        }

        let mut o = data.len() - SAUCE_LEN;
        if SAUCE_ID != data[o..(o + 5)] {
            return Ok((SauceFileType::Undefined, data.len()));
        }
        o += 5;

        if b"00" != &data[o..(o + 2)] {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Unsupported sauce version {}{}",
                    char::from_u32(data[o + 5] as u32).unwrap(),
                    char::from_u32(data[o + 6] as u32).unwrap()
                )
                .as_str(),
            ));
        }
        o += 2;
        o += self.title.read(&data[o..]);
        o += self.author.read(&data[o..]);
        o += self.group.read(&data[o..]);

        // skip date
        o += 8;

        // skip file_size - we can calculate it, better than to rely on random 3rd party software.
        // Question: are there files where that is important?
        o += 4;

        let data_type = SauceDataType::from(data[o]);
        o += 1;
        let file_type = data[o];
        o += 1;
        let t_info1 = data[o] as i32 + ((data[o + 1] as i32) << 8);
        o += 2;
        let t_info2 = data[o] as i32 + ((data[o + 1] as i32) << 8);
        o += 2;
        // let t_info3 = data[o] as u16 + ((data[o + 1] as u16) << 8);
        o += 2;
        // let t_info4 = data[o] as u16 + ((data[o + 1] as u16) << 8);
        o += 2;
        let num_comments: u8 = data[o];
        o += 1;
        let t_flags: u8 = data[o];
        o += 1;
        let mut t_info_str: SauceString<22, 0> = SauceString::new();
        o += t_info_str.read(&data[o..]);
        assert_eq!(data.len(), o);

        let mut sauce_file_type = SauceFileType::Undefined;

        match data_type {
            SauceDataType::BinaryText => {
                self.set_buffer_width((file_type as i32) << 1);
                sauce_file_type = SauceFileType::Bin;
                let use_ice = (t_flags & ANSI_FLAG_NON_BLINK_MODE) == ANSI_FLAG_NON_BLINK_MODE;
                if use_ice {
                    self.buffer_type = BufferType::LegacyIce;
                } else {
                    self.buffer_type = BufferType::LegacyDos;
                }
                // self.font = BitFont::from_name(&t_info_str.to_string()).unwrap_or_default();
            }
            SauceDataType::XBin => {
                self.set_buffer_width(t_info1);
                self.set_buffer_height(t_info2);
                sauce_file_type = SauceFileType::XBin;
                // no flags according to spec
            }
            SauceDataType::Character => {
                match file_type {
                    SAUCE_FILE_TYPE_ASCII => {
                        self.set_buffer_width(t_info1);
                        self.set_buffer_height(t_info2);
                        sauce_file_type = SauceFileType::Ascii;
                        let use_ice =
                            (t_flags & ANSI_FLAG_NON_BLINK_MODE) == ANSI_FLAG_NON_BLINK_MODE;
                        if use_ice {
                            self.buffer_type = BufferType::LegacyIce;
                        } else {
                            self.buffer_type = BufferType::LegacyDos;
                        }
                        // self.font = BitFont::from_name(&t_info_str.to_string()).unwrap_or_default();
                    }
                    SAUCE_FILE_TYPE_ANSI => {
                        self.set_buffer_width(t_info1);
                        self.set_buffer_height(t_info2);
                        sauce_file_type = SauceFileType::Ansi;
                        let use_ice =
                            (t_flags & ANSI_FLAG_NON_BLINK_MODE) == ANSI_FLAG_NON_BLINK_MODE;
                        if use_ice {
                            self.buffer_type = BufferType::LegacyIce;
                        } else {
                            self.buffer_type = BufferType::LegacyDos;
                        }
                        // self.font = BitFont::from_name(&t_info_str.to_string()).unwrap_or_default();
                    }
                    SAUCE_FILE_TYPE_ANSIMATION => {
                        self.set_buffer_width(t_info1);
                        self.set_buffer_height(t_info2);
                        sauce_file_type = SauceFileType::ANSiMation;
                        let use_ice =
                            (t_flags & ANSI_FLAG_NON_BLINK_MODE) == ANSI_FLAG_NON_BLINK_MODE;
                        if use_ice {
                            self.buffer_type = BufferType::LegacyIce;
                        } else {
                            self.buffer_type = BufferType::LegacyDos;
                        }
                        //  self.font = BitFont::from_name(&t_info_str.to_string()).unwrap_or_default();
                    }
                    SAUCE_FILE_TYPE_PCBOARD => {
                        self.set_buffer_width(t_info1);
                        self.set_buffer_height(t_info2);
                        sauce_file_type = SauceFileType::PCBoard;
                        // no flags according to spec
                    }
                    SAUCE_FILE_TYPE_AVATAR => {
                        self.set_buffer_width(t_info1);
                        self.set_buffer_height(t_info2);
                        sauce_file_type = SauceFileType::Avatar;
                        // no flags according to spec
                    }
                    SAUCE_FILE_TYPE_TUNDRA_DRAW => {
                        self.set_buffer_width(t_info1);
                        self.set_buffer_height(t_info2);
                        sauce_file_type = SauceFileType::TundraDraw;
                        // no flags according to spec
                    }
                    _ => {}
                }
            }
            _ => {
                log::error!(
                    "useless/invalid sauce info data type: {data_type:?} file type: {file_type}."
                );
            }
        }

        if num_comments == 0 {
            Ok((sauce_file_type, data.len() - SAUCE_LEN - 1)) // -1 is from the EOF char
        } else if (data.len() - SAUCE_LEN) as i32 - num_comments as i32 * 64 - 5 < 0 {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid sauce comment block",
            ))
        } else {
            let comment_start = (data.len() - SAUCE_LEN) - num_comments as usize * 64 - 5;
            o = comment_start;
            if SAUCE_COMMENT_ID != data[o..(o + 5)] {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Invalid SAUCE comment id {}",
                        String::from_utf8_lossy(&data[o..(o + 5)])
                    ),
                ));
            }
            o += 5;
            for _ in 0..num_comments {
                let mut comment: SauceString<64, 0> = SauceString::new();
                o += comment.read(&data[o..]);
                self.comments.push(comment);
            }
            Ok((sauce_file_type, comment_start - 1)) // -1 is from the EOF char
        }
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
    pub fn write_sauce_info(
        &self,
        sauce_file_type: &SauceFileType,
        vec: &mut Vec<u8>,
    ) -> io::Result<bool> {
        vec.push(0x1A); // EOF Char.
        let file_size = vec.len() as u32;
        if !self.comments.is_empty() {
            if self.comments.len() > 255 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "sauce comments exceed maximum of 255: {}.",
                        self.comments.len()
                    )
                    .as_str(),
                ));
            }
            vec.extend(SAUCE_COMMENT_ID);
            for cmt in &self.comments {
                cmt.append_to(vec);
            }
        }

        vec.extend(SAUCE_ID);
        vec.push(b'0');
        vec.push(b'0');
        self.title.append_to(vec);
        self.author.append_to(vec);
        self.group.append_to(vec);
        // TODO: Dates
        vec.extend(b"20130504");
        vec.extend(u32::to_le_bytes(file_size));

        let data_type;
        let file_type;
        let mut t_info1 = 0;
        let mut t_info2 = 0;
        let t_info3 = 0;
        let t_info4 = 0;
        let mut t_flags = 0;
        let mut t_info_str = &self.get_font(0).unwrap().name;

        match sauce_file_type {
            SauceFileType::Ascii => {
                data_type = SauceDataType::Character;
                file_type = SAUCE_FILE_TYPE_ASCII;
                t_info1 = self.get_buffer_width();
                t_info2 = self.get_buffer_height();

                if self.buffer_type.use_ice_colors() { t_flags |= ANSI_FLAG_NON_BLINK_MODE; }
            },
            SauceFileType::Undefined | // map everything else just to ANSI
            SauceFileType::Ansi => {
                data_type = SauceDataType::Character;
                file_type = SAUCE_FILE_TYPE_ANSI;
                t_info1 = self.get_buffer_width();
                t_info2 = self.get_buffer_height();
                if self.buffer_type.use_ice_colors() { t_flags |= ANSI_FLAG_NON_BLINK_MODE; }
            },
            SauceFileType::ANSiMation => {
                data_type = SauceDataType::Character;
                file_type = SAUCE_FILE_TYPE_ANSIMATION;
                t_info1 = self.get_buffer_width();
                t_info2 = self.get_buffer_height();
                if self.buffer_type.use_ice_colors() { t_flags |= ANSI_FLAG_NON_BLINK_MODE; }
            },
            SauceFileType::PCBoard => {
                data_type = SauceDataType::Character;
                file_type = SAUCE_FILE_TYPE_PCBOARD;
                t_info1 = self.get_buffer_width();
                t_info2 = self.get_buffer_height();
                // no flags
                t_info_str = &EMPTY_TINFO;
            },
            SauceFileType::Avatar => {
                data_type = SauceDataType::Character;
                file_type = SAUCE_FILE_TYPE_AVATAR;
                t_info1 = self.get_buffer_width();
                t_info2 = self.get_buffer_height();
                // no flags
                t_info_str = &EMPTY_TINFO;
            },
            SauceFileType::TundraDraw => {
                data_type = SauceDataType::Character;
                file_type = SAUCE_FILE_TYPE_TUNDRA_DRAW;
                t_info1 = self.get_buffer_width();
                // no flags
                t_info_str = &EMPTY_TINFO;
            }
            SauceFileType::Bin => {
                data_type = SauceDataType::BinaryText;
                let w = self.get_buffer_width() / 2;
                if w > u8::MAX as i32 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "BIN files can only be saved up to 510 width."));
                }
                file_type = w as u8;
                if self.buffer_type.use_ice_colors() { t_flags |= ANSI_FLAG_NON_BLINK_MODE; }
            },
            SauceFileType::XBin => {
                data_type = SauceDataType::XBin;
                file_type = 0;
                t_info1 = self.get_buffer_width();
                t_info2 = self.get_buffer_height();
                // no flags
                t_info_str = &EMPTY_TINFO;
            }
        }

        vec.push(data_type as u8);
        vec.push(file_type);
        vec.extend(u16::to_le_bytes(t_info1 as u16));
        vec.extend(u16::to_le_bytes(t_info2 as u16));
        vec.extend(u16::to_le_bytes(t_info3));
        vec.extend(u16::to_le_bytes(t_info4));
        vec.push(self.comments.len() as u8); // comment len is checked above for <= 255
        vec.push(t_flags);
        t_info_str.append_to(vec);

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::SauceString;

    #[test]
    fn test_sauce_string_string_conversion() {
        let str = SauceString::<20, 0>::from("Hello World!");
        assert_eq!("Hello World!", str.to_string());
    }
}
