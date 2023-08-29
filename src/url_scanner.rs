use crate::{Buffer, HyperLink, Position};

impl Buffer {
    pub fn get_string(&self, pos: Position, size: usize) -> String {
        let mut result = String::new();
        let mut pos = pos;
        for _ in 0..size {
            result.push(self.get_char(pos).ch);
            pos.x += 1;
            if pos.x >= self.get_width() {
                pos.x = 0;
                pos.y += 1;
            }
        }
        result
    }

    pub fn parse_hyperlinks(&self) -> Vec<HyperLink> {
        let mut result = Vec::new();

        let mut pos = Position::new(self.get_width() - 1, self.get_height() - 1);
        let mut parser = rfind_url::Parser::new();

        while pos.y >= 0 {
            let attr_char = self.get_char(pos);
            if let rfind_url::ParserState::Url(size) = parser.advance(attr_char.ch) {
                let p = crate::HyperLink {
                    url: None,
                    position: pos,
                    length: size,
                };
                result.push(p);
            }
            pos.x -= 1;
            if pos.x < 0 {
                pos.x = self.get_width() - 1;
                pos.y -= 1;
            }
        }

        result
    }

    fn underline(&mut self, pos: Position, size: usize) {
        let mut pos = pos;
        for _ in 0..size {
            let mut ch = self.get_char(pos);
            ch.attribute.set_is_underlined(true);
            self.layers[0].set_char(pos, ch);
            pos.x += 1;
            if pos.x >= self.get_width() {
                pos.x = 0;
                pos.y += 1;
            }
        }
    }

    pub fn is_position_in_range(&self, pos: Position, from: Position, size: usize) -> bool {
        match pos.y.cmp(&from.y) {
            std::cmp::Ordering::Less => false,
            std::cmp::Ordering::Equal => from.x <= pos.x && pos.x < from.x + size as i32,
            std::cmp::Ordering::Greater => {
                let remainder = (size as i32 - self.get_width() + from.x).max(0);
                let lines = remainder / self.get_width();
                let mut y = from.y + lines;
                let x = if remainder > 0 {
                    y += 1; // remainder > 1 wraps 1 extra line
                    remainder - lines * self.get_width()
                } else {
                    remainder
                };
                pos.y < y || pos.y == y && pos.x < x
            }
        }
    }

    pub fn join_hyperlinks(&mut self, hyperlinks: Vec<HyperLink>) {
        self.layers[0].hyperlinks.retain(|l| l.url.is_none());
        for hl in &hyperlinks {
            self.underline(hl.position, hl.length);
        }
        self.layers[0].hyperlinks.extend(hyperlinks);
    }

    pub fn update_hyperlinks(&mut self) {
        self.join_hyperlinks(self.parse_hyperlinks());
    }
}
