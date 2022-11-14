use super::TextAttribute;

#[derive(Clone, Copy, Debug)]
pub struct AttributedChar {
    pub ch: char,
    pub attribute: TextAttribute,
    font_page: usize,
}

impl Default for AttributedChar {
    fn default() -> Self {
        AttributedChar {
            ch: ' ',
            attribute: super::TextAttribute::default(),
            font_page: 0
        }
    }
}

impl AttributedChar {
    pub fn new(ch: char, attribute: TextAttribute) -> Self {
        AttributedChar {
            ch,
            attribute,
            font_page: 0
        }
    }

    #[inline(always)] 
    pub fn is_transparent(self) -> bool {
        (self.ch == '\0' || self.ch == ' ') && self.attribute.get_background() == 0
    }

    #[inline(always)] 
    pub fn get_font_page(&self) -> usize {
        self.font_page
    }
    pub fn set_font_page(&mut self, page: usize) {
        self.font_page = page;
    }
}

impl PartialEq for AttributedChar {
    fn eq(&self, other: &AttributedChar) -> bool {
        self.ch == other.ch && self.attribute == other.attribute
    }
}

impl std::fmt::Display for AttributedChar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(Char: {}/0x{0:X} '{}', Attr: {}, ExtFont: {})", self.ch as u32, char::from_u32(self.ch as u32).unwrap(),  self.attribute, self.font_page)
    }
}
 /*
pub fn get_color(color: u8) -> &'static str
{
    match color {
        0 => "Black",
        1 => "Blue",
        2 => "Green",
        3 => "Aqua",
        4 => "Red",
        5 => "Purple",
        6 => "Brown",
        7 => "Light Gray",
        8 => "Gray",
        9 => "Light Blue",
        10 => "Light Green",
        11 => "Light Aqua",
        12 => "Light Red",
        13 => "Light Purple",
        14 => "Light Yelllow",
        15 => "White",
        _ => "Unknown"
    }
}
*/