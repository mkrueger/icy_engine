use crate::{ EngineResult, Layer};

use super::{EditState, UndoState, undo_operations};

impl EditState {
    pub fn create_new_layer(&mut self) -> EngineResult<()> {
        let size = self.buffer.get_buffer_size();
        let mut new_layer = Layer::new("New Layer", size);
        new_layer.has_alpha_channel = true;
        if self.buffer.layers.is_empty() {
            new_layer.has_alpha_channel = false;
        }
        self.buffer.layers.push(new_layer);
        self.undo_stack.push(Box::new(undo_operations::CreateNewLayer::default()));
        Ok(())
    }
}




#[cfg(test)]
mod tests {
    use crate::{editor::{EditState, UndoState}};

    #[test]
    fn test_create_layer() {
        let mut state = EditState::default();
        assert_eq!(1, state.buffer.layers.len());
        state.create_new_layer().unwrap();
        assert_eq!(2, state.buffer.layers.len());
    }

    #[test]
    fn test_create_layer_undo_redo() {
        let mut state = EditState::default();
        assert_eq!(1, state.buffer.layers.len());
        state.create_new_layer().unwrap();
        assert_eq!(2, state.buffer.layers.len());
        state.undo();
        assert_eq!(1, state.buffer.layers.len());
        state.redo();
        assert_eq!(2, state.buffer.layers.len());
    }
}
