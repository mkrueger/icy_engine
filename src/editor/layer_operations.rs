#![allow(clippy::missing_errors_doc)]
use i18n_embed_fl::fl;

use crate::{EngineResult, Layer, Position, Size, TextPane};

use super::{undo_operations, EditState};

impl EditState {
    pub fn add_new_layer(&mut self, layer: usize) -> EngineResult<()> {
        let size = self.buffer.get_size();
        let mut new_layer = Layer::new(fl!(crate::LANGUAGE_LOADER, "layer-new-name"), size);
        new_layer.has_alpha_channel = true;
        if self.buffer.layers.is_empty() {
            new_layer.has_alpha_channel = false;
        }
        let idx = if self.buffer.layers.is_empty() {
            0
        } else {
            layer + 1
        };

        let op = undo_operations::AddLayer::new(idx, new_layer);
        self.push_undo(Box::new(op))?;
        self.current_layer = idx;
        Ok(())
    }

    pub fn remove_layer(&mut self, layer: usize) -> EngineResult<()> {
        let op = undo_operations::RemoveLayer::new(layer);
        self.push_undo(Box::new(op))
    }

    pub fn raise_layer(&mut self, layer: usize) -> EngineResult<()> {
        if layer + 1 >= self.buffer.layers.len() {
            return Ok(());
        }
        let op = undo_operations::RaiseLayer::new(layer);
        self.push_undo(Box::new(op))?;
        self.current_layer = layer + 1;
        Ok(())
    }

    pub fn lower_layer(&mut self, layer: usize) -> EngineResult<()> {
        if layer == 0 {
            return Ok(());
        }
        let op = undo_operations::LowerLayer::new(layer);
        self.push_undo(Box::new(op))?;
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
        let op = undo_operations::AddLayer::new(layer + 1, new_layer);
        self.push_undo(Box::new(op))?;
        self.current_layer = layer + 1;
        Ok(())
    }

    pub fn anchor_layer(&mut self) -> EngineResult<()> {
        let _op = self.begin_atomic_undo(fl!(crate::LANGUAGE_LOADER, "layer-anchor"));
        self.merge_layer_down(self.buffer.layers.len() - 1)
    }

    pub fn add_floating_layer(&mut self) -> EngineResult<()> {
        let op = undo_operations::AddFloatingLayer::default();
        self.push_undo(Box::new(op))
    }

    pub fn merge_layer_down(&mut self, layer: usize) -> EngineResult<()> {
        if layer == 0 {
            return Ok(());
        }

        let base_layer = &self.buffer.layers[layer - 1];
        let cur_layer = &self.buffer.layers[layer];

        let start = Position::new(
            base_layer.get_offset().x.min(cur_layer.get_offset().x),
            base_layer.get_offset().y.min(cur_layer.get_offset().y),
        );

        let mut merge_layer = base_layer.clone();
        merge_layer.clear();

        merge_layer.set_offset(start);

        let width = (base_layer.get_offset().x + base_layer.get_width())
            .max(cur_layer.get_offset().x + cur_layer.get_width())
            - start.x;
        let height = (base_layer.get_offset().y + base_layer.get_height())
            .max(cur_layer.get_offset().y + cur_layer.get_height())
            - start.y;
        if width < 0 || height < 0 {
            return Ok(());
        }
        merge_layer.set_size((width, height));

        for y in 0..base_layer.get_height() {
            for x in 0..base_layer.get_width() {
                let pos = Position::new(x, y);
                let ch = base_layer.get_char(pos);
                let pos = pos - merge_layer.get_offset() + base_layer.get_offset();
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

                let pos = pos - merge_layer.get_offset() + cur_layer.get_offset();
                merge_layer.set_char(pos, ch);
            }
        }

        let op = undo_operations::MergeLayerDown::new(layer, merge_layer);
        self.push_undo(Box::new(op))?;
        self.clamp_current_layer();
        Ok(())
    }

    pub fn toggle_layer_visibility(&mut self, layer: usize) -> EngineResult<()> {
        let op = undo_operations::ToggleLayerVisibility::new(layer);
        self.push_undo(Box::new(op))
    }

    pub fn move_layer(&mut self, to: Position) -> EngineResult<()> {
        let i = self.current_layer;
        let Some(cur_layer) = self.get_cur_layer_mut() else {
            return Ok(());
        };
        cur_layer.set_preview_offset(None);
        let op = undo_operations::MoveLayer::new(i, cur_layer.get_offset(), to);
        self.push_undo(Box::new(op))
    }

    pub fn set_layer_size(&mut self, layer: usize, size: impl Into<Size>) -> EngineResult<()> {
        let op = undo_operations::SetLayerSize::new(layer, size.into());
        self.push_undo(Box::new(op))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        editor::{EditState, UndoState},
        AttributedChar, Layer, Position, Size, TextAttribute, TextPane,
    };

    #[test]
    fn test_add_layer() {
        let mut state = EditState::default();
        assert_eq!(1, state.buffer.layers.len());
        state.add_new_layer(0).unwrap();
        assert_eq!(2, state.buffer.layers.len());
    }

    #[test]
    fn test_add_layer_size() {
        let mut state = EditState::default();
        let size = Size::new(160, 1000);
        state.buffer.set_size(size);
        state.add_new_layer(0).unwrap();
        assert_eq!(size, state.buffer.layers[1].get_size());
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
        assert_eq!(Position::new(0, 0), state.buffer.layers[1].get_offset());
        assert_eq!(Size::new(10, 10), state.buffer.layers[1].get_size());
        state.undo().unwrap();
        assert_eq!(3, state.buffer.layers.len());
    }

    #[test]
    fn test_merge_layer_down_case1() {
        let mut state = EditState::default();
        let mut new_layer = Layer::new("1", Size::new(10, 10));
        new_layer.has_alpha_channel = true;
        new_layer.set_offset((2, 2));
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
        assert_eq!(Position::new(0, 0), state.buffer.layers[1].get_offset());
        assert_eq!(Size::new(12, 12), state.buffer.layers[1].get_size());
    }

    #[test]
    fn test_merge_layer_down_case2() {
        let mut state = EditState::default();
        let mut new_layer = Layer::new("1", Size::new(10, 10));
        new_layer.has_alpha_channel = true;
        new_layer.set_offset((-1, -1));
        new_layer.set_char((5, 5), AttributedChar::new('a', TextAttribute::default()));
        state.buffer.layers.push(new_layer);

        let mut new_layer = Layer::new("2", Size::new(10, 10));
        new_layer.has_alpha_channel = true;
        new_layer.set_char((6, 6), AttributedChar::new('b', TextAttribute::default()));
        state.buffer.layers.push(new_layer);

        state.merge_layer_down(2).unwrap();

        assert_eq!(2, state.buffer.layers.len());

        assert_eq!(Position::new(-1, -1), state.buffer.layers[1].get_offset());
        assert_eq!(Size::new(11, 11), state.buffer.layers[1].get_size());

        assert_eq!('a', state.buffer.layers[1].get_char((5, 5)).ch);
        assert_eq!('b', state.buffer.layers[1].get_char((7, 7)).ch);
    }
}
