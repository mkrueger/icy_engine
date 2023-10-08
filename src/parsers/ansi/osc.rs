use regex::Regex;

use crate::{Buffer, CallbackAction, Caret, EngineResult, ParserError};

use super::{parse_next_number, Parser};
lazy_static::lazy_static! {
    static ref OSC_PALETTE: Regex = Regex::new(r"(\d+)?;[rR][gG][bB]:([0-9a-fA-F]{2})/([0-9a-fA-F]{2})/([0-9a-fA-F]{2})").unwrap();
}
impl Parser {
    pub(super) fn parse_osc(&mut self, buf: &mut Buffer, caret: &mut Caret) -> EngineResult<CallbackAction> {
        let mut i = 0;
        for ch in self.parse_string.chars() {
            match ch {
                '0'..='9' => {
                    let d = match self.parsed_numbers.pop() {
                        Some(number) => number,
                        _ => 0,
                    };
                    self.parsed_numbers.push(parse_next_number(d, ch as u8));
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

        if !self.parsed_numbers.is_empty() && self.parsed_numbers[0] == 4 {
            for a in OSC_PALETTE.captures_iter(&self.parse_string) {
                let color = a.get(1).unwrap().as_str().parse::<u32>()?;
                if color > 255 {
                    log::error!("Invalid color index: {}", color);
                    continue;
                }
                let r = u8::from_str_radix(a.get(2).unwrap().as_str(), 16)?;
                let g = u8::from_str_radix(a.get(3).unwrap().as_str(), 16)?;
                let b = u8::from_str_radix(a.get(4).unwrap().as_str(), 16)?;
                buf.palette.set_color_rgb(color, r, g, b);
            }
            return Ok(CallbackAction::Update);
        }

        if i == 3 && *self.parsed_numbers.first().unwrap() == 8 {
            self.handle_osc_hyperlinks(self.parse_string[3..].to_string(), buf, caret);
            return Ok(CallbackAction::NoUpdate);
        }

        Err(ParserError::UnsupportedOSCSequence(self.parse_string.clone()).into())
    }

    fn handle_osc_hyperlinks(&mut self, parse_string: impl Into<String>, buf: &mut Buffer, caret: &mut Caret) {
        let url = parse_string.into();
        if url.is_empty() {
            caret.attribute.set_is_underlined(false);
            let mut p = self.hyper_links.pop().unwrap();
            let cp = caret.get_position();
            if cp.y == p.position.y {
                p.length = cp.x - p.position.x;
            } else {
                p.length = buf.terminal_state.get_width() - p.position.x + (cp.y - p.position.y) * buf.terminal_state.get_width() + p.position.x;
            }
            buf.layers[0].add_hyperlink(p);
        } else {
            caret.attribute.set_is_underlined(true);
            self.hyper_links.push(crate::HyperLink {
                url: Some(url),
                position: caret.get_position(),
                length: 0,
            });
        }
    }
}
