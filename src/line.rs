use super::AttributedChar;
#[derive(Clone, Debug, Default)]
pub struct Line {
    pub chars: Vec<AttributedChar>,
}

impl Line {
    pub fn new() -> Self {
        Line::with_capacity(80)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Line {
            chars: Vec::with_capacity(capacity),
        }
    }

    pub fn create(width: u16) -> Self {
        let mut chars = Vec::new();
        chars.resize(width as usize, AttributedChar::default());
        Line { chars }
    }

    pub fn get_line_length(&self) -> usize {
        for i in 0..self.chars.len() {
            let idx = self.chars.len() - 1 - i;
            if self.chars[idx].is_visible() {
                return idx;
            }
        }
        0
    }

    pub fn insert_char(&mut self, index: i32, char_opt: AttributedChar) {
        if index > self.chars.len() as i32 {
            self.chars
                .resize(index as usize, AttributedChar::invisible());
        }
        self.chars.insert(index as usize, char_opt);
    }

    pub fn set_char(&mut self, index: i32, char_opt: AttributedChar) {
        if index >= self.chars.len() as i32 {
            self.chars
                .resize(index as usize + 1, AttributedChar::invisible());
        }
        self.chars[index as usize] = char_opt;
    }
}

#[cfg(test)]
mod tests {
    use crate::{AttributedChar, Line};

    #[test]
    fn test_insert_char() {
        let mut line = Line::new();
        line.insert_char(100, AttributedChar::default());
        assert_eq!(101, line.chars.len());
        line.insert_char(1, AttributedChar::default());
        assert_eq!(102, line.chars.len());
    }

    #[test]
    fn test_set_char() {
        let mut line = Line::new();
        line.set_char(100, AttributedChar::default());
        assert_eq!(101, line.chars.len());
        line.set_char(100, AttributedChar::default());
        assert_eq!(101, line.chars.len());
    }
}
