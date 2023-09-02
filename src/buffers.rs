use std::collections::{HashMap, VecDeque};
use std::ffi::OsStr;
use std::{
    cmp::max,
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

use crate::{
    parsers, BufferParser, Caret, EngineResult, Glyph, Layer, Rectangle, SauceData, Sixel,
    TerminalState, UPosition,
};

use super::{
    read_binary, read_xb, AttributedChar, BitFont, Palette, Position, SauceString, SaveOptions,
    Size,
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

    /// Letter-spacing (a.k.a. 8/9 pixel font selection)
    pub use_letter_spacing: bool,

    /// Define if the image should be stretched to emulate legacy aspects
    pub use_aspect_ratio: bool,

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
            .field("width", &self.get_width())
            .field("height", &self.get_height())
            .field("custom_palette", &self.palette)
            .field("layers", &self.layers)
            .finish()
    }
}

impl std::fmt::Display for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str = String::new();
        let p = parsers::ansi::Parser::default();

        for y in 0..self.get_line_count() {
            str.extend(format!("{y:3}: ").chars());
            for x in 0..self.get_width() {
                let ch = self.get_char((x, y));
                str.push(p.convert_to_unicode(ch));
            }
            str.push('\n');
        }
        write!(f, "{str}")
    }
}

impl Buffer {
    pub fn get_width(&self) -> usize {
        self.terminal_state.get_width()
    }

    pub fn get_line_count(&self) -> usize {
        if let Some(len) = self.layers.iter().map(|l| l.lines.len()).max() {
            len
        } else {
            self.terminal_state.get_height()
        }
    }

    pub fn get_char(&self, pos: impl Into<UPosition>) -> AttributedChar {
        let pos = pos.into();
        if let Some(overlay) = &self.overlay_layer {
            let ch = overlay.get_char(pos);
            if ch.is_visible() {
                return ch;
            }
        }

        let mut ch_opt = None;
        let mut attr_opt = None;

        for i in 0..self.layers.len() {
            let cur_layer = &self.layers[i];
            if !cur_layer.is_visible {
                continue;
            }
            let ch = cur_layer.get_char(pos);
            match cur_layer.mode {
                crate::Mode::Normal => {
                    if ch.is_visible() {
                        return merge(ch, ch_opt, attr_opt);
                    }
                    if !cur_layer.has_alpha_channel {
                        return merge(AttributedChar::default(), ch_opt, attr_opt);
                    }
                }
                crate::Mode::Chars => {
                    if !ch.is_transparent() {
                        ch_opt = Some(ch.ch);
                    }
                }
                crate::Mode::Attributes => {
                    if ch.is_visible() {
                        attr_opt = Some(ch.attribute);
                    }
                }
            }
        }

        if self.is_terminal_buffer {
            AttributedChar::default()
        } else {
            AttributedChar::invisible()
        }
    }

    pub fn get_line_length(&self, line: usize) -> usize {
        let mut length = 0;
        let mut pos = UPosition::new(0, line);
        for x in 0..self.get_width() {
            pos.x = x;
            let ch = self.get_char(pos);
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
        length
    }
}

fn merge(
    mut input_char: AttributedChar,
    ch_opt: Option<char>,
    attr_opt: Option<crate::TextAttribute>,
) -> AttributedChar {
    if let Some(ch) = ch_opt {
        input_char.ch = ch;
    }
    if let Some(attr) = attr_opt {
        input_char.attribute = attr;
    }
    input_char
}

impl Buffer {
    pub fn new(size: impl Into<Size>) -> Self {
        let mut font_table = HashMap::new();
        font_table.insert(0, BitFont::default());
        let size = size.into();
        Buffer {
            file_name: None,
            terminal_state: TerminalState::from(size),

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
            layers: vec![Layer::new("Background", size)],
            sixel_threads: VecDeque::new(), // file_name_changed: Box::new(|| {}),
            // undo_stack: Vec::new(),
            // redo_stack: Vec::new()
            use_letter_spacing: false,
            use_aspect_ratio: false,
        }
    }

