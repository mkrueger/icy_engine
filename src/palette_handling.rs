use std::fmt::Display;

#[derive(Debug, Clone, Copy, Default)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{Color: r={:02X}, g={:02X}, b{:02X}}}",
            self.r, self.g, self.b
        )
    }
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }
    pub fn get_rgb_f64(self) -> (f64, f64, f64) {
        (
            self.r as f64 / 255_f64,
            self.g as f64 / 255_f64,
            self.b as f64 / 255_f64,
        )
    }

    pub fn get_rgb_f32(self) -> (f32, f32, f32) {
        (
            self.r as f32 / 255_f32,
            self.g as f32 / 255_f32,
            self.b as f32 / 255_f32,
        )
    }

    #[inline(always)]
    pub fn get_rgb(self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }
}
impl PartialEq for Color {
    fn eq(&self, other: &Color) -> bool {
        self.r == other.r && self.g == other.g && self.b == other.b
    }
}

#[derive(Debug, Clone)]
pub struct Palette {
    pub colors: Vec<Color>,
}

static EGA_COLOR_OFFSETS: [usize; 16] = [0, 1, 2, 3, 4, 5, 20, 7, 56, 57, 58, 59, 60, 61, 62, 63];

pub const DOS_DEFAULT_PALETTE: [Color; 16] = [
    Color {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    }, // black
    Color {
        r: 0x00,
        g: 0x00,
        b: 0xAA,
    }, // blue
    Color {
        r: 0x00,
        g: 0xAA,
        b: 0x00,
    }, // green
    Color {
        r: 0x00,
        g: 0xAA,
        b: 0xAA,
    }, // cyan
    Color {
        r: 0xAA,
        g: 0x00,
        b: 0x00,
    }, // red
    Color {
        r: 0xAA,
        g: 0x00,
        b: 0xAA,
    }, // magenta
    Color {
        r: 0xAA,
        g: 0x55,
        b: 0x00,
    }, // brown
    Color {
        r: 0xAA,
        g: 0xAA,
        b: 0xAA,
    }, // lightgray
    Color {
        r: 0x55,
        g: 0x55,
        b: 0x55,
    }, // darkgray
    Color {
        r: 0x55,
        g: 0x55,
        b: 0xFF,
    }, // lightblue
    Color {
        r: 0x55,
        g: 0xFF,
        b: 0x55,
    }, // lightgreen
    Color {
        r: 0x55,
        g: 0xFF,
        b: 0xFF,
    }, // lightcyan
    Color {
        r: 0xFF,
        g: 0x55,
        b: 0x55,
    }, // lightred
    Color {
        r: 0xFF,
        g: 0x55,
        b: 0xFF,
    }, // lightmagenta
    Color {
        r: 0xFF,
        g: 0xFF,
        b: 0x55,
    }, // yellow
    Color {
        r: 0xFF,
        g: 0xFF,
        b: 0xFF,
    }, // white
];

// colors taken from "C64 Community Colors V1.2a" palette, see
// https://p1x3l.net/36/c64-community-colors-theor/
pub const C64_DEFAULT_PALETTE: [Color; 16] = [
    Color {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    }, // black
    Color {
        r: 0xFF,
        g: 0xFF,
        b: 0xFF,
    }, // white
    Color {
        r: 0xAF,
        g: 0x2A,
        b: 0x29,
    }, // red
    Color {
        r: 0x62,
        g: 0xD8,
        b: 0xCC,
    }, // cyan
    Color {
        r: 0xB0,
        g: 0x3F,
        b: 0xB6,
    }, // violett
    Color {
        r: 0x4A,
        g: 0xC6,
        b: 0x4A,
    }, // green
    Color {
        r: 0x37,
        g: 0x39,
        b: 0xC4,
    }, // blue
    Color {
        r: 0xE4,
        g: 0xED,
        b: 0x4E,
    }, // yellow
    Color {
        r: 0xB6,
        g: 0x59,
        b: 0x1C,
    }, // orange
    Color {
        r: 0x68,
        g: 0x38,
        b: 0x08,
    }, // brown
    Color {
        r: 0xEA,
        g: 0x74,
        b: 0x6C,
    }, // lightred
    Color {
        r: 0x4D,
        g: 0x4D,
        b: 0x4D,
    }, // gray1
    Color {
        r: 0x84,
        g: 0x84,
        b: 0x84,
    }, // gray2
    Color {
        r: 0xA6,
        g: 0xFA,
        b: 0x9E,
    }, // lightgreen
    Color {
        r: 0x70,
        g: 0x7C,
        b: 0xE6,
    }, // lightblue
    Color {
        r: 0xB6,
        g: 0xB6,
        b: 0xB5,
    }, // gray3
];

