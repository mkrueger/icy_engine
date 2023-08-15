use std::collections::{HashMap, VecDeque};
use std::ffi::OsStr;
use std::{
    cmp::max,
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

use num::NumCast;

use crate::{parsers, BufferParser, Caret, EngineResult, Glyph, Sixel, TerminalState};

use super::{
    read_binary, read_xb, AttributedChar, BitFont, Layer, Palette, Position, SauceString,
    SaveOptions, Size,
};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BufferType {
    LegacyDos = 0b_0000,  // 0-15 fg, 0-7 bg, blink
    LegacyIce = 0b_0001,  // 0-15 fg, 0-15 bg
    ExtFont = 0b_0010,    // 0-7 fg, 0-7 bg, blink + extended font
    ExtFontIce = 0b_0011, // 0-7 fg, 0-15 bg + extended font
    NoLimits = 0b_0111,   // free colors, blink + extended font
}

enum CharInterpreter {
    Ascii,
    Ansi,
    Avatar,
    Pcb,
    Petscii,
}

impl BufferType {
    #[must_use]
    pub fn use_ice_colors(self) -> bool {
        self == BufferType::LegacyIce || self == BufferType::ExtFontIce
    }

    #[must_use]
    pub fn use_blink(self) -> bool {
        self == BufferType::LegacyDos || self == BufferType::ExtFont || self == BufferType::NoLimits
    }

    #[must_use]
    pub fn use_extended_font(self) -> bool {
        self == BufferType::ExtFont || self == BufferType::ExtFontIce
    }

    #[must_use]
    pub fn get_fg_colors(self) -> u8 {
        match self {
            BufferType::LegacyDos | BufferType::LegacyIce | BufferType::NoLimits => 16, // may change in the future
            BufferType::ExtFont | BufferType::ExtFontIce => 8,
        }
    }

    #[must_use]
    pub fn get_bg_colors(self) -> u8 {
        match self {
            BufferType::LegacyDos | BufferType::ExtFont => 8,
            BufferType::LegacyIce | BufferType::ExtFontIce | BufferType::NoLimits => 16, // may change in the future
        }
    }
}

pub struct Buffer {
    pub file_name: Option<PathBuf>,
    //pub file_name_changed: Box<dyn Fn ()>,
    pub title: SauceString<35, b' '>,
    pub author: SauceString<20, b' '>,
    pub group: SauceString<20, b' '>,
    pub comments: Vec<SauceString<64, 0>>,

    pub terminal_state: TerminalState,
    pub buffer_type: BufferType,
    pub is_terminal_buffer: bool,

    pub palette: Palette,
    pub overlay_layer: Option<Layer>,

    font_table: HashMap<usize, BitFont>,
    is_font_table_dirty: bool,

    pub layers: Vec<Layer>,

    pub sixel_threads: VecDeque<std::thread::JoinHandle<Sixel>>, // pub undo_stack: Vec<Box<dyn UndoOperation>>,
                                                                 // pub redo_stack: Vec<Box<dyn UndoOperation>>,
}

#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Buffer")
            .field("file_name", &self.file_name)
            .field("width", &self.get_buffer_width())
            .field("height", &self.get_buffer_height())
            .field("custom_palette", &self.palette)
            .field("layers", &self.layers)
            .finish()
    }
}

impl std::fmt::Display for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str = String::new();
        let p = parsers::ansi::Parser::default();

        for y in 0..self.get_buffer_height() {
            str.extend(format!("{y:3}: ").chars());
            for x in 0..self.get_buffer_width() {
                let ch = self.get_char_xy(x, y).unwrap_or_default();
                str.push(p.convert_to_unicode(ch.ch));
            }
            str.push('\n');
        }
        write!(f, "{str}")
    }
}

impl Buffer {
    pub fn new() -> Self {
        let mut font_table = HashMap::new();
        font_table.insert(0, BitFont::default());
        Buffer {
            file_name: None,
            terminal_state: TerminalState::from(80, 25),

            title: SauceString::new(),
            author: SauceString::new(),
            group: SauceString::new(),
            comments: Vec::new(),

            buffer_type: BufferType::LegacyDos,
            is_terminal_buffer: false,
            palette: Palette::new(),

            font_table,
            is_font_table_dirty: false,
            overlay_layer: None,
            layers: vec![Layer::new()],
            sixel_threads: VecDeque::new(), // file_name_changed: Box::new(|| {}),
                                            // undo_stack: Vec::new(),
                                            // redo_stack: Vec::new()
        }
    }

