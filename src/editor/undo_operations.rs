use i18n_embed_fl::fl;

use crate::Layer;

use super::{EditState, UndoOperation};

pub(crate) struct AtomicUndo {
    description: String,
    stack: Vec<Box<dyn UndoOperation>>,
}

impl AtomicUndo {
    pub(crate) fn new(description: String, stack: Vec<Box<dyn UndoOperation>>) -> Self {
        Self { description, stack }
    }
}

impl UndoOperation for AtomicUndo {
    fn get_description(&self) -> String {
        self.description.clone()
    }

    fn undo(&mut self, buffer: &mut EditState) {
        for op in &mut self.stack {
            op.undo(buffer);
        }
    }

    fn redo(&mut self, buffer: &mut EditState) {
        for op in self.stack.iter_mut().rev() {
            op.redo(buffer);
        }
    }
}

#[derive(Default)]
pub struct CreateNewLayer {
    layer: Option<Layer>
}

impl UndoOperation for CreateNewLayer {
    fn get_description(&self) -> String {
        fl!(crate::LANGUAGE_LOADER, "undo-add_layer")
    }

    fn undo(&mut self, edit_state: &mut EditState) {
        self.layer = edit_state.buffer.layers.pop();
    }

    fn redo(&mut self, edit_state: &mut EditState) {
        if let Some(layer) = self.layer.take() {
            edit_state.buffer.layers.push(layer);
        }
    }
}



#[derive(Default)]
pub struct RemoveLayer {
    layer_index: usize,
    layer: Option<Layer>
}

impl RemoveLayer {
    pub fn new(layer_index: usize, layer: Layer) -> Self {
        Self {
            layer_index,
            layer: Some(layer)
        }
    }
}

impl UndoOperation for RemoveLayer {
    fn get_description(&self) -> String {
        fl!(crate::LANGUAGE_LOADER, "undo-remove_layer")
    }

    fn undo(&mut self, edit_state: &mut EditState) {
        if let Some(layer) = self.layer.take() {
            edit_state.buffer.layers.insert(self.layer_index, layer);
        }
    }

    fn redo(&mut self, edit_state: &mut EditState) {
        self.layer = Some(edit_state.buffer.layers.remove(self.layer_index));
    }
}