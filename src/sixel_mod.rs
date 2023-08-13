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

    pub picture_data: Vec<Vec<u8>>,
}

struct SixelParser {
    current_sixel_palette: Palette,
    current_sixel_color: i32,
    sixel_cursor: Position,
    parsed_numbers: Vec<i32>,
    state: SixelState,

    height_set: bool,
}

impl Default for SixelParser {
    fn default() -> Self {
        Self {
            current_sixel_palette: Palette::default(),
            current_sixel_color: 0,
            sixel_cursor: Position::default(),
            parsed_numbers: Vec::new(),
            state: SixelState::Read,
            height_set: false,
        }
    }
}

impl SixelParser {
    pub fn parse_from(
        &mut self,
        sixel: &mut Sixel,
        default_bg_color: [u8; 4],
        data: &str,
    ) -> EngineResult<bool> {
        for ch in data.chars() {
            self.parse_char(sixel, ch)?;
        }
        self.parse_char(sixel, '#')?;
        Ok(true)
    }
    fn parse_char(&mut self, sixel: &mut Sixel, ch: char) -> EngineResult<bool> {
        match self.state {
            SixelState::Read => {
                !self.parse_sixel_data(sixel, ch)?;
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
                    !self.parse_sixel_data(sixel, ch)?;
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
                    sixel.vertical_scale = self.parsed_numbers[0];
                    sixel.horizontal_scale = self.parsed_numbers[1];
                    if self.parsed_numbers.len() == 3 {
                        let height = self.parsed_numbers[2];
                        sixel.picture_data.resize(height as usize, Vec::new());
                        self.height_set = true;
                    }

                    if self.parsed_numbers.len() == 4 {
                        let height = self.parsed_numbers[3];
                        let width = self.parsed_numbers[2];
                        sixel
                            .picture_data
                            .resize(height as usize, vec![0; 4 * width as usize]);
                        self.height_set = true;
                    }
                    self.state = SixelState::Read;
                    self.parse_sixel_data(sixel, ch)?;
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
                            if !self.parse_sixel_data(sixel, ch)? {
                                break;
                            }
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

    fn translate_sixel_to_pixel(&mut self, sixel: &mut Sixel, ch: char) -> EngineResult<()> {
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
        if self.height_set && last_line > sixel.height() as usize {
            last_line = sixel.height() as usize;
        }

        if sixel.picture_data.len() < last_line {
            sixel
                .picture_data
                .resize(last_line, vec![0; sixel.width() as usize * 4]);
        }

        for i in 0..6 {
            if mask & (1 << i) != 0 {
                let translated_line = y_pos + i;
                if translated_line >= last_line {
                    break;
                }

                let cur_line = &mut sixel.picture_data[translated_line];

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

    fn parse_sixel_data(&mut self, sixel: &mut Sixel, ch: char) -> EngineResult<bool> {
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
                    return Ok(false);
                }
                self.translate_sixel_to_pixel(sixel, ch)?;
            }
        }
        Ok(true)
    }
}

impl Sixel {
    pub fn new(position: Position) -> Self {
        Self {
            position,
            vertical_scale: 1,
            horizontal_scale: 1,
            picture_data: Vec::new(),
        }
    }

    pub fn get_rect(&self) -> Rectangle {
        Rectangle {
            start: self.position,
            size: Size::new(self.width() as i32, self.height() as i32),
        }
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

    pub fn parse_from(
        pos: Position,
        vertical_scale: i32,
        horizontal_scale: i32,
        default_bg_color: [u8; 4],
        data: &str,
    ) -> EngineResult<Self> {
        let mut sixel = Self::new(pos);
        sixel.vertical_scale = vertical_scale;
        sixel.horizontal_scale = horizontal_scale;
        SixelParser::default().parse_from(&mut sixel, default_bg_color, data)?;
        Ok(sixel)
    }
}
