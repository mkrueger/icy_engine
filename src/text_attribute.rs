use super::BufferType;

mod attribute {
    pub const NONE:u8      = 0;
    pub const BOLD:u8      = 0b0000_0001;   // n = 1
    pub const FAINT:u8     = 0b0000_0010;  // n = 2
    pub const ITALIC:u8    = 0b0000_0100;  // n = 2
    pub const BLINK:u8     = 0b0000_1000;  // n = 5

    pub const UNDERLINE:u8       = 0b0001_0000;  // n = 4 
    pub const DOUBLE_UNDERLINE:u8 = 0b0011_0000;  // n = 21
    pub const CONCEAL:u8         = 0b0100_0000;  // n = 21
    pub const CROSSED_OUT:u8      = 0b1000_0000;  // n = 9
}

#[derive(Clone, Copy, Debug)]
pub struct TextAttribute {
    foreground_color: u8,
    background_color: u8,
    attr: u8
}

impl Default for TextAttribute {
    fn default() -> Self {
        Self { foreground_color: 7, background_color: 0, attr: attribute::NONE }
    }
}

impl std::fmt::Display for TextAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(Attr: {:X}, fg {}, bg {}, blink {})", self.as_u8(BufferType::LegacyDos), self.get_foreground(), self.get_background(), self.is_blinking())
    }
}

impl TextAttribute
{
    pub fn from_u8(attr: u8, buffer_type: BufferType) -> Self
    {
        let mut blink = false;
        let background_color = if buffer_type.use_ice_colors() {
            attr >> 4
        } else {
            blink = attr & 0b1000_0000 != 0;
            (attr >> 4) & 0b0111
        };

        let mut bold = false;
        let foreground_color = if buffer_type.use_extended_font() {
            attr & 0b0111
        } else {
            bold = attr & 0b1000 != 0;
            attr & 0b0111
        };

        let mut attr = TextAttribute {
            foreground_color,
            background_color,
            ..Default::default()
        };

        attr.set_is_bold(bold);
        attr.set_is_blinking(blink);

        return attr;
    }

    pub fn from_color(fg: u8, bg: u8) -> Self
    {
        let mut res = TextAttribute { foreground_color: fg & 0x7, background_color: bg & 0x7, ..Default::default() };
        res.set_is_bold((fg & 0b1000) != 0);
        res.set_is_blinking((bg & 0b1000) != 0);
        res
    }

    pub fn as_u8(self, buffer_type: BufferType) -> u8
    {
        let fg = if buffer_type.use_extended_font() {
            self.foreground_color & 0b_0111
        } else {
            self.foreground_color & 0b_0111 | if self.is_bold() { 0b_1000 } else { 0 }
        };

        let bg = self.background_color & 0b_0111 | if self.is_blinking() { 0b_1000 } else { 0 };
        fg | bg << 4
    }


    pub fn get_foreground(self) -> u8
    {
        self.foreground_color
    }

    pub fn set_foreground(&mut self, color: u8) 
    {
        self.foreground_color = color;
    }

    pub fn get_background(self) -> u8
    {
        self.background_color
    }

    pub fn set_background(&mut self, color: u8) 
    {
        self.background_color = color;
    }

    pub fn is_bold(self) -> bool
    {
        (self.attr & attribute::BOLD) == attribute::BOLD
    }

    pub fn set_is_bold(&mut self, is_bold: bool)
    {
        if is_bold {
            self.attr = attribute::BOLD;
        } else {
            self.attr &= !attribute::BOLD;
        }
    }

    pub fn is_faint(self) -> bool
    {
        (self.attr & attribute::FAINT) == attribute::FAINT
    }

    pub fn set_is_faint(&mut self, is_faint: bool)
    {
        if is_faint {
            self.attr = attribute::FAINT;
        } else {
            self.attr &= !attribute::FAINT;
        }
    }

    pub fn is_italic(self) -> bool
    {
        (self.attr & attribute::ITALIC) == attribute::ITALIC
    }

    pub fn set_is_italic(&mut self, is_italic: bool)
    {
        if is_italic {
            self.attr = attribute::ITALIC;
        } else {
            self.attr &= !attribute::ITALIC;
        }
    }

    pub fn is_blinking(self) -> bool
    {
        (self.attr & attribute::BLINK) == attribute::BLINK
    }

    pub fn set_is_blinking(&mut self, is_blink: bool)
    {
        if is_blink {
            self.attr = attribute::BLINK;
        } else {
            self.attr &= !attribute::BLINK;
        }
    }

    pub fn is_crossed_out(self) -> bool
    {
        (self.attr & attribute::CROSSED_OUT) == attribute::CROSSED_OUT
    }

    pub fn set_is_crossed_out(&mut self, is_crossed_out: bool)
    {
        if is_crossed_out {
            self.attr = attribute::CROSSED_OUT;
        } else {
            self.attr &= !attribute::CROSSED_OUT;
        }
    }

    pub fn is_underlined(self) -> bool
    {
        (self.attr & attribute::UNDERLINE) == attribute::UNDERLINE
    }

    pub fn set_is_underlined(&mut self, is_underline: bool)
    {
        if is_underline {
            self.attr = attribute::UNDERLINE;
        } else {
            self.attr &= !attribute::UNDERLINE;
        }
    }

    pub fn is_double_underlined(self) -> bool
    {
        (self.attr & attribute::DOUBLE_UNDERLINE) == attribute::DOUBLE_UNDERLINE
    }

    pub fn set_is_double_underlined(&mut self, is_double_underline: bool)
    {
        if is_double_underline {
            self.attr = attribute::DOUBLE_UNDERLINE;
        } else {
            self.attr &= !attribute::DOUBLE_UNDERLINE;
        }
    }

    pub fn is_concealed(self) -> bool
    {
        (self.attr & attribute::CONCEAL) == attribute::CONCEAL
    }

    pub fn set_is_concealed(&mut self, is_concealed: bool)
    {
        if is_concealed {
            self.attr = attribute::CONCEAL;
        } else {
            self.attr &= !attribute::CONCEAL;
        }
    }

}

impl PartialEq for TextAttribute {
    fn eq(&self, other: &TextAttribute) -> bool {
        self.foreground_color == other.foreground_color && self.background_color == other.background_color && self.attr == other.attr
    }
}
