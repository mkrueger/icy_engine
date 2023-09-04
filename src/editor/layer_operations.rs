#![allow(clippy::missing_errors_doc)]
use i18n_embed_fl::fl;

use crate::{EngineResult, Layer, Position, Size};

use super::{undo_operations, EditState, UndoOperation};

impl EditState {
    pub fn add_new_layer(&mut self, layer: usize) -> EngineResult<()> {
        let size = self.buffer.get_buffer_size();
        let mut new_layer = Layer::new(fl!(crate::LANGUAGE_LOADER, "layer-new-name"), size);
        new_layer.has_alpha_channel = true;
        if self.buffer.layers.is_empty() {
            new_layer.has_alpha_channel = false;
        }

        let mut op = undo_operations::AddLayer::new(layer + 1, new_layer);
        op.redo(self)?;
        self.push_undo(Box::new(op));
        self.current_layer = layer + 1;
        Ok(())
    }

    pub fn remove_layer(&mut self, layer: usize) -> EngineResult<()> {
        let mut op = undo_operations::RemoveLayer::new(layer);
        op.redo(self)?;
        self.push_undo(Box::new(op));
        Ok(())
    }

    pub fn raise_layer(&mut self, layer: usize) -> EngineResult<()> {
        if layer + 1 >= self.buffer.layers.len() {
            return Ok(());
        }
        let mut op = undo_operations::RaiseLayer::new(layer);
        op.redo(self)?;
        self.push_undo(Box::new(op));
        self.current_layer = layer + 1;
        Ok(())
    }

    pub fn lower_layer(&mut self, layer: usize) -> EngineResult<()> {
        if layer == 0 {
            return Ok(());
        }
        let mut op = undo_operations::LowerLayer::new(layer);
        op.redo(self)?;
        self.push_undo(Box::new(op));
        self.current_layer = layer - 1;
        Ok(())
    }

    pub fn duplicate_layer(&mut self, layer: usize) -> EngineResult<()> {
        let mut new_layer = self.buffer.layers[layer].clone();
        new_layer.title = fl!(
            crate::LANGUAGE_LOADER,
            "layer-duplicate-name",
            name = new_layer.title
        );
        let mut op = undo_operations::AddLayer::new(layer + 1, new_layer);
        op.redo(self)?;
        self.push_undo(Box::new(op));
        self.current_layer = layer + 1;
        Ok(())
    }

    pub fn merge_layer_down(&mut self, layer: usize) -> EngineResult<()> {
        if layer == 0 {
            return Ok(());
        }

        let base_layer = &self.buffer.layers[layer - 1];
        let cur_layer = &self.buffer.layers[layer];

        let start = Position::new(
            base_layer.offset.x.min(cur_layer.offset.x),
            base_layer.offset.y.min(cur_layer.offset.y),
        );

        let mut merge_layer = base_layer.clone();
        merge_layer.clear();

        merge_layer.offset = start;

        let width = (base_layer.offset.x + base_layer.size.width)
            .max(cur_layer.offset.x + cur_layer.size.width)
            - start.x;
        let height = (base_layer.offset.y + base_layer.size.height)
            .max(cur_layer.offset.y + cur_layer.size.height)
            - start.y;
        if width < 0 || height < 0 {
            return Ok(());
        }
        merge_layer.size = Size::new(width, height);

        for y in 0..base_layer.get_height() {
            for x in 0..base_layer.get_width() {
                let pos = Position::new(x, y);
                let ch = base_layer.get_char(pos);
                let pos = pos - merge_layer.offset + base_layer.offset;
                merge_layer.set_char(pos, ch);
            }
        }

        for y in 0..cur_layer.get_height() {
            for x in 0..cur_layer.get_width() {
                let pos = Position::new(x, y);
                let ch = cur_layer.get_char(pos);
                if !ch.is_visible() {
                    continue;
                }

                let pos = pos - merge_layer.offset + cur_layer.offset;
                merge_layer.set_char(pos, ch);
            }
        }

        let mut op = undo_operations::MergeLayerDown::new(layer, merge_layer);
        op.redo(self)?;
        self.push_undo(Box::new(op));
        Ok(())
    }

    pub fn toggle_layer_visibility(&mut self, layer: usize) -> EngineResult<()> {
        let mut op = undo_operations::ToggleLayerVisibility::new(layer);
        op.redo(self)?;
        self.push_undo(Box::new(op));
        Ok(())
    }

    pub fn move_layer(&mut self, from: Position, to: Position) -> EngineResult<()> {
        let mut op = undo_operations::MoveLayer::new(self.current_layer, from, to);
        op.redo(self)?;
        self.push_undo(Box::new(op));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        editor::{EditState, UndoState},
        AttributedChar, Layer, Position, Size, TextAttribute,
    };

    #[test]
    fn test_add_layer() {
        let mut state = EditState::default();
        assert_eq!(1, state.buffer.layers.len());
        state.add_new_layer(0).unwrap();
        assert_eq!(2, state.buffer.layers.len());
    }

    #[test]
    fn test_add_layer_undo_redo() {
        let mut state = EditState::default();
        assert_eq!(1, state.buffer.layers.len());
        state.add_new_layer(0).unwrap();
        assert_eq!(2, state.buffer.layers.len());
        state.undo().unwrap();
        assert_eq!(1, state.buffer.layers.len());
        state.redo().unwrap();
        assert_eq!(2, state.buffer.layers.len());
    }

