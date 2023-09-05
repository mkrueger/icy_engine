#![allow(clippy::missing_errors_doc)]

use std::io;

use i18n_embed_fl::fl;

use crate::{AttributedChar, EngineResult, Layer, Position, Sixel, Size, TextPane};

use super::{
    undo_operations::{Paste, UndoSetChar, UndoSwapChar},
    EditState,
};

impl EditState {
    pub fn set_char(
        &mut self,
        pos: impl Into<Position>,
        attributed_char: AttributedChar,
    ) -> EngineResult<()> {
        if let Some(layer) = self.get_cur_layer() {
            let pos = pos.into();
            let old = layer.get_char(pos);
            self.push_undo(Box::new(UndoSetChar {
                pos,
                layer: self.current_layer,
                old,
                new: attributed_char,
            }))
        } else {
            Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidData,
                "Current layer is invalid",
            )))
        }
    }

    pub fn swap_char(
        &mut self,
        pos1: impl Into<Position>,
        pos2: impl Into<Position>,
    ) -> EngineResult<()> {
        let pos1 = pos1.into();
        let pos2 = pos2.into();
        let layer = self.current_layer;
        let op = UndoSwapChar { layer, pos1, pos2 };
        self.push_undo(Box::new(op))
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn paste_clipboard_data(&mut self, data: &[u8]) -> EngineResult<()> {
        if let Some(layer) = Layer::from_clipboard_data(data) {
            let op = Paste::new(layer);
            self.push_undo(Box::new(op))?;
        }
        self.selection_opt = None;
        Ok(())
    }

    pub fn paste_sixel(&mut self, sixel: Sixel) -> EngineResult<()> {
        let dims = self.get_buffer().get_font_dimensions();

        let mut layer = Layer::new(
            fl!(crate::LANGUAGE_LOADER, "layer-pasted-name"),
            (
                (sixel.get_width() as f32 / dims.width as f32).ceil() as i32,
                (sixel.get_height() as f32 / dims.height as f32).ceil() as i32,
            ),
        );
        layer.role = crate::Role::PasteImage;
        layer.has_alpha_channel = true;
        layer.sixels.push(sixel);

        let op = Paste::new(layer);
        self.push_undo(Box::new(op))?;
        self.selection_opt = None;
        Ok(())
    }

    pub fn resize_buffer(&mut self, size: impl Into<Size>) -> EngineResult<()> {
        let op = super::undo_operations::ResizeBuffer::new(self.get_buffer().get_size(), size);
        self.push_undo(Box::new(op))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        editor::{EditState, UndoState},
        Size, TextPane,
    };

    #[test]
    fn test_resize_buffer() {
        let mut state = EditState::default();
        assert_eq!(Size::new(80, 25), state.buffer.get_size());
        state.resize_buffer(Size::new(10, 10)).unwrap();
        assert_eq!(Size::new(10, 10), state.buffer.get_size());
        state.undo().unwrap();
        assert_eq!(Size::new(80, 25), state.buffer.get_size());
        state.redo().unwrap();
        assert_eq!(Size::new(10, 10), state.buffer.get_size());
    }
}
