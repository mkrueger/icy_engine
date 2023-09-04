use std::mem;

use i18n_embed_fl::fl;

use crate::{AttributedChar, EngineResult, Layer, Position};

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

    fn undo(&mut self, edit_state: &mut EditState) -> EngineResult<()> {
        for op in self.stack.iter_mut().rev() {
            op.undo(edit_state)?;
        }
        Ok(())
    }

    fn redo(&mut self, edit_state: &mut EditState) -> EngineResult<()> {
        for op in &mut self.stack {
            op.redo(edit_state)?;
        }
        Ok(())
    }
}

pub struct UndoSetChar {
    pub pos: Position,
    pub layer: usize,
    pub old: AttributedChar,
    pub new: AttributedChar,
}

impl UndoOperation for UndoSetChar {
    fn get_description(&self) -> String {
        fl!(crate::LANGUAGE_LOADER, "undo-set_char")
    }

    fn undo(&mut self, edit_state: &mut EditState) -> EngineResult<()> {
        edit_state.buffer.layers[self.layer].set_char(self.pos, self.old);
        Ok(())
    }

    fn redo(&mut self, edit_state: &mut EditState) -> EngineResult<()> {
        edit_state.buffer.layers[self.layer].set_char(self.pos, self.new);
        Ok(())
    }
}

pub struct UndoSwapChar {
    pub layer: usize,
    pub pos1: Position,
    pub pos2: Position,
}
impl UndoOperation for UndoSwapChar {
    fn get_description(&self) -> String {
        String::new() // No stand alone operation.
    }

    fn undo(&mut self, edit_state: &mut EditState) -> EngineResult<()> {
        edit_state.buffer.layers[self.layer].swap_char(self.pos1, self.pos2);
        Ok(())
    }

    fn redo(&mut self, edit_state: &mut EditState) -> EngineResult<()> {
        edit_state.buffer.layers[self.layer].swap_char(self.pos1, self.pos2);
        Ok(())
    }
}

pub struct ClearLayerOperation {
    layer_num: usize,
    lines: Vec<crate::Line>,
}

impl ClearLayerOperation {
    pub fn _new(layer_num: usize) -> Self {
        Self {
            layer_num,
            lines: Vec::new(),
        }
    }
}

impl UndoOperation for ClearLayerOperation {
    fn get_description(&self) -> String {
        fl!(crate::LANGUAGE_LOADER, "undo-clear-layer")
    }

    fn undo(&mut self, edit_state: &mut EditState) -> EngineResult<()> {
        mem::swap(
            &mut self.lines,
            &mut edit_state.buffer.layers[self.layer_num].lines,
        );
        Ok(())
    }

    fn redo(&mut self, edit_state: &mut EditState) -> EngineResult<()> {
        mem::swap(
            &mut self.lines,
            &mut edit_state.buffer.layers[self.layer_num].lines,
        );
        Ok(())
    }
}

#[derive(Default)]
pub struct AddLayer {
    index: usize,
    layer: Option<Layer>,
}

impl AddLayer {
    pub(crate) fn new(index: usize, new_layer: Layer) -> Self {
        Self {
            index,
            layer: Some(new_layer),
        }
    }
}

impl UndoOperation for AddLayer {
    fn get_description(&self) -> String {
        fl!(crate::LANGUAGE_LOADER, "undo-add_layer")
    }

    fn undo(&mut self, edit_state: &mut EditState) -> EngineResult<()> {
        self.layer = Some(edit_state.buffer.layers.remove(self.index));
        Ok(())
    }

