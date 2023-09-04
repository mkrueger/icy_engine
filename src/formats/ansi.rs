use crate::ascii::CP437_TO_UNICODE;
use crate::Buffer;
use crate::{Position, TextAttribute};

use super::SaveOptions;

const FG_TABLE: [&[u8; 2]; 8] = [b"30", b"34", b"32", b"36", b"31", b"35", b"33", b"37"];
const BG_TABLE: [&[u8; 2]; 8] = [b"40", b"44", b"42", b"46", b"41", b"45", b"43", b"47"];

pub fn convert_to_ansi_data(
    buf: &Buffer,
    current_layer: usize,
    modern_terminal_output: bool,
) -> Vec<u8> {
    let mut result = Vec::new();
    let mut last_attr = TextAttribute::default();
    let mut pos = Position::default();
    let layer = &buf.layers[current_layer];
    let height = layer.get_line_count();
    let mut first_char = true;
    while pos.y < height {
        let line_length = if modern_terminal_output {
            layer.get_width()
        } else {
            layer.get_line_length(pos.y)
        };

        while pos.x < line_length {
            let mut space_count = 0;
            let mut ch = layer.get_char(pos);
            let mut cur_attr = ch.attribute;

            // doesn't work well with unix terminal - background color needs to be painted.
            if !modern_terminal_output {
                while (ch.ch == ' ' || ch.ch == '\0')
                    && ch.attribute.get_background() == 0
                    && pos.x < line_length
                {
                    space_count += 1;
                    pos.x += 1;
                    ch = layer.get_char(pos);
                }
            }

            // optimize color output for empty space lines.
            if space_count > 0 && cur_attr.get_background() == ch.attribute.get_background() {
                cur_attr = ch.attribute;
            }

            if last_attr != cur_attr || first_char {
                if modern_terminal_output {
                    if last_attr.get_foreground() != cur_attr.get_foreground() || first_char {
                        result.extend_from_slice(b"\x1b[38;2;");
                        let color = buf.palette.colors[cur_attr.get_foreground() as usize];
                        let (r, g, b) = color.get_rgb();
                        result.extend_from_slice(r.to_string().as_bytes());
                        result.push(b';');
                        result.extend_from_slice(g.to_string().as_bytes());
                        result.push(b';');
                        result.extend_from_slice(b.to_string().as_bytes());
                        result.push(b'm');
                    }

                    if last_attr.get_background() != cur_attr.get_background() || first_char {
                        result.extend_from_slice(b"\x1b[48;2;");
                        let color = buf.palette.colors[cur_attr.get_background() as usize];
                        let (r, g, b) = color.get_rgb();
                        result.extend_from_slice(r.to_string().as_bytes());
                        result.push(b';');
                        result.extend_from_slice(g.to_string().as_bytes());
                        result.push(b';');
                        result.extend_from_slice(b.to_string().as_bytes());
                        result.push(b'm');
                    }
                } else {
                    let mut write_24bit_fore_color = false;
                    let mut write_24bit_back_color = false;
                    result.extend_from_slice(b"\x1b[");

                    let mut wrote_part = false;

                    // handle bold change
                    if (!last_attr.is_bold() || first_char) && cur_attr.is_bold() {
                        // if blinking is turned off "0;" will be written which would reset the bold state here
                        // bold state is set again after blink reset.
                        if (!last_attr.is_blinking() && !first_char) || cur_attr.is_blinking() {
                            result.push(b'1');
                            wrote_part = true;
                        }
                    } else if (last_attr.is_bold() || first_char) && !cur_attr.is_bold() {
                        result.push(b'0');
                        last_attr = TextAttribute::default();
                        first_char = false; // attribute set.
                        wrote_part = true;
                    }

                    // handle blink change
                    if (!last_attr.is_blinking() || first_char) && cur_attr.is_blinking() {
                        if wrote_part {
                            result.push(b';');
                        }
                        result.push(b'5');
                        wrote_part = true;
                    } else if (last_attr.is_blinking() || first_char) && !cur_attr.is_blinking() {
                        if wrote_part {
                            result.push(b';');
                        }
                        result.push(b'0');
                        if cur_attr.is_bold() || first_char {
                            result.extend_from_slice(b";1");
                        }
                        last_attr = TextAttribute::default();
                        wrote_part = true;
                    }

                    // color changes
                    if last_attr.get_foreground() != cur_attr.get_foreground() {
                        let fg = cur_attr.get_foreground() as usize;
                        if fg < FG_TABLE.len() {
                            if wrote_part {
                                result.push(b';');
                            }
                            result.extend_from_slice(FG_TABLE[fg]);
                            wrote_part = true;
                        } else if let Some(col) = get_extended_color(buf, fg) {
                            if wrote_part {
                                result.push(b';');
                            }
                            result.extend_from_slice(b"38;5;");
                            result.extend_from_slice(col.to_string().as_bytes());
                            wrote_part = true;
                        } else {
                            write_24bit_fore_color = true;
                        }
                    }
                    if last_attr.get_background() != cur_attr.get_background() {
                        let bg = cur_attr.get_background() as usize;
                        if bg < BG_TABLE.len() {
                            if wrote_part {
                                result.push(b';');
                            }
                            result.extend_from_slice(BG_TABLE[bg]);
                        } else if let Some(col) = get_extended_color(buf, bg) {
                            if wrote_part {
                                result.push(b';');
                            }
                            result.extend_from_slice(b"48;5;");
                            result.extend_from_slice(col.to_string().as_bytes());
                        } else {
                            write_24bit_back_color = true;
                        }
                    }
                    result.push(b'm');

                    if write_24bit_fore_color {
                        result.extend_from_slice(b"\x1b[1;");
                        let color = buf.palette.colors[cur_attr.get_foreground() as usize];
                        let (r, g, b) = color.get_rgb();
                        result.extend_from_slice(r.to_string().as_bytes());
                        result.push(b';');
                        result.extend_from_slice(g.to_string().as_bytes());
                        result.push(b';');
                        result.extend_from_slice(b.to_string().as_bytes());
                        result.push(b't');
                    }

                    if write_24bit_back_color {
                        result.extend_from_slice(b"\x1b[0;");
                        let color = buf.palette.colors[cur_attr.get_background() as usize];
                        let (r, g, b) = color.get_rgb();
                        result.extend_from_slice(r.to_string().as_bytes());
                        result.push(b';');
                        result.extend_from_slice(g.to_string().as_bytes());
                        result.push(b';');
                        result.extend_from_slice(b.to_string().as_bytes());
                        result.push(b't');
                    }
                }
                last_attr = cur_attr;
            }

            first_char = false;

            if space_count > 0 {
                if space_count < 5 {
                    result.resize(result.len() + space_count, b' ');
                } else {
                    result.extend_from_slice(b"\x1b[");
                    push_int(&mut result, space_count);
                    result.push(b'C');
                }
                continue;
            }
            if modern_terminal_output {
                if ch.ch == '\0' {
                    result.push(b' ');
                } else {
                    let uni_ch = CP437_TO_UNICODE[ch.ch as usize].to_string();
                    result.extend(uni_ch.as_bytes());
                }
            } else {
                result.push(if ch.ch == '\0' { b' ' } else { ch.ch as u8 });
            }
            pos.x += 1;
        }
        // do not end with eol except for terminal support.
        if modern_terminal_output {
            result.extend_from_slice(b"\x1b[0m");
            result.push(10);
            first_char = true;
        } else if pos.x < layer.get_width() && pos.y + 1 < height {
            result.push(13);
            result.push(10);
        }

        pos.x = 0;
        pos.y += 1;
    }

    result
}

