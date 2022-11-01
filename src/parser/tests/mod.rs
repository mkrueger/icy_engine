mod buffer_operation_tests;
mod ansi_parser_tests;
mod ascii_parser_tests;

use crate::{Buffer, Caret, BufferParser};

fn create_buffer<T: BufferParser>(parser: &mut T, input: &[u8]) -> (Buffer, Caret) 
{
    let mut buf = Buffer::create(80, 25);
    let mut caret  = Caret::new();
    // remove editing layer
    buf.layers.remove(0);
    buf.layers[0].is_locked = false;
    buf.layers[0].is_transparent = false;
    
    update_buffer(&mut buf, &mut caret, parser, input);
    
    (buf, caret)
}

fn update_buffer<T: BufferParser>(buf: &mut Buffer, caret: &mut Caret, parser: &mut T, input: &[u8])
{
    for b in input {
        parser.print_char(buf,caret, *b).unwrap();
    }
}