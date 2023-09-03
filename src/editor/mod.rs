pub mod undo_stack;
pub use undo_stack::*;

mod undo_operations;

mod editor_operations;
pub use editor_operations::*;

use crate::{ansi, Buffer, BufferParser, Caret, Position, Selection, Shape};

pub struct EditState {
    buffer: Buffer,
    caret: Caret,
    selection_opt: Option<Selection>,
    parser: Box<dyn BufferParser>,

    current_layer: usize,

    atomic_undo_stack: Vec<(usize, String)>,
    undo_stack: Vec<Box<dyn UndoOperation>>,
    redo_stack: Vec<Box<dyn UndoOperation>>,
}

impl Default for EditState {
    fn default() -> Self {
        Self {
            parser: Box::<ansi::Parser>::default(),
            buffer: Buffer::default(),
            caret: Caret::default(),
            selection_opt: None,
            atomic_undo_stack: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            current_layer: 0,
        }
    }
}

impl EditState {
    pub fn from_buffer(buffer: Buffer) -> Self {
        Self {
            buffer,
            ..Default::default()
        }
    }

    pub fn set_parser(&mut self, parser: Box<dyn BufferParser>) {
        self.parser = parser;
    }

    pub fn get_parser(&self) -> &dyn BufferParser {
        &*self.parser
    }

    pub fn set_buffer(&mut self, buffer: Buffer) {
        self.buffer = buffer;
    }

    pub fn get_buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn get_buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    pub fn get_caret(&self) -> &Caret {
        &self.caret
    }

    pub fn get_caret_mut(&mut self) -> &mut Caret {
        &mut self.caret
    }

    pub fn get_buffer_and_caret_mut(
        &mut self,
    ) -> (&mut Buffer, &mut Caret, &mut Box<dyn BufferParser>) {
        (&mut self.buffer, &mut self.caret, &mut self.parser)
    }

    pub fn get_selection(&self) -> Option<Selection> {
        self.selection_opt
    }

    pub fn set_selection(&mut self, sel: Selection) {
        self.selection_opt = Some(sel);
    }

    pub fn clear_selection(&mut self) {
        self.selection_opt = None;
    }

    pub fn get_copy_text(&mut self) -> Option<String> {
        let Some(selection) = &self.selection_opt else {
            return None;
        };

        let mut res = String::new();
        if matches!(selection.shape, Shape::Rectangle) {
            let start = selection.min();
            let end = selection.max();
            for y in start.y..=end.y {
                for x in start.x..end.x {
                    let ch = self.buffer.get_char((x, y));
                    res.push(self.parser.convert_to_unicode(ch));
                }
                res.push('\n');
            }
        } else {
            let (start, end) = if selection.anchor.as_position() < selection.lead.as_position() {
                (selection.anchor.as_position(), selection.lead.as_position())
            } else {
                (selection.lead.as_position(), selection.anchor.as_position())
            };
            if start.y == end.y {
                for x in start.x..end.x {
                    let ch = self.buffer.get_char(Position::new(x, start.y));
                    res.push(self.parser.convert_to_unicode(ch));
                }
            } else {
                for x in start.x..(self.buffer.get_line_length(start.y as usize) as i32) {
                    let ch = self.buffer.get_char(Position::new(x, start.y));
                    res.push(self.parser.convert_to_unicode(ch));
                }
                res.push('\n');
                for y in start.y + 1..end.y {
                    for x in 0..(self.buffer.get_line_length(y as usize) as i32) {
                        let ch = self.buffer.get_char(Position::new(x, y));
                        res.push(self.parser.convert_to_unicode(ch));
                    }
                    res.push('\n');
                }
                for x in 0..end.x {
                    let ch = self.buffer.get_char(Position::new(x, end.y));
                    res.push(self.parser.convert_to_unicode(ch));
                }
            }
        }
        self.selection_opt = None;
        Some(res)
    }

    pub fn get_current_layer(&self) -> usize {
        self.current_layer
    }

    pub fn set_current_layer(&mut self, layer: usize) {
        self.current_layer = layer;
    }

    fn begin_atomic_undo(&mut self, description: String) {
        self.atomic_undo_stack
            .push((self.undo_stack.len(), description));
    }

    fn end_atomic_undo(&mut self) {
        let (base_count, description) = self.atomic_undo_stack.pop().unwrap();
        let count = self.undo_stack.len();
        if base_count == count {
            return;
        }

        let mut stack = Vec::new();
        while base_count < self.undo_stack.len() {
            let op = self.undo_stack.pop().unwrap();
            stack.push(op);
        }
        self.undo_stack
            .push(Box::new(undo_operations::AtomicUndo::new(
                description,
                stack,
            )));
    }
}

impl UndoState for EditState {
    fn undo_description(&self) -> Option<String> {
        self.undo_stack.last().map(|op| op.get_description())
    }

    fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    fn undo(&mut self) {
        if let Some(op) = self.undo_stack.pop() {
            op.undo(self);
            self.redo_stack.push(op);
        }
    }

    fn redo_description(&self) -> Option<String> {
        self.redo_stack.last().map(|op| op.get_description())
    }

    fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    fn redo(&mut self) {
        if let Some(op) = self.redo_stack.pop() {
            op.redo(self);
            self.undo_stack.push(op);
        }
    }
}
