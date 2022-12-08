use crate::Position;

#[derive(Debug, Clone)]
pub struct Selection {
    pub anchor: (f32, f32),
    pub lead: (f32, f32),
    pub block_selection: bool,

    pub anchor_pos: Position,
    pub lead_pos: Position,

    pub locked: bool,
}

impl Default for Selection {
    fn default() -> Self {
        Selection::new((0., 0.))
    }
}

impl Selection {
    pub fn new(pos: (f32, f32)) -> Self {
        Self {
            anchor: pos,
            lead: pos,
            anchor_pos: Position::new(pos.0 as i32, pos.1 as i32),
            lead_pos: Position::new(pos.0 as i32, pos.1 as i32),
            block_selection: false,
            locked: false,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.anchor_pos == self.lead_pos
    }
}

impl Selection {
    pub fn set_lead(&mut self, lead: (f32, f32)) {
        self.lead = lead;
        self.lead_pos = Position::new(lead.0 as i32, lead.1 as i32);
    }
}
