use std::collections::{HashMap, HashSet, VecDeque};
use std::{
    cmp::max,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use i18n_embed_fl::fl;

use crate::{
    parsers, BufferParser, EngineResult, Glyph, Layer, LoadingError, OutputFormat, Position, Rectangle, SauceData, Sixel, TerminalState, TextPane, FORMATS,
};

use super::{AttributedChar, BitFont, Palette, SaveOptions, Size};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BufferType {
    Unicode,
    CP437,
    Petscii,
    Atascii,
    Viewdata,
}

impl BufferType {
    pub fn from_byte(b: u8) -> Self {
        match b {
            // 0 => BufferType::Unicode,
            1 => BufferType::CP437,
            2 => BufferType::Petscii,
            3 => BufferType::Atascii,
            4 => BufferType::Viewdata,
            _ => BufferType::Unicode,
        }
    }

    pub fn to_byte(self) -> u8 {
        match self {
            BufferType::Unicode => 0,
            BufferType::CP437 => 1,
            BufferType::Petscii => 2,
            BufferType::Atascii => 3,
            BufferType::Viewdata => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IceMode {
    Unlimited,
    Blink,
    Ice,
}

impl IceMode {
    pub fn from_byte(b: u8) -> Self {
        match b {
            0 => IceMode::Unlimited,
            1 => IceMode::Blink,
            _ => IceMode::Ice,
        }
    }

    pub fn to_byte(self) -> u8 {
        match self {
            IceMode::Unlimited => 0,
            IceMode::Blink => 1,
            IceMode::Ice => 2,
        }
    }
    pub fn has_blink(self) -> bool {
        !matches!(self, IceMode::Ice)
    }

    pub fn has_high_bg_colors(self) -> bool {
        !matches!(self, IceMode::Blink)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PaletteMode {
    RGB,
    Fixed16,
    /// Extended font mode in XB + Blink limits to 8 colors
    Free8,
    Free16,
}

impl PaletteMode {
    pub fn from_byte(b: u8) -> Self {
        match b {
            // 0 => PaletteMode::RGB,
            1 => PaletteMode::Fixed16,
            2 => PaletteMode::Free8,
            3 => PaletteMode::Free16,
            _ => PaletteMode::RGB,
        }
    }

    pub fn to_byte(self) -> u8 {
        match self {
            PaletteMode::RGB => 0,
            PaletteMode::Fixed16 => 1,
            PaletteMode::Free8 => 2,
            PaletteMode::Free16 => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FontMode {
    /// Multiple fonts in the same document are possible without limit.
    Unlimited,
    /// Single font only sauce fonts apply
    Sauce,
    /// Single font all fonts are possible
    Single,
    /// Used to limit the the font pages
    /// For example 2 fonts for XB enhanced font mode.
    FixedSize,
}

impl FontMode {
    pub fn from_byte(b: u8) -> Self {
        match b {
            //  0 => FontMode::Unlimited,
            1 => FontMode::Sauce,
            2 => FontMode::Single,
            3 => FontMode::FixedSize,
            _ => FontMode::Unlimited,
        }
    }

    pub fn to_byte(self) -> u8 {
        match self {
            FontMode::Unlimited => 0,
            FontMode::Sauce => 1,
            FontMode::Single => 2,
            FontMode::FixedSize => 3,
        }
    }

    pub fn has_high_fg_colors(self) -> bool {
        !matches!(self, FontMode::FixedSize)
    }
}

pub struct Buffer {
    size: Size,
    pub file_name: Option<PathBuf>,

    pub terminal_state: TerminalState,
    pub buffer_type: BufferType,
    pub ice_mode: IceMode,
    pub palette_mode: PaletteMode,
    pub font_mode: FontMode,

    pub is_terminal_buffer: bool,

    sauce_data: Option<SauceData>,

    pub palette: Palette,
    pub overlay_layer: Option<Layer>,

    font_table: HashMap<usize, BitFont>,
    is_font_table_dirty: bool,

    pub layers: Vec<Layer>,

    pub sixel_threads: VecDeque<std::thread::JoinHandle<EngineResult<Sixel>>>, // pub undo_stack: Vec<Box<dyn UndoOperation>>,
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

        for y in 0..self.get_height() {
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
    pub fn scan_buffer_features(&self) -> BufferFeatures {
        let mut result = BufferFeatures::default();
        for layer in &self.layers {
            if !layer.sixels.is_empty() {
                result.use_sixels = true;
            }
            if !layer.hyperlinks.is_empty() {
                result.has_links = true;
            }
            for y in 0..layer.get_height() {
                for x in 0..layer.get_width() {
                    let ch = layer.get_char((x, y));

                    if ch.attribute.get_foreground() != 7 || ch.attribute.get_background() != 0 {
                        result.use_colors = true;
                    }

                    result.use_blink |= ch.attribute.is_blinking();
                    result.use_extended_attributes |= ch.attribute.is_crossed_out()
                        || ch.attribute.is_underlined()
                        || ch.attribute.is_concealed()
                        || ch.attribute.is_crossed_out()
                        || ch.attribute.is_double_height()
                        || ch.attribute.is_double_underlined()
                        || ch.attribute.is_overlined();
                }
            }
        }
        result.font_count = analyze_font_usage(self).len();
        result.use_extended_colors = self.palette.len() > 16;

        result
    }

    pub fn get_sauce(&self) -> &Option<SauceData> {
        &self.sauce_data
    }

    pub fn set_sauce(&mut self, sauce_opt: Option<SauceData>, resize_to_sauce: bool) -> Option<SauceData> {
        if resize_to_sauce {
            if let Some(sauce) = &sauce_opt {
                let mut size = sauce.buffer_size;
                // check limits, some files have wrong sauce data, even if 0 is specified
                // some files specify the pixel size there and don't have line breaks in the file
                if size.width == 0 || size.width > 1000 {
                    size.width = 80;
                }
                self.set_size(size);
                self.terminal_state.set_size(size);

                if !self.layers.is_empty() {
                    self.layers[0].set_size(size);
                }

                if let Some(font) = &sauce.font_opt {
                    if let Ok(font) = BitFont::from_sauce_name(font) {
                        self.set_font(0, font);
                    }
                }
                if sauce.use_ice {
                    self.ice_mode = IceMode::Ice;
                }
            }
        }
        self.is_font_table_dirty = true;
        let result = self.sauce_data.take();
        self.sauce_data = sauce_opt;
        result
    }

    pub fn has_sauce(&self) -> bool {
        self.sauce_data.is_some()
    }

    /// Clones the buffer (without sixel threads)
    pub fn flat_clone(&self) -> Buffer {
        let mut frame = Buffer::new(self.get_size());
        frame.file_name = self.file_name.clone();
        frame.terminal_state = self.terminal_state.clone();
        frame.buffer_type = self.buffer_type;
        frame.ice_mode = self.ice_mode;
        frame.palette_mode = self.palette_mode;
        frame.font_mode = self.font_mode;
        frame.is_terminal_buffer = self.is_terminal_buffer;
        frame.layers = self.layers.clone();
        frame.terminal_state = self.terminal_state.clone();
        frame.palette = self.palette.clone();
        frame.layers = Vec::new();
        frame.sauce_data = self.sauce_data.clone();
        for l in &self.layers {
            frame.layers.push(l.clone());
        }
        frame.clear_font_table();
        for f in self.font_iter() {
            frame.set_font(*f.0, f.1.clone());
        }
        frame
    }
}

pub fn analyze_font_usage(buf: &Buffer) -> Vec<usize> {
    let mut hash_set = HashSet::new();
    for y in 0..buf.get_height() {
        for x in 0..buf.get_width() {
            let ch = buf.get_char((x, y));
            hash_set.insert(ch.get_font_page());
        }
    }
    let mut v: Vec<usize> = hash_set.into_iter().collect();
    v.sort_unstable();
    v
}

#[derive(Default)]
pub struct BufferFeatures {
    pub use_sixels: bool,
    pub has_links: bool,
    pub font_count: usize,
    pub use_extended_colors: bool,
    pub use_colors: bool,
    pub use_blink: bool,
    pub use_extended_attributes: bool,
}

fn merge(mut input_char: AttributedChar, ch_opt: Option<char>, attr_opt: Option<crate::TextAttribute>) -> AttributedChar {
    if !input_char.is_visible() {
        return input_char;
    }
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
            size,
            terminal_state: TerminalState::from(size),
            sauce_data: None,

            buffer_type: BufferType::CP437,
            ice_mode: IceMode::Unlimited,
            palette_mode: PaletteMode::Fixed16,
            font_mode: FontMode::Sauce,

            is_terminal_buffer: false,
            palette: Palette::dos_default(),

            font_table,
            is_font_table_dirty: false,
            overlay_layer: None,
            layers: vec![Layer::new(fl!(crate::LANGUAGE_LOADER, "layer-background-name"), size)],
            sixel_threads: VecDeque::new(), // file_name_changed: Box::new(|| {}),
        }
    }

    /// Returns the update sixel threads of this [`Buffer`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn update_sixel_threads(&mut self) -> EngineResult<bool> {
        let mut updated_sixel = false;
        while let Some(handle) = self.sixel_threads.front() {
            if !handle.is_finished() {
                return Ok(false);
            }
            let Some(handle) = self.sixel_threads.pop_front() else {
                continue;
            };
            let Ok(result) = handle.join() else {
                continue;
            };

            let sixel = result?;

            updated_sixel = true;

            let font_dims = self.get_font_dimensions();
            let screen_rect = sixel.get_screen_rect(font_dims);

            let vec = &mut self.layers[0].sixels;
            let mut sixel_count = vec.len();
            // remove old sixel that are shadowed by the new one
            let mut i = 0;
            while i < sixel_count {
                let old_rect = vec[i].get_screen_rect(font_dims);
                if screen_rect.contains_rect(&old_rect) {
                    vec.remove(i);
                    sixel_count -= 1;
                } else {
                    i += 1;
                }
            }
            vec.push(sixel);
        }
        Ok(updated_sixel)
    }

    pub fn clear_font_table(&mut self) {
        self.font_table.clear();
        self.is_font_table_dirty = true;
    }

    pub fn has_fonts(&self) -> bool {
        !self.font_table.is_empty()
    }

    pub fn has_font(&self, id: usize) -> bool {
        self.font_table.contains_key(&id)
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
            if font.name == name {
                return Some(*i);
            }
        }
        None
    }

    pub fn font_iter(&self) -> impl Iterator<Item = (&usize, &BitFont)> {
        self.font_table.iter()
    }

    pub fn font_iter_mut(&mut self) -> impl Iterator<Item = (&usize, &mut BitFont)> {
        self.font_table.iter_mut()
    }

    pub fn get_font(&self, font_number: usize) -> Option<&BitFont> {
        self.font_table.get(&font_number)
    }

    pub fn set_font(&mut self, font_number: usize, font: BitFont) {
        self.font_table.insert(font_number, font);
        self.is_font_table_dirty = true;
    }

    pub fn remove_font(&mut self, font_number: usize) -> Option<BitFont> {
        self.font_table.remove(&font_number)
    }

    pub fn font_count(&self) -> usize {
        self.font_table.len()
    }

    pub fn get_font_table(&self) -> HashMap<usize, BitFont> {
        self.font_table.clone()
    }

    pub fn set_font_table(&mut self, font_table: HashMap<usize, BitFont>) {
        self.font_table = font_table;
    }

    pub fn append_font(&mut self, font: BitFont) -> usize {
        let mut i = 0;
        while self.font_table.contains_key(&i) {
            i += 1;
        }
        self.font_table.insert(i, font);
        i
    }

    pub fn get_real_buffer_width(&self) -> i32 {
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
    pub fn set_size(&mut self, size: impl Into<Size>) {
        let size = size.into();
        self.size = size;
        if let Some(sauce) = &mut self.sauce_data {
            sauce.buffer_size = size;
        }
    }

    pub fn set_width(&mut self, width: i32) {
        self.size.width = width;
        if let Some(sauce) = &mut self.sauce_data {
            sauce.buffer_size.width = width;
        }
    }

    pub fn set_height(&mut self, height: i32) {
        self.size.height = height;
        if let Some(sauce) = &mut self.sauce_data {
            sauce.buffer_size.height = height;
        }
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
    pub fn get_first_visible_line(&self) -> i32 {
        if self.is_terminal_buffer {
            max(0, self.size.height.saturating_sub(self.terminal_state.get_height()))
        } else {
            0
        }
    }

    pub fn get_last_visible_line(&self) -> i32 {
        self.get_first_visible_line() + self.get_height()
    }

    pub fn get_first_editable_line(&self) -> i32 {
        if self.is_terminal_buffer {
            if let Some((start, _)) = self.terminal_state.get_margins_top_bottom() {
                return self.get_first_visible_line() + start;
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

    pub fn get_last_editable_column(&self) -> i32 {
        if self.is_terminal_buffer {
            if let Some((_, end)) = self.terminal_state.get_margins_left_right() {
                return end;
            }
        }
        self.get_width().saturating_sub(1)
    }

    #[must_use]
    pub fn needs_scrolling(&self) -> bool {
        self.is_terminal_buffer && self.terminal_state.get_margins_top_bottom().is_some()
    }

    #[must_use]
    pub fn get_last_editable_line(&self) -> i32 {
        if self.is_terminal_buffer {
            if let Some((_, end)) = self.terminal_state.get_margins_top_bottom() {
                self.get_first_visible_line() + end
            } else {
                (self.get_first_visible_line() + self.get_height()).saturating_sub(1)
            }
        } else {
            max(self.layers[0].lines.len() as i32, self.get_height().saturating_sub(1))
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
    pub fn create(size: impl Into<Size>) -> Self {
        let size = size.into();
        let mut res = Buffer::new(size);
        res.layers[0].lines.resize(size.height as usize, crate::Line::create(size.width));

        res
    }

    pub fn get_overlay_layer(&mut self) -> &mut Option<Layer> {
        if self.overlay_layer.is_none() {
            let mut l = Layer::new("Overlay", self.get_size());
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
    /// # Panics
    ///
    /// Panics if .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn load_buffer(file_name: &Path, skip_errors: bool) -> EngineResult<Buffer> {
        let mut f = match File::open(file_name) {
            Ok(f) => f,
            Err(err) => {
                return Err(LoadingError::OpenFileError(format!("{err}")).into());
            }
        };
        let mut bytes = Vec::new();
        if let Err(err) = f.read_to_end(&mut bytes) {
            return Err(LoadingError::ReadFileError(format!("{err}")).into());
        }

        Buffer::from_bytes(file_name, skip_errors, &bytes)
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn to_bytes(&self, extension: &str, options: &SaveOptions) -> EngineResult<Vec<u8>> {
        let extension = extension.to_ascii_lowercase();
        for fmt in &*crate::FORMATS {
            if fmt.get_file_extension() == extension || fmt.get_alt_extensions().contains(&extension) {
                if options.lossles_output {
                    return fmt.to_bytes(self, options);
                }
                let optimizer = crate::ColorOptimizer::new(self, options);
                return fmt.to_bytes(&optimizer.optimize(self), options);
            }
        }
        Err(anyhow::anyhow!("Unknown format"))
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
    pub fn from_bytes(file_name: &Path, _skip_errors: bool, bytes: &[u8]) -> EngineResult<Buffer> {
        let ext = file_name.extension().unwrap().to_str().unwrap();
        let mut len = bytes.len();
        let sauce_data = match SauceData::extract(bytes) {
            Ok(Some(sauce)) => {
                len -= sauce.sauce_header_len;
                Some(sauce)
            }
            Ok(None) => None,
            Err(err) => {
                log::error!("Error reading sauce data: {}", err);
                None
            }
        };

        let ext = ext.to_ascii_lowercase();
        for format in &*FORMATS {
            if format.get_file_extension() == ext {
                return format.load_buffer(file_name, &bytes[..len], sauce_data);
            }
        }

        crate::Ansi::default().load_buffer(file_name, &bytes[..len], sauce_data)
    }

    pub fn to_screenx(&self, x: i32) -> f64 {
        let font_dimensions = self.get_font_dimensions();
        x as f64 * font_dimensions.width as f64
    }

    pub fn to_screeny(&self, y: i32) -> f64 {
        let font_dimensions = self.get_font_dimensions();
        y as f64 * font_dimensions.height as f64
    }

    /// .
    ///
    /// # Panics
    ///
    /// Panics if .
    pub fn render_to_rgba(&self, rect: Rectangle) -> (Size, Vec<u8>) {
        let font_size = self.get_font(0).unwrap().size;

        let px_width = rect.get_width() * font_size.width;
        let px_height = rect.get_height() * font_size.height;
        let line_bytes = px_width * 4;
        let mut pixels = vec![0; (line_bytes * px_height) as usize];

        for y in 0..rect.get_height() {
            for x in 0..rect.get_width() {
                let ch = self.get_char((x + rect.start.x, y + rect.start.y));
                let font = self.get_font(ch.get_font_page()).unwrap();

                let fg = if ch.attribute.is_bold() && ch.attribute.get_foreground() < 8 {
                    ch.attribute.get_foreground() + 8
                } else {
                    ch.attribute.get_foreground()
                };

                let (f_r, f_g, f_b) = self.palette.get_rgb(fg as usize);
                let (b_r, b_g, b_b) = self.palette.get_rgb(ch.attribute.get_background() as usize);

                if let Some(glyph) = font.get_glyph(ch.ch) {
                    let cur_font_size = font.size;
                    for cy in 0..cur_font_size.height.min(font_size.height) {
                        for cx in 0..cur_font_size.width.min(font_size.width) {
                            let offset = ((x * font_size.width + cx) * 4 + (y * font_size.height + cy) * line_bytes) as usize;
                            if glyph.data[cy as usize] & (128 >> cx) == 0 {
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

        for layer in &self.layers {
            for sixel in &layer.sixels {
                let sx = layer.get_offset().x + sixel.position.x - rect.start.x;
                let sx_px = sx * font_size.width;
                let sy = layer.get_offset().y + sixel.position.y - rect.start.y;
                let sy_pix = sy * font_size.height;
                let sixel_line_bytes = (sixel.get_width() * 4) as usize;

                let mut sixel_line = 0;
                for y in sy_pix..(sy_pix + sixel.get_height()) {
                    if y < 0 {
                        continue;
                    }
                    let y = y as usize;
                    let offset = y * line_bytes as usize + sx_px as usize * 4;
                    let o = sixel_line * sixel_line_bytes;
                    if offset + sixel_line_bytes > pixels.len() {
                        break;
                    }
                    pixels[offset..(offset + sixel_line_bytes)].copy_from_slice(&sixel.picture_data[o..(o + sixel_line_bytes)]);
                    sixel_line += 1;
                }
            }
        }
        (Size::new(px_width, px_height), pixels)
    }

    pub fn use_letter_spacing(&self) -> bool {
        if let Some(data) = &self.sauce_data {
            return data.use_letter_spacing;
        }
        false
    }

    pub fn use_aspect_ratio(&self) -> bool {
        if let Some(data) = &self.sauce_data {
            return data.use_aspect_ratio;
        }
        false
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Buffer::new((80, 25))
    }
}

impl TextPane for Buffer {
    fn get_width(&self) -> i32 {
        self.size.width
    }

    fn get_height(&self) -> i32 {
        self.size.height
    }

    fn get_line_count(&self) -> i32 {
        if let Some(len) = self.layers.iter().map(|l| l.lines.len()).max() {
            len as i32
        } else {
            self.size.height
        }
    }

    fn get_char(&self, pos: impl Into<Position>) -> AttributedChar {
        let pos = pos.into();
        if let Some(overlay) = &self.overlay_layer {
            let pos = pos - overlay.get_offset();
            let ch = overlay.get_char(pos);
            if ch.is_visible() {
                return ch;
            }
        }

        let mut ch_opt = None;
        let mut attr_opt = None;
        let mut default_font_page = 0;
        for i in (0..self.layers.len()).rev() {
            let cur_layer = &self.layers[i];
            if !cur_layer.is_visible {
                continue;
            }
            let pos = pos - cur_layer.get_offset();
            if pos.x < 0 || pos.y < 0 || pos.x >= cur_layer.get_width() || pos.y >= cur_layer.get_height() {
                continue;
            }
            let ch = cur_layer.get_char(pos);
            default_font_page = cur_layer.default_font_page;
            match cur_layer.mode {
                crate::Mode::Normal => {
                    if ch.is_visible() {
                        return merge(ch, ch_opt, attr_opt);
                    }
                    if !cur_layer.has_alpha_channel {
                        return merge(AttributedChar::default().with_font_page(default_font_page), ch_opt, attr_opt);
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

        let mut ch = if self.is_terminal_buffer {
            AttributedChar::default()
        } else {
            AttributedChar::invisible()
        };
        ch.attribute.set_font_page(default_font_page);
        ch
    }

    fn get_line_length(&self, line: i32) -> i32 {
        let mut length = 0;
        let mut pos = Position::new(0, line);
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

    fn get_size(&self) -> Size {
        self.size
    }

    fn get_rectangle(&self) -> Rectangle {
        Rectangle::from_min_size((0, 0), (self.get_width(), self.get_height()))
    }
}

#[cfg(test)]
mod tests {
    use crate::{AttributedChar, Buffer, Layer, SaveOptions, Size, TextAttribute, TextPane};

    #[test]
    fn test_respect_sauce_width() {
        let mut buf = Buffer::default();
        buf.set_width(10);
        for x in 0..buf.get_width() {
            buf.layers[0].set_char((x, 0), AttributedChar::new('1', TextAttribute::default()));
            buf.layers[0].set_char((x, 1), AttributedChar::new('2', TextAttribute::default()));
            buf.layers[0].set_char((x, 2), AttributedChar::new('3', TextAttribute::default()));
        }

        let mut opt = SaveOptions::new();
        opt.save_sauce = true;
        let ansi_bytes = buf.to_bytes("ans", &opt).unwrap();

        let loaded_buf = Buffer::from_bytes(&std::path::PathBuf::from("test.ans"), false, &ansi_bytes).unwrap();
        assert_eq!(10, loaded_buf.get_width());
        assert_eq!(10, loaded_buf.layers[0].get_width());
    }

    #[test]
    fn test_layer_offset() {
        let mut buf: Buffer = Buffer::default();

        let mut new_layer = Layer::new("1", Size::new(10, 10));
        new_layer.has_alpha_channel = true;
        new_layer.set_offset((2, 2));
        new_layer.set_char((5, 5), AttributedChar::new('a', TextAttribute::default()));
        buf.layers.push(new_layer);

        assert_eq!('a', buf.get_char((7, 7)).ch);
    }

    #[test]
    fn test_layer_negative_offset() {
        let mut buf: Buffer = Buffer::default();

        let mut new_layer = Layer::new("1", Size::new(10, 10));
        new_layer.has_alpha_channel = true;
        new_layer.set_offset((-2, -2));
        new_layer.set_char((5, 5), AttributedChar::new('a', TextAttribute::default()));
        buf.layers.push(new_layer);

        let mut new_layer = Layer::new("2", Size::new(10, 10));
        new_layer.has_alpha_channel = true;
        new_layer.set_offset((2, 2));
        new_layer.set_char((5, 5), AttributedChar::new('b', TextAttribute::default()));
        buf.layers.push(new_layer);

        assert_eq!('a', buf.get_char((3, 3)).ch);
        assert_eq!('b', buf.get_char((7, 7)).ch);
    }
}
