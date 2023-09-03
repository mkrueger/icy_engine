use crate::{ EngineResult, Layer};

use super::{EditState, undo_operations};

impl EditState {
    pub fn add_layer(&mut self) -> EngineResult<()> {
        let size = self.buffer.get_buffer_size();
        let mut new_layer = Layer::new("New Layer", size);
        new_layer.has_alpha_channel = true;
        if self.buffer.layers.is_empty() {
            new_layer.has_alpha_channel = false;
        }
        self.buffer.layers.push(new_layer);
        self.undo_stack.push(Box::<undo_operations::CreateNewLayer>::default());
        Ok(())
    }

    pub fn remove_layer(&mut self, layer: usize) -> EngineResult<()> {
        let layer_struct = self.get_buffer_mut().layers.remove(layer);

        self.set_current_layer(
            self.get_current_layer().clamp(
                0,
                self
                    .get_buffer()
                    .layers
                    .len()
                    .saturating_sub(1),
            ),
        );
        self.undo_stack.push(Box::new(undo_operations::RemoveLayer::new(layer, layer_struct)));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{editor::{EditState, UndoState}};

    #[test]
    fn test_add_layer() {
        let mut state = EditState::default();
        assert_eq!(1, state.buffer.layers.len());
        state.add_layer().unwrap();
        assert_eq!(2, state.buffer.layers.len());
    }

    #[test]
    fn test_add_layer_undo_redo() {
        let mut state = EditState::default();
        assert_eq!(1, state.buffer.layers.len());
        state.add_layer().unwrap();
        assert_eq!(2, state.buffer.layers.len());
        state.undo();
        assert_eq!(1, state.buffer.layers.len());
        state.redo();
        assert_eq!(2, state.buffer.layers.len());
    }

    #[test]
    fn test_remove_layer() {
        let mut state = EditState::default();
        state.add_layer().unwrap();
        state.add_layer().unwrap();
        assert_eq!(3, state.buffer.layers.len());
        state.remove_layer(1).unwrap();
        assert_eq!(2, state.buffer.layers.len());
    }

    #[test]
    fn test_remove_layer_undo_redo() {
        let mut state = EditState::default();
        state.add_layer().unwrap();
        state.add_layer().unwrap();
        assert_eq!(3, state.buffer.layers.len());
        state.remove_layer(1).unwrap();
        assert_eq!(2, state.buffer.layers.len());
        state.undo();
        assert_eq!(3, state.buffer.layers.len());
        state.redo();
        assert_eq!(2, state.buffer.layers.len());
    }
}