pub const ATARI_DEFAULT_PALETTE: [Color; 16] = [
    Color {
        r: 0x09,
        g: 0x51,
        b: 0x83,
    }, // background
    Color {
        r: 0x00,
        g: 0x00,
        b: 0xAA,
    }, // unused
    Color {
        r: 0x00,
        g: 0xAA,
        b: 0x00,
    }, // unused
    Color {
        r: 0x00,
        g: 0xAA,
        b: 0xAA,
    }, // unused
    Color {
        r: 0xAA,
        g: 0x00,
        b: 0x00,
    }, // unused
    Color {
        r: 0xAA,
        g: 0x00,
        b: 0xAA,
    }, // unused
    Color {
        r: 0xAA,
        g: 0x55,
        b: 0x00,
    }, // unused
    Color {
        r: 0x65,
        g: 0xB7,
        b: 0xE9,
    }, // foreground
    Color {
        r: 0x55,
        g: 0x55,
        b: 0x55,
    }, // unused
    Color {
        r: 0x55,
        g: 0x55,
        b: 0xFF,
    }, // unused
    Color {
        r: 0x55,
        g: 0xFF,
        b: 0x55,
    }, // unused
    Color {
        r: 0x55,
        g: 0xFF,
        b: 0xFF,
    }, // unused
    Color {
        r: 0xFF,
        g: 0x55,
        b: 0x55,
    }, // unused
    Color {
        r: 0xFF,
        g: 0x55,
        b: 0xFF,
    }, // unused
    Color {
        r: 0xFF,
        g: 0xFF,
        b: 0x55,
    }, // unused
    Color {
        r: 0xFF,
        g: 0xFF,
        b: 0xFF,
    }, // unused
];

