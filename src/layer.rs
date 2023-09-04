use crate::{Buffer, BufferParser, Color, Line, Position, Sixel, Size};

use super::AttributedChar;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Normal,
    Chars,
    Attributes,
}

#[derive(Debug, Default, Clone)]
pub struct Layer {
    pub title: String,
    pub is_visible: bool,
    pub is_locked: bool,
    pub is_position_locked: bool,
    pub is_alpha_channel_locked: bool,
    pub has_alpha_channel: bool,

    pub color: Option<Color>,

    pub mode: Mode,

    pub offset: Position,
    pub size: Size,
    pub lines: Vec<Line>,

    pub sixels: Vec<Sixel>,
    pub(crate) hyperlinks: Vec<HyperLink>,
}

impl std::fmt::Display for Layer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str = String::new();
        let p = crate::parsers::ansi::Parser::default();

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

#[derive(Debug, Default, Clone)]
pub struct HyperLink {
    pub url: Option<String>,
    pub position: Position,
    pub length: i32,
}

impl HyperLink {
    pub fn get_url(&self, buf: &Buffer) -> String {
        if let Some(ref url) = self.url {
            url.clone()
        } else {
            buf.get_string(self.position, self.length as usize)
        }
    }
}

impl Layer {
    pub fn new(title: impl Into<String>, size: impl Into<Size>) -> Self {
        let size = size.into();
        let mut lines = Vec::new();
        lines.resize(size.height as usize, Line::create(size.width));

        Layer {
            title: title.into(),
            is_visible: true,
            size,
            lines,
            ..Default::default()
        }
    }

    pub fn get_offset(&self) -> Position {
        self.offset
    }

    pub fn set_offset(&mut self, pos: Position) {
        if self.is_position_locked {
            return;
        }
        self.offset = pos;
    }

