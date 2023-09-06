#![allow(clippy::missing_errors_doc)]

use std::io;

use i18n_embed_fl::fl;

use crate::{AttributedChar, EngineResult, Layer, Position, Rectangle, Sixel, Size, TextPane};

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

    pub fn center_line(&mut self) -> EngineResult<()> {
        let offset = if let Some(layer) = self.get_cur_layer() {
            layer.get_offset().y
        } else {
            0
        };
        let y = self.get_caret().get_position().y + offset;
        self.set_selection(Rectangle::from_coords(-1_000_000, y, 1_000_000, y + 1));
        let res = self.center();
        self.clear_selection();
        res
    }

    pub fn justify_line_left(&mut self) -> EngineResult<()> {
        let offset: i32 = if let Some(layer) = self.get_cur_layer() {
            layer.get_offset().y
        } else {
            0
        };
        let y = self.get_caret().get_position().y + offset;
        self.set_selection(Rectangle::from_coords(-1_000_000, y, 1_000_000, y + 1));
        let res = self.justify_left();
        self.clear_selection();
        res
    }

    pub fn justify_line_right(&mut self) -> EngineResult<()> {
        let offset: i32 = if let Some(layer) = self.get_cur_layer() {
            layer.get_offset().y
        } else {
            0
        };
        let y = self.get_caret().get_position().y + offset;
        self.set_selection(Rectangle::from_coords(-1_000_000, y, 1_000_000, y + 1));
        let res = self.justify_right();
        self.clear_selection();
        res
    }

    pub fn delete_row(&mut self) -> EngineResult<()> {
        let y = self.get_caret().get_position().y;
        let layer = self.get_current_layer();
        let op = super::undo_operations::DeleteRow::new(layer, y);
        self.push_undo(Box::new(op))
    }

    pub fn insert_row(&mut self) -> EngineResult<()> {
        let y = self.get_caret().get_position().y;
        let layer = self.get_current_layer();
        let op = super::undo_operations::InsertRow::new(layer, y);
        self.push_undo(Box::new(op))
    }

    pub fn insert_column(&mut self) -> EngineResult<()> {
        let x = self.get_caret().get_position().x;
        let layer = self.get_current_layer();
        let op = super::undo_operations::InsertColumn::new(layer, x);
        self.push_undo(Box::new(op))
    }

    pub fn delete_column(&mut self) -> EngineResult<()> {
        let x = self.get_caret().get_position().x;
        let layer = self.get_current_layer();
        let op = super::undo_operations::DeleteColumn::new(layer, x);
        self.push_undo(Box::new(op))
    }

    pub fn erase_row(&mut self) -> EngineResult<()> {
        let offset = if let Some(layer) = self.get_cur_layer() {
            layer.get_offset().y
        } else {
            0
        };
        let y = self.get_caret().get_position().y + offset;
        self.set_selection(Rectangle::from_coords(-1_000_000, y, 1_000_000, y + 1));
        self.delete_selection()
    }

    pub fn erase_row_to_start(&mut self) -> EngineResult<()> {
        let offset = if let Some(layer) = self.get_cur_layer() {
            layer.get_offset()
        } else {
            Position::default()
        };
        let y = self.get_caret().get_position().y + offset.y;
        let x = self.get_caret().get_position().x + offset.x;
        self.set_selection(Rectangle::from_coords(-1_000_000, y, x, y + 1));
        self.delete_selection()
    }

    pub fn erase_row_to_end(&mut self) -> EngineResult<()> {
        let offset = if let Some(layer) = self.get_cur_layer() {
            layer.get_offset()
        } else {
            Position::default()
        };
        let y = self.get_caret().get_position().y + offset.y;
        let x = self.get_caret().get_position().x + offset.x;
        self.set_selection(Rectangle::from_coords(x, y, 1_000_000, y + 1));
        self.delete_selection()
    }

    pub fn erase_column(&mut self) -> EngineResult<()> {
        let offset = if let Some(layer) = self.get_cur_layer() {
            layer.get_offset()
        } else {
            Position::default()
        };
        let x = self.get_caret().get_position().x + offset.x;
        self.set_selection(Rectangle::from_coords(x, -1_000_000, x, 1_000_000));
        self.delete_selection()
    }

    pub fn erase_column_to_start(&mut self) -> EngineResult<()> {
        let offset = if let Some(layer) = self.get_cur_layer() {
            layer.get_offset()
        } else {
            Position::default()
        };
        let y = self.get_caret().get_position().y + offset.y;
        let x = self.get_caret().get_position().x + offset.x;
        self.set_selection(Rectangle::from_coords(x, -1_000_000, x, y));
        self.delete_selection()
    }

    pub fn erase_column_to_end(&mut self) -> EngineResult<()> {
        let offset = if let Some(layer) = self.get_cur_layer() {
            layer.get_offset()
        } else {
            Position::default()
        };
        let y = self.get_caret().get_position().y + offset.y;
        let x = self.get_caret().get_position().x + offset.x;
        self.set_selection(Rectangle::from_coords(x, y, x, 1_000_000));
        self.delete_selection()
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
