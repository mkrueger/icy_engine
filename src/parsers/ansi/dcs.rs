use base64::{engine::general_purpose, Engine};

use crate::{BitFont, Buffer, CallbackAction, Caret, EngineResult, ParserError, Sixel, HEX_TABLE};

use super::Parser;

#[derive(Debug, Clone, Copy)]
enum HexMacroState {
    FirstHex,
    SecondHex(char),
    RepeatNumber(i32),
}

impl Parser {
    pub(super) fn execute_dcs(
        &mut self,
        buf: &mut Buffer,
        caret: &mut Caret,
    ) -> EngineResult<CallbackAction> {
        if self.dcs_string.starts_with("CTerm:Font:") {
            return self.load_custom_font(buf);
        }
        let mut i = 0;
        for ch in self.dcs_string.chars() {
            match ch {
                '0'..='9' => {
                    let d = match self.parsed_numbers.pop() {
                        Some(number) => number,
                        _ => 0,
                    };
                    self.parsed_numbers.push(d * 10 + ch as i32 - b'0' as i32);
                }
                ';' => {
                    self.parsed_numbers.push(0);
                }
                _ => {
                    break;
                }
            }
            i += 1;
        }

        if self.dcs_string[i..].starts_with("!z") {
            return self.parse_macro(buf, i + 2);
        }

        if self.dcs_string[i..].starts_with('q') {
            let horizontal_scale = match self.parsed_numbers.first() {
                Some(0 | 1 | 5 | 6) => 2,
                Some(2) => 5,
                Some(3 | 4) => 3,
                _ => 1,
            };

            let bg_color = if let Some(1) = self.parsed_numbers.get(1) {
                [0, 0, 0, 0]
            } else {
                let (r, g, b) = buf.palette.colors[caret.attr.get_background() as usize].get_rgb();
                [0xff, r, g, b]
            };

            let sixel = Sixel::parse_from(
                caret.get_position(),
                1,
                horizontal_scale,
                bg_color,
                &self.dcs_string[i + 1..],
            )?;
            buf.layers[0].sixels.push(sixel);
            buf.layers[0].updated_sixels = true;
            return Ok(CallbackAction::None);
        }

        Err(Box::new(ParserError::UnsupportedDCSSequence(format!(
            "encountered unsupported dcs: '{}'",
            self.dcs_string
        ))))
    }

    fn parse_macro(
        &mut self,
        buf: &mut Buffer,
        start_index: usize,
    ) -> EngineResult<CallbackAction> {
        if let Some(pid) = self.parsed_numbers.first() {
            if let Some(pdt) = self.parsed_numbers.get(1) {
                // 0 - or omitted overwrites macro
                // 1 - clear all macros before defining this macro
                if *pdt == 1 {
                    self.macros.clear();
                }
            }

            match self.parsed_numbers.get(2) {
                Some(0) => {
                    self.parse_macro_sequence(*pid as usize, start_index);
                }
                Some(1) => {
                    self.parse_hex_macro_sequence(*pid as usize, start_index)?;
                }
                _ => {
                    return Err(Box::new(ParserError::UnsupportedDCSSequence(format!(
                        "encountered p3 in macro definition: '{}' only 0 and 1 are valid.",
                        self.dcs_string
                    ))))
                }
            };
            return Ok(CallbackAction::None);
        }
        Err(Box::new(ParserError::UnsupportedDCSSequence(format!(
            "encountered unsupported macro definition: '{}'",
            self.dcs_string
        ))))
    }

    fn parse_macro_sequence(&mut self, id: usize, start_index: usize) {
        self.macros
            .insert(id, self.dcs_string[start_index..].to_string());
    }