    pub fn join(&mut self, layer: &Layer) {
        for y in 0..layer.lines.len() {
            let line = &layer.lines[y];
            for x in 0..line.chars.len() {
                let ch = line.chars[x];
                if ch.is_visible() {
                    self.set_char((x as i32, y as i32), ch);
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.hyperlinks.clear();
        self.sixels.clear();
    }

    pub fn set_char(&mut self, pos: impl Into<Position>, attributed_char: AttributedChar) {
        let pos = pos.into();
        if pos.x < 0 || pos.y < 0 || pos.x >= self.get_width() || pos.y >= self.get_height() {
            return;
        }
        if self.is_locked || !self.is_visible {
            return;
        }
        if pos.y >= self.lines.len() as i32 {
            self.lines
                .resize(pos.y as usize + 1, Line::create(self.size.width));
        }

        if self.has_alpha_channel && self.is_alpha_channel_locked {
            let old_char = self.get_char(pos);
            if !old_char.is_visible() {
                return;
            }
        }

        let cur_line = &mut self.lines[pos.y as usize];
        cur_line.set_char(pos.x, attributed_char);
    }

    pub(crate) fn set_char_xy(&mut self, x: i32, y: i32, attributed_char: AttributedChar) {
        self.set_char((x, y), attributed_char);
    }

    pub fn get_char(&self, pos: impl Into<Position>) -> AttributedChar {
        let pos = pos.into();
        if pos.x < 0
            || pos.y < 0
            || pos.x >= self.get_width()
            || pos.y >= self.get_height() && pos.y >= self.lines.len() as i32
        {
            return if self.has_alpha_channel {
                AttributedChar::invisible()
            } else {
                AttributedChar::default()
            };
        }

        let y = pos.y;
        if y < self.lines.len() as i32 {
            let cur_line = &self.lines[y as usize];

            if pos.x < cur_line.chars.len() as i32 {
                let ch = cur_line.chars[pos.x as usize];
                if !self.has_alpha_channel && !ch.is_visible() {
                    return AttributedChar::default();
                }
                return ch;
            }
        }

        if self.has_alpha_channel {
            AttributedChar::invisible()
        } else {
            AttributedChar::default()
        }
    }

    /// .
    ///
    /// # Panics
    ///
    /// Panics if .
    pub fn remove_line(&mut self, index: i32) {
        if self.is_locked || !self.is_visible {
            return;
        }
        assert!(
            !(index < 0 || index >= self.lines.len() as i32),
            "line out of range"
        );
        self.lines.remove(index as usize);
    }

    /// .
    ///
    /// # Panics
    ///
    /// Panics if .
    pub fn insert_line(&mut self, index: i32, line: Line) {
        if self.is_locked || !self.is_visible {
            return;
        }
        assert!(index >= 0, "line out of range");
        if index > self.lines.len() as i32 {
            self.lines
                .resize(index as usize, Line::create(self.size.width));
        }

        self.lines.insert(index as usize, line);
    }

    pub fn swap_char(&mut self, pos1: impl Into<Position>, pos2: impl Into<Position>) {
        let pos1 = pos1.into();
        let pos2 = pos2.into();
        let tmp = self.get_char(pos1);
        self.set_char(pos1, self.get_char(pos2));
        self.set_char(pos2, tmp);
    }

    pub fn add_hyperlink(&mut self, p: HyperLink) {
        self.hyperlinks.push(p);
    }

    pub fn hyperlinks(&self) -> &Vec<HyperLink> {
        &self.hyperlinks
    }

    pub fn set_width(&mut self, width: i32) {
        self.size.width = width;
    }

    pub fn get_width(&self) -> i32 {
        self.size.width
    }

    pub fn set_height(&mut self, height: i32) {
        self.size.height = height;
    }

    pub fn get_height(&self) -> i32 {
        self.size.height
    }

    pub fn get_line_count(&self) -> i32 {
        self.lines.len() as i32
    }

    pub fn get_line_length(&self, line: i32) -> i32 {
        self.lines[line as usize].get_line_length()
    }
}

#[cfg(test)]
mod tests {
    use i18n_embed_fl::fl;

    use crate::{AttributedChar, Layer, Line, TextAttribute};

    #[test]
    fn test_get_char() {
        let mut layer = Layer::new(
            fl!(crate::LANGUAGE_LOADER, "layer-background-name"),
            (20, 20),
        );
        layer.has_alpha_channel = false;
        let mut line = Line::new();
        line.set_char(10, AttributedChar::new('a', TextAttribute::default()));

        layer.insert_line(0, line);

        assert_eq!(AttributedChar::default(), layer.get_char((-1, -1)));
        assert_eq!(AttributedChar::default(), layer.get_char((1000, 1000)));
        assert_eq!('a', layer.get_char((10, 0)).ch);
        assert_eq!(AttributedChar::default(), layer.get_char((9, 0)));
        assert_eq!(AttributedChar::default(), layer.get_char((11, 0)));
    }

    #[test]
    fn test_get_char_intransparent() {
        let mut layer = Layer::new(
            fl!(crate::LANGUAGE_LOADER, "layer-background-name"),
            (20, 20),
        );
        layer.has_alpha_channel = true;

        let mut line = Line::new();
        line.set_char(10, AttributedChar::new('a', TextAttribute::default()));

        layer.insert_line(0, line);

        assert_eq!(AttributedChar::invisible(), layer.get_char((-1, -1)));
        assert_eq!(AttributedChar::invisible(), layer.get_char((1000, 1000)));
        assert_eq!('a', layer.get_char((10, 0)).ch);
        assert_eq!(AttributedChar::invisible(), layer.get_char((9, 0)));
        assert_eq!(AttributedChar::invisible(), layer.get_char((11, 0)));
    }

    #[test]
    fn test_insert_line() {
        let mut layer = Layer::new(
            fl!(crate::LANGUAGE_LOADER, "layer-background-name"),
            (80, 0),
        );
        let mut line = Line::new();
        line.chars
            .push(AttributedChar::new('a', TextAttribute::default()));
        layer.insert_line(10, line);

        assert_eq!('a', layer.lines[10].chars[0].ch);
        assert_eq!(11, layer.lines.len());

        layer.insert_line(11, Line::new());
        assert_eq!(12, layer.lines.len());
    }
}
