use crate::{Buffer, HyperLink, Position};

impl Buffer {

    pub fn get_string(&self, pos: Position, size: usize) -> String {
        let mut result = String::new();
        let mut pos = pos;
        for _ in 0..size {
            result.push(self.get_char(pos).ch);
            pos.x += 1;
            if pos.x >= self.get_buffer_width() {
                pos.x = 0;
                pos.y += 1;
            }
        }
        result 
    }

    pub fn parse_hyperlinks(&self) -> Vec<HyperLink>
    {
        let mut result = Vec::new();

        let mut pos = Position::new(self.get_buffer_width() - 1, self.get_buffer_height() - 1);
        let mut parser = rfind_url::Parser::new();

        while pos.y >= 0 {
            let attr_char = self.get_char(pos);
            if let rfind_url::ParserState::Url(size) =  parser.advance(attr_char.ch) {
                let p =  crate::HyperLink {
                    url: None,
                    position: pos,
                    length: size,
                };
                result.push(p);
            }
            pos.x -= 1;
            if pos.x < 0 {
                pos.x = self.get_buffer_width() - 1;
                pos.y -= 1;
            }
        }

        result
    }


    fn underline(&mut self, pos: Position, size: usize){
        let mut pos = pos;
        for _ in 0..size {
            let mut ch = self.get_char(pos);
            ch.attribute.set_is_underlined(true);
            self.set_char(0, pos, ch);
            pos.x += 1;
            if pos.x >= self.get_buffer_width() {
                pos.x = 0;
                pos.y += 1;
            }
        }
    }

    pub fn is_position_in_range(&self, pos: Position, from: Position, size: usize) -> bool { 
        if pos.y < from.y {
            false
        } else  if pos.y == from.y {
            from.x <= pos.x && pos.x < from.x + size as i32
        } else {
            let mut x = from.x + size as i32;
            let remainder = x - self.get_buffer_width();
            let lines = remainder / self.get_buffer_width();
            let mut y = from.y + lines;

            if remainder > 0 {
                x = remainder - lines * self.get_buffer_width();
            }

            pos.y < y  || pos.y == y && pos.x < x
        }
    }

    pub fn join_hyperlinks(&mut self, hyperlinks: Vec<HyperLink>)
    {
        self.layers[0].hyperlinks.retain(|l| l.url.is_none());
        for hl in &hyperlinks {
            self.underline(hl.position, hl.length as usize);
        }
        self.layers[0].hyperlinks.extend(hyperlinks);
    }

    pub fn update_hyperlinks(&mut self)
    {
        self.join_hyperlinks(self.parse_hyperlinks());
    }
}
