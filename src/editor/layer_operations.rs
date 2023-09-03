#![allow(clippy::missing_errors_doc)]
use i18n_embed_fl::fl;

use crate::{ EngineResult, Layer, Position};

use super::{EditState, undo_operations, UndoOperation};

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
        self.undo_stack.push(Box::new(op));
        Ok(())
    }

    pub fn remove_layer(&mut self, layer: usize) -> EngineResult<()> {
        let mut op = undo_operations::RemoveLayer::new(layer);
        op.redo(self)?;
        self.undo_stack.push(Box::new(op));
        Ok(())
    }

    pub fn raise_layer(&mut self, layer: usize) -> EngineResult<()> {
        if layer + 1 >= self.buffer.layers.len() {
            return Ok(());
        }
        let mut op = undo_operations::RaiseLayer::new(layer);
        op.redo(self)?;
        self.undo_stack.push(Box::new(op));
        self.current_layer = layer + 1;
        Ok(())
    }

    pub fn lower_layer(&mut self, layer: usize) -> EngineResult<()> {
        if layer == 0 {
            return Ok(());
        }
        let mut op = undo_operations::LowerLayer::new(layer);
        op.redo(self)?;
        self.undo_stack.push(Box::new(op));
        self.current_layer =  layer - 1;
        Ok(())
    }

    pub fn duplicate_layer(&mut self, layer: usize) -> EngineResult<()> {
        let mut new_layer = self.buffer.layers[layer].clone();
        new_layer.title = fl!(crate::LANGUAGE_LOADER, "layer-duplicate-name", name = new_layer.title);
        let mut op = undo_operations::AddLayer::new(layer, new_layer);
        op.redo(self)?;
        self.undo_stack.push(Box::new(op));
        Ok(())
    }

    pub fn merge_layer_down(&mut self, layer: usize) -> EngineResult<()> {
        if layer == 0 {
            return Ok(());
        }

        let mut merge_layer = self.buffer.layers[layer - 1].clone();
        let cur_layer = &self.buffer.layers[layer];
        
        let position_delta = cur_layer.get_offset() - merge_layer.get_offset();

        for y in 0..cur_layer.get_height() {
            for x in 0..cur_layer.get_width() {
                let ch = cur_layer.get_char((x, y));
                if ch.is_visible() {
                    let pos = Position::new(x as i32, y as i32) - position_delta;
                    if 0 <= pos.x && pos.x < merge_layer.get_width() as i32 && 
                       0 <= pos.y && pos.y < merge_layer.get_height() as i32 {
                        merge_layer.set_char(pos , ch);
                    }
                }
            }
        }
        
        let mut op = undo_operations::MergeLayerDown::new(layer, merge_layer);
        op.redo(self)?;
        self.undo_stack.push(Box::new(op));
        Ok(())
    }

    pub fn toggle_layer_visibility(&mut self, layer: usize) -> EngineResult<()> {
        let mut op = undo_operations::ToggleLayerVisibility::new(layer);
        op.redo(self)?;
        self.undo_stack.push(Box::new(op));
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use crate::editor::{EditState, UndoState};

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
        state.undo();
        assert_eq!(1, state.buffer.layers.len());
        state.redo();
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
        state.undo();
        assert_eq!(3, state.buffer.layers.len());
        state.redo();
        assert_eq!(2, state.buffer.layers.len());
    }

    #[test]
    fn test_raise_layer() {
        let mut state = EditState::default();
        let name = state.buffer.layers[0].title.clone();
        state.add_new_layer(0).unwrap();
        state.raise_layer(0).unwrap();
        assert_eq!(name, state.buffer.layers[1].title);
        state.undo();
        assert_ne!(name, state.buffer.layers[1].title);
    }

    #[test]
    fn test_lower_layer() {
        let mut state = EditState::default();
        state.add_new_layer(0).unwrap();
        let name = state.buffer.layers[1].title.clone();
        state.lower_layer(1).unwrap();
        assert_eq!(name, state.buffer.layers[0].title);
        state.undo();
        assert_ne!(name, state.buffer.layers[0].title);
    }

    #[test]
    fn test_toggle_layer_visibility() {
        let mut state = EditState::default();
        assert!(state.buffer.layers[0].is_visible);
        state.toggle_layer_visibility(0).unwrap();
        assert!(!state.buffer.layers[0].is_visible);
        state.undo();
        assert!(state.buffer.layers[0].is_visible);
    }
}