    fn parse_hex_macro_sequence(
        &mut self,
        id: usize,
        start_index: usize,
    ) -> EngineResult<CallbackAction> {
        let mut state = HexMacroState::FirstHex;
        let mut read_repeat = false;
        let mut repeat_rec = String::new();
        let mut repeat_number = 0;
        let mut marco_rec = String::new();

        for ch in self.dcs_string[start_index..].chars() {
            match &state {
                HexMacroState::FirstHex => {
                    if ch == ';' && read_repeat {
                        read_repeat = false;
                        (0..repeat_number).for_each(|_| marco_rec.push_str(&repeat_rec));
                        continue;
                    }
                    if ch == '!' {
                        state = HexMacroState::RepeatNumber(0);
                        continue;
                    }
                    state = HexMacroState::SecondHex(ch);
                }
                HexMacroState::SecondHex(first) => {
                    let cc: char =
                        unsafe { char::from_u32_unchecked(ch as u32) }.to_ascii_uppercase();
                    let second = HEX_TABLE.iter().position(|&x| x == cc as u8);
                    let first = HEX_TABLE.iter().position(|&x| x == *first as u8);
                    if let (Some(first), Some(second)) = (first, second) {
                        let cc = unsafe { char::from_u32_unchecked((first * 16 + second) as u32) };
                        if read_repeat {
                            repeat_rec.push(cc);
                        } else {
                            marco_rec.push(cc);
                        }
                        state = HexMacroState::FirstHex;
                    } else {
                        return Err(Box::new(ParserError::Error(
                            "Invalid hex number in macro sequence".to_string(),
                        )));
                    }
                    state = HexMacroState::FirstHex;
                }
                HexMacroState::RepeatNumber(n) => {
                    if ch.is_ascii_digit() {
                        state = HexMacroState::RepeatNumber(*n * 10 + ch as i32 - b'0' as i32);
                        continue;
                    }
                    if ch == ';' {
                        repeat_number = *n;
                        repeat_rec.clear();
                        read_repeat = true;
                        state = HexMacroState::FirstHex;
                        continue;
                    }
                    return Err(Box::new(ParserError::Error(format!(
                        "Invalid end of repeat number {ch}"
                    ))));
                }
            }
        }
        if read_repeat {
            read_repeat = false;
            (0..repeat_number).for_each(|_| marco_rec.push_str(&repeat_rec));
        }

        self.macros.insert(id, marco_rec);

        Ok(CallbackAction::None)
    }

    fn load_custom_font(&mut self, buf: &mut Buffer) -> EngineResult<CallbackAction> {
        let start_index = "CTerm:Font:".len();
        if let Some(idx) = self.dcs_string[start_index..].find(':') {
            let idx = idx + start_index;

            if let Ok(num) = self.dcs_string[start_index..idx].parse::<usize>() {
                if let Ok(font_data) =
                    general_purpose::STANDARD.decode(self.dcs_string[idx + 1..].as_bytes())
                {
                    if let Ok(font) = BitFont::from_bytes(format!("custom font {num}"), &font_data)
                    {
                        buf.set_font(num, font);
                        return Ok(CallbackAction::None);
                    }
                }
            }
            return Ok(CallbackAction::None);
        }

        Err(Box::new(ParserError::UnsupportedDCSSequence(format!(
            "invalid custom font in dcs: {}",
            self.dcs_string
        ))))
    }
}

/*
            EngineState::ReadSixel(state) => {
                match state {
                    SixelState::EndSequence => {
                        let current_sixel = buf.layers[0].sixels.len() - 1;
                        let layer = &mut buf.layers[0];
                        let new_sixel_rect = layer.sixels[current_sixel].get_rect();
                        layer.sixels[current_sixel].read_status = SixelReadStatus::Finished;

                        let char_width = 8;
                        let char_height = 16;
                        // Draw Sixel upon each other.
                        if current_sixel > 0 {
                            for i in 0..current_sixel {
                                let old_sixel_rect = layer.sixels[i].get_rect();
                                if old_sixel_rect.start.x <= new_sixel_rect.start.x
                                    && new_sixel_rect.start.x * char_width
                                        + new_sixel_rect.size.width
                                        <= old_sixel_rect.start.x * char_width
                                            + old_sixel_rect.size.width
                                    && old_sixel_rect.start.y <= new_sixel_rect.start.y
                                    && new_sixel_rect.start.y * char_height
                                        + new_sixel_rect.size.height
                                        <= old_sixel_rect.start.y * char_height
                                            + old_sixel_rect.size.height
                                {
                                    let replace_sixel = layer.sixels.pop().unwrap();

                                    let start_y = ((new_sixel_rect.start.y
                                        - old_sixel_rect.start.y)
                                        * char_height)
                                        as usize;
                                    let start_x = ((new_sixel_rect.start.x
                                        - old_sixel_rect.start.x)
                                        * char_width)
                                        as usize;
                                    let sx = &mut layer.sixels[i];

                                    if sx.picture_data.len() < new_sixel_rect.size.height as usize {
                                        sx.picture_data
                                            .resize(new_sixel_rect.size.height as usize, vec![]);
                                    }

                                    let end_y = start_y + new_sixel_rect.size.height as usize;
                                    let end_x = start_x + new_sixel_rect.size.width as usize;

                                    for y in start_y..end_y {
                                        if sx.picture_data[y].len() < end_x - start_x {
                                            sx.picture_data[y]
                                                .resize(end_x - start_x, Color::default());
                                        }

                                        for x in start_x..end_x {
                                            let line = &replace_sixel.picture_data[y - start_y];
                                            if line.len() > x - start_x {
                                                sx.picture_data[y][x] = line[x - start_x];
                                            }
                                        }
                                    }
                                    sx.read_status = SixelReadStatus::Updated;
                                }
                            }
                        }

                        if ch == '\\' {
                            self.state = EngineState::Default;
                        } else {
                            return Err(Box::new(ParserError::UnexpectedSixelEnd(ch)));
                        }
                    }

                }
            }
*/