    /// Returns the update sixel threads of this [`Buffer`].
    ///
    /// # Panics
    ///
    /// Panics if .
    pub fn update_sixel_threads(&mut self) {
        if let Some(handle) = self.sixel_threads.front() {
            if !handle.is_finished() {
                return;
            }
            let handle = self.sixel_threads.pop_front().unwrap();
            let sixel = handle.join().unwrap();
            {
                let screen_rect = sixel.get_screen_rect();

                let vec = &mut self.layers[0].sixels;
                let mut sixel_count = vec.len();
                // remove old sixel that are shadowed by the new one
                let mut i = 0;
                while i < sixel_count {
                    let old_rect = vec[i].get_screen_rect();
                    if screen_rect.contains_rect(&old_rect) {
                        vec.remove(i);
                        sixel_count -= 1;
                    } else {
                        i += 1;
                    }
                }
                vec.push(sixel);
            }
            self.layers[0].updated_sixels = true;
        }
    }

    pub fn clear_font_table(&mut self) {
        self.font_table.clear();
        self.is_font_table_dirty = true;
    }

    pub fn has_fonts(&self) -> bool {
        !self.font_table.is_empty()
    }

    pub fn is_font_table_updated(&self) -> bool {
        self.is_font_table_dirty
    }

    pub fn set_font_table_is_updated(&mut self) {
        self.is_font_table_dirty = false;
    }

    pub fn search_font_by_name(&self, name: impl Into<String>) -> Option<usize> {
        let name = name.into();
        for (i, font) in &self.font_table {
            if font.name.to_string() == name {
                return Some(*i);
            }
        }
        None
    }

    pub fn font_iter(&self) -> impl Iterator<Item = (&usize, &BitFont)> {
        self.font_table.iter()
    }

    pub fn get_font(&self, font_number: usize) -> Option<&BitFont> {
        self.font_table.get(&font_number)
    }

    pub fn set_font(&mut self, font_number: usize, font: BitFont) {
        self.font_table.insert(font_number, font);
        self.is_font_table_dirty = true;
    }

    pub fn font_count(&self) -> usize {
        self.font_table.len()
    }

    pub fn append_font(&mut self, font: BitFont) -> usize {
        let mut i = 0;
        while self.font_table.contains_key(&i) {
            i += 1;
        }
        self.font_table.insert(i, font);
        i
    }

    pub fn get_buffer_width(&self) -> i32 {
        self.terminal_state.width
    }

    pub fn get_buffer_height(&self) -> i32 {
        self.terminal_state.height
    }

    pub fn get_real_buffer_height(&self) -> i32 {
        if let Some(len) = self.layers.iter().map(|l| l.lines.len()).max() {
            len as i32
        } else {
            self.terminal_state.height
        }
    }

    pub fn get_real_buffer_width(&self) -> i32 {
        let mut w = 0;
        for layer in &self.layers {
            for line in &layer.lines {
                w = max(w, line.get_line_length());
            }
        }
        (w as i32) + 1
    }

    /// Sets the buffer size of this [`Buffer`].
    ///
    /// # Panics
    ///
    /// Panics if .
    pub fn set_buffer_size<T: NumCast>(&mut self, size: Size<T>) {
        self.terminal_state.width = num::cast(size.width).unwrap();
        self.terminal_state.height = num::cast(size.height).unwrap();
        self.terminal_state.reset();
    }

    /// Sets the buffer width of this [`Buffer`].
    ///
    /// # Panics
    ///
    /// Panics if .
    pub fn set_buffer_width<T: NumCast>(&mut self, width: T) {
        self.terminal_state.width = num::cast(width).unwrap();
        self.terminal_state.reset();
    }

    /// Sets the buffer height of this [`Buffer`].
    ///
    /// # Panics
    ///
    /// Panics if .
    pub fn set_buffer_height<T: NumCast>(&mut self, height: T) {
        self.terminal_state.height = num::cast(height).unwrap();
    }

