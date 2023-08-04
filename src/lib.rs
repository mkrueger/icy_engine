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

use ::num::NumCast;
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

mod parsers;
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

pub type EngineResult<T> = Result<T, Box<dyn Error>>;

#[derive(Copy, Clone, Debug, Default)]
pub struct Size<T: NumCast> {
    pub width: T,
    pub height: T,
}

impl<T> PartialEq for Size<T>
where
    T: NumCast + Eq,
{
    fn eq(&self, other: &Size<T>) -> bool {
        self.width == other.width && self.height == other.height
    }
}

impl<T> Size<T>
where
    T: NumCast,
{
    pub fn new(width: T, height: T) -> Self {
        Size { width, height }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Rectangle {
    pub start: Position,
    pub size: Size<i32>,
}

impl Rectangle {
    pub fn new(start: Position, size: Size<i32>) -> Self {
        Self { start, size }
    }

    pub fn from(x: i32, y: i32, width: i32, height: i32) -> Self {
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
            size: Size::new((x2 - x1) + 1, (y2 - y1) + 1),
        }
    }

    pub fn from_pt(p1: Position, p2: Position) -> Self {
        let start = Position::new(min(p1.x, p2.x), min(p1.y, p2.y));

        Rectangle {
            start,
            size: Size::new((p1.x - p2.x).abs(), (p1.y - p2.y).abs()),
        }
    }

    pub fn is_inside(&self, p: Position) -> bool {
        self.start.x <= p.x
            && self.start.y <= p.y
            && p.x < self.start.x + self.size.width
            && p.y < self.start.y + self.size.height
    }

    pub fn lower_right(&self) -> Position {
        Position {
            x: self.start.x + self.size.width,
            y: self.start.y + self.size.height,
        }
    }

    pub fn contains(&self, other: Rectangle) -> bool {
        self.start <= other.start && self.lower_right() <= other.lower_right()
    }
}
