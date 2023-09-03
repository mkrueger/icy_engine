use super::EditState;

pub trait UndoState {
    fn undo_description(&self) -> Option<String>;
    fn can_undo(&self) -> bool;
    fn undo(&mut self);

    fn redo_description(&self) -> Option<String>;
    fn can_redo(&self) -> bool;
    fn redo(&mut self);
}

pub trait UndoOperation: Send {
    fn get_description(&self) -> String;

    fn undo(&self, edit_state: &mut EditState);
    fn redo(&self, edit_state: &mut EditState);
}