pub const EGA_PALETTE: [Color; 64] = [
    Color {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0x00,
        g: 0x00,
        b: 0xAA,
    },
    Color {
        r: 0x00,
        g: 0xAA,
        b: 0x00,
    },
    Color {
        r: 0x00,
        g: 0xAA,
        b: 0xAA,
    },
    Color {
        r: 0xAA,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0xAA,
        g: 0x00,
        b: 0xAA,
    },
    Color {
        r: 0xAA,
        g: 0xAA,
        b: 0x00,
    },
    Color {
        r: 0xAA,
        g: 0xAA,
        b: 0xAA,
    },
    Color {
        r: 0x00,
        g: 0x00,
        b: 0x55,
    },
    Color {
        r: 0x00,
        g: 0x00,
        b: 0xFF,
    },
    Color {
        r: 0x00,
        g: 0xAA,
        b: 0x55,
    },
    Color {
        r: 0x00,
        g: 0xAA,
        b: 0xFF,
    },
    Color {
        r: 0xAA,
        g: 0x00,
        b: 0x55,
    },
    Color {
        r: 0xAA,
        g: 0x00,
        b: 0xFF,
    },
    Color {
        r: 0xAA,
        g: 0xAA,
        b: 0x55,
    },
    Color {
        r: 0xAA,
        g: 0xAA,
        b: 0xFF,
    },
    Color {
        r: 0x00,
        g: 0x55,
        b: 0x00,
    },
    Color {
        r: 0x00,
        g: 0x55,
        b: 0xAA,
    },
    Color {
        r: 0x00,
        g: 0xFF,
        b: 0x00,
    },
    Color {
        r: 0x00,
        g: 0xFF,
        b: 0xAA,
    },
    Color {
        r: 0xAA,
        g: 0x55,
        b: 0x00,
    },
    Color {
        r: 0xAA,
        g: 0x55,
        b: 0xAA,
    },
    Color {
        r: 0xAA,
        g: 0xFF,
        b: 0x00,
    },
    Color {
        r: 0xAA,
        g: 0xFF,
        b: 0xAA,
    },
    Color {
        r: 0x00,
        g: 0x55,
        b: 0x55,
    },
    Color {
        r: 0x00,
        g: 0x55,
        b: 0xFF,
    },
    Color {
        r: 0x00,
        g: 0xFF,
        b: 0x55,
    },
    Color {
        r: 0x00,
        g: 0xFF,
        b: 0xFF,
    },
    Color {
        r: 0xAA,
        g: 0x55,
        b: 0x55,
    },
    Color {
        r: 0xAA,
        g: 0x55,
        b: 0xFF,
    },
    Color {
        r: 0xAA,
        g: 0xFF,
        b: 0x55,
    },
    Color {
        r: 0xAA,
        g: 0xFF,
        b: 0xFF,
    },
    Color {
        r: 0x55,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0x55,
        g: 0x00,
        b: 0xAA,
    },
    Color {
        r: 0x55,
        g: 0xAA,
        b: 0x00,
    },
    Color {
        r: 0x55,
        g: 0xAA,
        b: 0xAA,
    },
    Color {
        r: 0xFF,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0xFF,
        g: 0x00,
        b: 0xAA,
    },
    Color {
        r: 0xFF,
        g: 0xAA,
        b: 0x00,
    },
    Color {
        r: 0xFF,
        g: 0xAA,
        b: 0xAA,
    },
    Color {
        r: 0x55,
        g: 0x00,
        b: 0x55,
    },
    Color {
        r: 0x55,
        g: 0x00,
        b: 0xFF,
    },
    Color {
        r: 0x55,
        g: 0xAA,
        b: 0x55,
    },
    Color {
        r: 0x55,
        g: 0xAA,
        b: 0xFF,
    },
    Color {
        r: 0xFF,
        g: 0x00,
        b: 0x55,
    },
    Color {
        r: 0xFF,
        g: 0x00,
        b: 0xFF,
    },
    Color {
        r: 0xFF,
        g: 0xAA,
        b: 0x55,
    },
    Color {
        r: 0xFF,
        g: 0xAA,
        b: 0xFF,
    },
    Color {
        r: 0x55,
        g: 0x55,
        b: 0x00,
    },
    Color {
        r: 0x55,
        g: 0x55,
        b: 0xAA,
    },
    Color {
        r: 0x55,
        g: 0xFF,
        b: 0x00,
    },
    Color {
        r: 0x55,
        g: 0xFF,
        b: 0xAA,
    },
    Color {
        r: 0xFF,
        g: 0x55,
        b: 0x00,
    },
    Color {
        r: 0xFF,
        g: 0x55,
        b: 0xAA,
    },
    Color {
        r: 0xFF,
        g: 0xFF,
        b: 0x00,
    },
    Color {
        r: 0xFF,
        g: 0xFF,
        b: 0xAA,
    },
    Color {
        r: 0x55,
        g: 0x55,
        b: 0x55,
    },
    Color {
        r: 0x55,
        g: 0x55,
        b: 0xFF,
    },
    Color {
        r: 0x55,
        g: 0xFF,
        b: 0x55,
    },
    Color {
        r: 0x55,
        g: 0xFF,
        b: 0xFF,
    },
    Color {
        r: 0xFF,
        g: 0x55,
        b: 0x55,
    },
    Color {
        r: 0xFF,
        g: 0x55,
        b: 0xFF,
    },
    Color {
        r: 0xFF,
        g: 0xFF,
        b: 0x55,
    },
    Color {
        r: 0xFF,
        g: 0xFF,
        b: 0xFF,
    },
];

