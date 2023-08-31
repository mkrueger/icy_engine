use crate::{Position, Size, UPosition};

#[derive(Debug, Clone)]
pub struct Coordinates {
    pub x: f32,
    pub y: f32,
}

impl Coordinates {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn as_position(&self) -> Position {
        Position::new(self.x as i32, self.y as i32)
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
            locked: false,
            shape: Shape::Lines,
        }
    }

    pub fn from_rectangle(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            anchor: Coordinates::new(x, y),
            lead: Coordinates::new(x + width, y + height),
            locked: false,
            shape: Shape::Rectangle,
        }
    }

    pub fn is_empty(&self) -> bool {
        let anchor_pos = self.anchor.as_position();
        let lead_pos = self.lead.as_position();

        anchor_pos == lead_pos
    }

    pub fn is_inside(&self, pos: impl Into<UPosition>) -> bool {
        let pos = pos.into();
        let anchor_pos = self.anchor.as_position();
        let lead_pos = self.lead.as_position();

        anchor_pos.as_uposition() <= pos && lead_pos.as_uposition() < pos
            || lead_pos.as_uposition() <= pos && anchor_pos.as_uposition() < pos
    }

    pub fn min(&self) -> Position {
        let anchor_pos = self.anchor.as_position();
        let lead_pos = self.lead.as_position();

        Position::new(anchor_pos.x.min(lead_pos.x), anchor_pos.y.min(lead_pos.y))
    }

    pub fn max(&self) -> Position {
        let anchor_pos = Position::new(self.anchor.x as i32, self.anchor.y as i32);
        let lead_pos = Position::new(self.lead.x as i32, self.lead.y as i32);

        Position::new(anchor_pos.x.max(lead_pos.x), anchor_pos.y.max(lead_pos.y))
    }

    pub fn size(&self) -> Size {
        let anchor_pos = Position::new(self.anchor.x as i32, self.anchor.y as i32);
        let lead_pos = Position::new(self.lead.x as i32, self.lead.y as i32);

        Size::new(
            (anchor_pos.x - lead_pos.x).unsigned_abs() as usize,
            (anchor_pos.y - lead_pos.y).unsigned_abs() as usize,
        )
    }
}

impl Selection {
    pub fn set_lead(&mut self, x: f32, y: f32) {
        self.lead = Coordinates::new(x, y);
    }
}
