#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::cast_sign_loss, clippy::cast_possible_truncation, clippy::cast_possible_wrap, clippy::too_many_lines, clippy::cast_lossless, clippy::cast_precision_loss)]
mod text_attribute;
use std::error::Error;

pub use text_attribute::*;

mod attributed_char;
pub use attributed_char::*;

mod layer;
pub use layer::*;

mod line;
pub use line::*;

mod position;
pub use position::*;

mod buffer;
pub use  buffer::*;

mod palette_handling;
pub use palette_handling::*;

mod fonts;
pub use fonts::*;

mod parser;
pub use parser::*;

mod caret;
pub use caret::*;

mod formats;
pub use formats::*;

mod tdf_font;
pub use tdf_font::*;

mod sauce;
pub use sauce::*;

mod undo_stack;
pub use undo_stack::*;

mod crc;
pub use crc::*;

mod terminal_state;
pub use terminal_state::*;

mod sixel;
pub use sixel::*;

pub type EngineResult<T> = Result<T, Box<dyn Error>>;

#[derive(Copy, Clone, Debug, Default)]
pub struct Size<T> 
{
    pub width: T,
    pub height: T
}

impl<T> PartialEq for Size<T>
where T: PartialEq {
    fn eq(&self, other: &Size<T>) -> bool {
        self.width == other.width && self.height == other.height
    }
}

impl<T> Size<T> 
where T: Default
{
    pub fn new(width: T, height: T) -> Self
    {
        Size { width, height }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Rectangle
{
    pub start: Position,
    pub size: Size<i32>
}

impl Rectangle {
    pub fn lower_right(&self) -> Position {
        Position { x: self.start.x + self.size.width, y: self.start.y + self.size.height }
    }
    pub fn contains(&self, other: Rectangle) -> bool {
        self.start <= other.start && self.lower_right() <= other.lower_right()
    }
}