pub const XTERM_256_PALETTE: [Color; 256] = [
    Color {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0x80,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0x00,
        g: 0x80,
        b: 0x80,
    },
    Color {
        r: 0x80,
        g: 0x80,
        b: 0x80,
    },
    Color {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0x80,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0x00,
        g: 0x80,
        b: 0x80,
    },
    Color {
        r: 0xc0,
        g: 0xc0,
        b: 0xc0,
    },
    Color {
        r: 0x80,
        g: 0x80,
        b: 0x80,
    },
    Color {
        r: 0xff,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0x00,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0xff,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0xff,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0x00,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0xff,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0x00,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0x00,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0x00,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0x00,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0x00,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0x00,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0x00,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0x00,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0x00,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0x00,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0x00,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0x00,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0x00,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0x00,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0x00,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0x00,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0x00,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0x00,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0x00,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0x00,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0x00,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0x00,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0x00,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0x00,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0x00,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0x00,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0x00,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0x00,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0x00,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0x00,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0x5f,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0x5f,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0x5f,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0x5f,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0x5f,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0x5f,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0x5f,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0x5f,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0x5f,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0x5f,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0x5f,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0x5f,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0x5f,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0x5f,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0x5f,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0x5f,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0x5f,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0x5f,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0x5f,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0x5f,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0x5f,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0x5f,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0x5f,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0x5f,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0x5f,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0x5f,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0x5f,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0x5f,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0x5f,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0x5f,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0x5f,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0x5f,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0x5f,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0x5f,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0x5f,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0x5f,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0x87,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0x87,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0x87,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0x87,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0x87,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0x87,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0x87,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0x87,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0x87,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0x87,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0x87,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0x87,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0x87,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0x87,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0x87,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0x87,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0x87,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0x87,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0x87,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0x87,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0x87,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0x87,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0x87,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0x87,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0x87,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0x87,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0x87,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0x87,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0x87,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0x87,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0x87,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0x87,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0x87,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0x87,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0x87,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0x87,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0xaf,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0xaf,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0xaf,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0xaf,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0xaf,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0xaf,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0xaf,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0xaf,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0xaf,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0xaf,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0xaf,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0xaf,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0xaf,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0xaf,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0xaf,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0xaf,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0xaf,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0xaf,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0xaf,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0xaf,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0xaf,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0xaf,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0xaf,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0xaf,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0xaf,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0xaf,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0xaf,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0xaf,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0xaf,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0xaf,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0xaf,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0xaf,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0xaf,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0xaf,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0xaf,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0xaf,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0xd7,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0xd7,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0xd7,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0xd7,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0xd7,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0xd7,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0xd7,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0xd7,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0xd7,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0xd7,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0xd7,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0xd7,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0xd7,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0xd7,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0xd7,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0xd7,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0xd7,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0xd7,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0xd7,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0xd7,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0xd7,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0xd7,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0xd7,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0xd7,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0xd7,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0xd7,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0xd7,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0xd7,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0xd7,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0xd7,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0xd7,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0xd7,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0xd7,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0xd7,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0xd7,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0xd7,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0xff,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0xff,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0xff,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0xff,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0xff,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0xff,
        g: 0x00,
        b: 0x00,
    },
    Color {
        r: 0xff,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0xff,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0xff,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0xff,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0xff,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0xff,
        g: 0x5f,
        b: 0x5f,
    },
    Color {
        r: 0xff,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0xff,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0xff,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0xff,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0xff,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0xff,
        g: 0x87,
        b: 0x87,
    },
    Color {
        r: 0xff,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0xff,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0xff,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0xff,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0xff,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0xff,
        g: 0xaf,
        b: 0xaf,
    },
    Color {
        r: 0xff,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0xff,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0xff,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0xff,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0xff,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0xff,
        g: 0xd7,
        b: 0xd7,
    },
    Color {
        r: 0xff,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0xff,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0xff,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0xff,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0xff,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0xff,
        g: 0xff,
        b: 0xff,
    },
    Color {
        r: 0x08,
        g: 0x08,
        b: 0x08,
    },
    Color {
        r: 0x12,
        g: 0x12,
        b: 0x12,
    },
    Color {
        r: 0x1c,
        g: 0x1c,
        b: 0x1c,
    },
    Color {
        r: 0x26,
        g: 0x26,
        b: 0x26,
    },
    Color {
        r: 0x30,
        g: 0x30,
        b: 0x30,
    },
    Color {
        r: 0x3a,
        g: 0x3a,
        b: 0x3a,
    },
    Color {
        r: 0x44,
        g: 0x44,
        b: 0x44,
    },
    Color {
        r: 0x4e,
        g: 0x4e,
        b: 0x4e,
    },
    Color {
        r: 0x58,
        g: 0x58,
        b: 0x58,
    },
    Color {
        r: 0x60,
        g: 0x60,
        b: 0x60,
    },
    Color {
        r: 0x66,
        g: 0x66,
        b: 0x66,
    },
    Color {
        r: 0x76,
        g: 0x76,
        b: 0x76,
    },
    Color {
        r: 0x80,
        g: 0x80,
        b: 0x80,
    },
    Color {
        r: 0x8a,
        g: 0x8a,
        b: 0x8a,
    },
    Color {
        r: 0x94,
        g: 0x94,
        b: 0x94,
    },
    Color {
        r: 0x9e,
        g: 0x9e,
        b: 0x9e,
    },
    Color {
        r: 0xa8,
        g: 0xa8,
        b: 0xa8,
    },
    Color {
        r: 0xb2,
        g: 0xb2,
        b: 0xb2,
    },
    Color {
        r: 0xbc,
        g: 0xbc,
        b: 0xbc,
    },
    Color {
        r: 0xc6,
        g: 0xc6,
        b: 0xc6,
    },
    Color {
        r: 0xd0,
        g: 0xd0,
        b: 0xd0,
    },
    Color {
        r: 0xda,
        g: 0xda,
        b: 0xda,
    },
    Color {
        r: 0xe4,
        g: 0xe4,
        b: 0xe4,
    },
    Color {
        r: 0xee,
        g: 0xee,
        b: 0xee,
    },
];