    /// Returns the update sixel threads of this [`Buffer`].
    ///
    /// # Panics
    ///
    /// Panics if .
    pub fn update_sixel_threads(&mut self) -> bool {
        let mut updated_sixel = false;
        while let Some(handle) = self.sixel_threads.front() {
            if !handle.is_finished() {
                return false;
            }
            let handle = self.sixel_threads.pop_front().unwrap();
            let sixel = handle.join().unwrap();
            {
                updated_sixel = true;

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
        }
        updated_sixel
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

    pub fn get_height(&self) -> usize {
        self.terminal_state.get_height()
    }

    pub fn get_real_buffer_width(&self) -> usize {
        let mut w = 0;
        for layer in &self.layers {
            for line in &layer.lines {
                w = max(w, line.get_line_length());
            }
        }
        w
    }

    pub fn reset_terminal(&mut self) {
        self.terminal_state = TerminalState::from(self.terminal_state.get_size());
    }

    /// Sets the buffer size of this [`Buffer`].
    ///
    /// # Panics
    ///
    /// Panics if .
    pub fn set_buffer_size(&mut self, size: impl Into<Size>) {
        self.terminal_state.set_size(size);
    }

    /// Sets the buffer width of this [`Buffer`].
    ///
    /// # Panics
    ///
    /// Panics if .
    pub fn set_buffer_width(&mut self, width: usize) {
        self.terminal_state.set_width(width);
    }
    pub fn get_buffer_size(&self) -> Size {
        self.terminal_state.get_size()
    }
    /// Sets the buffer height of this [`Buffer`].
    ///
    /// # Panics
    ///
    /// Panics if .
    pub fn set_buffer_height(&mut self, height: usize) {
        self.terminal_state.set_height(height);
    }

    /// Returns the clear of this [`Buffer`].
    ///
    /// # Panics
    ///
    /// Panics if .
    pub fn stop_sixel_threads(&mut self) {
        self.sixel_threads.clear();
    }

    /// terminal buffers have a viewport on the bottom of the buffer
    /// this function gives back the first visible line.
    #[must_use]
    pub fn get_first_visible_line(&self) -> usize {
        if self.is_terminal_buffer {
            max(
                0,
                self.layers[0].lines.len().saturating_sub(self.get_height()),
            )
        } else {
            0
        }
    }

    pub fn get_last_visible_line(&self) -> usize {
        self.get_first_visible_line() + self.get_height()
    }

    pub fn get_first_editable_line(&self) -> usize {
        if self.is_terminal_buffer {
            if let Some((start, _)) = self.terminal_state.get_margins_top_bottom() {
                return self.get_first_visible_line() + start as usize;
            }
        }
        self.get_first_visible_line()
    }

    pub fn get_first_editable_column(&self) -> i32 {
        if self.is_terminal_buffer {
            if let Some((start, _)) = self.terminal_state.get_margins_left_right() {
                return start;
            }
        }
        0
    }

    pub fn get_last_editable_column(&self) -> usize {
        if self.is_terminal_buffer {
            if let Some((_, end)) = self.terminal_state.get_margins_left_right() {
                return end as usize;
            }
        }
        self.get_width().saturating_sub(1)
    }

    #[must_use]
    pub fn needs_scrolling(&self) -> bool {
        self.is_terminal_buffer && self.terminal_state.get_margins_top_bottom().is_some()
    }

    #[must_use]
    pub fn get_last_editable_line(&self) -> usize {
        if self.is_terminal_buffer {
            if let Some((_, end)) = self.terminal_state.get_margins_top_bottom() {
                self.get_first_visible_line() + end as usize
            } else {
                (self.get_first_visible_line() + self.get_height()).saturating_sub(1)
            }
        } else {
            max(
                self.layers[0].lines.len(),
                self.get_height().saturating_sub(1),
            )
        }
    }

    #[must_use]
    pub fn upper_left_position(&self) -> Position {
        match self.terminal_state.origin_mode {
            crate::OriginMode::UpperLeftCorner => Position {
                x: 0,
                y: self.get_first_visible_line() as i32,
            },
            crate::OriginMode::WithinMargins => Position {
                x: 0,
                y: self.get_first_editable_line() as i32,
            },
        }
    }

    #[must_use]
    pub fn create(size: impl Into<Size>) -> Self {
        let size = size.into();
        let mut res = Buffer::new(size);
        res.layers[0]
            .lines
            .resize(size.height, crate::Line::create(size.width));

        res
    }

    pub fn get_overlay_layer(&mut self) -> &mut Option<Layer> {
        if self.overlay_layer.is_none() {
            let mut l = Layer::new("Overlay", self.get_buffer_size());
            l.has_alpha_channel = true;
            self.overlay_layer = Some(l);
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
    pub fn get_font_dimensions(&self) -> Size {
        self.font_table[&0].size
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
            "icd" => super::convert_to_icd(self),
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
            || self.get_width() != 80
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
        let mut result = Buffer {
            is_terminal_buffer: true,
            file_name: Some(file_name.to_path_buf()),
            ..Default::default()
        };

        let ext = file_name.extension();
        if let Some(ext) = ext {
            // mdf doesn't need sauce info.
            let ext = OsStr::to_str(ext).unwrap().to_lowercase();
            if ext.as_str() == "icd" {
                super::read_icd(&mut result, bytes)?;
                return Ok(result);
            }
        }
        let sauce_data = match SauceData::extract(bytes) {
            Ok(sauce) => Some(sauce),
            Err(err) => {
                log::error!("Error reading sauce data: {}", err);
                None
            }
        };
        let mut sauce_type = super::SauceFileType::Undefined;
        let mut file_size = bytes.len();
        let mut use_ice = false;
        if let Some(sauce) = &sauce_data {
            result.title = sauce.title.clone();
            result.author = sauce.author.clone();
            result.group = sauce.group.clone();
            result.comments = sauce.comments.clone();
            result.set_buffer_size(sauce.buffer_size);
            result.layers[0].size = sauce.buffer_size;
            result.use_aspect_ratio = sauce.use_letter_spacing;
            result.use_letter_spacing = sauce.use_letter_spacing;
            sauce_type = sauce.sauce_file_type;
            use_ice = sauce.use_ice;
            file_size -= sauce.sauce_header_len;
        }

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
                log::warn!("WARNING: ANSiMations are not fully supported.");
            }
            super::SauceFileType::PCBoard => {
                interpreter = CharInterpreter::Pcb;
            }
            super::SauceFileType::Avatar => {
                interpreter = CharInterpreter::Avatar;
            }
            super::SauceFileType::TundraDraw => {
                if result.get_width() == 0 {
                    result.set_buffer_width(80);
                }
                super::read_tnd(&mut result, bytes, file_size)?;
                return Ok(result);
            }
            super::SauceFileType::Bin => {
                if result.get_width() == 0 {
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
                        if result.get_width() == 0 {
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
                        if result.get_width() == 0 {
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
                        if result.get_width() == 0 {
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

        if result.get_width() == 0 {
            result.set_buffer_width(80);
        }
        result.set_buffer_height(25);
        result.layers[0].lines.clear();

        let mut interpreter: Box<dyn BufferParser> = match interpreter {
            CharInterpreter::Ascii => Box::<parsers::ascii::Parser>::default(),
            CharInterpreter::Ansi => {
                let mut parser = Box::<parsers::ansi::Parser>::default();
                parser.bs_is_ctrl_char = false;
                parser
            }
            CharInterpreter::Avatar => Box::<parsers::avatar::Parser>::default(),
            CharInterpreter::Pcb => Box::<parsers::pcboard::Parser>::default(),
            CharInterpreter::Petscii => Box::<parsers::petscii::Parser>::default(),
        };

        let mut caret = Caret::default();
        caret.set_ice_mode(use_ice);

        for b in bytes.iter().take(file_size) {
            let res = interpreter.as_mut().print_char(
                &mut result,
                0,
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

    pub fn set_height_for_pos(&mut self, pos: impl Into<UPosition>) {
        let pos = pos.into();
        self.set_buffer_height(if pos.x == 0 { pos.y } else { pos.y + 1 });
    }

    pub fn render_to_rgba(&self, rect: Rectangle) -> (Size, Vec<u8>) {
        let font_size = self.get_font(0).unwrap().size;

        let px_width = rect.get_width() * font_size.width;
        let px_height = rect.get_height() * font_size.height;
        let line_bytes = px_width * 4;
        let mut pixels = vec![0; line_bytes * px_height];

        for y in 0..rect.get_height() {
            for x in 0..rect.get_width() {
                let x = x + rect.start.x as usize;
                let y = y + rect.start.y as usize;

                let ch = self.get_char((x, y));
                let font = self.get_font(ch.get_font_page()).unwrap();

                let fg = if ch.attribute.is_bold() && ch.attribute.get_foreground() < 8 {
                    ch.attribute.get_foreground() + 8
                } else {
                    ch.attribute.get_foreground()
                };

                let (f_r, f_g, f_b) = self.palette.colors[fg as usize].get_rgb();
                let (b_r, b_g, b_b) =
                    self.palette.colors[ch.attribute.get_background() as usize].get_rgb();

                if let Some(glyph) = font.get_glyph(ch.ch) {
                    for cy in 0..font_size.height {
                        for cx in 0..font_size.width {
                            let offset = (x * font_size.width + cx) * 4
                                + (y * font_size.height + cy) * line_bytes;
                            if glyph.data[cy] & (128 >> cx) == 0 {
                                pixels[offset] = b_r;
                                pixels[offset + 1] = b_g;
                                pixels[offset + 2] = b_b;
                            } else {
                                pixels[offset] = f_r;
                                pixels[offset + 1] = f_g;
                                pixels[offset + 2] = f_b;
                            }
                            pixels[offset + 3] = 0xFF;
                        }
                    }
                }
            }
        }

        (Size::new(px_width, px_height), pixels)
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Buffer::new((80, 25))
    }
}

#[cfg(test)]
mod tests {
    use crate::{AttributedChar, Buffer, SaveOptions, TextAttribute};

    #[test]
    fn test_respect_sauce_width() {
        let mut buf = Buffer::default();
        buf.set_buffer_width(10);
        for x in 0..buf.get_width() {
            buf.layers[0].set_char_xy(
                x as i32,
                0,
                AttributedChar::new('1', TextAttribute::default()),
            );
            buf.layers[0].set_char_xy(
                x as i32,
                1,
                AttributedChar::new('2', TextAttribute::default()),
            );
            buf.layers[0].set_char_xy(
                x as i32,
                2,
                AttributedChar::new('3', TextAttribute::default()),
            );
        }

        let mut opt = SaveOptions::new();
        opt.save_sauce = true;
        let ansi_bytes = buf.to_bytes("ans", &opt).unwrap();

        let loaded_buf =
            Buffer::from_bytes(&std::path::PathBuf::from("test.ans"), false, &ansi_bytes).unwrap();
        assert_eq!(10, loaded_buf.get_width());
        assert_eq!(10, loaded_buf.layers[0].get_width());
    }
}