    /// Returns the clear of this [`Buffer`].
    ///
    /// # Panics
    ///
    /// Panics if .
    pub fn clear(&mut self) {
        self.layers[0].clear();
        self.layers[0].sixels.clear();
        self.sixel_threads.clear();
    }

    /// terminal buffers have a viewport on the bottom of the buffer
    /// this function gives back the first visible line.
    #[must_use]
    pub fn get_first_visible_line(&self) -> i32 {
        if self.is_terminal_buffer {
            max(
                0,
                self.layers[0].lines.len() as i32 - self.get_buffer_height(),
            )
        } else {
            0
        }
    }

    pub fn get_last_visible_line(&self) -> i32 {
        self.get_first_visible_line() + self.get_buffer_height()
    }

    pub fn get_first_editable_line(&self) -> i32 {
        if self.is_terminal_buffer {
            if let Some((start, _)) = self.terminal_state.margins_up_down {
                return self.get_first_visible_line() + start;
            }
        }
        self.get_first_visible_line()
    }

    pub fn get_first_editable_column(&self) -> i32 {
        if self.is_terminal_buffer {
            if let Some((start, _)) = self.terminal_state.margins_left_right {
                return start;
            }
        }
        0
    }

    pub fn get_last_editable_column(&self) -> i32 {
        if self.is_terminal_buffer {
            if let Some((_, end)) = self.terminal_state.margins_left_right {
                return end;
            }
        }
        self.get_buffer_width() - 1
    }

    #[must_use]
    pub fn needs_scrolling(&self) -> bool {
        self.is_terminal_buffer /*&& self.terminal_state.margins.is_some()*/
    }

    #[must_use]
    pub fn get_last_editable_line(&self) -> i32 {
        if self.is_terminal_buffer {
            if let Some((_, end)) = self.terminal_state.margins_up_down {
                self.get_first_visible_line() + end
            } else {
                self.get_first_visible_line() + self.get_buffer_height() - 1
            }
        } else {
            max(
                self.layers[0].lines.len() as i32,
                self.get_buffer_height() - 1,
            )
        }
    }

    #[must_use]
    pub fn upper_left_position(&self) -> Position {
        match self.terminal_state.origin_mode {
            crate::OriginMode::UpperLeftCorner => Position {
                x: 0,
                y: self.get_first_visible_line(),
            },
            crate::OriginMode::WithinMargins => Position {
                x: 0,
                y: self.get_first_editable_line(),
            },
        }
    }

    #[must_use]
    pub fn create(width: i32, height: i32) -> Self {
        let mut res = Buffer::new();
        res.set_buffer_width(width);
        res.set_buffer_height(height);
        res.layers[0].is_locked = true;
        res.layers[0].is_transparent = false;
        res.layers[0]
            .lines
            .resize(height as usize, crate::Line::create(width as u16));

        let mut editing_layer = Layer::new();
        editing_layer.title = "Editing".to_string();
        res.layers.insert(0, editing_layer);
        res
    }

    pub fn get_overlay_layer(&mut self) -> &mut Option<Layer> {
        if self.overlay_layer.is_none() {
            self.overlay_layer = Some(Layer::new());
        }

        &mut self.overlay_layer
    }

    pub fn remove_overlay(&mut self) -> Option<Layer> {
        self.overlay_layer.take()
    }

    #[must_use]
    pub fn get_glyph(&self, ch: &AttributedChar) -> Option<&Glyph> {
        if let Some(ext) = &self.get_font(ch.get_font_page()) {
            return ext.get_glyph(ch.ch);
        }
        None
    }

    #[must_use]
    pub fn get_font_dimensions(&self) -> Size<u8> {
        self.font_table[&0].size
    }

    pub fn set_char(&mut self, layer: usize, pos: Position, dos_char: Option<AttributedChar>) {
        if layer >= self.layers.len() {
            return;
        }

        let cur_layer = &mut self.layers[layer];
        cur_layer.set_char(pos, dos_char);
    }

    pub fn get_char_from_layer(&self, layer: usize, pos: Position) -> Option<AttributedChar> {
        if let Some(layer) = self.layers.get(layer) {
            layer.get_char(pos)
        } else {
            None
        }
    }

