#![allow(clippy::many_single_char_names)]
use std::{fmt::Display, path::Path};

use regex::Regex;

#[derive(Debug, Clone, Default)]
pub struct Color {
    pub name: Option<String>,
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
        Color {
            name: None,
            r,
            g,
            b,
        }
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
            name: None,
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
            name: None,
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
            name: None,
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
            name: None,
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

pub enum PaletteFormat {
    Hex,
    Pal,
    Gpl,
    Txt,
    Ase,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Palette {
    pub title: String,
    pub description: String,
    pub author: String,
    colors: Vec<Color>,
}

impl Palette {
    pub fn from_colors(colors: Vec<Color>) -> Self {
        Self {
            title: String::new(),
            description: String::new(),
            author: String::new(),
            colors,
        }
    }

    pub fn get_color(&self, index: usize) -> Color {
        if index >= self.colors.len() {
            return Color::new(0, 0, 0);
        }
        self.colors[index].clone()
    }

    pub fn are_colors_equal(&self, other: &Palette) -> bool {
        self.colors == other.colors        
    }

    pub fn get_rgb(&self, index: usize) -> (u8, u8, u8) {
        if index >= self.colors.len() {
            (0, 0, 0)
        } else {
            let c = &self.colors[index];
            (c.r, c.g, c.b)
        }
    }

    pub fn color_iter(&self) -> impl Iterator<Item = &Color> {
        self.colors.iter()
    }

    pub fn push(&mut self, b: Color) {
        self.colors.push(b);
    }

