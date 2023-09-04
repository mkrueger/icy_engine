#![allow(clippy::missing_errors_doc)]

use crate::{AttributedChar, EngineResult, Layer, Position};

use super::{
    undo_operations::{Paste, UndoSetChar, UndoSwapChar},
    EditState, UndoOperation,
};

impl EditState {
    pub fn set_char(
        &mut self,
        pos: impl Into<Position>,
        attributed_char: AttributedChar,
    ) -> EngineResult<()> {
        let pos = pos.into();
        self.redo_stack.clear();
        let old = self.buffer.layers[self.current_layer].get_char(pos);
        self.buffer.layers[self.current_layer].set_char(pos, attributed_char);
        self.push_undo(Box::new(UndoSetChar {
            pos,
            layer: self.current_layer,
            old,
            new: attributed_char,
        }));
        Ok(())
    }

    pub fn swap_char(
        &mut self,
        pos1: impl Into<Position>,
        pos2: impl Into<Position>,
    ) -> EngineResult<()> {
        let pos1 = pos1.into();
        let pos2 = pos2.into();
        let layer = self.current_layer;
        self.get_buffer_mut().layers[layer].swap_char(pos1, pos2);
        self.push_undo(Box::new(UndoSwapChar { layer, pos1, pos2 }));
        Ok(())
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn paste_clipboard_data(&mut self, data: &[u8]) -> EngineResult<()> {
        if let Some(layer) = Layer::from_clipboard_data(data) {
            let mut op = Paste::new(layer);
            op.redo(self)?;
            self.push_undo(Box::new(op));
        }
        self.selection_opt = None;
        Ok(())
    }
}