    fn redo(&mut self, edit_state: &mut EditState) -> EngineResult<()> {
        if let Some(layer) = self.layer.take() {
            edit_state.buffer.layers.insert(self.index, layer);
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct RemoveLayer {
    layer_index: usize,
    layer: Option<Layer>,
}

impl RemoveLayer {
    pub fn new(layer_index: usize) -> Self {
        Self {
            layer_index,
            layer: None,
        }
    }
}

impl UndoOperation for RemoveLayer {
    fn get_description(&self) -> String {
        fl!(crate::LANGUAGE_LOADER, "undo-remove_layer")
    }

    fn undo(&mut self, edit_state: &mut EditState) -> EngineResult<()> {
        if let Some(layer) = self.layer.take() {
            edit_state.buffer.layers.insert(self.layer_index, layer);
        }
        Ok(())
    }

    fn redo(&mut self, edit_state: &mut EditState) -> EngineResult<()> {
        self.layer = Some(edit_state.buffer.layers.remove(self.layer_index));
        edit_state.clamp_current_layer();
        Ok(())
    }
}

#[derive(Default)]
pub struct RaiseLayer {
    layer_index: usize,
}

impl RaiseLayer {
    pub fn new(layer_index: usize) -> Self {
        Self { layer_index }
    }
}

impl UndoOperation for RaiseLayer {
    fn get_description(&self) -> String {
        fl!(crate::LANGUAGE_LOADER, "undo-raise_layer")
    }

    fn undo(&mut self, edit_state: &mut EditState) -> EngineResult<()> {
        edit_state
            .buffer
            .layers
            .swap(self.layer_index, self.layer_index + 1);
        Ok(())
    }

    fn redo(&mut self, edit_state: &mut EditState) -> EngineResult<()> {
        edit_state
            .buffer
            .layers
            .swap(self.layer_index, self.layer_index + 1);
        Ok(())
    }
}

#[derive(Default)]
pub struct LowerLayer {
    layer_index: usize,
}

impl LowerLayer {
    pub fn new(layer_index: usize) -> Self {
        Self { layer_index }
    }
}

impl UndoOperation for LowerLayer {
    fn get_description(&self) -> String {
        fl!(crate::LANGUAGE_LOADER, "undo-lower_layer")
    }

    fn undo(&mut self, edit_state: &mut EditState) -> EngineResult<()> {
        edit_state
            .buffer
            .layers
            .swap(self.layer_index, self.layer_index - 1);
        Ok(())
    }

    fn redo(&mut self, edit_state: &mut EditState) -> EngineResult<()> {
        edit_state
            .buffer
            .layers
            .swap(self.layer_index, self.layer_index - 1);
        Ok(())
    }
}

#[derive(Default)]
pub struct MergeLayerDown {
    index: usize,
    merged_layer: Option<Layer>,
    orig_layers: Option<Vec<Layer>>,
}

impl MergeLayerDown {
    pub(crate) fn new(index: usize, merged_layer: Layer) -> Self {
        Self {
            index,
            merged_layer: Some(merged_layer),
            orig_layers: None,
        }
    }
}

impl UndoOperation for MergeLayerDown {
    fn get_description(&self) -> String {
        fl!(crate::LANGUAGE_LOADER, "undo-merge_down_layer")
    }

    fn undo(&mut self, edit_state: &mut EditState) -> EngineResult<()> {
        if let Some(mut orig_layers) = self.orig_layers.take() {
            while let Some(layer) = orig_layers.pop() {
                edit_state.buffer.layers.insert(self.index - 1, layer);
            }
            self.merged_layer = Some(edit_state.buffer.layers.remove(self.index + 1));
        }
        Ok(())
    }

    fn redo(&mut self, edit_state: &mut EditState) -> EngineResult<()> {
        if let Some(layer) = self.merged_layer.take() {
            self.orig_layers = Some(
                edit_state
                    .buffer
                    .layers
                    .drain((self.index - 1)..=self.index)
                    .collect(),
            );
            edit_state.buffer.layers.insert(self.index - 1, layer);
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct ToggleLayerVisibility {
    index: usize,
}

impl ToggleLayerVisibility {
    pub(crate) fn new(index: usize) -> Self {
        Self { index }
    }
}

impl UndoOperation for ToggleLayerVisibility {
    fn get_description(&self) -> String {
        fl!(crate::LANGUAGE_LOADER, "undo-toggle_layer_visibility")
    }

    fn undo(&mut self, edit_state: &mut EditState) -> EngineResult<()> {
        edit_state.buffer.layers[self.index].is_visible =
            !edit_state.buffer.layers[self.index].is_visible;
        Ok(())
    }

    fn redo(&mut self, edit_state: &mut EditState) -> EngineResult<()> {
        edit_state.buffer.layers[self.index].is_visible =
            !edit_state.buffer.layers[self.index].is_visible;
        Ok(())
    }
}

#[derive(Default)]
pub struct MoveLayer {
    index: usize,
    from: Position,
    to: Position,
}

impl MoveLayer {
    pub(crate) fn new(index: usize, from: Position, to: Position) -> Self {
        Self { index, from, to }
    }
}

impl UndoOperation for MoveLayer {
    fn get_description(&self) -> String {
        fl!(crate::LANGUAGE_LOADER, "undo-move_layer")
    }

    fn undo(&mut self, edit_state: &mut EditState) -> EngineResult<()> {
        edit_state.buffer.layers[self.index].offset = self.from;
        Ok(())
    }

    fn redo(&mut self, edit_state: &mut EditState) -> EngineResult<()> {
        edit_state.buffer.layers[self.index].offset = self.to;
        Ok(())
    }
}
