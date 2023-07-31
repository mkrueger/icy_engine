use std::io;

use crate::{Buffer, Position, TextAttribute};

use super::SaveOptions;

/// Starts Avatar command
const AVT_CMD: u8 = 22;

/// clear the current window and set current attribute to default.
const AVT_CLR: u8 = 12;

///  Read two bytes from the modem. Send the first one to the screen as many times as the binary value
///  of the second one. This is the exception where the two bytes may have their high bit set. Do not reset it here!
// const AVT_REP: u8 = 25;

pub enum AvtReadState {
    Chars,
    RepeatChars,
    ReadCommand,
    MoveCursor,
    ReadColor,
}

pub fn convert_to_avt(buf: &Buffer, options: &SaveOptions) -> io::Result<Vec<u8>> {
    let mut result = Vec::new();
    let mut last_attr = TextAttribute::default();
    let mut pos = Position::default();
    let height = buf.get_real_buffer_height() as i32;
    let mut first_char = true;

    match options.screen_preparation {
        super::ScreenPreperation::None => {}
        super::ScreenPreperation::ClearScreen => {
            result.push(AVT_CLR);
        }
        super::ScreenPreperation::Home => {
            result.push(AVT_CMD);
            result.push(8); // move caret
            result.push(1); // x
            result.push(1); // y
        }
    }

    // TODO: implement repeat pattern compression (however even TheDraw never bothered to implement this cool RLE from fsc0037)
    while pos.y < height {
        let line_length = buf.get_line_length(pos.y);

        while pos.x < line_length {
            let mut repeat_count = 1;
            let mut ch = buf.get_char(pos).unwrap_or_default();

            while pos.x < buf.get_buffer_width() as i32 - 3
                && ch == buf.get_char(pos + Position::new(1, 0)).unwrap_or_default()
            {
                repeat_count += 1;
                pos.x += 1;
                ch = buf.get_char(pos).unwrap_or_default();
            }

            if first_char || ch.attribute != last_attr {
                result.push(22);
                result.push(1);
                result.push(ch.attribute.as_u8(buf.buffer_type));
                last_attr = ch.attribute;
            }
            first_char = false;

            if repeat_count > 1 {
                if repeat_count < 4 && (ch.ch != '\x16' && ch.ch != '\x0C' && ch.ch != '\x19') {
                    result.resize(result.len() + repeat_count, ch.ch as u8);
                } else {
                    result.push(25);
                    result.push(ch.ch as u8);
                    result.push(repeat_count as u8);
                }
                pos.x += 1;

                continue;
            }

            // avt control codes need to be represented as repeat once.
            if ch.ch == '\x16' || ch.ch == '\x0C' || ch.ch == '\x19' {
                result.push(25);
                result.push(ch.ch as u8);
                result.push(1);
            } else {
                result.push(if ch.ch == '\0' { b' ' } else { ch.ch as u8 });
            }
            pos.x += 1;
        }
        // do not end with eol
        if pos.x < buf.get_buffer_width() as i32 && pos.y + 1 < height {
            result.push(13);
            result.push(10);
        }

        pos.x = 0;
        pos.y += 1;
    }
    if options.save_sauce {
        buf.write_sauce_info(&crate::SauceFileType::Avatar, &mut result)?;
    }
    Ok(result)
}

pub fn get_save_sauce_default_avt(buf: &Buffer) -> (bool, String) {
    if buf.get_buffer_width() != 80 {
        return (true, "width != 80".to_string());
    }

    if buf.has_sauce_relevant_data() {
        return (true, String::new());
    }

    (false, String::new())
}

#[cfg(test)]
mod tests {
    use crate::{Buffer, Position, SaveOptions};

    #[test]
    fn test_clear() {
        let buf =
            Buffer::from_bytes(&std::path::PathBuf::from("test.avt"), false,&[b'X', 12, b'X']).unwrap();
        assert_eq!(1, buf.get_buffer_height());
        assert_eq!(1, buf.get_buffer_width());
    }

