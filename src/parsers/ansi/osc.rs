use crate::{Buffer, CallbackAction, Caret, EngineResult, ParserError};

use super::{parse_next_number, Parser};

impl Parser {
    pub(super) fn parse_osc(
        &mut self,
        buf: &mut Buffer,
        caret: &mut Caret,
    ) -> EngineResult<CallbackAction> {
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

        if i == 3 && *self.parsed_numbers.first().unwrap() == 8 {
            self.handle_osc_hyperlinks(self.parse_string[3..].to_string(), buf, caret);
            return Ok(CallbackAction::None);
        }

        Err(Box::new(ParserError::UnsupportedOSCSequence(
            self.parse_string.clone(),
        )))
    }

    fn handle_osc_hyperlinks(
        &mut self,
        parse_string: impl Into<String>,
        buf: &mut Buffer,
        caret: &mut Caret,
    ) {
        let url = parse_string.into();
        if url.is_empty() {
            caret.attribute.set_is_underlined(false);
            let mut p = self.hyper_links.pop().unwrap();
            let cp = caret.get_position();
            if cp.y == p.position.y {
                p.length = cp.x - p.position.x;
            } else {
                p.length = buf.get_width() - p.position.x
                    + (cp.y - p.position.y) * buf.get_width()
                    + p.position.x;
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
