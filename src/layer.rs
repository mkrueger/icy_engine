use super::{DosChar, Position};

#[derive(Clone, Debug, Default)]
pub struct Line {
    pub chars: Vec<Option<DosChar>>,
}

impl Line {
    pub fn new() -> Self {
        Line { chars: Vec::new() }
    }

    pub fn get_line_length(&self) -> usize {
        let mut res = 0;
        for i in 0..self.chars.len() {
            if self.chars[i].is_some() {
                res = i;
            }
        }
        res
    }

    pub fn insert_char(&mut self, index: i32, char_opt: Option<DosChar>)
    {
        assert!(index >= 0 , "char out of range");
        if index > self.chars.len() as i32 {
            self.chars.resize(index as usize, None);
        }
        self.chars.insert(index as usize, char_opt);
    }

}

#[derive(Clone, Debug, Default)]
pub struct Layer {
    pub title: String,
    pub is_visible: bool,
    pub is_locked: bool,
    pub is_position_locked: bool,
    pub is_transparent: bool,

    pub offset: Position,
    pub lines: Vec<Line>,
}

impl Layer {
    pub fn new() -> Self {
        Layer {
            title: "Background".to_string(),
            is_visible: true,
            is_locked: false,
            is_position_locked: false,
            is_transparent: true,
            lines: Vec::new(),
            offset: Position::new(),
        }
    }

    pub fn get_offset(&self) -> Position {
        self.offset
    }

    pub fn set_offset(&mut self, pos: Position) {
        if self.is_position_locked {
            return;
        }
        self.offset = pos;
    }

    pub fn join(&mut self, layer: &Layer)
    {  
        for y in 0..layer.lines.len() {
            let line = &layer.lines[y];
            for x in 0..line.chars.len() {
                let ch = line.chars[x];
                if ch.is_some() {
                    self.set_char(Position::from(x as i32, y as i32), ch);
                }
            }
        }
    }

    pub fn clear(&mut self)
    {
        self.lines.clear();
    }

    pub fn set_char(&mut self, pos: Position, dos_char: Option<DosChar>) {
        let pos = pos - self.offset;
        if pos.x < 0 || pos.y < 0 || self.is_locked || !self.is_visible {
            return;
        }
        if pos.y >= self.lines.len() as i32 {
            self.lines.resize(pos.y as usize + 1, Line::new());
        }

        let cur_line = &mut self.lines[pos.y as usize];
        if pos.x >= cur_line.chars.len() as i32 {
            cur_line.chars.resize(pos.x as usize + 1, None);
        }
        cur_line.chars[pos.x as usize] = dos_char;
    }

    pub fn get_char(&self, pos: Position) -> Option<DosChar> {
        let pos = pos - self.offset;
        let y = pos.y as usize;
        if y < self.lines.len() {
            let cur_line = &self.lines[y];
            if pos.x >= 0 && pos.x < cur_line.chars.len() as i32 {
                let ch = cur_line.chars[pos.x as usize];
                if ch.is_some() {
                    return ch;
                }
            }
        }
        if self.is_transparent {
            None
        } else {
            Some(DosChar::default())
        }
    }

    pub fn remove_line(&mut self, index: i32)
    {
        if self.is_locked || !self.is_visible {
            return;
        }
        assert!(!(index < 0 || index >= self.lines.len() as i32), "line out of range");
        self.lines.remove(index as usize);
    }

    pub fn insert_line(&mut self, index: i32, line: Line)
    {
        if self.is_locked || !self.is_visible {
            return;
        }
        assert!(index >= 0 , "line out of range");
        if index > self.lines.len() as i32 {
            self.lines.resize(index as usize, Line::new());
        }
        self.lines.insert(index as usize, line);
    }

    pub fn swap_char(&mut self, pos1: Position, pos2: Position) 
    {
        let tmp = self.get_char(pos1);
        self.set_char(pos1, self.get_char(pos2));
        self.set_char(pos2, tmp);
    }
}




#[cfg(test)]
mod tests {
    use crate::{Layer, Line, DosChar, TextAttribute};

    #[test]
    fn test_insert_line() {
        let mut layer = Layer::new();
        let mut line = Line::new();
        line.chars.push(Some(DosChar::from(b'a' as u16, TextAttribute::default())));
        layer.insert_line(10, line);

        assert_eq!(b'a' as u16, layer.lines[10].chars[0].unwrap().char_code);
        assert_eq!(11, layer.lines.len());
        
        layer.insert_line(11, Line::new());
        assert_eq!(12, layer.lines.len());
    }
}
