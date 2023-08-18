use crate::{EngineResult, Palette, ParserError, Position, Rectangle, Size};

#[derive(Clone, Debug, Copy)]
pub enum SixelState {
    Read,
    ReadColor,
    ReadSize,
    Repeat,
}

#[derive(Clone, Debug, Default)]
pub struct Sixel {
    pub position: Position,

    pub vertical_scale: i32,
    pub horizontal_scale: i32,
    pub picture_data: Vec<u8>,

    pub(crate) width: u32,
    pub(crate) height: u32,
}

struct SixelParser {
    pos: Position,
    current_sixel_palette: Palette,
    current_sixel_color: i32,
    sixel_cursor: Position,
    parsed_numbers: Vec<i32>,
    state: SixelState,
    picture_data: Vec<Vec<u8>>,
    vertical_scale: i32,
    horizontal_scale: i32,

    height_set: bool,
}

impl Default for SixelParser {
    fn default() -> Self {
        Self {
            pos: Position::default(),
            current_sixel_palette: Palette::default(),
            current_sixel_color: 0,
            sixel_cursor: Position::default(),
            parsed_numbers: Vec::new(),
            state: SixelState::Read,
            height_set: false,
            picture_data: Vec::new(),
            vertical_scale: 1,
            horizontal_scale: 1,
        }
    }
}

impl SixelParser {
    pub fn parse_from(&mut self, _default_bg_color: [u8; 4], data: &str) -> EngineResult<Sixel> {
        for ch in data.chars() {
            self.parse_char(ch)?;
        }
        self.parse_char('#')?;
        let mut picture_data = Vec::new();
        for y in 0..self.height() {
            let line = &self.picture_data[y as usize];
            picture_data.extend(line);
        }
        Ok(Sixel {
            position: self.pos,
            vertical_scale: self.vertical_scale,
            horizontal_scale: self.horizontal_scale,
            picture_data,
            width: self.width(),
            height: self.height(),
        })
    }

    pub fn width(&self) -> u32 {
        if let Some(first_line) = self.picture_data.get(0) {
            first_line.len() as u32 / 4
        } else {
            0
        }
    }

    pub fn height(&self) -> u32 {
        self.picture_data.len() as u32
    }

    fn parse_char(&mut self, ch: char) -> EngineResult<bool> {
        match self.state {
            SixelState::Read => {
                self.parse_sixel_data(ch)?;
            }
            SixelState::ReadColor => {
                if ch.is_ascii_digit() {
                    let d = match self.parsed_numbers.pop() {
                        Some(number) => number,
                        _ => 0,
                    };
                    self.parsed_numbers.push(d * 10 + ch as i32 - b'0' as i32);
                } else if ch == ';' {
                    self.parsed_numbers.push(0);
                } else {
                    if let Some(color) = self.parsed_numbers.first() {
                        self.current_sixel_color = *color;
                    }
                    if self.parsed_numbers.len() > 1 {
                        if self.parsed_numbers.len() != 5 {
                            return Err(Box::new(ParserError::InvalidColorInSixelSequence));
                        }

                        match self.parsed_numbers.get(1).unwrap() {
                            2 => {
                                self.current_sixel_palette.set_color_rgb(
                                    self.current_sixel_color as usize,
                                    (self.parsed_numbers[2] * 255 / 100) as u8,
                                    (self.parsed_numbers[3] * 255 / 100) as u8,
                                    (self.parsed_numbers[4] * 255 / 100) as u8,
                                );
                            }
                            1 => {
                                self.current_sixel_palette.set_color_hsl(
                                    self.current_sixel_color as usize,
                                    self.parsed_numbers[2] as f32 * 360.0
                                        / (2.0 * std::f32::consts::PI),
                                    self.parsed_numbers[4] as f32 / 100.0, // sixel is hls
                                    self.parsed_numbers[3] as f32 / 100.0,
                                );
                            }
                            n => {
                                return Err(Box::new(ParserError::UnsupportedSixelColorformat(*n)));
                            }
                        }
                    }
                    self.parse_sixel_data(ch)?;
                }
            }
            SixelState::ReadSize => {
                if ch.is_ascii_digit() {
                    let d = match self.parsed_numbers.pop() {
                        Some(number) => number,
                        _ => 0,
                    };
                    self.parsed_numbers.push(d * 10 + ch as i32 - b'0' as i32);
                } else if ch == ';' {
                    self.parsed_numbers.push(0);
                } else {
                    if self.parsed_numbers.len() < 2 || self.parsed_numbers.len() > 4 {
                        return Err(Box::new(ParserError::InvalidPictureSize));
                    }
                    self.vertical_scale = self.parsed_numbers[0];
                    self.horizontal_scale = self.parsed_numbers[1];
                    if self.parsed_numbers.len() == 3 {
                        let height = self.parsed_numbers[2];
                        self.picture_data.resize(height as usize, Vec::new());
                        self.height_set = true;
                    }

                    if self.parsed_numbers.len() == 4 {
                        let height = self.parsed_numbers[3];
                        let width = self.parsed_numbers[2];
                        self.picture_data
                            .resize(height as usize, vec![0; 4 * width as usize]);
                        self.height_set = true;
                    }
                    self.state = SixelState::Read;
                    self.parse_sixel_data(ch)?;
                }
            }
            SixelState::Repeat => {
                if ch.is_ascii_digit() {
                    let d = match self.parsed_numbers.pop() {
                        Some(number) => number,
                        _ => 0,
                    };
                    self.parsed_numbers.push(d * 10 + ch as i32 - '0' as i32);
                } else {
                    if let Some(i) = self.parsed_numbers.first() {
                        for _ in 0..*i {
                            self.parse_sixel_data(ch)?;
                        }
                    } else {
                        return Err(Box::new(ParserError::NumberMissingInSixelRepeat));
                    }
                    self.state = SixelState::Read;
                }
            }
        }
        Ok(true)
    }

