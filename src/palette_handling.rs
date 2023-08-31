#![allow(clippy::many_single_char_names)]
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

    pub fn get_rgb(self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }
}
impl PartialEq for Color {
    fn eq(&self, other: &Color) -> bool {
        self.r == other.r && self.g == other.g && self.b == other.b
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(value: (u8, u8, u8)) -> Self {
        Color {
            r: value.0,
            g: value.1,
            b: value.2,
        }
    }
}

impl From<Color> for (u8, u8, u8) {
    fn from(value: Color) -> (u8, u8, u8) {
        (value.r, value.g, value.b)
    }
}

impl From<[u8; 3]> for Color {
    fn from(value: [u8; 3]) -> Self {
        Color {
            r: value[0],
            g: value[1],
            b: value[2],
        }
    }
}

impl From<Color> for [u8; 3] {
    fn from(value: Color) -> [u8; 3] {
        [value.r, value.g, value.b]
    }
}

impl From<(f32, f32, f32)> for Color {
    fn from(value: (f32, f32, f32)) -> Self {
        Color {
            r: (value.0 * 255_f32) as u8,
            g: (value.1 * 255_f32) as u8,
            b: (value.2 * 255_f32) as u8,
        }
    }
}

impl From<Color> for (f32, f32, f32) {
    fn from(value: Color) -> (f32, f32, f32) {
        (
            (value.r as f32 / 255_f32),
            (value.g as f32 / 255_f32),
            (value.b as f32 / 255_f32),
        )
    }
}

impl From<[f32; 3]> for Color {
    fn from(value: [f32; 3]) -> Self {
        Color {
            r: (value[0] * 255_f32) as u8,
            g: (value[1] * 255_f32) as u8,
            b: (value[2] * 255_f32) as u8,
        }
    }
}

impl From<Color> for [f32; 3] {
    fn from(value: Color) -> [f32; 3] {
        [
            (value.r as f32 / 255_f32),
            (value.g as f32 / 255_f32),
            (value.b as f32 / 255_f32),
        ]
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
    // 0: Black (SYSTEM)
    Color {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    },
    // 1: Maroon (SYSTEM)
    Color {
        r: 0x80,
        g: 0x00,
        b: 0x00,
    },
    // 2: Green (SYSTEM)
    Color {
        r: 0x00,
        g: 0x80,
        b: 0x00,
    },
    // 3: Olive (SYSTEM)
    Color {
        r: 0x80,
        g: 0x80,
        b: 0x00,
    },
    // 4: Navy (SYSTEM)
    Color {
        r: 0x00,
        g: 0x00,
        b: 0x80,
    },
    // 5: Purple (SYSTEM)
    Color {
        r: 0x80,
        g: 0x00,
        b: 0x80,
    },
    // 6: Teal (SYSTEM)
    Color {
        r: 0x00,
        g: 0x80,
        b: 0x80,
    },
    // 7: Silver (SYSTEM)
    Color {
        r: 0xc0,
        g: 0xc0,
        b: 0xc0,
    },
    // 8: Grey (SYSTEM)
    Color {
        r: 0x80,
        g: 0x80,
        b: 0x80,
    },
    // 9: Red (SYSTEM)
    Color {
        r: 0xff,
        g: 0x00,
        b: 0x00,
    },
    // 10: Lime (SYSTEM)
    Color {
        r: 0x00,
        g: 0xff,
        b: 0x00,
    },
    // 11: Yellow (SYSTEM)
    Color {
        r: 0xff,
        g: 0xff,
        b: 0x00,
    },
    // 12: Blue (SYSTEM)
    Color {
        r: 0x00,
        g: 0x00,
        b: 0xff,
    },
    // 13: Fuchsia (SYSTEM)
    Color {
        r: 0xff,
        g: 0x00,
        b: 0xff,
    },
    // 14: Aqua (SYSTEM)
    Color {
        r: 0x00,
        g: 0xff,
        b: 0xff,
    },
    // 15: White (SYSTEM)
    Color {
        r: 0xff,
        g: 0xff,
        b: 0xff,
    },
    // 16: Grey0
    Color {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    },
    // 17: NavyBlue
    Color {
        r: 0x00,
        g: 0x00,
        b: 0x5f,
    },
    // 18: DarkBlue
    Color {
        r: 0x00,
        g: 0x00,
        b: 0x87,
    },
    // 19: Blue3
    Color {
        r: 0x00,
        g: 0x00,
        b: 0xaf,
    },
    // 20: Blue3
    Color {
        r: 0x00,
        g: 0x00,
        b: 0xd7,
    },
    // 21: Blue1
    Color {
        r: 0x00,
        g: 0x00,
        b: 0xff,
    },
    // 22: DarkGreen
    Color {
        r: 0x00,
        g: 0x5f,
        b: 0x00,
    },
    // 23: DeepSkyBlue4
    Color {
        r: 0x00,
        g: 0x5f,
        b: 0x5f,
    },
    // 24: DeepSkyBlue4
    Color {
        r: 0x00,
        g: 0x5f,
        b: 0x87,
    },
    // 25: DeepSkyBlue4
    Color {
        r: 0x00,
        g: 0x5f,
        b: 0xaf,
    },
    // 26: DodgerBlue3
    Color {
        r: 0x00,
        g: 0x5f,
        b: 0xd7,
    },
    // 27: DodgerBlue2
    Color {
        r: 0x00,
        g: 0x5f,
        b: 0xff,
    },
    // 28: Green4
    Color {
        r: 0x00,
        g: 0x87,
        b: 0x00,
    },
    // 29: SpringGreen4
    Color {
        r: 0x00,
        g: 0x87,
        b: 0x5f,
    },
    // 30: Turquoise4
    Color {
        r: 0x00,
        g: 0x87,
        b: 0x87,
    },
    // 31: DeepSkyBlue3
    Color {
        r: 0x00,
        g: 0x87,
        b: 0xaf,
    },
    // 32: DeepSkyBlue3
    Color {
        r: 0x00,
        g: 0x87,
        b: 0xd7,
    },
    // 33: DodgerBlue1
    Color {
        r: 0x00,
        g: 0x87,
        b: 0xff,
    },
    // 34: Green3
    Color {
        r: 0x00,
        g: 0xaf,
        b: 0x00,
    },
    // 35: SpringGreen3
    Color {
        r: 0x00,
        g: 0xaf,
        b: 0x5f,
    },
    // 36: DarkCyan
    Color {
        r: 0x00,
        g: 0xaf,
        b: 0x87,
    },
    // 37: LightSeaGreen
    Color {
        r: 0x00,
        g: 0xaf,
        b: 0xaf,
    },
    // 38: DeepSkyBlue2
    Color {
        r: 0x00,
        g: 0xaf,
        b: 0xd7,
    },
    // 39: DeepSkyBlue1
    Color {
        r: 0x00,
        g: 0xaf,
        b: 0xff,
    },
    // 40: Green3
    Color {
        r: 0x00,
        g: 0xd7,
        b: 0x00,
    },
    // 41: SpringGreen3
    Color {
        r: 0x00,
        g: 0xd7,
        b: 0x5f,
    },
    // 42: SpringGreen2
    Color {
        r: 0x00,
        g: 0xd7,
        b: 0x87,
    },
    // 43: Cyan3
    Color {
        r: 0x00,
        g: 0xd7,
        b: 0xaf,
    },
    // 44: DarkTurquoise
    Color {
        r: 0x00,
        g: 0xd7,
        b: 0xd7,
    },
    // 45: Turquoise2
    Color {
        r: 0x00,
        g: 0xd7,
        b: 0xff,
    },
    // 46: Green1
    Color {
        r: 0x00,
        g: 0xff,
        b: 0x00,
    },
    // 47: SpringGreen2
    Color {
        r: 0x00,
        g: 0xff,
        b: 0x5f,
    },
    // 48: SpringGreen1
    Color {
        r: 0x00,
        g: 0xff,
        b: 0x87,
    },
    // 49: MediumSpringGreen
    Color {
        r: 0x00,
        g: 0xff,
        b: 0xaf,
    },
    // 50: Cyan2
    Color {
        r: 0x00,
        g: 0xff,
        b: 0xd7,
    },
    // 51: Cyan1
    Color {
        r: 0x00,
        g: 0xff,
        b: 0xff,
    },
    // 52: DarkRed
    Color {
        r: 0x5f,
        g: 0x00,
        b: 0x00,
    },
    // 53: DeepPink4
    Color {
        r: 0x5f,
        g: 0x00,
        b: 0x5f,
    },
    // 54: Purple4
    Color {
        r: 0x5f,
        g: 0x00,
        b: 0x87,
    },
    // 55: Purple4
    Color {
        r: 0x5f,
        g: 0x00,
        b: 0xaf,
    },
    // 56: Purple3
    Color {
        r: 0x5f,
        g: 0x00,
        b: 0xd7,
    },
    // 57: BlueViolet
    Color {
        r: 0x5f,
        g: 0x00,
        b: 0xff,
    },
    // 58: Orange4
    Color {
        r: 0x5f,
        g: 0x5f,
        b: 0x00,
    },
    // 59: Grey37
    Color {
        r: 0x5f,
        g: 0x5f,
        b: 0x5f,
    },
    // 60: MediumPurple4
    Color {
        r: 0x5f,
        g: 0x5f,
        b: 0x87,
    },
    // 61: SlateBlue3
    Color {
        r: 0x5f,
        g: 0x5f,
        b: 0xaf,
    },
    // 62: SlateBlue3
    Color {
        r: 0x5f,
        g: 0x5f,
        b: 0xd7,
    },
    // 63: RoyalBlue1
    Color {
        r: 0x5f,
        g: 0x5f,
        b: 0xff,
    },
    // 64: Chartreuse4
    Color {
        r: 0x5f,
        g: 0x87,
        b: 0x00,
    },
    // 65: DarkSeaGreen4
    Color {
        r: 0x5f,
        g: 0x87,
        b: 0x5f,
    },
    // 66: PaleTurquoise4
    Color {
        r: 0x5f,
        g: 0x87,
        b: 0x87,
    },
    // 67: SteelBlue
    Color {
        r: 0x5f,
        g: 0x87,
        b: 0xaf,
    },
    // 68: SteelBlue3
    Color {
        r: 0x5f,
        g: 0x87,
        b: 0xd7,
    },
    // 69: CornflowerBlue
    Color {
        r: 0x5f,
        g: 0x87,
        b: 0xff,
    },
    // 70: Chartreuse3
    Color {
        r: 0x5f,
        g: 0xaf,
        b: 0x00,
    },
    // 71: DarkSeaGreen4
    Color {
        r: 0x5f,
        g: 0xaf,
        b: 0x5f,
    },
    // 72: CadetBlue
    Color {
        r: 0x5f,
        g: 0xaf,
        b: 0x87,
    },
    // 73: CadetBlue
    Color {
        r: 0x5f,
        g: 0xaf,
        b: 0xaf,
    },
    // 74: SkyBlue3
    Color {
        r: 0x5f,
        g: 0xaf,
        b: 0xd7,
    },
    // 75: SteelBlue1
    Color {
        r: 0x5f,
        g: 0xaf,
        b: 0xff,
    },
    // 76: Chartreuse3
    Color {
        r: 0x5f,
        g: 0xd7,
        b: 0x00,
    },
    // 77: PaleGreen3
    Color {
        r: 0x5f,
        g: 0xd7,
        b: 0x5f,
    },
    // 78: SeaGreen3
    Color {
        r: 0x5f,
        g: 0xd7,
        b: 0x87,
    },
    // 79: Aquamarine3
    Color {
        r: 0x5f,
        g: 0xd7,
        b: 0xaf,
    },
    // 80: MediumTurquoise
    Color {
        r: 0x5f,
        g: 0xd7,
        b: 0xd7,
    },
    // 81: SteelBlue1
    Color {
        r: 0x5f,
        g: 0xd7,
        b: 0xff,
    },
    // 82: Chartreuse2
    Color {
        r: 0x5f,
        g: 0xff,
        b: 0x00,
    },
    // 83: SeaGreen2
    Color {
        r: 0x5f,
        g: 0xff,
        b: 0x5f,
    },
    // 84: SeaGreen1
    Color {
        r: 0x5f,
        g: 0xff,
        b: 0x87,
    },
    // 85: SeaGreen1
    Color {
        r: 0x5f,
        g: 0xff,
        b: 0xaf,
    },
    // 86: Aquamarine1
    Color {
        r: 0x5f,
        g: 0xff,
        b: 0xd7,
    },
    // 87: DarkSlateGray2
    Color {
        r: 0x5f,
        g: 0xff,
        b: 0xff,
    },
    // 88: DarkRed
    Color {
        r: 0x87,
        g: 0x00,
        b: 0x00,
    },
    // 89: DeepPink4
    Color {
        r: 0x87,
        g: 0x00,
        b: 0x5f,
    },
    // 90: DarkMagenta
    Color {
        r: 0x87,
        g: 0x00,
        b: 0x87,
    },
    // 91: DarkMagenta
    Color {
        r: 0x87,
        g: 0x00,
        b: 0xaf,
    },
    // 92: DarkViolet
    Color {
        r: 0x87,
        g: 0x00,
        b: 0xd7,
    },
    // 93: Purple
    Color {
        r: 0x87,
        g: 0x00,
        b: 0xff,
    },
    // 94: Orange4
    Color {
        r: 0x87,
        g: 0x5f,
        b: 0x00,
    },
    // 95: LightPink4
    Color {
        r: 0x87,
        g: 0x5f,
        b: 0x5f,
    },
    // 96: Plum4
    Color {
        r: 0x87,
        g: 0x5f,
        b: 0x87,
    },
    // 97: MediumPurple3
    Color {
        r: 0x87,
        g: 0x5f,
        b: 0xaf,
    },
    // 98: MediumPurple3
    Color {
        r: 0x87,
        g: 0x5f,
        b: 0xd7,
    },
    // 99: SlateBlue1
    Color {
        r: 0x87,
        g: 0x5f,
        b: 0xff,
    },
    // 100: Yellow4
    Color {
        r: 0x87,
        g: 0x87,
        b: 0x00,
    },
    // 101: Wheat4
    Color {
        r: 0x87,
        g: 0x87,
        b: 0x5f,
    },
    // 102: Grey53
    Color {
        r: 0x87,
        g: 0x87,
        b: 0x87,
    },
    // 103: LightSlateGrey
    Color {
        r: 0x87,
        g: 0x87,
        b: 0xaf,
    },
    // 104: MediumPurple
    Color {
        r: 0x87,
        g: 0x87,
        b: 0xd7,
    },
    // 105: LightSlateBlue
    Color {
        r: 0x87,
        g: 0x87,
        b: 0xff,
    },
    // 106: Yellow4
    Color {
        r: 0x87,
        g: 0xaf,
        b: 0x00,
    },
    // 107: DarkOliveGreen3
    Color {
        r: 0x87,
        g: 0xaf,
        b: 0x5f,
    },
    // 108: DarkSeaGreen
    Color {
        r: 0x87,
        g: 0xaf,
        b: 0x87,
    },
    // 109: LightSkyBlue3
    Color {
        r: 0x87,
        g: 0xaf,
        b: 0xaf,
    },
    // 110: LightSkyBlue3
    Color {
        r: 0x87,
        g: 0xaf,
        b: 0xd7,
    },
    // 111: SkyBlue2
    Color {
        r: 0x87,
        g: 0xaf,
        b: 0xff,
    },
    // 112: Chartreuse2
    Color {
        r: 0x87,
        g: 0xd7,
        b: 0x00,
    },
    // 113: DarkOliveGreen3
    Color {
        r: 0x87,
        g: 0xd7,
        b: 0x5f,
    },
    // 114: PaleGreen3
    Color {
        r: 0x87,
        g: 0xd7,
        b: 0x87,
    },
    // 115: DarkSeaGreen3
    Color {
        r: 0x87,
        g: 0xd7,
        b: 0xaf,
    },
    // 116: DarkSlateGray3
    Color {
        r: 0x87,
        g: 0xd7,
        b: 0xd7,
    },
    // 117: SkyBlue1
    Color {
        r: 0x87,
        g: 0xd7,
        b: 0xff,
    },
    // 118: Chartreuse1
    Color {
        r: 0x87,
        g: 0xff,
        b: 0x00,
    },
    // 119: LightGreen
    Color {
        r: 0x87,
        g: 0xff,
        b: 0x5f,
    },
    // 120: LightGreen
    Color {
        r: 0x87,
        g: 0xff,
        b: 0x87,
    },
    // 121: PaleGreen1
    Color {
        r: 0x87,
        g: 0xff,
        b: 0xaf,
    },
    // 122: Aquamarine1
    Color {
        r: 0x87,
        g: 0xff,
        b: 0xd7,
    },
    // 123: DarkSlateGray1
    Color {
        r: 0x87,
        g: 0xff,
        b: 0xff,
    },
    // 124: Red3
    Color {
        r: 0xaf,
        g: 0x00,
        b: 0x00,
    },
    // 125: DeepPink4
    Color {
        r: 0xaf,
        g: 0x00,
        b: 0x5f,
    },
    // 126: MediumVioletRed
    Color {
        r: 0xaf,
        g: 0x00,
        b: 0x87,
    },
    // 127: Magenta3
    Color {
        r: 0xaf,
        g: 0x00,
        b: 0xaf,
    },
    // 128: DarkViolet
    Color {
        r: 0xaf,
        g: 0x00,
        b: 0xd7,
    },
    // 129: Purple
    Color {
        r: 0xaf,
        g: 0x00,
        b: 0xff,
    },
    // 130: DarkOrange3
    Color {
        r: 0xaf,
        g: 0x5f,
        b: 0x00,
    },
    // 131: IndianRed
    Color {
        r: 0xaf,
        g: 0x5f,
        b: 0x5f,
    },
    // 132: HotPink3
    Color {
        r: 0xaf,
        g: 0x5f,
        b: 0x87,
    },
    // 133: MediumOrchid3
    Color {
        r: 0xaf,
        g: 0x5f,
        b: 0xaf,
    },
    // 134: MediumOrchid
    Color {
        r: 0xaf,
        g: 0x5f,
        b: 0xd7,
    },
    // 135: MediumPurple2
    Color {
        r: 0xaf,
        g: 0x5f,
        b: 0xff,
    },
    // 136: DarkGoldenrod
    Color {
        r: 0xaf,
        g: 0x87,
        b: 0x00,
    },
    // 137: LightSalmon3
    Color {
        r: 0xaf,
        g: 0x87,
        b: 0x5f,
    },
    // 138: RosyBrown
    Color {
        r: 0xaf,
        g: 0x87,
        b: 0x87,
    },
    // 139: Grey63
    Color {
        r: 0xaf,
        g: 0x87,
        b: 0xaf,
    },
    // 140: MediumPurple2
    Color {
        r: 0xaf,
        g: 0x87,
        b: 0xd7,
    },
    // 141: MediumPurple1
    Color {
        r: 0xaf,
        g: 0x87,
        b: 0xff,
    },
    // 142: Gold3
    Color {
        r: 0xaf,
        g: 0xaf,
        b: 0x00,
    },
    // 143: DarkKhaki
    Color {
        r: 0xaf,
        g: 0xaf,
        b: 0x5f,
    },
    // 144: NavajoWhite3
    Color {
        r: 0xaf,
        g: 0xaf,
        b: 0x87,
    },
    // 145: Grey69
    Color {
        r: 0xaf,
        g: 0xaf,
        b: 0xaf,
    },
    // 146: LightSteelBlue3
    Color {
        r: 0xaf,
        g: 0xaf,
        b: 0xd7,
    },
    // 147: LightSteelBlue
    Color {
        r: 0xaf,
        g: 0xaf,
        b: 0xff,
    },
    // 148: Yellow3
    Color {
        r: 0xaf,
        g: 0xd7,
        b: 0x00,
    },
    // 149: DarkOliveGreen3
    Color {
        r: 0xaf,
        g: 0xd7,
        b: 0x5f,
    },
    // 150: DarkSeaGreen3
    Color {
        r: 0xaf,
        g: 0xd7,
        b: 0x87,
    },
    // 151: DarkSeaGreen2
    Color {
        r: 0xaf,
        g: 0xd7,
        b: 0xaf,
    },
    // 152: LightCyan3
    Color {
        r: 0xaf,
        g: 0xd7,
        b: 0xd7,
    },
    // 153: LightSkyBlue1
    Color {
        r: 0xaf,
        g: 0xd7,
        b: 0xff,
    },
    // 154: GreenYellow
    Color {
        r: 0xaf,
        g: 0xff,
        b: 0x00,
    },
    // 155: DarkOliveGreen2
    Color {
        r: 0xaf,
        g: 0xff,
        b: 0x5f,
    },
    // 156: PaleGreen1
    Color {
        r: 0xaf,
        g: 0xff,
        b: 0x87,
    },
    // 157: DarkSeaGreen2
    Color {
        r: 0xaf,
        g: 0xff,
        b: 0xaf,
    },
    // 158: DarkSeaGreen1
    Color {
        r: 0xaf,
        g: 0xff,
        b: 0xd7,
    },
    // 159: PaleTurquoise1
    Color {
        r: 0xaf,
        g: 0xff,
        b: 0xff,
    },
    // 160: Red3
    Color {
        r: 0xd7,
        g: 0x00,
        b: 0x00,
    },
    // 161: DeepPink3
    Color {
        r: 0xd7,
        g: 0x00,
        b: 0x5f,
    },
    // 162: DeepPink3
    Color {
        r: 0xd7,
        g: 0x00,
        b: 0x87,
    },
    // 163: Magenta3
    Color {
        r: 0xd7,
        g: 0x00,
        b: 0xaf,
    },
    // 164: Magenta3
    Color {
        r: 0xd7,
        g: 0x00,
        b: 0xd7,
    },
    // 165: Magenta2
    Color {
        r: 0xd7,
        g: 0x00,
        b: 0xff,
    },
    // 166: DarkOrange3
    Color {
        r: 0xd7,
        g: 0x5f,
        b: 0x00,
    },
    // 167: IndianRed
    Color {
        r: 0xd7,
        g: 0x5f,
        b: 0x5f,
    },
    // 168: HotPink3
    Color {
        r: 0xd7,
        g: 0x5f,
        b: 0x87,
    },
    // 169: HotPink2
    Color {
        r: 0xd7,
        g: 0x5f,
        b: 0xaf,
    },
    // 170: Orchid
    Color {
        r: 0xd7,
        g: 0x5f,
        b: 0xd7,
    },
    // 171: MediumOrchid1
    Color {
        r: 0xd7,
        g: 0x5f,
        b: 0xff,
    },
    // 172: Orange3
    Color {
        r: 0xd7,
        g: 0x87,
        b: 0x00,
    },
    // 173: LightSalmon3
    Color {
        r: 0xd7,
        g: 0x87,
        b: 0x5f,
    },
    // 174: LightPink3
    Color {
        r: 0xd7,
        g: 0x87,
        b: 0x87,
    },
    // 175: Pink3
    Color {
        r: 0xd7,
        g: 0x87,
        b: 0xaf,
    },
    // 176: Plum3
    Color {
        r: 0xd7,
        g: 0x87,
        b: 0xd7,
    },
    // 177: Violet
    Color {
        r: 0xd7,
        g: 0x87,
        b: 0xff,
    },
    // 178: Gold3
    Color {
        r: 0xd7,
        g: 0xaf,
        b: 0x00,
    },
    // 179: LightGoldenrod3
    Color {
        r: 0xd7,
        g: 0xaf,
        b: 0x5f,
    },
    // 180: Tan
    Color {
        r: 0xd7,
        g: 0xaf,
        b: 0x87,
    },
    // 181: MistyRose3
    Color {
        r: 0xd7,
        g: 0xaf,
        b: 0xaf,
    },
    // 182: Thistle3
    Color {
        r: 0xd7,
        g: 0xaf,
        b: 0xd7,
    },
    // 183: Plum2
    Color {
        r: 0xd7,
        g: 0xaf,
        b: 0xff,
    },
    // 184: Yellow3
    Color {
        r: 0xd7,
        g: 0xd7,
        b: 0x00,
    },
    // 185: Khaki3
    Color {
        r: 0xd7,
        g: 0xd7,
        b: 0x5f,
    },
    // 186: LightGoldenrod2
    Color {
        r: 0xd7,
        g: 0xd7,
        b: 0x87,
    },
    // 187: LightYellow3
    Color {
        r: 0xd7,
        g: 0xd7,
        b: 0xaf,
    },
    // 188: Grey84
    Color {
        r: 0xd7,
        g: 0xd7,
        b: 0xd7,
    },
    // 189: LightSteelBlue1
    Color {
        r: 0xd7,
        g: 0xd7,
        b: 0xff,
    },
    // 190: Yellow2
    Color {
        r: 0xd7,
        g: 0xff,
        b: 0x00,
    },
    // 191: DarkOliveGreen1
    Color {
        r: 0xd7,
        g: 0xff,
        b: 0x5f,
    },
    // 192: DarkOliveGreen1
    Color {
        r: 0xd7,
        g: 0xff,
        b: 0x87,
    },
    // 193: DarkSeaGreen1
    Color {
        r: 0xd7,
        g: 0xff,
        b: 0xaf,
    },
    // 194: Honeydew2
    Color {
        r: 0xd7,
        g: 0xff,
        b: 0xd7,
    },
    // 195: LightCyan1
    Color {
        r: 0xd7,
        g: 0xff,
        b: 0xff,
    },
    // 196: Red1
    Color {
        r: 0xff,
        g: 0x00,
        b: 0x00,
    },
    // 197: DeepPink2
    Color {
        r: 0xff,
        g: 0x00,
        b: 0x5f,
    },
    // 198: DeepPink1
    Color {
        r: 0xff,
        g: 0x00,
        b: 0x87,
    },
    // 199: DeepPink1
    Color {
        r: 0xff,
        g: 0x00,
        b: 0xaf,
    },
    // 200: Magenta2
    Color {
        r: 0xff,
        g: 0x00,
        b: 0xd7,
    },
    // 201: Magenta1
    Color {
        r: 0xff,
        g: 0x00,
        b: 0xff,
    },
    // 202: OrangeRed1
    Color {
        r: 0xff,
        g: 0x5f,
        b: 0x00,
    },
    // 203: IndianRed1
    Color {
        r: 0xff,
        g: 0x5f,
        b: 0x5f,
    },
    // 204: IndianRed1
    Color {
        r: 0xff,
        g: 0x5f,
        b: 0x87,
    },
    // 205: HotPink
    Color {
        r: 0xff,
        g: 0x5f,
        b: 0xaf,
    },
    // 206: HotPink
    Color {
        r: 0xff,
        g: 0x5f,
        b: 0xd7,
    },
    // 207: MediumOrchid1
    Color {
        r: 0xff,
        g: 0x5f,
        b: 0xff,
    },
    // 208: DarkOrange
    Color {
        r: 0xff,
        g: 0x87,
        b: 0x00,
    },
    // 209: Salmon1
    Color {
        r: 0xff,
        g: 0x87,
        b: 0x5f,
    },
    // 210: LightCoral
    Color {
        r: 0xff,
        g: 0x87,
        b: 0x87,
    },
    // 211: PaleVioletRed1
    Color {
        r: 0xff,
        g: 0x87,
        b: 0xaf,
    },
    // 212: Orchid2
    Color {
        r: 0xff,
        g: 0x87,
        b: 0xd7,
    },
    // 213: Orchid1
    Color {
        r: 0xff,
        g: 0x87,
        b: 0xff,
    },
    // 214: Orange1
    Color {
        r: 0xff,
        g: 0xaf,
        b: 0x00,
    },
    // 215: SandyBrown
    Color {
        r: 0xff,
        g: 0xaf,
        b: 0x5f,
    },
    // 216: LightSalmon1
    Color {
        r: 0xff,
        g: 0xaf,
        b: 0x87,
    },
    // 217: LightPink1
    Color {
        r: 0xff,
        g: 0xaf,
        b: 0xaf,
    },
    // 218: Pink1
    Color {
        r: 0xff,
        g: 0xaf,
        b: 0xd7,
    },
    // 219: Plum1
    Color {
        r: 0xff,
        g: 0xaf,
        b: 0xff,
    },
    // 220: Gold1
    Color {
        r: 0xff,
        g: 0xd7,
        b: 0x00,
    },
    // 221: LightGoldenrod2
    Color {
        r: 0xff,
        g: 0xd7,
        b: 0x5f,
    },
    // 222: LightGoldenrod2
    Color {
        r: 0xff,
        g: 0xd7,
        b: 0x87,
    },
    // 223: NavajoWhite1
    Color {
        r: 0xff,
        g: 0xd7,
        b: 0xaf,
    },
    // 224: MistyRose1
    Color {
        r: 0xff,
        g: 0xd7,
        b: 0xd7,
    },
    // 225: Thistle1
    Color {
        r: 0xff,
        g: 0xd7,
        b: 0xff,
    },
    // 226: Yellow1
    Color {
        r: 0xff,
        g: 0xff,
        b: 0x00,
    },
    // 227: LightGoldenrod1
    Color {
        r: 0xff,
        g: 0xff,
        b: 0x5f,
    },
    // 228: Khaki1
    Color {
        r: 0xff,
        g: 0xff,
        b: 0x87,
    },
    // 229: Wheat1
    Color {
        r: 0xff,
        g: 0xff,
        b: 0xaf,
    },
    // 230: Cornsilk1
    Color {
        r: 0xff,
        g: 0xff,
        b: 0xd7,
    },
    // 231: Grey100
    Color {
        r: 0xff,
        g: 0xff,
        b: 0xff,
    },
    // 232: Grey3
    Color {
        r: 0x08,
        g: 0x08,
        b: 0x08,
    },
    // 233: Grey7
    Color {
        r: 0x12,
        g: 0x12,
        b: 0x12,
    },
    // 234: Grey11
    Color {
        r: 0x1c,
        g: 0x1c,
        b: 0x1c,
    },
    // 235: Grey15
    Color {
        r: 0x26,
        g: 0x26,
        b: 0x26,
    },
    // 236: Grey19
    Color {
        r: 0x30,
        g: 0x30,
        b: 0x30,
    },
    // 237: Grey23
    Color {
        r: 0x3a,
        g: 0x3a,
        b: 0x3a,
    },
    // 238: Grey27
    Color {
        r: 0x44,
        g: 0x44,
        b: 0x44,
    },
    // 239: Grey30
    Color {
        r: 0x4e,
        g: 0x4e,
        b: 0x4e,
    },
    // 240: Grey35
    Color {
        r: 0x58,
        g: 0x58,
        b: 0x58,
    },
    // 241: Grey39
    Color {
        r: 0x62,
        g: 0x62,
        b: 0x62,
    },
    // 242: Grey42
    Color {
        r: 0x6c,
        g: 0x6c,
        b: 0x6c,
    },
    // 243: Grey46
    Color {
        r: 0x76,
        g: 0x76,
        b: 0x76,
    },
    // 244: Grey50
    Color {
        r: 0x80,
        g: 0x80,
        b: 0x80,
    },
    // 245: Grey54
    Color {
        r: 0x8a,
        g: 0x8a,
        b: 0x8a,
    },
    // 246: Grey58
    Color {
        r: 0x94,
        g: 0x94,
        b: 0x94,
    },
    // 247: Grey62
    Color {
        r: 0x9e,
        g: 0x9e,
        b: 0x9e,
    },
    // 248: Grey66
    Color {
        r: 0xa8,
        g: 0xa8,
        b: 0xa8,
    },
    // 249: Grey70
    Color {
        r: 0xb2,
        g: 0xb2,
        b: 0xb2,
    },
    // 250: Grey74
    Color {
        r: 0xbc,
        g: 0xbc,
        b: 0xbc,
    },
    // 251: Grey78
    Color {
        r: 0xc6,
        g: 0xc6,
        b: 0xc6,
    },
    // 252: Grey82
    Color {
        r: 0xd0,
        g: 0xd0,
        b: 0xd0,
    },
    // 253: Grey85
    Color {
        r: 0xda,
        g: 0xda,
        b: 0xda,
    },
    // 254: Grey89
    Color {
        r: 0xe4,
        g: 0xe4,
        b: 0xe4,
    },
    // 255: Grey93
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

    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
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
        let mut res = vec![0; 3 * self.colors.len()];
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
