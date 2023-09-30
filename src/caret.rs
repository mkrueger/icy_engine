use super::{Position, TextAttribute};

#[derive(Clone)]
pub struct Caret {
    pub(super) pos: Position,
    pub(super) attribute: TextAttribute,
    pub insert_mode: bool,
    pub is_visible: bool,
    pub is_blinking: bool,
    ice_mode: bool,
}

impl Caret {
    pub fn new(pos: Position) -> Self {
        Self { pos, ..Default::default() }
    }

    pub fn new_xy(x: i32, y: i32) -> Self {
        Self {
            pos: Position { x, y },
            ..Default::default()
        }
    }

    pub fn get_attribute(&self) -> TextAttribute {
        let mut result = self.attribute;
        if self.ice_mode {
            let bg = result.get_background();
            if bg < 8 && result.is_blinking() {
                result.set_background(bg + 8);
            }
            // ice mode is not blinking
            result.set_is_blinking(false);
        }
        result
    }

    pub fn get_position(&self) -> Position {
        self.pos
    }

    pub fn set_position(&mut self, pos: Position) {
        self.pos = pos;
    }

    pub fn set_position_xy(&mut self, x: i32, y: i32) {
        self.pos = Position::new(x, y);
    }

    pub fn set_x_position(&mut self, x: i32) {
        self.pos.x = x;
    }

    pub fn set_y_position(&mut self, y: i32) {
        self.pos.y = y;
    }

    pub fn set_attr(&mut self, attr: TextAttribute) {
        self.attribute = attr;
    }

    pub fn set_foreground(&mut self, color: u32) {
        self.attribute.set_foreground(color);
    }

    pub fn set_background(&mut self, color: u32) {
        self.attribute.set_background(color);
    }

    pub(crate) fn reset(&mut self) {
        self.pos = Position::default();
        self.attribute = TextAttribute::default();
        self.insert_mode = false;
        self.is_visible = true;
        self.is_blinking = true;
        self.ice_mode = false;
    }

    pub fn get_font_page(&self) -> usize {
        self.attribute.get_font_page()
    }

    pub fn set_font_page(&mut self, page: usize) {
        self.attribute.set_font_page(page);
    }

    pub fn reset_color_attribute(&mut self) {
        let font_page = self.attribute.get_font_page();
        self.attribute = TextAttribute::default();
        self.attribute.set_font_page(font_page);
    }

    pub fn ice_mode(&self) -> bool {
        self.ice_mode
    }
    pub fn set_ice_mode(&mut self, ice_mode: bool) {
        self.ice_mode = ice_mode;
    }
}

#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for Caret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cursor")
            .field("pos", &self.pos)
            .field("attr", &self.attribute)
            .field("insert_mode", &self.insert_mode)
            .finish()
    }
}

impl Default for Caret {
    fn default() -> Self {
        Self {
            pos: Position::default(),
            attribute: TextAttribute::default(),
            insert_mode: false,
            is_visible: true,
            is_blinking: true,
            ice_mode: false,
        }
    }
}

impl PartialEq for Caret {
    fn eq(&self, other: &Caret) -> bool {
        self.pos == other.pos && self.attribute == other.attribute
    }
}
