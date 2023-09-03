#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::too_many_lines,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::must_use_candidate,
    clippy::struct_excessive_bools,
    clippy::return_self_not_must_use
)]
mod text_attribute;
use std::{cmp::min, error::Error};

pub use text_attribute::*;

mod attributed_char;
pub use attributed_char::*;

mod layer;
pub use layer::*;

mod line;
pub use line::*;

mod position;
pub use position::*;

mod buffers;
pub use buffers::*;

mod palette_handling;
pub use palette_handling::*;

mod fonts;
pub use fonts::*;

pub mod parsers;
pub use parsers::*;

mod caret;
pub use caret::*;

mod formats;
pub use formats::*;

mod tdf_font;
pub use tdf_font::*;

mod sauce_mod;
pub use sauce_mod::*;

mod crc;
pub use crc::*;

mod terminal_state;
pub use terminal_state::*;

mod sixel_mod;
pub use sixel_mod::*;

mod selection;
pub use selection::*;

mod url_scanner;
pub use url_scanner::*;

pub type EngineResult<T> = Result<T, Box<dyn Error>>;

pub mod editor;

#[derive(Copy, Clone, Debug, Default)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

impl std::fmt::Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(width: {}, height: {})", self.width, self.height)
    }
}

impl PartialEq for Size {
    fn eq(&self, other: &Size) -> bool {
        self.width == other.width && self.height == other.height
    }
}

impl Size {
    pub fn new(width: usize, height: usize) -> Self {
        Size { width, height }
    }
}

impl From<(usize, usize)> for Size {
    fn from(value: (usize, usize)) -> Self {
        Size {
            width: value.0,
            height: value.1,
        }
    }
}
impl From<(i32, i32)> for Size {
    fn from(value: (i32, i32)) -> Self {
        Size {
            width: value.0 as usize,
            height: value.1 as usize,
        }
    }
}
impl From<(u32, u32)> for Size {
    fn from(value: (u32, u32)) -> Self {
        Size {
            width: value.0 as usize,
            height: value.1 as usize,
        }
    }
}

impl From<(u16, u16)> for Size {
    fn from(value: (u16, u16)) -> Self {
        Size {
            width: value.0 as usize,
            height: value.1 as usize,
        }
    }
}

impl From<(u8, u8)> for Size {
    fn from(value: (u8, u8)) -> Self {
        Size {
            width: value.0 as usize,
            height: value.1 as usize,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Rectangle {
    pub start: Position,
    pub size: Size,
}
impl std::fmt::Display for Rectangle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(x:{}, y:{}, width: {}, height: {})",
            self.start.x, self.start.y, self.size.width, self.size.height
        )
    }
}

impl Rectangle {
    pub fn new(start: Position, size: Size) -> Self {
        Self { start, size }
    }

    pub fn from(x: i32, y: i32, width: usize, height: usize) -> Self {
        Self {
            start: Position::new(x, y),
            size: Size::new(width, height),
        }
    }

    /// .
    ///
    /// # Panics
    ///
    /// Panics if .
    pub fn from_coords(x1: i32, y1: i32, x2: i32, y2: i32) -> Self {
        assert!(x1 <= x2);
        assert!(y1 <= y2);
        Rectangle {
            start: Position::new(x1, y1),
            size: Size::new((x2 - x1) as usize + 1, (y2 - y1) as usize + 1),
        }
    }

    pub fn from_pt(p1: Position, p2: Position) -> Self {
        let start = Position::new(min(p1.x, p2.x), min(p1.y, p2.y));

        Rectangle {
            start,
            size: Size::new(
                (p1.x - p2.x).unsigned_abs() as usize,
                (p1.y - p2.y).unsigned_abs() as usize,
            ),
        }
    }

    pub fn lower_right(&self) -> Position {
        Position {
            x: self.start.x + self.size.width as i32,
            y: self.start.y + self.size.height as i32,
        }
    }

    pub fn contains_pt(&self, point: Position) -> bool {
        self.start.x <= point.x
            && point.x <= self.start.x + self.size.width as i32
            && self.start.y <= point.y
            && point.y <= self.start.y + self.size.height as i32
    }

    pub fn contains_rect(&self, other: &Rectangle) -> bool {
        self.contains_pt(other.start) && self.contains_pt(other.lower_right())
    }

    fn get_width(&self) -> usize {
        self.size.width
    }

    fn get_height(&self) -> usize {
        self.size.height
    }
}