    #[test]
    fn test_remove_layer() {
        let mut state = EditState::default();
        state.add_new_layer(0).unwrap();
        state.add_new_layer(0).unwrap();
        assert_eq!(3, state.buffer.layers.len());
        state.remove_layer(1).unwrap();
        assert_eq!(2, state.buffer.layers.len());
    }

    #[test]
    fn test_remove_layer_undo_redo() {
        let mut state = EditState::default();
        state.add_new_layer(0).unwrap();
        state.add_new_layer(0).unwrap();
        assert_eq!(3, state.buffer.layers.len());
        state.remove_layer(1).unwrap();
        assert_eq!(2, state.buffer.layers.len());
        state.undo().unwrap();
        assert_eq!(3, state.buffer.layers.len());
        state.redo().unwrap();
        assert_eq!(2, state.buffer.layers.len());
    }

    #[test]
    fn test_raise_layer() {
        let mut state = EditState::default();
        let name = state.buffer.layers[0].title.clone();
        state.add_new_layer(0).unwrap();
        state.raise_layer(0).unwrap();
        assert_eq!(name, state.buffer.layers[1].title);
        state.undo().unwrap();
        assert_ne!(name, state.buffer.layers[1].title);
    }

    #[test]
    fn test_lower_layer() {
        let mut state: EditState = EditState::default();
        state.add_new_layer(0).unwrap();
        let name = state.buffer.layers[1].title.clone();
        state.lower_layer(1).unwrap();
        assert_eq!(name, state.buffer.layers[0].title);
        state.undo().unwrap();
        assert_ne!(name, state.buffer.layers[0].title);
    }

    #[test]
    fn test_toggle_layer_visibility() {
        let mut state = EditState::default();
        assert!(state.buffer.layers[0].is_visible);
        state.toggle_layer_visibility(0).unwrap();
        assert!(!state.buffer.layers[0].is_visible);
        state.undo().unwrap();
        assert!(state.buffer.layers[0].is_visible);
    }

    #[test]
    fn test_merge_layer_down() {
        let mut state = EditState::default();
        let mut new_layer = Layer::new("1", Size::new(10, 10));
        new_layer.has_alpha_channel = true;
        new_layer.set_char((5, 5), AttributedChar::new('a', TextAttribute::default()));
        state.buffer.layers.push(new_layer);

        let mut new_layer = Layer::new("2", Size::new(10, 10));
        new_layer.has_alpha_channel = true;
        new_layer.set_char((6, 6), AttributedChar::new('b', TextAttribute::default()));
        state.buffer.layers.push(new_layer);

        state.merge_layer_down(2).unwrap();
        assert_eq!(2, state.buffer.layers.len());

        assert_eq!('a', state.buffer.get_char((5, 5)).ch);
        assert_eq!('b', state.buffer.get_char((6, 6)).ch);
        assert_eq!(Position::new(0, 0), state.buffer.layers[1].offset);
        assert_eq!(Size::new(10, 10), state.buffer.layers[1].size);
        state.undo().unwrap();
        assert_eq!(3, state.buffer.layers.len());
    }

    #[test]
    fn test_merge_layer_down_case1() {
        let mut state = EditState::default();
        let mut new_layer = Layer::new("1", Size::new(10, 10));
        new_layer.has_alpha_channel = true;
        new_layer.offset = Position::new(2, 2);
        new_layer.set_char((5, 5), AttributedChar::new('a', TextAttribute::default()));
        state.buffer.layers.push(new_layer);

        let mut new_layer = Layer::new("2", Size::new(10, 10));
        new_layer.has_alpha_channel = true;
        new_layer.set_char((6, 6), AttributedChar::new('b', TextAttribute::default()));
        state.buffer.layers.push(new_layer);

        state.merge_layer_down(2).unwrap();
        assert_eq!(2, state.buffer.layers.len());

        assert_eq!('a', state.buffer.get_char((7, 7)).ch);
        assert_eq!('b', state.buffer.get_char((6, 6)).ch);
        assert_eq!(Position::new(0, 0), state.buffer.layers[1].offset);
        assert_eq!(Size::new(12, 12), state.buffer.layers[1].size);
    }

    #[test]
    fn test_merge_layer_down_case2() {
        let mut state = EditState::default();
        let mut new_layer = Layer::new("1", Size::new(10, 10));
        new_layer.has_alpha_channel = true;
        new_layer.offset = Position::new(-1, -1);
        new_layer.set_char((5, 5), AttributedChar::new('a', TextAttribute::default()));
        state.buffer.layers.push(new_layer);

        let mut new_layer = Layer::new("2", Size::new(10, 10));
        new_layer.has_alpha_channel = true;
        new_layer.set_char((6, 6), AttributedChar::new('b', TextAttribute::default()));
        state.buffer.layers.push(new_layer);

        state.merge_layer_down(2).unwrap();

        assert_eq!(2, state.buffer.layers.len());

        assert_eq!(Position::new(-1, -1), state.buffer.layers[1].offset);
        assert_eq!(Size::new(11, 11), state.buffer.layers[1].size);

        assert_eq!('a', state.buffer.layers[1].get_char((5, 5)).ch);
        assert_eq!('b', state.buffer.layers[1].get_char((7, 7)).ch);
    }
}