pub const VIEWDATA_PALETTE: [Color; 16] = [
    Color {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    }, // black
    Color {
        r: 0xFF,
        g: 0x00,
        b: 0x00,
    }, // red
    Color {
        r: 0x00,
        g: 0xFF,
        b: 0x00,
    }, // green
    Color {
        r: 0xFF,
        g: 0xFF,
        b: 0x00,
    }, // yellow
    Color {
        r: 0x00,
        g: 0x00,
        b: 0xFF,
    }, // blue
    Color {
        r: 0xFF,
        g: 0x00,
        b: 0xFF,
    }, // magenta
    Color {
        r: 0x00,
        g: 0xFF,
        b: 0xFF,
    }, // cyan
    Color {
        r: 0xFF,
        g: 0xFF,
        b: 0xFF,
    }, // white
    Color {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    }, // black
    Color {
        r: 0xFF,
        g: 0x00,
        b: 0x00,
    }, // red
    Color {
        r: 0x00,
        g: 0xFF,
        b: 0x00,
    }, // green
    Color {
        r: 0xFF,
        g: 0xFF,
        b: 0x00,
    }, // yellow
    Color {
        r: 0x00,
        g: 0x00,
        b: 0xFF,
    }, // blue
    Color {
        r: 0xFF,
        g: 0x00,
        b: 0xFF,
    }, // magenta
    Color {
        r: 0x00,
        g: 0xFF,
        b: 0xFF,
    }, // cyan
    Color {
        r: 0xFF,
        g: 0xFF,
        b: 0xFF,
    }, // white
];

impl Palette {
    pub fn new() -> Self {
        Palette {
            colors: DOS_DEFAULT_PALETTE.to_vec(),
        }
    }

    pub fn len(&self) -> u32 {
        self.colors.len() as u32
    }

    pub fn clear(&mut self) {
        self.colors.clear();
    }

    pub fn fill_to_16(&mut self) {
        if self.colors.len() < DOS_DEFAULT_PALETTE.len() {
            self.colors
                .extend(&DOS_DEFAULT_PALETTE[self.colors.len()..]);
        }
    }

    pub fn is_default(&self) -> bool {
        if self.colors.len() != DOS_DEFAULT_PALETTE.len() {
            return false;
        }
        #[allow(clippy::needless_range_loop)]
        for i in 0..DOS_DEFAULT_PALETTE.len() {
            if self.colors[i] != DOS_DEFAULT_PALETTE[i] {
                return false;
            }
        }
        true
    }

    pub fn set_color_rgb(&mut self, number: usize, r: u8, g: u8, b: u8) {
        if self.colors.len() <= number {
            self.colors.resize(number + 1, Color::default());
        }
        self.colors[number] = Color { r, g, b };
    }

