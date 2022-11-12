use crate::{Color, Position};

#[derive(Clone, Debug, Default)]
pub struct Sixel {
    pub position: Position,
    pub aspect_ratio: u8,
    pub background_color: Option<Color>,
    pub picture: Vec<Vec<Option<Color>>>
}

impl Sixel {
    pub fn new(position: Position, aspect_ratio: u8) -> Self {
        Self { 
            position,
            aspect_ratio,
            background_color: None,
            picture: Vec::new()
        }
    }
}