    pub fn set_color(&mut self, index: usize, color: Color) {
        self.colors[index] = color;
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
    pub fn load_palette(format: &PaletteFormat, bytes: &[u8]) -> anyhow::Result<Self> {
        let mut colors = Vec::new();
        let mut title = String::new();
        let author = String::new();
        let mut description = String::new();
        match format {
            PaletteFormat::Hex => match String::from_utf8(bytes.to_vec()) {
                Ok(data) => {
                    let re = Regex::new(r"([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})")?;
                    for (_, [r, g, b]) in re.captures_iter(&data).map(|c| c.extract()) {
                        let r = u32::from_str_radix(r, 16)?;
                        let g = u32::from_str_radix(g, 16)?;
                        let b = u32::from_str_radix(b, 16)?;
                        colors.push(Color::new(r as u8, g as u8, b as u8));
                    }
                }
                Err(err) => return Err(anyhow::anyhow!("Invalid input: {err}")),
            },
            PaletteFormat::Pal => {
                match String::from_utf8(bytes.to_vec()) {
                    Ok(data) => {
                        let re = Regex::new(r"(\d+)\s+(\d+)\s+(\d+)")?;

                        for (i, line) in data.lines().enumerate() {
                            match i {
                                0 => {
                                    if line != "JASC-PAL" {
                                        return Err(anyhow::anyhow!(
                                            "Only JASC-PAL supported: {line}"
                                        ));
                                    }
                                }
                                1 | 2 => {
                                    // Ignore
                                }
                                _ => {
                                    for (_, [r, g, b]) in
                                        re.captures_iter(line).map(|c| c.extract())
                                    {
                                        let r = r.parse::<u32>()?;
                                        let g = g.parse::<u32>()?;
                                        let b = b.parse::<u32>()?;
                                        colors.push(Color::new(r as u8, g as u8, b as u8));
                                    }
                                }
                            }
                        }
                    }
                    Err(err) => return Err(anyhow::anyhow!("Invalid input: {err}")),
                }
            }
            PaletteFormat::Gpl => match String::from_utf8(bytes.to_vec()) {
                Ok(data) => {
                    let color_regex = Regex::new(r"(\d+)\s+(\d+)\s+(\d+)\s+\S+")?;
                    let name_regex = Regex::new(r"\s*#Palette Name:\s*(.*)\s*")?;
                    let description_regex = Regex::new(r"\s*#Description:\s*(.*)\s*")?;
                    for (i, line) in data.lines().enumerate() {
                        match i {
                            0 => {
                                if line != "GIMP Palette" {
                                    return Err(anyhow::anyhow!(
                                        "Only GIMP Palette supported: {line}"
                                    ));
                                }
                            }
                            _ => {
                                if line.starts_with('#') {
                                    if let Some(cap) = name_regex.captures(line) {
                                        if let Some(name) = cap.get(1) {
                                            title = name.as_str().to_string();
                                        }
                                    }
                                    if let Some(cap) = description_regex.captures(line) {
                                        if let Some(name) = cap.get(1) {
                                            description = name.as_str().to_string();
                                        }
                                    }
                                } else if let Some(cap) = color_regex.captures(line) {
                                    let (_, [r, g, b]) = cap.extract();

                                    let r = r.parse::<u32>()?;
                                    let g = g.parse::<u32>()?;
                                    let b = b.parse::<u32>()?;
                                    colors.push(Color::new(r as u8, g as u8, b as u8));
                                }
                            }
                        }
                    }
                }
                Err(err) => return Err(anyhow::anyhow!("Invalid input: {err}")),
            },
            PaletteFormat::Txt => match String::from_utf8(bytes.to_vec()) {
                Ok(data) => {
                    let color_regex = Regex::new(
                        r"([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})",
                    )?;
                    let name_regex = Regex::new(r"\s*;Palette Name:\s*(.*)\s*")?;
                    let description_regex = Regex::new(r"\s*;Description:\s*(.*)\s*")?;
                    for line in data.lines() {
                        if line.starts_with(';') {
                            if let Some(cap) = name_regex.captures(line) {
                                if let Some(name) = cap.get(1) {
                                    title = name.as_str().to_string();
                                }
                            }
                            if let Some(cap) = description_regex.captures(line) {
                                if let Some(name) = cap.get(1) {
                                    description = name.as_str().to_string();
                                }
                            }
                        } else if let Some(cap) = color_regex.captures(line) {
                            let (_, [_a, r, g, b]) = cap.extract();

                            let r = u32::from_str_radix(r, 16).unwrap();
                            let g = u32::from_str_radix(g, 16).unwrap();
                            let b = u32::from_str_radix(b, 16).unwrap();
                            colors.push(Color::new(r as u8, g as u8, b as u8));
                        }
                    }
                }
                Err(err) => return Err(anyhow::anyhow!("Invalid input: {err}")),
            },
            PaletteFormat::Ase => todo!(),
        }
        Ok(Self {
            title,
            description,
            author,
            colors,
        })
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn import_palette(file_name: &Path, bytes: &[u8]) -> anyhow::Result<Self> {
        let Some(ext) = file_name.extension() else {
            return Err(anyhow::anyhow!("No file extension"));
        };
        let ascii_lowercase = &ext.to_ascii_lowercase();
        let Some(ext) = ascii_lowercase.to_str() else {
            return Err(anyhow::anyhow!("Error in string conversion."));
        };

        match ext {
            "pal" => Palette::load_palette(&PaletteFormat::Pal, bytes),
            "gpl" => Palette::load_palette(&PaletteFormat::Gpl, bytes),
            "txt" => Palette::load_palette(&PaletteFormat::Txt, bytes),
            "hex" => Palette::load_palette(&PaletteFormat::Hex, bytes),
            _ => Err(anyhow::anyhow!("Unsupported file extension: {ext}")),
        }
    }

    pub fn export_palette(&self, format: &PaletteFormat) -> Vec<u8> {
        match format {
            PaletteFormat::Hex => {
                let mut res = String::new();
                for c in &self.colors {
                    res.push_str(format!("{:02x}{:02x}{:02x}\n", c.r, c.g, c.b).as_str());
                }
                return res.as_bytes().to_vec();
            }
            PaletteFormat::Pal => {
                let mut res = String::new();
                res.push_str("JASC-PAL\n");
                res.push_str("0100\n");
                res.push_str(format!("{}\n", self.colors.len()).as_str());

                for c in &self.colors {
                    res.push_str(format!("{} {} {}\n", c.r, c.g, c.b).as_str());
                }

                return res.as_bytes().to_vec();
            }
            PaletteFormat::Gpl => {
                let mut res = String::new();
                res.push_str("GIMP Palette\n");

                res.push_str(format!("#Palette Name: {}\n", self.title).as_str());
                res.push_str(format!("#Author: {}\n", self.author).as_str());
                res.push_str(format!("#Description: {}\n", self.description).as_str());
                res.push_str(format!("#Colors: {}\n", self.colors.len()).as_str());

                for c in &self.colors {
                    res.push_str(
                        format!(
                            "{:3} {:3} {:3} {:02x}{:02x}{:02x}\n",
                            c.r, c.g, c.b, c.r, c.g, c.b
                        )
                        .as_str(),
                    );
                }

                return res.as_bytes().to_vec();
            }
            PaletteFormat::Txt => {
                let mut res = String::new();
                res.push_str(";paint.net Palette File\n");

                res.push_str(format!(";Palette Name: {}\n", self.title).as_str());
                res.push_str(format!(";Author: {}\n", self.author).as_str());
                res.push_str(format!(";Description: {}\n", self.description).as_str());
                res.push_str(format!(";Colors: {}\n", self.colors.len()).as_str());

                for c in &self.colors {
                    res.push_str(format!("FF{:02x}{:02x}{:02x}\n", c.r, c.g, c.b,).as_str());
                }

                return res.as_bytes().to_vec();
            }
            PaletteFormat::Ase => todo!(),
        }
    }

    pub fn new() -> Self {
        Palette {
            title: String::new(),
            description: String::new(),
            author: String::new(),
            colors: DOS_DEFAULT_PALETTE.to_vec(),
        }
    }

    pub fn len(&self) -> usize {
        self.colors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }

    pub fn clear(&mut self) {
        self.colors.clear();
    }

    pub fn fill_to_16(&mut self) {
        if self.colors.len() < DOS_DEFAULT_PALETTE.len() {
            (self.colors.len()..DOS_DEFAULT_PALETTE.len()).for_each(|i| {
                self.colors.push(DOS_DEFAULT_PALETTE[i].clone());
            });
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
        self.colors[number] = Color {
            name: None,
            r,
            g,
            b,
        };
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

        self.colors[number] = Color {
            name: None,
            r,
            g,
            b,
        };
    }

    pub fn insert_color(&mut self, color: Color) -> u32 {
        for i in 0..self.colors.len() {
            let col = self.colors[i].clone();
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
                name: None,
                r: pal[o] << 2 | pal[o] >> 4,
                g: pal[o + 1] << 2 | pal[o + 1] >> 4,
                b: pal[o + 2] << 2 | pal[o + 2] >> 4,
            });
            o += 3;
        }

        Palette {
            title: String::new(),
            description: String::new(),
            author: String::new(),
            colors,
        }
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
        Palette {
            title: String::new(),
            description: String::new(),
            author: String::new(),
            colors,
        }
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
                ega_colors[EGA_COLOR_OFFSETS[i]] = self.colors[i].clone();
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
                self.colors[i].clone()
            } else {
                DOS_DEFAULT_PALETTE[i].clone()
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

static EGA_COLOR_OFFSETS: [usize; 16] = [0, 1, 2, 3, 4, 5, 20, 7, 56, 57, 58, 59, 60, 61, 62, 63];

pub const DOS_DEFAULT_PALETTE: [Color; 16] = [
    Color {
        name: None,
        r: 0x00,
        g: 0x00,
        b: 0x00,
    }, // black
    Color {
        name: None,
        r: 0x00,
        g: 0x00,
        b: 0xAA,
    }, // blue
    Color {
        name: None,
        r: 0x00,
        g: 0xAA,
        b: 0x00,
    }, // green
    Color {
        name: None,
        r: 0x00,
        g: 0xAA,
        b: 0xAA,
    }, // cyan
    Color {
        name: None,
        r: 0xAA,
        g: 0x00,
        b: 0x00,
    }, // red
    Color {
        name: None,
        r: 0xAA,
        g: 0x00,
        b: 0xAA,
    }, // magenta
    Color {
        name: None,
        r: 0xAA,
        g: 0x55,
        b: 0x00,
    }, // brown
    Color {
        name: None,
        r: 0xAA,
        g: 0xAA,
        b: 0xAA,
    }, // lightgray
    Color {
        name: None,
        r: 0x55,
        g: 0x55,
        b: 0x55,
    }, // darkgray
    Color {
        name: None,
        r: 0x55,
        g: 0x55,
        b: 0xFF,
    }, // lightblue
    Color {
        name: None,
        r: 0x55,
        g: 0xFF,
        b: 0x55,
    }, // lightgreen
    Color {
        name: None,
        r: 0x55,
        g: 0xFF,
        b: 0xFF,
    }, // lightcyan
    Color {
        name: None,
        r: 0xFF,
        g: 0x55,
        b: 0x55,
    }, // lightred
    Color {
        name: None,
        r: 0xFF,
        g: 0x55,
        b: 0xFF,
    }, // lightmagenta
    Color {
        name: None,
        r: 0xFF,
        g: 0xFF,
        b: 0x55,
    }, // yellow
    Color {
        name: None,
        r: 0xFF,
        g: 0xFF,
        b: 0xFF,
    }, // white
];

// colors taken from "C64 Community Colors V1.2a" palette, see
// https://p1x3l.net/36/c64-community-colors-theor/
pub const C64_DEFAULT_PALETTE: [Color; 16] = [
    Color {
        name: None,
        r: 0x00,
        g: 0x00,
        b: 0x00,
    }, // black
    Color {
        name: None,
        r: 0xFF,
        g: 0xFF,
        b: 0xFF,
    }, // white
    Color {
        name: None,
        r: 0xAF,
        g: 0x2A,
        b: 0x29,
    }, // red
    Color {
        name: None,
        r: 0x62,
        g: 0xD8,
        b: 0xCC,
    }, // cyan
    Color {
        name: None,
        r: 0xB0,
        g: 0x3F,
        b: 0xB6,
    }, // violett
    Color {
        name: None,
        r: 0x4A,
        g: 0xC6,
        b: 0x4A,
    }, // green
    Color {
        name: None,
        r: 0x37,
        g: 0x39,
        b: 0xC4,
    }, // blue
    Color {
        name: None,
        r: 0xE4,
        g: 0xED,
        b: 0x4E,
    }, // yellow
    Color {
        name: None,
        r: 0xB6,
        g: 0x59,
        b: 0x1C,
    }, // orange
    Color {
        name: None,
        r: 0x68,
        g: 0x38,
        b: 0x08,
    }, // brown
    Color {
        name: None,
        r: 0xEA,
        g: 0x74,
        b: 0x6C,
    }, // lightred
    Color {
        name: None,
        r: 0x4D,
        g: 0x4D,
        b: 0x4D,
    }, // gray1
    Color {
        name: None,
        r: 0x84,
        g: 0x84,
        b: 0x84,
    }, // gray2
    Color {
        name: None,
        r: 0xA6,
        g: 0xFA,
        b: 0x9E,
    }, // lightgreen
    Color {
        name: None,
        r: 0x70,
        g: 0x7C,
        b: 0xE6,
    }, // lightblue
    Color {
        name: None,
        r: 0xB6,
        g: 0xB6,
        b: 0xB5,
    }, // gray3
];

pub const ATARI_DEFAULT_PALETTE: [Color; 16] = [
    Color {
        name: None,
        r: 0x09,
        g: 0x51,
        b: 0x83,
    }, // background
    Color {
        name: None,
        r: 0x00,
        g: 0x00,
        b: 0xAA,
    }, // unused
    Color {
        name: None,
        r: 0x00,
        g: 0xAA,
        b: 0x00,
    }, // unused
    Color {
        name: None,
        r: 0x00,
        g: 0xAA,
        b: 0xAA,
    }, // unused
    Color {
        name: None,
        r: 0xAA,
        g: 0x00,
        b: 0x00,
    }, // unused
    Color {
        name: None,
        r: 0xAA,
        g: 0x00,
        b: 0xAA,
    }, // unused
    Color {
        name: None,
        r: 0xAA,
        g: 0x55,
        b: 0x00,
    }, // unused
    Color {
        name: None,
        r: 0x65,
        g: 0xB7,
        b: 0xE9,
    }, // foreground
    Color {
        name: None,
        r: 0x55,
        g: 0x55,
        b: 0x55,
    }, // unused
    Color {
        name: None,
        r: 0x55,
        g: 0x55,
        b: 0xFF,
    }, // unused
    Color {
        name: None,
        r: 0x55,
        g: 0xFF,
        b: 0x55,
    }, // unused
    Color {
        name: None,
        r: 0x55,
        g: 0xFF,
        b: 0xFF,
    }, // unused
    Color {
        name: None,
        r: 0xFF,
        g: 0x55,
        b: 0x55,
    }, // unused
    Color {
        name: None,
        r: 0xFF,
        g: 0x55,
        b: 0xFF,
    }, // unused
    Color {
        name: None,
        r: 0xFF,
        g: 0xFF,
        b: 0x55,
    }, // unused
    Color {
        name: None,
        r: 0xFF,
        g: 0xFF,
        b: 0xFF,
    }, // unused
];

pub const EGA_PALETTE: [Color; 64] = [
    Color {
        name: None,
        r: 0x00,
        g: 0x00,
        b: 0x00,
    },
    Color {
        name: None,
        r: 0x00,
        g: 0x00,
        b: 0xAA,
    },
    Color {
        name: None,
        r: 0x00,
        g: 0xAA,
        b: 0x00,
    },
    Color {
        name: None,
        r: 0x00,
        g: 0xAA,
        b: 0xAA,
    },
    Color {
        name: None,
        r: 0xAA,
        g: 0x00,
        b: 0x00,
    },
    Color {
        name: None,
        r: 0xAA,
        g: 0x00,
        b: 0xAA,
    },
    Color {
        name: None,
        r: 0xAA,
        g: 0xAA,
        b: 0x00,
    },
    Color {
        name: None,
        r: 0xAA,
        g: 0xAA,
        b: 0xAA,
    },
    Color {
        name: None,
        r: 0x00,
        g: 0x00,
        b: 0x55,
    },
    Color {
        name: None,
        r: 0x00,
        g: 0x00,
        b: 0xFF,
    },
    Color {
        name: None,
        r: 0x00,
        g: 0xAA,
        b: 0x55,
    },
    Color {
        name: None,
        r: 0x00,
        g: 0xAA,
        b: 0xFF,
    },
    Color {
        name: None,
        r: 0xAA,
        g: 0x00,
        b: 0x55,
    },
    Color {
        name: None,
        r: 0xAA,
        g: 0x00,
        b: 0xFF,
    },
    Color {
        name: None,
        r: 0xAA,
        g: 0xAA,
        b: 0x55,
    },
    Color {
        name: None,
        r: 0xAA,
        g: 0xAA,
        b: 0xFF,
    },
    Color {
        name: None,
        r: 0x00,
        g: 0x55,
        b: 0x00,
    },
    Color {
        name: None,
        r: 0x00,
        g: 0x55,
        b: 0xAA,
    },
    Color {
        name: None,
        r: 0x00,
        g: 0xFF,
        b: 0x00,
    },
    Color {
        name: None,
        r: 0x00,
        g: 0xFF,
        b: 0xAA,
    },
    Color {
        name: None,
        r: 0xAA,
        g: 0x55,
        b: 0x00,
    },
    Color {
        name: None,
        r: 0xAA,
        g: 0x55,
        b: 0xAA,
    },
    Color {
        name: None,
        r: 0xAA,
        g: 0xFF,
        b: 0x00,
    },
    Color {
        name: None,
        r: 0xAA,
        g: 0xFF,
        b: 0xAA,
    },
    Color {
        name: None,
        r: 0x00,
        g: 0x55,
        b: 0x55,
    },
    Color {
        name: None,
        r: 0x00,
        g: 0x55,
        b: 0xFF,
    },
    Color {
        name: None,
        r: 0x00,
        g: 0xFF,
        b: 0x55,
    },
    Color {
        name: None,
        r: 0x00,
        g: 0xFF,
        b: 0xFF,
    },
    Color {
        name: None,
        r: 0xAA,
        g: 0x55,
        b: 0x55,
    },
    Color {
        name: None,
        r: 0xAA,
        g: 0x55,
        b: 0xFF,
    },
    Color {
        name: None,
        r: 0xAA,
        g: 0xFF,
        b: 0x55,
    },
    Color {
        name: None,
        r: 0xAA,
        g: 0xFF,
        b: 0xFF,
    },
    Color {
        name: None,
        r: 0x55,
        g: 0x00,
        b: 0x00,
    },
    Color {
        name: None,
        r: 0x55,
        g: 0x00,
        b: 0xAA,
    },
    Color {
        name: None,
        r: 0x55,
        g: 0xAA,
        b: 0x00,
    },
    Color {
        name: None,
        r: 0x55,
        g: 0xAA,
        b: 0xAA,
    },
    Color {
        name: None,
        r: 0xFF,
        g: 0x00,
        b: 0x00,
    },
    Color {
        name: None,
        r: 0xFF,
        g: 0x00,
        b: 0xAA,
    },
    Color {
        name: None,
        r: 0xFF,
        g: 0xAA,
        b: 0x00,
    },
    Color {
        name: None,
        r: 0xFF,
        g: 0xAA,
        b: 0xAA,
    },
    Color {
        name: None,
        r: 0x55,
        g: 0x00,
        b: 0x55,
    },
    Color {
        name: None,
        r: 0x55,
        g: 0x00,
        b: 0xFF,
    },
    Color {
        name: None,
        r: 0x55,
        g: 0xAA,
        b: 0x55,
    },
    Color {
        name: None,
        r: 0x55,
        g: 0xAA,
        b: 0xFF,
    },
    Color {
        name: None,
        r: 0xFF,
        g: 0x00,
        b: 0x55,
    },
    Color {
        name: None,
        r: 0xFF,
        g: 0x00,
        b: 0xFF,
    },
    Color {
        name: None,
        r: 0xFF,
        g: 0xAA,
        b: 0x55,
    },
    Color {
        name: None,
        r: 0xFF,
        g: 0xAA,
        b: 0xFF,
    },
    Color {
        name: None,
        r: 0x55,
        g: 0x55,
        b: 0x00,
    },
    Color {
        name: None,
        r: 0x55,
        g: 0x55,
        b: 0xAA,
    },
    Color {
        name: None,
        r: 0x55,
        g: 0xFF,
        b: 0x00,
    },
    Color {
        name: None,
        r: 0x55,
        g: 0xFF,
        b: 0xAA,
    },
    Color {
        name: None,
        r: 0xFF,
        g: 0x55,
        b: 0x00,
    },
    Color {
        name: None,
        r: 0xFF,
        g: 0x55,
        b: 0xAA,
    },
    Color {
        name: None,
        r: 0xFF,
        g: 0xFF,
        b: 0x00,
    },
    Color {
        name: None,
        r: 0xFF,
        g: 0xFF,
        b: 0xAA,
    },
    Color {
        name: None,
        r: 0x55,
        g: 0x55,
        b: 0x55,
    },
    Color {
        name: None,
        r: 0x55,
        g: 0x55,
        b: 0xFF,
    },
    Color {
        name: None,
        r: 0x55,
        g: 0xFF,
        b: 0x55,
    },
    Color {
        name: None,
        r: 0x55,
        g: 0xFF,
        b: 0xFF,
    },
    Color {
        name: None,
        r: 0xFF,
        g: 0x55,
        b: 0x55,
    },
    Color {
        name: None,
        r: 0xFF,
        g: 0x55,
        b: 0xFF,
    },
    Color {
        name: None,
        r: 0xFF,
        g: 0xFF,
        b: 0x55,
    },
    Color {
        name: None,
        r: 0xFF,
        g: 0xFF,
        b: 0xFF,
    },
];

pub const XTERM_256_PALETTE: [(&str, Color); 256] = [
    (
        "Black (SYSTEM)",
        Color {
            name: None,
            r: 0x00,
            g: 0x00,
            b: 0x00,
        },
    ),
    (
        "Maroon (SYSTEM)",
        Color {
            name: None,
            r: 0x80,
            g: 0x00,
            b: 0x00,
        },
    ),
    (
        "Green (SYSTEM)",
        Color {
            name: None,
            r: 0x00,
            g: 0x80,
            b: 0x00,
        },
    ),
    (
        "Olive (SYSTEM)",
        Color {
            name: None,
            r: 0x80,
            g: 0x80,
            b: 0x00,
        },
    ),
    (
        "Navy (SYSTEM)",
        Color {
            name: None,
            r: 0x00,
            g: 0x00,
            b: 0x80,
        },
    ),
    (
        "Purple (SYSTEM)",
        Color {
            name: None,
            r: 0x80,
            g: 0x00,
            b: 0x80,
        },
    ),
    (
        "Teal (SYSTEM)",
        Color {
            name: None,
            r: 0x00,
            g: 0x80,
            b: 0x80,
        },
    ),
    (
        "Silver (SYSTEM)",
        Color {
            name: None,
            r: 0xc0,
            g: 0xc0,
            b: 0xc0,
        },
    ),
    (
        "Grey (SYSTEM)",
        Color {
            name: None,
            r: 0x80,
            g: 0x80,
            b: 0x80,
        },
    ),
    (
        "Red (SYSTEM)",
        Color {
            name: None,
            r: 0xff,
            g: 0x00,
            b: 0x00,
        },
    ),
    (
        "Lime (SYSTEM)",
        Color {
            name: None,
            r: 0x00,
            g: 0xff,
            b: 0x00,
        },
    ),
    (
        "Yellow (SYSTEM)",
        Color {
            name: None,
            r: 0xff,
            g: 0xff,
            b: 0x00,
        },
    ),
    (
        "Blue (SYSTEM)",
        Color {
            name: None,
            r: 0x00,
            g: 0x00,
            b: 0xff,
        },
    ),
    (
        "Fuchsia (SYSTEM)",
        Color {
            name: None,
            r: 0xff,
            g: 0x00,
            b: 0xff,
        },
    ),
    (
        "Aqua (SYSTEM)",
        Color {
            name: None,
            r: 0x00,
            g: 0xff,
            b: 0xff,
        },
    ),
    (
        "White (SYSTEM)",
        Color {
            name: None,
            r: 0xff,
            g: 0xff,
            b: 0xff,
        },
    ),
    (
        "Grey0",
        Color {
            name: None,
            r: 0x00,
            g: 0x00,
            b: 0x00,
        },
    ),
    (
        "NavyBlue",
        Color {
            name: None,
            r: 0x00,
            g: 0x00,
            b: 0x5f,
        },
    ),
    (
        "DarkBlue",
        Color {
            name: None,
            r: 0x00,
            g: 0x00,
            b: 0x87,
        },
    ),
    (
        "Blue3",
        Color {
            name: None,
            r: 0x00,
            g: 0x00,
            b: 0xaf,
        },
    ),
    (
        "Blue3",
        Color {
            name: None,
            r: 0x00,
            g: 0x00,
            b: 0xd7,
        },
    ),
    (
        "Blue1",
        Color {
            name: None,
            r: 0x00,
            g: 0x00,
            b: 0xff,
        },
    ),
    (
        "DarkGreen",
        Color {
            name: None,
            r: 0x00,
            g: 0x5f,
            b: 0x00,
        },
    ),
    (
        "DeepSkyBlue4",
        Color {
            name: None,
            r: 0x00,
            g: 0x5f,
            b: 0x5f,
        },
    ),
    (
        "DeepSkyBlue4",
        Color {
            name: None,
            r: 0x00,
            g: 0x5f,
            b: 0x87,
        },
    ),
    (
        "DeepSkyBlue4",
        Color {
            name: None,
            r: 0x00,
            g: 0x5f,
            b: 0xaf,
        },
    ),
    (
        "DodgerBlue3",
        Color {
            name: None,
            r: 0x00,
            g: 0x5f,
            b: 0xd7,
        },
    ),
    (
        "DodgerBlue2",
        Color {
            name: None,
            r: 0x00,
            g: 0x5f,
            b: 0xff,
        },
    ),
    (
        "Green4",
        Color {
            name: None,
            r: 0x00,
            g: 0x87,
            b: 0x00,
        },
    ),
    (
        "SpringGreen4",
        Color {
            name: None,
            r: 0x00,
            g: 0x87,
            b: 0x5f,
        },
    ),
    (
        "Turquoise4",
        Color {
            name: None,
            r: 0x00,
            g: 0x87,
            b: 0x87,
        },
    ),
    (
        "DeepSkyBlue3",
        Color {
            name: None,
            r: 0x00,
            g: 0x87,
            b: 0xaf,
        },
    ),
    (
        "DeepSkyBlue3",
        Color {
            name: None,
            r: 0x00,
            g: 0x87,
            b: 0xd7,
        },
    ),
    (
        "DodgerBlue1",
        Color {
            name: None,
            r: 0x00,
            g: 0x87,
            b: 0xff,
        },
    ),
    (
        "Green3",
        Color {
            name: None,
            r: 0x00,
            g: 0xaf,
            b: 0x00,
        },
    ),
    (
        "SpringGreen3",
        Color {
            name: None,
            r: 0x00,
            g: 0xaf,
            b: 0x5f,
        },
    ),
    (
        "DarkCyan",
        Color {
            name: None,
            r: 0x00,
            g: 0xaf,
            b: 0x87,
        },
    ),
    (
        "LightSeaGreen",
        Color {
            name: None,
            r: 0x00,
            g: 0xaf,
            b: 0xaf,
        },
    ),
    (
        "DeepSkyBlue2",
        Color {
            name: None,
            r: 0x00,
            g: 0xaf,
            b: 0xd7,
        },
    ),
    (
        "DeepSkyBlue1",
        Color {
            name: None,
            r: 0x00,
            g: 0xaf,
            b: 0xff,
        },
    ),
    (
        "Green3",
        Color {
            name: None,
            r: 0x00,
            g: 0xd7,
            b: 0x00,
        },
    ),
    (
        "SpringGreen3",
        Color {
            name: None,
            r: 0x00,
            g: 0xd7,
            b: 0x5f,
        },
    ),
    (
        "SpringGreen2",
        Color {
            name: None,
            r: 0x00,
            g: 0xd7,
            b: 0x87,
        },
    ),
    (
        "Cyan3",
        Color {
            name: None,
            r: 0x00,
            g: 0xd7,
            b: 0xaf,
        },
    ),
    (
        "DarkTurquoise",
        Color {
            name: None,
            r: 0x00,
            g: 0xd7,
            b: 0xd7,
        },
    ),
    (
        "Turquoise2",
        Color {
            name: None,
            r: 0x00,
            g: 0xd7,
            b: 0xff,
        },
    ),
    (
        "Green1",
        Color {
            name: None,
            r: 0x00,
            g: 0xff,
            b: 0x00,
        },
    ),
    (
        "SpringGreen2",
        Color {
            name: None,
            r: 0x00,
            g: 0xff,
            b: 0x5f,
        },
    ),
    (
        "SpringGreen1",
        Color {
            name: None,
            r: 0x00,
            g: 0xff,
            b: 0x87,
        },
    ),
    (
        "MediumSpringGreen",
        Color {
            name: None,
            r: 0x00,
            g: 0xff,
            b: 0xaf,
        },
    ),
    (
        "Cyan2",
        Color {
            name: None,
            r: 0x00,
            g: 0xff,
            b: 0xd7,
        },
    ),
    (
        "Cyan1",
        Color {
            name: None,
            r: 0x00,
            g: 0xff,
            b: 0xff,
        },
    ),
    (
        "DarkRed",
        Color {
            name: None,
            r: 0x5f,
            g: 0x00,
            b: 0x00,
        },
    ),
    (
        "DeepPink4",
        Color {
            name: None,
            r: 0x5f,
            g: 0x00,
            b: 0x5f,
        },
    ),
    (
        "Purple4",
        Color {
            name: None,
            r: 0x5f,
            g: 0x00,
            b: 0x87,
        },
    ),
    (
        "Purple4",
        Color {
            name: None,
            r: 0x5f,
            g: 0x00,
            b: 0xaf,
        },
    ),
    (
        "Purple3",
        Color {
            name: None,
            r: 0x5f,
            g: 0x00,
            b: 0xd7,
        },
    ),
    (
        "BlueViolet",
        Color {
            name: None,
            r: 0x5f,
            g: 0x00,
            b: 0xff,
        },
    ),
    (
        "Orange4",
        Color {
            name: None,
            r: 0x5f,
            g: 0x5f,
            b: 0x00,
        },
    ),
    (
        "Grey37",
        Color {
            name: None,
            r: 0x5f,
            g: 0x5f,
            b: 0x5f,
        },
    ),
    (
        "MediumPurple4",
        Color {
            name: None,
            r: 0x5f,
            g: 0x5f,
            b: 0x87,
        },
    ),
    (
        "SlateBlue3",
        Color {
            name: None,
            r: 0x5f,
            g: 0x5f,
            b: 0xaf,
        },
    ),
    (
        "SlateBlue3",
        Color {
            name: None,
            r: 0x5f,
            g: 0x5f,
            b: 0xd7,
        },
    ),
    (
        "RoyalBlue1",
        Color {
            name: None,
            r: 0x5f,
            g: 0x5f,
            b: 0xff,
        },
    ),
    (
        "Chartreuse4",
        Color {
            name: None,
            r: 0x5f,
            g: 0x87,
            b: 0x00,
        },
    ),
    (
        "DarkSeaGreen4",
        Color {
            name: None,
            r: 0x5f,
            g: 0x87,
            b: 0x5f,
        },
    ),
    (
        "PaleTurquoise4",
        Color {
            name: None,
            r: 0x5f,
            g: 0x87,
            b: 0x87,
        },
    ),
    (
        "SteelBlue",
        Color {
            name: None,
            r: 0x5f,
            g: 0x87,
            b: 0xaf,
        },
    ),
    (
        "SteelBlue3",
        Color {
            name: None,
            r: 0x5f,
            g: 0x87,
            b: 0xd7,
        },
    ),
    (
        "CornflowerBlue",
        Color {
            name: None,
            r: 0x5f,
            g: 0x87,
            b: 0xff,
        },
    ),
    (
        "Chartreuse3",
        Color {
            name: None,
            r: 0x5f,
            g: 0xaf,
            b: 0x00,
        },
    ),
    (
        "DarkSeaGreen4",
        Color {
            name: None,
            r: 0x5f,
            g: 0xaf,
            b: 0x5f,
        },
    ),
    (
        "CadetBlue",
        Color {
            name: None,
            r: 0x5f,
            g: 0xaf,
            b: 0x87,
        },
    ),
    (
        "CadetBlue",
        Color {
            name: None,
            r: 0x5f,
            g: 0xaf,
            b: 0xaf,
        },
    ),
    (
        "SkyBlue3",
        Color {
            name: None,
            r: 0x5f,
            g: 0xaf,
            b: 0xd7,
        },
    ),
    (
        "SteelBlue1",
        Color {
            name: None,
            r: 0x5f,
            g: 0xaf,
            b: 0xff,
        },
    ),
    (
        "Chartreuse3",
        Color {
            name: None,
            r: 0x5f,
            g: 0xd7,
            b: 0x00,
        },
    ),
    (
        "PaleGreen3",
        Color {
            name: None,
            r: 0x5f,
            g: 0xd7,
            b: 0x5f,
        },
    ),
    (
        "SeaGreen3",
        Color {
            name: None,
            r: 0x5f,
            g: 0xd7,
            b: 0x87,
        },
    ),
    (
        "Aquamarine3",
        Color {
            name: None,
            r: 0x5f,
            g: 0xd7,
            b: 0xaf,
        },
    ),
    (
        "MediumTurquoise",
        Color {
            name: None,
            r: 0x5f,
            g: 0xd7,
            b: 0xd7,
        },
    ),
    (
        "SteelBlue1",
        Color {
            name: None,
            r: 0x5f,
            g: 0xd7,
            b: 0xff,
        },
    ),
    (
        "Chartreuse2",
        Color {
            name: None,
            r: 0x5f,
            g: 0xff,
            b: 0x00,
        },
    ),
    (
        "SeaGreen2",
        Color {
            name: None,
            r: 0x5f,
            g: 0xff,
            b: 0x5f,
        },
    ),
    (
        "SeaGreen1",
        Color {
            name: None,
            r: 0x5f,
            g: 0xff,
            b: 0x87,
        },
    ),
    (
        "SeaGreen1",
        Color {
            name: None,
            r: 0x5f,
            g: 0xff,
            b: 0xaf,
        },
    ),
    (
        "Aquamarine1",
        Color {
            name: None,
            r: 0x5f,
            g: 0xff,
            b: 0xd7,
        },
    ),
    (
        "DarkSlateGray2",
        Color {
            name: None,
            r: 0x5f,
            g: 0xff,
            b: 0xff,
        },
    ),
    (
        "DarkRed",
        Color {
            name: None,
            r: 0x87,
            g: 0x00,
            b: 0x00,
        },
    ),
    (
        "DeepPink4",
        Color {
            name: None,
            r: 0x87,
            g: 0x00,
            b: 0x5f,
        },
    ),
    (
        "DarkMagenta",
        Color {
            name: None,
            r: 0x87,
            g: 0x00,
            b: 0x87,
        },
    ),
    (
        "DarkMagenta",
        Color {
            name: None,
            r: 0x87,
            g: 0x00,
            b: 0xaf,
        },
    ),
    (
        "DarkViolet",
        Color {
            name: None,
            r: 0x87,
            g: 0x00,
            b: 0xd7,
        },
    ),
    (
        "Purple",
        Color {
            name: None,
            r: 0x87,
            g: 0x00,
            b: 0xff,
        },
    ),
    (
        "Orange4",
        Color {
            name: None,
            r: 0x87,
            g: 0x5f,
            b: 0x00,
        },
    ),
    (
        "LightPink4",
        Color {
            name: None,
            r: 0x87,
            g: 0x5f,
            b: 0x5f,
        },
    ),
    (
        "Plum4",
        Color {
            name: None,
            r: 0x87,
            g: 0x5f,
            b: 0x87,
        },
    ),
    (
        "MediumPurple3",
        Color {
            name: None,
            r: 0x87,
            g: 0x5f,
            b: 0xaf,
        },
    ),
    (
        "MediumPurple3",
        Color {
            name: None,
            r: 0x87,
            g: 0x5f,
            b: 0xd7,
        },
    ),
    (
        "SlateBlue1",
        Color {
            name: None,
            r: 0x87,
            g: 0x5f,
            b: 0xff,
        },
    ),
    (
        "Yellow4",
        Color {
            name: None,
            r: 0x87,
            g: 0x87,
            b: 0x00,
        },
    ),
    (
        "Wheat4",
        Color {
            name: None,
            r: 0x87,
            g: 0x87,
            b: 0x5f,
        },
    ),
    (
        "Grey53",
        Color {
            name: None,
            r: 0x87,
            g: 0x87,
            b: 0x87,
        },
    ),
    (
        "LightSlateGrey",
        Color {
            name: None,
            r: 0x87,
            g: 0x87,
            b: 0xaf,
        },
    ),
    (
        "MediumPurple",
        Color {
            name: None,
            r: 0x87,
            g: 0x87,
            b: 0xd7,
        },
    ),
    (
        "LightSlateBlue",
        Color {
            name: None,
            r: 0x87,
            g: 0x87,
            b: 0xff,
        },
    ),
    (
        "Yellow4",
        Color {
            name: None,
            r: 0x87,
            g: 0xaf,
            b: 0x00,
        },
    ),
    (
        "DarkOliveGreen3",
        Color {
            name: None,
            r: 0x87,
            g: 0xaf,
            b: 0x5f,
        },
    ),
    (
        "DarkSeaGreen",
        Color {
            name: None,
            r: 0x87,
            g: 0xaf,
            b: 0x87,
        },
    ),
    (
        "LightSkyBlue3",
        Color {
            name: None,
            r: 0x87,
            g: 0xaf,
            b: 0xaf,
        },
    ),
    (
        "LightSkyBlue3",
        Color {
            name: None,
            r: 0x87,
            g: 0xaf,
            b: 0xd7,
        },
    ),
    (
        "SkyBlue2",
        Color {
            name: None,
            r: 0x87,
            g: 0xaf,
            b: 0xff,
        },
    ),
    (
        "Chartreuse2",
        Color {
            name: None,
            r: 0x87,
            g: 0xd7,
            b: 0x00,
        },
    ),
    (
        "DarkOliveGreen3",
        Color {
            name: None,
            r: 0x87,
            g: 0xd7,
            b: 0x5f,
        },
    ),
    (
        "PaleGreen3",
        Color {
            name: None,
            r: 0x87,
            g: 0xd7,
            b: 0x87,
        },
    ),
    (
        "DarkSeaGreen3",
        Color {
            name: None,
            r: 0x87,
            g: 0xd7,
            b: 0xaf,
        },
    ),
    (
        "DarkSlateGray3",
        Color {
            name: None,
            r: 0x87,
            g: 0xd7,
            b: 0xd7,
        },
    ),
    (
        "SkyBlue1",
        Color {
            name: None,
            r: 0x87,
            g: 0xd7,
            b: 0xff,
        },
    ),
    (
        "Chartreuse1",
        Color {
            name: None,
            r: 0x87,
            g: 0xff,
            b: 0x00,
        },
    ),
    (
        "LightGreen",
        Color {
            name: None,
            r: 0x87,
            g: 0xff,
            b: 0x5f,
        },
    ),
    (
        "LightGreen",
        Color {
            name: None,
            r: 0x87,
            g: 0xff,
            b: 0x87,
        },
    ),
    (
        "PaleGreen1",
        Color {
            name: None,
            r: 0x87,
            g: 0xff,
            b: 0xaf,
        },
    ),
    (
        "Aquamarine1",
        Color {
            name: None,
            r: 0x87,
            g: 0xff,
            b: 0xd7,
        },
    ),
    (
        "DarkSlateGray1",
        Color {
            name: None,
            r: 0x87,
            g: 0xff,
            b: 0xff,
        },
    ),
    (
        "Red3",
        Color {
            name: None,
            r: 0xaf,
            g: 0x00,
            b: 0x00,
        },
    ),
    (
        "DeepPink4",
        Color {
            name: None,
            r: 0xaf,
            g: 0x00,
            b: 0x5f,
        },
    ),
    (
        "MediumVioletRed",
        Color {
            name: None,
            r: 0xaf,
            g: 0x00,
            b: 0x87,
        },
    ),
    (
        "Magenta3",
        Color {
            name: None,
            r: 0xaf,
            g: 0x00,
            b: 0xaf,
        },
    ),
    (
        "DarkViolet",
        Color {
            name: None,
            r: 0xaf,
            g: 0x00,
            b: 0xd7,
        },
    ),
    (
        "Purple",
        Color {
            name: None,
            r: 0xaf,
            g: 0x00,
            b: 0xff,
        },
    ),
    (
        "DarkOrange3",
        Color {
            name: None,
            r: 0xaf,
            g: 0x5f,
            b: 0x00,
        },
    ),
    (
        "IndianRed",
        Color {
            name: None,
            r: 0xaf,
            g: 0x5f,
            b: 0x5f,
        },
    ),
    (
        "HotPink3",
        Color {
            name: None,
            r: 0xaf,
            g: 0x5f,
            b: 0x87,
        },
    ),
    (
        "MediumOrchid3",
        Color {
            name: None,
            r: 0xaf,
            g: 0x5f,
            b: 0xaf,
        },
    ),
    (
        "MediumOrchid",
        Color {
            name: None,
            r: 0xaf,
            g: 0x5f,
            b: 0xd7,
        },
    ),
    (
        "MediumPurple2",
        Color {
            name: None,
            r: 0xaf,
            g: 0x5f,
            b: 0xff,
        },
    ),
    (
        "DarkGoldenrod",
        Color {
            name: None,
            r: 0xaf,
            g: 0x87,
            b: 0x00,
        },
    ),
    (
        "LightSalmon3",
        Color {
            name: None,
            r: 0xaf,
            g: 0x87,
            b: 0x5f,
        },
    ),
    (
        "RosyBrown",
        Color {
            name: None,
            r: 0xaf,
            g: 0x87,
            b: 0x87,
        },
    ),
    (
        "Grey63",
        Color {
            name: None,
            r: 0xaf,
            g: 0x87,
            b: 0xaf,
        },
    ),
    (
        "MediumPurple2",
        Color {
            name: None,
            r: 0xaf,
            g: 0x87,
            b: 0xd7,
        },
    ),
    (
        "MediumPurple1",
        Color {
            name: None,
            r: 0xaf,
            g: 0x87,
            b: 0xff,
        },
    ),
    (
        "Gold3",
        Color {
            name: None,
            r: 0xaf,
            g: 0xaf,
            b: 0x00,
        },
    ),
    (
        "DarkKhaki",
        Color {
            name: None,
            r: 0xaf,
            g: 0xaf,
            b: 0x5f,
        },
    ),
    (
        "NavajoWhite3",
        Color {
            name: None,
            r: 0xaf,
            g: 0xaf,
            b: 0x87,
        },
    ),
    (
        "Grey69",
        Color {
            name: None,
            r: 0xaf,
            g: 0xaf,
            b: 0xaf,
        },
    ),
    (
        "LightSteelBlue3",
        Color {
            name: None,
            r: 0xaf,
            g: 0xaf,
            b: 0xd7,
        },
    ),
    (
        "LightSteelBlue",
        Color {
            name: None,
            r: 0xaf,
            g: 0xaf,
            b: 0xff,
        },
    ),
    (
        "Yellow3",
        Color {
            name: None,
            r: 0xaf,
            g: 0xd7,
            b: 0x00,
        },
    ),
    (
        "DarkOliveGreen3",
        Color {
            name: None,
            r: 0xaf,
            g: 0xd7,
            b: 0x5f,
        },
    ),
    (
        "DarkSeaGreen3",
        Color {
            name: None,
            r: 0xaf,
            g: 0xd7,
            b: 0x87,
        },
    ),
    (
        "DarkSeaGreen2",
        Color {
            name: None,
            r: 0xaf,
            g: 0xd7,
            b: 0xaf,
        },
    ),
    (
        "LightCyan3",
        Color {
            name: None,
            r: 0xaf,
            g: 0xd7,
            b: 0xd7,
        },
    ),
    (
        "LightSkyBlue1",
        Color {
            name: None,
            r: 0xaf,
            g: 0xd7,
            b: 0xff,
        },
    ),
    (
        "GreenYellow",
        Color {
            name: None,
            r: 0xaf,
            g: 0xff,
            b: 0x00,
        },
    ),
    (
        "DarkOliveGreen2",
        Color {
            name: None,
            r: 0xaf,
            g: 0xff,
            b: 0x5f,
        },
    ),
    (
        "PaleGreen1",
        Color {
            name: None,
            r: 0xaf,
            g: 0xff,
            b: 0x87,
        },
    ),
    (
        "DarkSeaGreen2",
        Color {
            name: None,
            r: 0xaf,
            g: 0xff,
            b: 0xaf,
        },
    ),
    (
        "DarkSeaGreen1",
        Color {
            name: None,
            r: 0xaf,
            g: 0xff,
            b: 0xd7,
        },
    ),
    (
        "PaleTurquoise1",
        Color {
            name: None,
            r: 0xaf,
            g: 0xff,
            b: 0xff,
        },
    ),
    (
        "Red3",
        Color {
            name: None,
            r: 0xd7,
            g: 0x00,
            b: 0x00,
        },
    ),
    (
        "DeepPink3",
        Color {
            name: None,
            r: 0xd7,
            g: 0x00,
            b: 0x5f,
        },
    ),
    (
        "DeepPink3",
        Color {
            name: None,
            r: 0xd7,
            g: 0x00,
            b: 0x87,
        },
    ),
    (
        "Magenta3",
        Color {
            name: None,
            r: 0xd7,
            g: 0x00,
            b: 0xaf,
        },
    ),
    (
        "Magenta3",
        Color {
            name: None,
            r: 0xd7,
            g: 0x00,
            b: 0xd7,
        },
    ),
    (
        "Magenta2",
        Color {
            name: None,
            r: 0xd7,
            g: 0x00,
            b: 0xff,
        },
    ),
    (
        "DarkOrange3",
        Color {
            name: None,
            r: 0xd7,
            g: 0x5f,
            b: 0x00,
        },
    ),
    (
        "IndianRed",
        Color {
            name: None,
            r: 0xd7,
            g: 0x5f,
            b: 0x5f,
        },
    ),
    (
        "HotPink3",
        Color {
            name: None,
            r: 0xd7,
            g: 0x5f,
            b: 0x87,
        },
    ),
    (
        "HotPink2",
        Color {
            name: None,
            r: 0xd7,
            g: 0x5f,
            b: 0xaf,
        },
    ),
    (
        "Orchid",
        Color {
            name: None,
            r: 0xd7,
            g: 0x5f,
            b: 0xd7,
        },
    ),
    (
        "MediumOrchid1",
        Color {
            name: None,
            r: 0xd7,
            g: 0x5f,
            b: 0xff,
        },
    ),
    (
        "Orange3",
        Color {
            name: None,
            r: 0xd7,
            g: 0x87,
            b: 0x00,
        },
    ),
    (
        "LightSalmon3",
        Color {
            name: None,
            r: 0xd7,
            g: 0x87,
            b: 0x5f,
        },
    ),
    (
        "LightPink3",
        Color {
            name: None,
            r: 0xd7,
            g: 0x87,
            b: 0x87,
        },
    ),
    (
        "Pink3",
        Color {
            name: None,
            r: 0xd7,
            g: 0x87,
            b: 0xaf,
        },
    ),
    (
        "Plum3",
        Color {
            name: None,
            r: 0xd7,
            g: 0x87,
            b: 0xd7,
        },
    ),
    (
        "Violet",
        Color {
            name: None,
            r: 0xd7,
            g: 0x87,
            b: 0xff,
        },
    ),
    (
        "Gold3",
        Color {
            name: None,
            r: 0xd7,
            g: 0xaf,
            b: 0x00,
        },
    ),
    (
        "LightGoldenrod3",
        Color {
            name: None,
            r: 0xd7,
            g: 0xaf,
            b: 0x5f,
        },
    ),
    (
        "Tan",
        Color {
            name: None,
            r: 0xd7,
            g: 0xaf,
            b: 0x87,
        },
    ),
    (
        "MistyRose3",
        Color {
            name: None,
            r: 0xd7,
            g: 0xaf,
            b: 0xaf,
        },
    ),
    (
        "Thistle3",
        Color {
            name: None,
            r: 0xd7,
            g: 0xaf,
            b: 0xd7,
        },
    ),
    (
        "Plum2",
        Color {
            name: None,
            r: 0xd7,
            g: 0xaf,
            b: 0xff,
        },
    ),
    (
        "Yellow3",
        Color {
            name: None,
            r: 0xd7,
            g: 0xd7,
            b: 0x00,
        },
    ),
    (
        "Khaki3",
        Color {
            name: None,
            r: 0xd7,
            g: 0xd7,
            b: 0x5f,
        },
    ),
    (
        "LightGoldenrod2",
        Color {
            name: None,
            r: 0xd7,
            g: 0xd7,
            b: 0x87,
        },
    ),
    (
        "LightYellow3",
        Color {
            name: None,
            r: 0xd7,
            g: 0xd7,
            b: 0xaf,
        },
    ),
    (
        "Grey84",
        Color {
            name: None,
            r: 0xd7,
            g: 0xd7,
            b: 0xd7,
        },
    ),
    (
        "LightSteelBlue1",
        Color {
            name: None,
            r: 0xd7,
            g: 0xd7,
            b: 0xff,
        },
    ),
    (
        "Yellow2",
        Color {
            name: None,
            r: 0xd7,
            g: 0xff,
            b: 0x00,
        },
    ),
    (
        "DarkOliveGreen1",
        Color {
            name: None,
            r: 0xd7,
            g: 0xff,
            b: 0x5f,
        },
    ),
    (
        "DarkOliveGreen1",
        Color {
            name: None,
            r: 0xd7,
            g: 0xff,
            b: 0x87,
        },
    ),
    (
        "DarkSeaGreen1",
        Color {
            name: None,
            r: 0xd7,
            g: 0xff,
            b: 0xaf,
        },
    ),
    (
        "Honeydew2",
        Color {
            name: None,
            r: 0xd7,
            g: 0xff,
            b: 0xd7,
        },
    ),
    (
        "LightCyan1",
        Color {
            name: None,
            r: 0xd7,
            g: 0xff,
            b: 0xff,
        },
    ),
    (
        "Red1",
        Color {
            name: None,
            r: 0xff,
            g: 0x00,
            b: 0x00,
        },
    ),
    (
        "DeepPink2",
        Color {
            name: None,
            r: 0xff,
            g: 0x00,
            b: 0x5f,
        },
    ),
    (
        "DeepPink1",
        Color {
            name: None,
            r: 0xff,
            g: 0x00,
            b: 0x87,
        },
    ),
    (
        "DeepPink1",
        Color {
            name: None,
            r: 0xff,
            g: 0x00,
            b: 0xaf,
        },
    ),
    (
        "Magenta2",
        Color {
            name: None,
            r: 0xff,
            g: 0x00,
            b: 0xd7,
        },
    ),
    (
        "Magenta1",
        Color {
            name: None,
            r: 0xff,
            g: 0x00,
            b: 0xff,
        },
    ),
    (
        "OrangeRed1",
        Color {
            name: None,
            r: 0xff,
            g: 0x5f,
            b: 0x00,
        },
    ),
    (
        "IndianRed1",
        Color {
            name: None,
            r: 0xff,
            g: 0x5f,
            b: 0x5f,
        },
    ),
    (
        "IndianRed1",
        Color {
            name: None,
            r: 0xff,
            g: 0x5f,
            b: 0x87,
        },
    ),
    (
        "HotPink",
        Color {
            name: None,
            r: 0xff,
            g: 0x5f,
            b: 0xaf,
        },
    ),
    (
        "HotPink",
        Color {
            name: None,
            r: 0xff,
            g: 0x5f,
            b: 0xd7,
        },
    ),
    (
        "MediumOrchid1",
        Color {
            name: None,
            r: 0xff,
            g: 0x5f,
            b: 0xff,
        },
    ),
    (
        "DarkOrange",
        Color {
            name: None,
            r: 0xff,
            g: 0x87,
            b: 0x00,
        },
    ),
    (
        "Salmon1",
        Color {
            name: None,
            r: 0xff,
            g: 0x87,
            b: 0x5f,
        },
    ),
    (
        "LightCoral",
        Color {
            name: None,
            r: 0xff,
            g: 0x87,
            b: 0x87,
        },
    ),
    (
        "PaleVioletRed1",
        Color {
            name: None,
            r: 0xff,
            g: 0x87,
            b: 0xaf,
        },
    ),
    (
        "Orchid2",
        Color {
            name: None,
            r: 0xff,
            g: 0x87,
            b: 0xd7,
        },
    ),
    (
        "Orchid1",
        Color {
            name: None,
            r: 0xff,
            g: 0x87,
            b: 0xff,
        },
    ),
    (
        "Orange1",
        Color {
            name: None,
            r: 0xff,
            g: 0xaf,
            b: 0x00,
        },
    ),
    (
        "SandyBrown",
        Color {
            name: None,
            r: 0xff,
            g: 0xaf,
            b: 0x5f,
        },
    ),
    (
        "LightSalmon1",
        Color {
            name: None,
            r: 0xff,
            g: 0xaf,
            b: 0x87,
        },
    ),
    (
        "LightPink1",
        Color {
            name: None,
            r: 0xff,
            g: 0xaf,
            b: 0xaf,
        },
    ),
    (
        "Pink1",
        Color {
            name: None,
            r: 0xff,
            g: 0xaf,
            b: 0xd7,
        },
    ),
    (
        "Plum1",
        Color {
            name: None,
            r: 0xff,
            g: 0xaf,
            b: 0xff,
        },
    ),
    (
        "Gold1",
        Color {
            name: None,
            r: 0xff,
            g: 0xd7,
            b: 0x00,
        },
    ),
    (
        "LightGoldenrod2",
        Color {
            name: None,
            r: 0xff,
            g: 0xd7,
            b: 0x5f,
        },
    ),
    (
        "LightGoldenrod2",
        Color {
            name: None,
            r: 0xff,
            g: 0xd7,
            b: 0x87,
        },
    ),
    (
        "NavajoWhite1",
        Color {
            name: None,
            r: 0xff,
            g: 0xd7,
            b: 0xaf,
        },
    ),
    (
        "MistyRose1",
        Color {
            name: None,
            r: 0xff,
            g: 0xd7,
            b: 0xd7,
        },
    ),
    (
        "Thistle1",
        Color {
            name: None,
            r: 0xff,
            g: 0xd7,
            b: 0xff,
        },
    ),
    (
        "Yellow1",
        Color {
            name: None,
            r: 0xff,
            g: 0xff,
            b: 0x00,
        },
    ),
    (
        "LightGoldenrod1",
        Color {
            name: None,
            r: 0xff,
            g: 0xff,
            b: 0x5f,
        },
    ),
    (
        "Khaki1",
        Color {
            name: None,
            r: 0xff,
            g: 0xff,
            b: 0x87,
        },
    ),
    (
        "Wheat1",
        Color {
            name: None,
            r: 0xff,
            g: 0xff,
            b: 0xaf,
        },
    ),
    (
        "Cornsilk1",
        Color {
            name: None,
            r: 0xff,
            g: 0xff,
            b: 0xd7,
        },
    ),
    (
        "Grey100",
        Color {
            name: None,
            r: 0xff,
            g: 0xff,
            b: 0xff,
        },
    ),
    (
        "Grey3",
        Color {
            name: None,
            r: 0x08,
            g: 0x08,
            b: 0x08,
        },
    ),
    (
        "Grey7",
        Color {
            name: None,
            r: 0x12,
            g: 0x12,
            b: 0x12,
        },
    ),
    (
        "Grey11",
        Color {
            name: None,
            r: 0x1c,
            g: 0x1c,
            b: 0x1c,
        },
    ),
    (
        "Grey15",
        Color {
            name: None,
            r: 0x26,
            g: 0x26,
            b: 0x26,
        },
    ),
    (
        "Grey19",
        Color {
            name: None,
            r: 0x30,
            g: 0x30,
            b: 0x30,
        },
    ),
    (
        "Grey23",
        Color {
            name: None,
            r: 0x3a,
            g: 0x3a,
            b: 0x3a,
        },
    ),
    (
        "Grey27",
        Color {
            name: None,
            r: 0x44,
            g: 0x44,
            b: 0x44,
        },
    ),
    (
        "Grey30",
        Color {
            name: None,
            r: 0x4e,
            g: 0x4e,
            b: 0x4e,
        },
    ),
    (
        "Grey35",
        Color {
            name: None,
            r: 0x58,
            g: 0x58,
            b: 0x58,
        },
    ),
    (
        "Grey39",
        Color {
            name: None,
            r: 0x62,
            g: 0x62,
            b: 0x62,
        },
    ),
    (
        "Grey42",
        Color {
            name: None,
            r: 0x6c,
            g: 0x6c,
            b: 0x6c,
        },
    ),
    (
        "Grey46",
        Color {
            name: None,
            r: 0x76,
            g: 0x76,
            b: 0x76,
        },
    ),
    (
        "Grey50",
        Color {
            name: None,
            r: 0x80,
            g: 0x80,
            b: 0x80,
        },
    ),
    (
        "Grey54",
        Color {
            name: None,
            r: 0x8a,
            g: 0x8a,
            b: 0x8a,
        },
    ),
    (
        "Grey58",
        Color {
            name: None,
            r: 0x94,
            g: 0x94,
            b: 0x94,
        },
    ),
    (
        "Grey62",
        Color {
            name: None,
            r: 0x9e,
            g: 0x9e,
            b: 0x9e,
        },
    ),
    (
        "Grey66",
        Color {
            name: None,
            r: 0xa8,
            g: 0xa8,
            b: 0xa8,
        },
    ),
    (
        "Grey70",
        Color {
            name: None,
            r: 0xb2,
            g: 0xb2,
            b: 0xb2,
        },
    ),
    (
        "Grey74",
        Color {
            name: None,
            r: 0xbc,
            g: 0xbc,
            b: 0xbc,
        },
    ),
    (
        "Grey78",
        Color {
            name: None,
            r: 0xc6,
            g: 0xc6,
            b: 0xc6,
        },
    ),
    (
        "Grey82",
        Color {
            name: None,
            r: 0xd0,
            g: 0xd0,
            b: 0xd0,
        },
    ),
    (
        "Grey85",
        Color {
            name: None,
            r: 0xda,
            g: 0xda,
            b: 0xda,
        },
    ),
    (
        "Grey89",
        Color {
            name: None,
            r: 0xe4,
            g: 0xe4,
            b: 0xe4,
        },
    ),
    (
        "Grey93",
        Color {
            name: None,
            r: 0xee,
            g: 0xee,
            b: 0xee,
        },
    ),
];

pub const VIEWDATA_PALETTE: [Color; 16] = [
    Color {
        name: None,
        r: 0x00,
        g: 0x00,
        b: 0x00,
    }, // black
    Color {
        name: None,
        r: 0xFF,
        g: 0x00,
        b: 0x00,
    }, // red
    Color {
        name: None,
        r: 0x00,
        g: 0xFF,
        b: 0x00,
    }, // green
    Color {
        name: None,
        r: 0xFF,
        g: 0xFF,
        b: 0x00,
    }, // yellow
    Color {
        name: None,
        r: 0x00,
        g: 0x00,
        b: 0xFF,
    }, // blue
    Color {
        name: None,
        r: 0xFF,
        g: 0x00,
        b: 0xFF,
    }, // magenta
    Color {
        name: None,
        r: 0x00,
        g: 0xFF,
        b: 0xFF,
    }, // cyan
    Color {
        name: None,
        r: 0xFF,
        g: 0xFF,
        b: 0xFF,
    }, // white
    Color {
        name: None,
        r: 0x00,
        g: 0x00,
        b: 0x00,
    }, // black
    Color {
        name: None,
        r: 0xFF,
        g: 0x00,
        b: 0x00,
    }, // red
    Color {
        name: None,
        r: 0x00,
        g: 0xFF,
        b: 0x00,
    }, // green
    Color {
        name: None,
        r: 0xFF,
        g: 0xFF,
        b: 0x00,
    }, // yellow
    Color {
        name: None,
        r: 0x00,
        g: 0x00,
        b: 0xFF,
    }, // blue
    Color {
        name: None,
        r: 0xFF,
        g: 0x00,
        b: 0xFF,
    }, // magenta
    Color {
        name: None,
        r: 0x00,
        g: 0xFF,
        b: 0xFF,
    }, // cyan
    Color {
        name: None,
        r: 0xFF,
        g: 0xFF,
        b: 0xFF,
    }, // white
];

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
