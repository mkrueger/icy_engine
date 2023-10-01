use crate::{ascii::CP437_TO_UNICODE, EngineResult, IceMode, Size, TextPane};

use super::Buffer;

use chrono::{NaiveDateTime, Utc};
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

#[derive(Default, Debug, Clone, Copy)]
pub enum SauceFileType {
    #[default]
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

#[derive(Clone, Default)]
pub struct SauceData {
    pub title: SauceString<35, b' '>,
    pub author: SauceString<20, b' '>,
    pub group: SauceString<20, b' '>,
    pub comments: Vec<SauceString<64, 0>>,

    pub data_type: SauceDataType,
    pub buffer_size: Size,

    pub creation_time: NaiveDateTime,

    pub font_opt: Option<String>,
    pub use_ice: bool,
    pub use_letter_spacing: bool,
    pub use_aspect_ratio: bool,
    pub sauce_header_len: usize,

    pub sauce_file_type: SauceFileType,
}

impl SauceData {
    /// .
    ///
    /// # Panics
    /// # Errors
    ///
    /// This function will return an error if the file con
    pub fn extract(data: &[u8]) -> EngineResult<Option<SauceData>> {
        if data.len() < SAUCE_LEN {
            return Ok(None);
        }

        let mut o = data.len() - SAUCE_LEN;
        if SAUCE_ID != data[o..(o + 5)] {
            return Ok(None);
        }
        o += 5;

        if b"00" != &data[o..(o + 2)] {
            return Err(SauceError::UnsupportedSauceVersion(String::from_utf8_lossy(&data[o..(o + 2)]).to_string()).into());
        }

        let mut title = SauceString::<35, b' '>::new();
        let mut author = SauceString::<20, b' '>::new();
        let mut group = SauceString::<20, b' '>::new();
        let mut comments: Vec<SauceString<64, 0>> = Vec::new();
        let mut buffer_size = Size::new(80, 25);
        let mut font_opt = None;
        let mut use_ice = false;
        let mut use_letter_spacing = false;
        let mut use_aspect_ratio = false;

        o += 2;
        o += title.read(&data[o..]);
        o += author.read(&data[o..]);
        o += group.read(&data[o..]);

        let mut date_string = String::from_utf8_lossy(&data[o..(o + 8)]).to_string();
        date_string.push_str("000000"); // otherwise the datetime parser will fail, it needs full info
        let date_time = match NaiveDateTime::parse_from_str(&date_string, "%Y%m%d%H%M%S") {
            Ok(d) => d,
            Err(err) => return Err(SauceError::UnsupportedSauceDate(err.to_string()).into()),
        };
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
                buffer_size.width = ((file_type as u16) << 1) as i32;
                sauce_file_type = SauceFileType::Bin;
                use_ice = (t_flags & ANSI_FLAG_NON_BLINK_MODE) == ANSI_FLAG_NON_BLINK_MODE;
                font_opt = Some(t_info_str.to_string());
            }
            SauceDataType::XBin => {
                buffer_size = Size::new(t_info1, t_info2);
                sauce_file_type = SauceFileType::XBin;
                // no flags according to spec
            }
            SauceDataType::Character => {
                match file_type {
                    SAUCE_FILE_TYPE_ASCII => {
                        buffer_size = Size::new(t_info1, t_info2);
                        sauce_file_type = SauceFileType::Ascii;
                        use_ice = (t_flags & ANSI_FLAG_NON_BLINK_MODE) == ANSI_FLAG_NON_BLINK_MODE;

                        match t_flags & ANSI_MASK_LETTER_SPACING {
                            ANSI_LETTER_SPACING_LEGACY | ANSI_LETTER_SPACING_8PX => {
                                use_letter_spacing = false;
                            }
                            ANSI_LETTER_SPACING_9PX => use_letter_spacing = true,
                            _ => {}
                        }

                        match t_flags & ANSI_MASK_ASPECT_RATIO {
                            ANSI_ASPECT_RATIO_SQUARE | ANSI_ASPECT_RATIO_LEGACY => {
                                use_aspect_ratio = false;
                            }
                            ANSI_ASPECT_RATIO_STRETCH => use_aspect_ratio = true,
                            _ => {}
                        }

                        font_opt = Some(t_info_str.to_string());
                    }
                    SAUCE_FILE_TYPE_ANSI => {
                        buffer_size = Size::new(t_info1, t_info2);
                        sauce_file_type = SauceFileType::Ansi;
                        use_ice = (t_flags & ANSI_FLAG_NON_BLINK_MODE) == ANSI_FLAG_NON_BLINK_MODE;
                        font_opt = Some(t_info_str.to_string());
                    }
                    SAUCE_FILE_TYPE_ANSIMATION => {
                        buffer_size = Size::new(t_info1, t_info2);
                        sauce_file_type = SauceFileType::ANSiMation;
                        use_ice = (t_flags & ANSI_FLAG_NON_BLINK_MODE) == ANSI_FLAG_NON_BLINK_MODE;
                        font_opt = Some(t_info_str.to_string());
                    }

                    SAUCE_FILE_TYPE_PCBOARD => {
                        buffer_size = Size::new(t_info1, t_info2);
                        sauce_file_type = SauceFileType::PCBoard;
                        // no flags according to spec
                    }
                    SAUCE_FILE_TYPE_AVATAR => {
                        buffer_size = Size::new(t_info1, t_info2);
                        sauce_file_type = SauceFileType::Avatar;
                        // no flags according to spec
                    }
                    SAUCE_FILE_TYPE_TUNDRA_DRAW => {
                        buffer_size = Size::new(t_info1, t_info2);
                        sauce_file_type = SauceFileType::TundraDraw;
                        // no flags according to spec
                    }
                    _ => {}
                }
            }
            _ => {
                log::error!("useless/invalid sauce info data type: {data_type:?} file type: {file_type}.");
            }
        }
        let len = if num_comments > 0 {
            if (data.len() - SAUCE_LEN) as i32 - num_comments as i32 * 64 - 5 < 0 {
                return Err(SauceError::InvalidCommentBlock.into());
            }
            let comment_start = (data.len() - SAUCE_LEN) - num_comments as usize * 64 - 5;
            o = comment_start;
            if SAUCE_COMMENT_ID != data[o..(o + 5)] {
                return Err(SauceError::InvalidCommentId(String::from_utf8_lossy(&data[o..(o + 5)]).to_string()).into());
            }
            o += 5;
            for _ in 0..num_comments {
                let mut comment: SauceString<64, 0> = SauceString::new();
                o += comment.read(&data[o..]);
                comments.push(comment);
            }
            comment_start
        } else {
            data.len() - SAUCE_LEN
        };

