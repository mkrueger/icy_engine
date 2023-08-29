use crate::{Buffer, Line, Sixel, Size};

use super::{AttributedChar, Position};

#[derive(Debug, Default, Clone)]
pub struct Layer {
    pub title: String,
    pub is_visible: bool,
    pub is_locked: bool,
    pub is_position_locked: bool,

    pub offset: Position,
    pub size: Size<i32>,
    pub lines: Vec<Line>,

    pub sixels: Vec<Sixel>,
    pub(crate) hyperlinks: Vec<HyperLink>,
}

#[derive(Debug, Default, Clone)]
pub struct HyperLink {
    pub url: Option<String>,
    pub position: Position,
    pub length: usize,
}

impl HyperLink {
    pub fn get_url(&self, buf: &Buffer) -> String {
        if let Some(ref url) = self.url {
            url.clone()
        } else {
            buf.get_string(self.position, self.length)
        }
    }
}

impl Layer {
    pub fn new(title: impl Into<String>, width: i32, height: i32) -> Self {
        Layer {
            title: title.into(),
            is_visible: true,
            is_locked: false,
            is_position_locked: false,
            lines: Vec::new(),
            sixels: Vec::new(),
            offset: Position::default(),
            hyperlinks: Vec::new(),
            size: Size::new(width, height),
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
                    self.set_char(Position::new(x as i32, y as i32), ch);
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.hyperlinks.clear();
        self.sixels.clear();
    }

    pub fn set_char(&mut self, pos: Position, attributed_char: AttributedChar) {
        let pos = pos - self.offset;
        if pos.x < 0 || pos.y < 0 || self.is_locked || !self.is_visible {
            return;
        }
        if pos.y >= self.lines.len() as i32 {
            self.lines.resize(pos.y as usize + 1, Line::new());
        }
        let cur_line = &mut self.lines[pos.y as usize];
        cur_line.set_char(pos.x, attributed_char);
    }

    pub(crate) fn set_char_xy(&mut self, x: i32, y: i32, attributed_char: AttributedChar) {
        self.set_char(Position::new(x, y), attributed_char);
    }
    pub fn get_char(&self, pos: Position) -> AttributedChar {
        let pos = pos - self.offset;
        let y = pos.y as usize;
        if y < self.lines.len() {
            let cur_line = &self.lines[y];
            if pos.x < cur_line.chars.len() as i32 {
                return cur_line.chars[pos.x as usize];
            }
        }
        AttributedChar::invisible()
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
            self.lines.resize(index as usize, Line::new());
        }

        self.lines.insert(index as usize, line);
    }

    pub fn swap_char(&mut self, pos1: Position, pos2: Position) {
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

    pub fn get_width(&self) -> i32 {
        self.size.width
    }

    pub fn get_line_count(&self) -> i32 {
        self.lines.len() as i32
    }

    pub fn get_line_length(&self, line: i32) -> i32 {
        self.lines[line as usize].get_line_length() as i32
    }
}

#[cfg(test)]
mod tests {
    use crate::{AttributedChar, Layer, Line, Position, TextAttribute};

    #[test]
    fn test_get_char() {
        let mut layer = Layer::new("Background", 80, 25);
        let mut line = Line::new();
        line.set_char(10, AttributedChar::new('a', TextAttribute::default()));

        layer.insert_line(0, line);

        assert_eq!(
            AttributedChar::invisible(),
            layer.get_char(Position::new(-1, -1))
        );
        assert_eq!(
            AttributedChar::invisible(),
            layer.get_char(Position::new(1000, 1000))
        );
        assert_eq!('a', layer.get_char(Position::new(10, 0)).ch);
        assert_eq!(
            AttributedChar::invisible(),
            layer.get_char(Position::new(9, 0))
        );
        assert_eq!(
            AttributedChar::invisible(),
            layer.get_char(Position::new(11, 0))
        );
    }

    #[test]
    fn test_get_char_intransparent() {
        let mut layer = Layer::new("Background", 80, 25);
        let mut line = Line::new();
        line.set_char(10, AttributedChar::new('a', TextAttribute::default()));

        layer.insert_line(0, line);

        assert_eq!(
            AttributedChar::default(),
            layer.get_char(Position::new(-1, -1))
        );
        assert_eq!(
            AttributedChar::default(),
            layer.get_char(Position::new(1000, 1000))
        );
        assert_eq!('a', layer.get_char(Position::new(10, 0)).ch);
        assert_eq!(
            AttributedChar::invisible(),
            layer.get_char(Position::new(9, 0))
        );
        assert_eq!(
            AttributedChar::default(),
            layer.get_char(Position::new(11, 0))
        );
    }

    #[test]
    fn test_insert_line() {
        let mut layer = Layer::new("Background", 80, 25);
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
