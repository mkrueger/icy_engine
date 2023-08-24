use crate::Position;

#[derive(Debug, Clone)]
pub struct Coordinates {
    pub x: f32,
    pub y: f32,
}

impl Coordinates {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Debug)]
pub enum Shape {
    Rectangle,
    Lines,
}

#[derive(Debug, Clone)]
pub struct Selection {
    pub anchor: Coordinates,
    pub lead: Coordinates,

    pub anchor_pos: Position,
    pub lead_pos: Position,

    pub locked: bool,
    pub shape: Shape,
}

impl Default for Selection {
    fn default() -> Self {
        Selection::new(0., 0.)
    }
}

impl Selection {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            anchor: Coordinates::new(x, y),
            lead: Coordinates::new(x, y),
            anchor_pos: Position::new(x as i32, y as i32),
            lead_pos: Position::new(x as i32, y as i32),
            locked: false,
            shape: Shape::Lines,
        }
    }

    pub fn from_rectangle(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            anchor: Coordinates::new(x, y),
            lead: Coordinates::new(x + width, y + height),
            anchor_pos: Position::new(x as i32, y as i32),
            lead_pos: Position::new((x + height) as i32, (y + height) as i32),
            locked: false,
            shape: Shape::Rectangle,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.anchor_pos == self.lead_pos
    }
}

impl Selection {
    pub fn set_lead(&mut self, x: f32, y: f32) {
        self.lead = Coordinates::new(x, y);
        self.lead_pos = Position::new(x as i32, y as i32);
    }
}