    pub fn get_char_xy(&self, x: i32, y: i32) -> Option<AttributedChar> {
        self.get_char(Position::new(x, y))
    }

    pub fn get_char(&self, pos: Position) -> Option<AttributedChar> {
        if let Some(overlay) = &self.overlay_layer {
            let ch = overlay.get_char(pos);
            if ch.is_some() {
                return ch;
            }
        }

        for i in 0..self.layers.len() {
            let cur_layer = &self.layers[i];
            if !cur_layer.is_visible {
                continue;
            }
            let ch = cur_layer.get_char(pos);
            if ch.is_some() {
                return ch;
            }
        }

        None
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn load_buffer(file_name: &Path, skip_errors: bool) -> EngineResult<Buffer> {
        let mut f = File::open(file_name)?;
        let mut bytes = Vec::new();
        f.read_to_end(&mut bytes)?;

        Buffer::from_bytes(file_name, skip_errors, &bytes)
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn to_bytes(&self, extension: &str, options: &SaveOptions) -> io::Result<Vec<u8>> {
        match extension {
            "mdf" => super::convert_to_mdf(self),
            "bin" => super::convert_to_binary(self, options),
            "xb" => super::convert_to_xb(self, options),
            "ice" | "ans" => super::convert_to_ans(self, options),
            "avt" => super::convert_to_avt(self, options),
            "pcb" => super::convert_to_pcb(self, options),
            "adf" => super::convert_to_adf(self, options),
            "idf" => super::convert_to_idf(self, options),
            "tnd" => super::convert_to_tnd(self, options),
            _ => super::convert_to_asc(self, options),
        }
    }

    pub fn get_save_sauce_default(&self, extension: &str) -> (bool, String) {
        match extension {
            "bin" => super::get_save_sauce_default_binary(self),
            "xb" => super::get_save_sauce_default_xb(self),
            "ice" | "ans" => super::get_save_sauce_default_ans(self),
            "avt" => super::get_save_sauce_default_avt(self),
            "pcb" => super::get_save_sauce_default_pcb(self),
            "adf" => super::get_save_sauce_default_adf(self),
            "idf" => super::get_save_sauce_default_idf(self),
            "tnd" => super::get_save_sauce_default_tnd(self),
            _ => super::get_save_sauce_default_asc(self),
        }
    }

    pub fn has_sauce_relevant_data(&self) -> bool {
        !self.title.is_empty()
            || !self.group.is_empty()
            || !self.author.is_empty()
            || !self.comments.is_empty()
            || self.font_table[&0].name.to_string() != super::DEFAULT_FONT_NAME
                && self.font_table[&0].name.to_string() != super::ALT_DEFAULT_FONT_NAME
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
    pub fn from_bytes(file_name: &Path, skip_errors: bool, bytes: &[u8]) -> EngineResult<Buffer> {
        let mut result = Buffer::new();
        result.is_terminal_buffer = false;
        result.file_name = Some(file_name.to_path_buf());
        let ext = file_name.extension();

        if let Some(ext) = ext {
            // mdf doesn't need sauce info.
            let ext = OsStr::to_str(ext).unwrap().to_lowercase();
            if ext.as_str() == "mdf" {
                super::read_mdf(&mut result, bytes)?;
                return Ok(result);
            }
        }

        let (sauce_type, file_size) = result.read_sauce_info(bytes)?;
        let mut check_extension = false;
        let mut interpreter = CharInterpreter::Ascii;

        match sauce_type {
            super::SauceFileType::Ascii => {
                interpreter = CharInterpreter::Ansi; /* There are files that are marked as Ascii but contain ansi codes. */
            }
            super::SauceFileType::Ansi => {
                interpreter = CharInterpreter::Ansi;
                check_extension = true;
            }
            super::SauceFileType::ANSiMation => {
                interpreter = CharInterpreter::Ansi;
                eprintln!("WARNING: ANSiMations are not fully supported.");
            }
            super::SauceFileType::PCBoard => {
                interpreter = CharInterpreter::Pcb;
            }
            super::SauceFileType::Avatar => {
                interpreter = CharInterpreter::Avatar;
            }
            super::SauceFileType::TundraDraw => {
                if result.get_buffer_width() == 0 {
                    result.set_buffer_width(80);
                }
                super::read_tnd(&mut result, bytes, file_size)?;
                return Ok(result);
            }
            super::SauceFileType::Bin => {
                if result.get_buffer_width() == 0 {
                    result.set_buffer_width(160);
                }
                read_binary(&mut result, bytes, file_size)?;
                return Ok(result);
            }
            super::SauceFileType::XBin => {
                read_xb(&mut result, bytes, file_size)?;
                return Ok(result);
            }
            super::SauceFileType::Undefined => {
                check_extension = true;
            }
        }

        if check_extension {
            if let Some(ext) = ext {
                let ext = OsStr::to_str(ext).unwrap().to_lowercase();
                match ext.as_str() {
                    "bin" => {
                        if result.get_buffer_width() == 0 {
                            result.set_buffer_width(160);
                        }
                        read_binary(&mut result, bytes, file_size)?;
                        return Ok(result);
                    }
                    "xb" => {
                        read_xb(&mut result, bytes, file_size)?;
                        return Ok(result);
                    }
                    "adf" => {
                        if result.get_buffer_width() == 0 {
                            result.set_buffer_width(80);
                        }
                        super::read_adf(&mut result, bytes, file_size)?;
                        return Ok(result);
                    }
                    "idf" => {
                        super::read_idf(&mut result, bytes, file_size)?;
                        return Ok(result);
                    }
                    "tnd" => {
                        if result.get_buffer_width() == 0 {
                            result.set_buffer_width(80);
                        }
                        super::read_tnd(&mut result, bytes, file_size)?;
                        return Ok(result);
                    }
                    "ans" => {
                        interpreter = CharInterpreter::Ansi;
                    }
                    "ice" => {
                        interpreter = CharInterpreter::Ansi;
                        result.buffer_type = BufferType::LegacyIce;
                    }
                    "avt" => {
                        interpreter = CharInterpreter::Avatar;
                    }
                    "pcb" => {
                        interpreter = CharInterpreter::Pcb;
                    }
                    "seq" => {
                        interpreter = CharInterpreter::Petscii;
                    }
                    _ => {}
                }
            }
        }

        if result.get_buffer_width() == 0 {
            result.set_buffer_width(80);
        }
        result.set_buffer_height(25);

        let mut interpreter: Box<dyn BufferParser> = match interpreter {
            CharInterpreter::Ascii => Box::<parsers::ascii::Parser>::default(),
            CharInterpreter::Ansi => Box::<parsers::ansi::Parser>::default(),
            CharInterpreter::Avatar => Box::<parsers::avatar::Parser>::default(),
            CharInterpreter::Pcb => Box::<parsers::pcboard::Parser>::default(),
            CharInterpreter::Petscii => Box::<parsers::petscii::Parser>::default(),
        };

        let mut caret = Caret::default();
        for b in bytes.iter().take(file_size) {
            let res = interpreter.as_mut().print_char(
                &mut result,
                &mut caret,
                char::from_u32(*b as u32).unwrap(),
            );
            if !skip_errors && res.is_err() {
                res?;
            }
        }
        Ok(result)
    }

    pub fn to_screenx(&self, x: i32) -> f64 {
        let font_dimensions = self.get_font_dimensions();
        x as f64 * font_dimensions.width as f64
    }

    pub fn to_screeny(&self, y: i32) -> f64 {
        let font_dimensions = self.get_font_dimensions();
        y as f64 * font_dimensions.height as f64
    }

    pub fn get_line_length(&self, line: i32) -> i32 {
        let mut length = 0;
        let mut pos = Position::new(0, line);
        for x in 0..self.get_buffer_width() {
            pos.x = x;
            if let Some(ch) = self.get_char(pos) {
                /*if x > 0 && ch.is_transparent() {
                    if let Some(prev) = self.get_char(pos  + Position::from(-1, 0)) {
                        if prev.attribute.get_background() > 0 {
                            length = x + 1;
                        }

                    }
                } else */
                if !ch.is_transparent() {
                    length = x + 1;
                }
            }
        }
        length
    }

    pub fn set_height_for_pos(&mut self, pos: Position) {
        self.set_buffer_height(if pos.x == 0 { pos.y } else { pos.y + 1 });
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Buffer::new()
    }
}
