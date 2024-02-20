use crate::{
    parsers::{update_buffer, viewdata::Parser},
    Buffer, BufferParser, Caret, Position, TextPane,
};

fn create_mode7_buffer<T: BufferParser>(parser: &mut T, input: &[u8]) -> (Buffer, Caret) {
    let mut buf = Buffer::create((40, 25));
    buf.is_terminal_buffer = true;
    let mut caret = Caret::default();

    update_buffer(&mut buf, &mut caret, parser, input);

    (buf, caret)
}