/// .
///
/// # Errors
///
/// This function will return an error if .
pub fn convert_to_ans(buf: &Buffer, options: &SaveOptions) -> std::io::Result<Vec<u8>> {
    let mut result = Vec::new();

    match options.screen_preparation {
        super::ScreenPreperation::None => {}
        super::ScreenPreperation::ClearScreen => {
            result.extend_from_slice(b"\x1b[2J");
        }
        super::ScreenPreperation::Home => {
            result.extend_from_slice(b"\x1b[1;1H");
        }
    }

    result.extend(convert_to_ansi_data(buf, 0, options.modern_terminal_output));

    if options.save_sauce {
        buf.write_sauce_info(crate::SauceFileType::Ansi, &mut result)?;
    }
    Ok(result)
}

fn get_extended_color(buf: &Buffer, color: usize) -> Option<usize> {
    let color = buf.palette.colors[color];
    (0..crate::XTERM_256_PALETTE.len()).find(|&i| color == crate::XTERM_256_PALETTE[i].1)
}

fn push_int(result: &mut Vec<u8>, number: usize) {
    result.extend_from_slice(number.to_string().as_bytes());
}

pub fn get_save_sauce_default_ans(buf: &Buffer) -> (bool, String) {
    if buf.get_width() != 80 {
        return (true, "width != 80".to_string());
    }

    if buf.has_sauce_relevant_data() {
        return (true, String::new());
    }

    (false, String::new())
}
