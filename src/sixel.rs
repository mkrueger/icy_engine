use crate::{Color, Position};

#[derive(Clone, Debug)]
pub enum SixelReadStatus {
    NotStarted,
    Finished,
    Error,
    Position(i32, i32)
}

impl Default for SixelReadStatus {
    fn default() -> Self {
        SixelReadStatus::NotStarted
    }
}

#[derive(Clone, Debug, Default)]
pub struct Sixel {
    pub position: Position,
    pub aspect_ratio: u8,
    pub background_color: Option<Color>,
    pub picture: Vec<Vec<Option<Color>>>,
    pub len: usize,
    pub read_status: SixelReadStatus
}

impl Sixel {
    pub fn new(position: Position, aspect_ratio: u8) -> Self {
        Self { 
            position,
            aspect_ratio,
            background_color: None,
            picture: Vec::new(),
            read_status: Default::default(),
            len: 0
        }
    }

    pub fn width(&self) -> u32 {
        self.picture.len() as u32
    }

    pub fn height(&self) -> u32 {
        if let Some(first_line) = self.picture.get(0) {
            first_line.len() as u32
        } else {
            0
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }
}