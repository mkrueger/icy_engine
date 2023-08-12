use crate::{Color, Position, Rectangle, Size};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SixelReadStatus {
    #[default]
    NotStarted,
    Finished,
    Error,
    Position(i32, i32),
    Updated,
}

#[derive(Clone, Debug, Default)]
pub struct Sixel {
    pub position: Position,

    pub vertical_size: i32,
    pub horizontal_size: i32,

    pub background_color: Option<Color>,
    pub picture_data: Vec<Vec<Color>>,
    pub len: usize,
    pub read_status: SixelReadStatus,

    pub defined_height: Option<usize>,
    pub defined_width: Option<usize>,
}

impl Sixel {
    pub fn new(position: Position) -> Self {
        Self {
            position,
            vertical_size: 1,
            horizontal_size: 1,
            background_color: None,
            picture_data: Vec::new(),
            read_status: SixelReadStatus::default(),
            len: 0,
            defined_height: None,
            defined_width: None,
        }
    }

    pub fn get_rect(&self) -> Rectangle {
        Rectangle {
            start: self.position,
            size: Size::new(self.width() as i32, self.height() as i32),
        }
    }

    pub fn width(&self) -> u32 {
        if let Some(first_line) = self.picture_data.get(0) {
            first_line.len() as u32
        } else {
            0
        }
    }

    pub fn height(&self) -> u32 {
        self.picture_data.len() as u32
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}
