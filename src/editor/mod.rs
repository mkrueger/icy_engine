pub mod undo_stack;
use std::sync::{Arc, Mutex};

use i18n_embed_fl::fl;
pub use undo_stack::*;

mod undo_operations;

mod layer_operations;
pub use layer_operations::*;
mod edit_operations;
pub use edit_operations::*;

use crate::{
    ansi, AttributedChar, Buffer, BufferParser, Caret, EngineResult, Position, Selection, Shape,
    TextAttribute, UPosition,
};

pub struct EditState {
    buffer: Buffer,
    caret: Caret,
    selection_opt: Option<Selection>,
    parser: Box<dyn BufferParser>,

    current_layer: usize,

    undo_stack: Arc<Mutex<Vec<Box<dyn UndoOperation>>>>,
    redo_stack: Vec<Box<dyn UndoOperation>>,
}

pub struct AtomicUndoGuard {
    base_count: usize,
    description: String,

    undo_stack: Arc<Mutex<Vec<Box<dyn UndoOperation>>>>,
}

impl AtomicUndoGuard {
    fn new(description: String, undo_stack: Arc<Mutex<Vec<Box<dyn UndoOperation>>>>) -> Self {
        let base_count = undo_stack.lock().unwrap().len();
        Self {
            base_count,
            description,
            undo_stack,
        }
    }
}

impl Drop for AtomicUndoGuard {
    fn drop(&mut self) {
        let count = self.undo_stack.lock().unwrap().len();
        if self.base_count >= count {
            return;
        }

        let stack = self
            .undo_stack
            .lock()
            .unwrap()
            .drain(self.base_count..)
            .collect();

        self.undo_stack
            .lock()
            .unwrap()
            .push(Box::new(undo_operations::AtomicUndo::new(
                self.description.clone(),
                stack,
            )));
    }
}

impl Default for EditState {
    fn default() -> Self {
        Self {
            parser: Box::<ansi::Parser>::default(),
            buffer: Buffer::default(),
            caret: Caret::default(),
            selection_opt: None,
            undo_stack: Arc::new(Mutex::new(Vec::new())),
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

    pub fn get_clipboard_data(&self) -> Option<Vec<u8>> {
        let Some(selection) = &self.selection_opt else {
            return None;
        };

        let mut data = Vec::new();
        if matches!(selection.shape, Shape::Rectangle) {
            data.push(0);
            data.extend(u32::to_le_bytes(selection.size().width as u32));
            data.extend(u32::to_le_bytes(selection.size().height as u32));

            let start = selection.min();
            let end = selection.max();
            println!("{}, {} - {}", selection.size(), start, end);

            for y in start.y..end.y {
                for x in start.x..end.x {
                    let ch = self.buffer.get_char((x, y));
                    data.extend(u16::to_le_bytes(ch.ch as u16));
                    data.extend(u16::to_le_bytes(ch.attribute.attr));
                    data.extend(u16::to_le_bytes(ch.attribute.font_page as u16));
                    data.extend(u32::to_le_bytes(ch.attribute.background_color));
                    data.extend(u32::to_le_bytes(ch.attribute.foreground_color));
                }
            }
        } else {
            // TODO
        }
        Some(data)
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn paste_clipboard_data(&mut self, data: &[u8]) -> EngineResult<()> {
        if data[0] != 0 {
            return Ok(());
        }

        let width = u32::from_le_bytes([data[1], data[2], data[3], data[4]]) as usize;
        let height = u32::from_le_bytes([data[5], data[6], data[7], data[8]]) as usize;
        let _paste = self.begin_atomic_undo(fl!(crate::LANGUAGE_LOADER, "undo-paste"));
        let mut data = &data[9..];

        let pos = self.caret.get_position().as_uposition();

        for y in 0..height {
            for x in 0..width {
                let ch = AttributedChar {
                    ch: unsafe {
                        char::from_u32_unchecked(u16::from_le_bytes([data[0], data[1]]) as u32)
                    },
                    attribute: TextAttribute {
                        attr: u16::from_le_bytes([data[2], data[3]]),
                        font_page: u16::from_le_bytes([data[4], data[5]]) as usize,
                        background_color: u32::from_le_bytes([data[6], data[7], data[8], data[9]]),
                        foreground_color: u32::from_le_bytes([
                            data[10], data[11], data[12], data[13],
                        ]),
                    },
                };
                self.set_char(UPosition::new(x, y) + pos, ch)?;
                data = &data[14..];
            }
        }
        self.selection_opt = None;
        Ok(())
    }

    pub fn get_current_layer(&self) -> usize {
        self.current_layer
    }

    pub fn set_current_layer(&mut self, layer: usize) {
        self.current_layer = layer;
    }

    #[must_use]
    pub fn begin_atomic_undo(&mut self, description: String) -> AtomicUndoGuard {
        self.redo_stack.clear();
        AtomicUndoGuard::new(description, self.undo_stack.clone())
    }

    fn clamp_current_layer(&mut self) {
        self.current_layer = self
            .current_layer
            .clamp(0, self.buffer.layers.len().saturating_sub(1));
    }

    fn push_undo(&mut self, op: Box<dyn UndoOperation>) {
        self.undo_stack.lock().unwrap().push(op);
        self.redo_stack.clear();
    }

    pub fn delete_selection(&mut self) {
        if let Some(selection) = &self.get_selection() {
            let _paste =
                self.begin_atomic_undo(fl!(crate::LANGUAGE_LOADER, "undo-delete-selection"));
            let min = selection.min();
            let max = selection.max();

            for y in min.y..max.y {
                if y < 0 {
                    continue;
                }
                for x in min.x..max.x {
                    if x < 0 {
                        continue;
                    }
                    self.set_char((x, y), AttributedChar::invisible());
                }
            }
            self.clear_selection();
        }
    }
}

impl UndoState for EditState {
    fn undo_description(&self) -> Option<String> {
        self.undo_stack
            .lock()
            .unwrap()
            .last()
            .map(|op| op.get_description())
    }

    fn can_undo(&self) -> bool {
        !self.undo_stack.lock().unwrap().is_empty()
    }

    fn undo(&mut self) -> EngineResult<()> {
        let Some(mut op) = self.undo_stack.lock().unwrap().pop() else {
            return Ok(());
        };

        let res = op.undo(self);
        self.redo_stack.push(op);
        res
    }

    fn redo_description(&self) -> Option<String> {
        self.redo_stack.last().map(|op| op.get_description())
    }

    fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    fn redo(&mut self) -> EngineResult<()> {
        if let Some(mut op) = self.redo_stack.pop() {
            let res = op.redo(self);
            self.undo_stack.lock().unwrap().push(op);
            return res;
        }
        Ok(())
    }
}