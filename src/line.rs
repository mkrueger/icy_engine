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

    pub fn create(width: usize) -> Self {
        let mut chars = Vec::new();
        chars.resize(width, AttributedChar::invisible());
        Line { chars }
    }

    pub fn get_line_length(&self) -> usize {
        for idx in (0..self.chars.len()).rev() {
            if !self.chars[idx].is_transparent() {
                return idx + 1;
            }
        }
        0
    }

    pub fn insert_char(&mut self, index: usize, char_opt: AttributedChar) {
        if index > self.chars.len() {
            self.chars.resize(index, AttributedChar::invisible());
        }
        self.chars.insert(index, char_opt);
    }

    pub fn set_char(&mut self, index: usize, char: AttributedChar) {
        if index >= self.chars.len() {
            self.chars.resize(index + 1, AttributedChar::invisible());
        }
        self.chars[index] = char;
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