    fn translate_sixel_to_pixel(&mut self, ch: char) -> EngineResult<()> {
        /*let current_sixel = buf.layers[0].sixels.len() - 1;

        let sixel = &mut buf.layers[0].sixels[current_sixel];*/
        if ch < '?' {
            return Err(Box::new(ParserError::InvalidSixelChar(ch)));
        }
        let mask = ch as u8 - b'?';

        let fg_color = self.current_sixel_palette.colors
            [(self.current_sixel_color as usize) % self.current_sixel_palette.colors.len()];
        let x_pos = self.sixel_cursor.x as usize;
        let y_pos = self.sixel_cursor.y as usize * 6;

        let mut last_line = y_pos + 6;
        if self.height_set && last_line > self.height() as usize {
            last_line = self.height() as usize;
        }

        if self.picture_data.len() < last_line {
            self.picture_data
                .resize(last_line, vec![0; self.width() as usize * 4]);
        }

        for i in 0..6 {
            if mask & (1 << i) != 0 {
                let translated_line = y_pos + i;
                if translated_line >= last_line {
                    break;
                }

                let cur_line = &mut self.picture_data[translated_line];

                let offset = x_pos * 4;
                if cur_line.len() <= offset {
                    cur_line.resize((x_pos + 1) * 4, 0);
                }

                let (r, g, b) = fg_color.get_rgb();
                cur_line[offset] = r;
                cur_line[offset + 1] = g;
                cur_line[offset + 2] = b;
                cur_line[offset + 3] = 0xFF;
            }
        }
        self.sixel_cursor.x += 1;
        Ok(())
    }

    fn parse_sixel_data(&mut self, ch: char) -> EngineResult<()> {
        match ch {
            '#' => {
                self.parsed_numbers.clear();
                self.state = SixelState::ReadColor;
            }
            '!' => {
                self.parsed_numbers.clear();
                self.state = SixelState::Repeat;
            }
            '-' => {
                self.sixel_cursor.x = 0;
                self.sixel_cursor.y += 1;
            }
            '$' => {
                self.sixel_cursor.x = 0;
            }
            '"' => {
                self.parsed_numbers.clear();
                self.state = SixelState::ReadSize;
            }
            _ => {
                if ch > '\x7F' {
                    return Ok(());
                }
                self.translate_sixel_to_pixel(ch)?;
            }
        }
        Ok(())
    }
}

impl Sixel {
    pub fn new(position: Position) -> Self {
        Self {
            position,
            vertical_scale: 1,
            horizontal_scale: 1,
            picture_data: Vec::new(),
            width: 0,
            height: 0,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn get_screen_rect(&self) -> Rectangle {
        let x = self.position.x * 8;
        let y = self.position.y * 16;
        Rectangle {
            start: Position::new(x, y),
            size: Size::new(self.width as i32, self.height as i32),
        }
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn parse_from(
        pos: Position,
        horizontal_scale: i32,
        vertical_scale: i32,
        default_bg_color: [u8; 4],
        data: &str,
    ) -> EngineResult<Self> {
        let mut parser = SixelParser {
            pos,
            vertical_scale,
            horizontal_scale,
            ..SixelParser::default()
        };
        parser.parse_from(default_bg_color, data)
    }
}
