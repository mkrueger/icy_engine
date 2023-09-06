use crate::{Position, Rectangle, Size};

#[derive(Clone, Copy, Debug)]
pub enum Shape {
    Rectangle,
    Lines,
}

#[derive(Debug, Clone, Copy)]
pub struct Selection {
    pub anchor: Position,
    pub lead: Position,

    pub locked: bool,
    pub shape: Shape,
}

impl Default for Selection {
    fn default() -> Self {
        Selection::new((0, 0))
    }
}

impl Selection {
    pub fn new(pos: impl Into<Position>) -> Self {
        let pos = pos.into();
        Self {
            anchor: pos,
            lead: pos,
            locked: false,
            shape: Shape::Lines,
        }
    }

    pub fn is_empty(&self) -> bool {
        let anchor_pos = self.anchor;
        let lead_pos = self.lead;

        anchor_pos == lead_pos
    }

    pub fn is_inside(&self, pos: impl Into<Position>) -> bool {
        let pos = pos.into();
        let anchor_pos = self.anchor;
        let lead_pos = self.lead;

        anchor_pos <= pos && lead_pos < pos || lead_pos <= pos && anchor_pos < pos
    }

    pub fn min(&self) -> Position {
        let anchor_pos = self.anchor;
        let lead_pos = self.lead;

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
            (anchor_pos.x - lead_pos.x).abs(),
            (anchor_pos.y - lead_pos.y).abs(),
        )
    }

    pub fn as_rectangle(&self) -> Rectangle {
        Rectangle::from_min_size(self.min(), self.size())
    }
}

impl From<Rectangle> for Selection {
    fn from(value: Rectangle) -> Self {
        Selection {
            anchor: value.top_left(),
            lead: value.lower_right(),
            locked: false,
            shape: Shape::Rectangle,
        }
    }
}

impl From<(f32, f32, f32, f32)> for Selection {
    fn from(value: (f32, f32, f32, f32)) -> Self {
        Selection {
            anchor: (value.0, value.1).into(),
            lead: (value.2, value.3).into(),
            locked: false,
            shape: Shape::Rectangle,
        }
    }
}

impl From<(i32, i32, i32, i32)> for Selection {
    fn from(value: (i32, i32, i32, i32)) -> Self {
        Selection {
            anchor: (value.0, value.1).into(),
            lead: (value.2, value.3).into(),
            locked: false,
            shape: Shape::Rectangle,
        }
    }
}
