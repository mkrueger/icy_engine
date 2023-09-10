pub mod undo_stack;
use std::sync::{Arc, Mutex};

pub use undo_stack::*;

mod undo_operations;

mod editor_error;
pub use editor_error::*;

mod layer_operations;
pub use layer_operations::*;
mod edit_operations;
pub use edit_operations::*;
mod area_operations;
pub use area_operations::*;

use crate::{
    ansi, Buffer, BufferParser, Caret, EngineResult, Layer, Position, Selection, Shape, TextPane,
};

pub struct EditState {
    buffer: Buffer,
    caret: Caret,
    selection_opt: Option<Selection>,
    parser: Box<dyn BufferParser>,

    current_layer: usize,
    outline_style: usize,
    mirror_mode: bool,

    undo_stack: Arc<Mutex<Vec<Box<dyn UndoOperation>>>>,
    redo_stack: Vec<Box<dyn UndoOperation>>,
}

pub struct AtomicUndoGuard {
    base_count: usize,
    description: String,
    operation_type: OperationType,

    undo_stack: Arc<Mutex<Vec<Box<dyn UndoOperation>>>>,
}

impl AtomicUndoGuard {
    fn new(
        description: String,
        undo_stack: Arc<Mutex<Vec<Box<dyn UndoOperation>>>>,
        operation_type: OperationType,
    ) -> Self {
        let base_count = undo_stack.lock().unwrap().len();
        Self {
            base_count,
            description,
            operation_type,
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
        let stack = Arc::new(Mutex::new(stack));
        self.undo_stack
            .lock()
            .unwrap()
            .push(Box::new(undo_operations::AtomicUndo::new(
                self.description.clone(),
                stack,
                self.operation_type,
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
            outline_style: 0,
            mirror_mode: false,
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

    pub fn get_cur_layer(&self) -> Option<&Layer> {
        self.buffer.layers.get(self.current_layer)
    }

    pub fn get_cur_layer_mut(&mut self) -> Option<&mut Layer> {
        self.buffer.layers.get_mut(self.current_layer)
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

    pub fn set_selection(&mut self, sel: impl Into<Selection>) {
        self.selection_opt = Some(sel.into());
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
            let (start, end) = if selection.anchor < selection.lead {
                (selection.anchor, selection.lead)
            } else {
                (selection.lead, selection.anchor)
            };
            if start.y == end.y {
                for x in start.x..end.x {
                    let ch = self.buffer.get_char(Position::new(x, start.y));
                    res.push(self.parser.convert_to_unicode(ch));
                }
            } else {
                for x in start.x..(self.buffer.get_line_length(start.y)) {
                    let ch = self.buffer.get_char(Position::new(x, start.y));
                    res.push(self.parser.convert_to_unicode(ch));
                }
                res.push('\n');
                for y in start.y + 1..end.y {
                    for x in 0..(self.buffer.get_line_length(y)) {
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
            data.extend(i32::to_le_bytes(selection.min().x));
            data.extend(i32::to_le_bytes(selection.min().y));

            data.extend(u32::to_le_bytes(selection.size().width as u32));
            data.extend(u32::to_le_bytes(selection.size().height as u32));

            let start = selection.min();
            let end = selection.max();

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

    pub fn get_current_layer(&self) -> usize {
        self.current_layer
    }

    pub fn set_current_layer(&mut self, layer: usize) {
        self.current_layer = layer;
    }

    pub fn get_outline_style(&self) -> usize {
        self.outline_style
    }

    pub fn set_outline_style(&mut self, outline_style: usize) {
        self.outline_style = outline_style;
    }

    #[must_use]
    pub fn begin_atomic_undo(&mut self, description: impl Into<String>) -> AtomicUndoGuard {
        self.begin_typed_atomic_undo(description, OperationType::Unknown)
    }

    #[must_use]
    pub fn begin_typed_atomic_undo(
        &mut self,
        description: impl Into<String>,
        operation_type: OperationType,
    ) -> AtomicUndoGuard {
        self.redo_stack.clear();
        AtomicUndoGuard::new(description.into(), self.undo_stack.clone(), operation_type)
    }

    fn clamp_current_layer(&mut self) {
        self.current_layer = self
            .current_layer
            .clamp(0, self.buffer.layers.len().saturating_sub(1));
    }

    fn push_undo(&mut self, mut op: Box<dyn UndoOperation>) -> EngineResult<()> {
        op.redo(self)?;
        self.undo_stack.lock().unwrap().push(op);
        self.redo_stack.clear();
        Ok(())
    }

    /// Returns the undo stack len of this [`EditState`].
    ///
    /// # Panics
    ///
    /// Panics if .
    pub fn undo_stack_len(&self) -> usize {
        self.undo_stack.lock().unwrap().len()
    }

    pub fn get_undo_stack(&self) -> Arc<Mutex<Vec<Box<dyn UndoOperation>>>> {
        self.undo_stack.clone()
    }

    pub fn has_floating_layer(&self) -> bool {
        for layer in &self.buffer.layers {
            if layer.role.is_paste() {
                return true;
            }
        }
        false
    }

    pub fn get_mirror_mode(&self) -> bool {
        self.mirror_mode
    }

    pub fn set_mirror_mode(&mut self, mirror_mode: bool) {
        self.mirror_mode = mirror_mode;
        println!("set mirror mode to {}", mirror_mode);
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