        let offset = len - 1; // -1 is from the EOF char

        Ok(Some(SauceData {
            title,
            author,
            group,
            comments,
            data_type,
            creation_time: date_time,
            buffer_size,
            font_opt,
            use_ice,
            use_letter_spacing,
            use_aspect_ratio,
            sauce_header_len: data.len() - offset,
            sauce_file_type,
        }))
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
        write!(f, "(SauceString<{}> {})", LEN, String::from_utf8_lossy(&self.0))
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
const ANSI_FLAG_NON_BLINK_MODE: u8 = 0b0000_0001;
const ANSI_MASK_LETTER_SPACING: u8 = 0b0000_0110;
const ANSI_LETTER_SPACING_LEGACY: u8 = 0b0000_0000;
const ANSI_LETTER_SPACING_8PX: u8 = 0b0000_0010;
const ANSI_LETTER_SPACING_9PX: u8 = 0b0000_0100;

const ANSI_MASK_ASPECT_RATIO: u8 = 0b0001_1000;
const ANSI_ASPECT_RATIO_LEGACY: u8 = 0b0000_0000;
const ANSI_ASPECT_RATIO_STRETCH: u8 = 0b0000_1000;
const ANSI_ASPECT_RATIO_SQUARE: u8 = 0b0001_0000;

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
    pub fn write_sauce_info(&self, sauce_file_type: SauceFileType, vec: &mut Vec<u8>) -> EngineResult<bool> {
        vec.push(0x1A); // EOF Char.
        let file_size = vec.len() as u32;
        let mut comment_len = 0;
        if let Some(data) = self.get_sauce() {
            if !data.comments.is_empty() {
                if data.comments.len() > 255 {
                    return Err(SauceError::CommentLimitExceeded(data.comments.len()).into());
                }
                comment_len = data.comments.len() as u8;
                vec.extend(SAUCE_COMMENT_ID);
                for cmt in &data.comments {
                    cmt.append_to(vec);
                }
            }
        }
        vec.extend(SAUCE_ID);
        vec.push(b'0');
        vec.push(b'0');

        if let Some(data) = self.get_sauce() {
            data.title.append_to(vec);
            data.author.append_to(vec);
            data.group.append_to(vec);
        } else {
            SauceData::default().title.append_to(vec);
            SauceData::default().author.append_to(vec);
            SauceData::default().group.append_to(vec);
        }

        let cur_time = Utc::now();
        let date_time = cur_time.format("%Y%m%d").to_string();
        assert_eq!(date_time.len(), 8);
        vec.extend(date_time.bytes());
        vec.extend(u32::to_le_bytes(file_size));

        let data_type;
        let file_type;
        let mut t_info1 = 0;
        let mut t_info2 = 0;
        let t_info3 = 0;
        let t_info4 = 0;
        let mut t_flags = 0;
        let mut t_info_str = self.get_font(0).unwrap().name.clone();

        match sauce_file_type {
            SauceFileType::Ascii => {
                data_type = SauceDataType::Character;
                file_type = SAUCE_FILE_TYPE_ASCII;
                t_info1 = self.get_width();
                t_info2 = self.get_height();

                if matches!(self.ice_mode, IceMode::Ice) { t_flags |= ANSI_FLAG_NON_BLINK_MODE; }
            },
            SauceFileType::Undefined | // map everything else just to ANSI
            SauceFileType::Ansi => {
                data_type = SauceDataType::Character;
                file_type = SAUCE_FILE_TYPE_ANSI;
                t_info1 = self.get_width();
                t_info2 = self.get_height();
                if matches!(self.ice_mode, IceMode::Ice) { t_flags |= ANSI_FLAG_NON_BLINK_MODE; }
                if let Some(sauce_data) = self.get_sauce() {
                    if sauce_data.use_aspect_ratio { t_flags |= ANSI_ASPECT_RATIO_LEGACY; }
                    if sauce_data.use_letter_spacing { t_flags |= ANSI_LETTER_SPACING_9PX; }
                }
            },
            SauceFileType::ANSiMation => {
                data_type = SauceDataType::Character;
                file_type = SAUCE_FILE_TYPE_ANSIMATION;
                t_info1 = self.get_width();
                t_info2 = self.get_height();
                if matches!(self.ice_mode, IceMode::Ice) { t_flags |= ANSI_FLAG_NON_BLINK_MODE; }
            },
            SauceFileType::PCBoard => {
                data_type = SauceDataType::Character;
                file_type = SAUCE_FILE_TYPE_PCBOARD;
                t_info1 = self.get_width();
                t_info2 = self.get_height();
                // no flags
                t_info_str = String::new();
            },
            SauceFileType::Avatar => {
                data_type = SauceDataType::Character;
                file_type = SAUCE_FILE_TYPE_AVATAR;
                t_info1 = self.get_width();
                t_info2 = self.get_height();
                // no flags
                t_info_str = String::new();
            },
            SauceFileType::TundraDraw => {
                data_type = SauceDataType::Character;
                file_type = SAUCE_FILE_TYPE_TUNDRA_DRAW;
                t_info1 = self.get_width();
                // no flags
                t_info_str = String::new();
            }
            SauceFileType::Bin => {
                data_type = SauceDataType::BinaryText;
                let w = self.get_width() / 2;
                if w > u8::MAX as i32 {
                    return Err(SauceError::BinFileWidthLimitExceeded(w).into());
                }
                file_type = w as u8;
                if matches!(self.ice_mode, IceMode::Ice) { t_flags |= ANSI_FLAG_NON_BLINK_MODE; }
            },
            SauceFileType::XBin => {
                data_type = SauceDataType::XBin;
                file_type = 0;
                t_info1 = self.get_width();
                t_info2 = self.get_height();
                // no flags
                t_info_str = String::new();
            }
        }

        vec.push(data_type as u8);
        vec.push(file_type);
        vec.extend(u16::to_le_bytes(t_info1 as u16));
        vec.extend(u16::to_le_bytes(t_info2 as u16));
        vec.extend(u16::to_le_bytes(t_info3));
        vec.extend(u16::to_le_bytes(t_info4));
        vec.push(comment_len); // comment len is checked above for <= 255
        vec.push(t_flags);
        let t_info_str: SauceString<22, 0> = SauceString::from(t_info_str);
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