    pub fn set_color_hsl(&mut self, number: usize, h: f32, s: f32, l: f32) {
        if self.colors.len() <= number {
            self.colors.resize(number + 1, Color::default());
        }

        let (r, g, b) = if l == 0.0 {
            (0, 0, 0)
        } else if s == 0.0 {
            let l = (l * 255.0) as u8;
            (l, l, l)
        } else {
            let temp2 = if l <= 0.5 {
                l * (1.0 + s)
            } else {
                l + s - (l * s)
            };
            let temp1 = 2.0 * l - temp2;
            (
                convert_vector(temp2, temp1, h + 1.0 / 3.0),
                convert_vector(temp2, temp1, h),
                convert_vector(temp2, temp1, h - 1.0 / 3.0),
            )
        };

        self.colors[number] = Color { r, g, b };
    }

    pub fn insert_color(&mut self, color: Color) -> u32 {
        for i in 0..self.colors.len() {
            let col = self.colors[i];
            if col == color {
                return i as u32;
            }
        }
        self.colors.push(color);
        (self.colors.len() - 1) as u32
    }

    pub fn insert_color_rgb(&mut self, r: u8, g: u8, b: u8) -> u32 {
        self.insert_color(Color::new(r, g, b))
    }

    pub fn from(pal: &[u8]) -> Self {
        let mut colors = Vec::new();
        let mut o = 0;
        while o < pal.len() {
            colors.push(Color {
                r: pal[o] << 2 | pal[o] >> 4,
                g: pal[o + 1] << 2 | pal[o + 1] >> 4,
                b: pal[o + 2] << 2 | pal[o + 2] >> 4,
            });
            o += 3;
        }

        Palette { colors }
    }

    pub fn cycle_ega_colors(&self) -> Palette {
        let mut colors = self.colors.clone();
        #[allow(clippy::needless_range_loop)]
        for i in 0..EGA_COLOR_OFFSETS.len() {
            let offset = EGA_COLOR_OFFSETS[i];
            if i == offset {
                continue;
            }
            colors.swap(i, offset);
        }
        Palette { colors }
    }

    pub fn to_ega_palette(&self) -> Vec<u8> {
        let mut ega_colors;

        if self.colors.len() == 64 {
            //assume ega palette
            ega_colors = self.colors.clone();
            #[allow(clippy::needless_range_loop)]
            for i in 0..EGA_COLOR_OFFSETS.len() {
                let offset = EGA_COLOR_OFFSETS[i];
                if i == offset {
                    continue;
                }
                ega_colors.swap(i, offset);
            }
        } else {
            // just store the first 16 colors to the standard EGA palette
            ega_colors = EGA_PALETTE.to_vec();
            for i in 0..16 {
                if i >= self.colors.len() {
                    break;
                }
                ega_colors[EGA_COLOR_OFFSETS[i]] = self.colors[i];
            }
        }
        let mut res = Vec::with_capacity(3 * 64);
        for col in ega_colors {
            res.push(col.r >> 2 | col.r << 4);
            res.push(col.g >> 2 | col.g << 4);
            res.push(col.b >> 2 | col.b << 4);
        }
        res
    }

    pub fn to_16color_vec(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(3 * 16);
        #[allow(clippy::needless_range_loop)]
        for i in 0..16 {
            let col = if i < self.colors.len() {
                self.colors[i]
            } else {
                DOS_DEFAULT_PALETTE[i]
            };

            res.push(col.r >> 2 | col.r << 4);
            res.push(col.g >> 2 | col.g << 4);
            res.push(col.b >> 2 | col.b << 4);
        }
        res
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut res = Vec::new();
        res.resize(3 * self.colors.len(), 0);
        for col in &self.colors {
            res.push(col.r >> 2 | col.r << 4);
            res.push(col.g >> 2 | col.g << 4);
            res.push(col.b >> 2 | col.b << 4);
        }
        res
    }
}

fn convert_vector(temp2: f32, temp1: f32, mut x: f32) -> u8 {
    if x < 0.0 {
        x += 1.0;
    }
    if x > 1.0 {
        x -= 1.0;
    }
    let v = if 6.0 * x < 1.0 {
        temp1 + (temp2 - temp1) * x * 6.0
    } else if 2.0 * x < 1.0 {
        temp2
    } else if 3.0 * x < 2.0 {
        temp1 + (temp2 - temp1) * ((2.0 / 3.0) - x) * 6.0
    } else {
        temp1
    };

    (v * 255.0) as u8
}

impl Default for Palette {
    fn default() -> Self {
        Self::new()
    }
}