    #[test]
    fn test_repeat() {
        let buf = Buffer::from_bytes(
            &std::path::PathBuf::from("test.avt"),
            false,
            &[b'X', 25, b'b', 3, b'X'],
        )
        .unwrap();
        assert_eq!(1, buf.get_buffer_height());
        assert_eq!(5, buf.get_buffer_width());
        assert_eq!(
            b'X',
            buf.get_char(Position::new(0, 0)).unwrap_or_default().ch as u8
        );
        assert_eq!(
            b'b',
            buf.get_char(Position::new(1, 0)).unwrap_or_default().ch as u8
        );
        assert_eq!(
            b'b',
            buf.get_char(Position::new(2, 0)).unwrap_or_default().ch as u8
        );
        assert_eq!(
            b'b',
            buf.get_char(Position::new(3, 0)).unwrap_or_default().ch as u8
        );
        assert_eq!(
            b'X',
            buf.get_char(Position::new(4, 0)).unwrap_or_default().ch as u8
        );
    }

    #[test]
    fn test_zero_repeat() {
        let buf =
            Buffer::from_bytes(&std::path::PathBuf::from("test.avt"), false,&[25, b'b', 0]).unwrap();
        assert_eq!(0, buf.get_buffer_height());
        assert_eq!(0, buf.get_buffer_width());
    }

    #[test]
    fn test_linebreak_bug() {
        let buf = Buffer::from_bytes(
            &std::path::PathBuf::from("test.avt"),
            false,
            &[
                12, 22, 1, 8, 32, 88, 22, 1, 15, 88, 25, 32, 4, 88, 22, 1, 8, 88, 32, 32, 32, 22,
                1, 3, 88, 88, 22, 1, 57, 88, 88, 88, 25, 88, 7, 22, 1, 9, 25, 88, 4, 22, 1, 25, 88,
                88, 88, 88, 88, 88, 22, 1, 1, 25, 88, 13,
            ],
        )
        .unwrap();
        assert_eq!(1, buf.get_buffer_height());
        assert_eq!(47, buf.get_buffer_width());
    }

    fn output_avt(data: &[u8]) -> Vec<u8> {
        let mut result = Vec::new();
        let mut prev = 0;

        for d in data {
            match d {
                12 => result.extend_from_slice(b"^L"),
                25 => result.extend_from_slice(b"^Y"),
                22 => result.extend_from_slice(b"^V"),
                _ => {
                    if prev == 22 {
                        match d {
                            1 => result.extend_from_slice(b"<SET_COLOR>"),
                            2 => result.extend_from_slice(b"<BLINK_ON>"),
                            3 => result.extend_from_slice(b"<MOVE_UP>"),
                            4 => result.extend_from_slice(b"<MOVE_DOWN>"),
                            5 => result.extend_from_slice(b"<MOVE_RIGHT"),
                            6 => result.extend_from_slice(b"<MOVE_LEFT>"),
                            7 => result.extend_from_slice(b"<CLR_EOL>"),
                            8 => result.extend_from_slice(b"<GOTO_XY>"),
                            _ => result.extend_from_slice(b"<UNKNOWN_CMD>"),
                        }
                        prev = *d;
                        continue;
                    }

                    result.push(*d);
                }
            }
            prev = *d;
        }
        result
    }

    fn test_avt(data: &[u8]) {
        let buf = Buffer::from_bytes(&std::path::PathBuf::from("test.avt"), false, data).unwrap();
        let converted = super::convert_to_avt(&buf, &SaveOptions::new()).unwrap();

        // more gentle output.
        let b: Vec<u8> = output_avt(&converted);
        let converted = String::from_utf8_lossy(b.as_slice());

        let b: Vec<u8> = output_avt(data);
        let expected = String::from_utf8_lossy(b.as_slice());

        assert_eq!(expected, converted);
    }

    #[test]
    fn test_char_compression() {
        let data = b"\x16\x01\x07A-A--A---A\x19-\x04A\x19-\x05A\x19-\x06A\x19-\x07A";
        test_avt(data);
    }
}
