use crate::EngineResult;

use super::EditState;

pub trait UndoState {
    fn undo_description(&self) -> Option<String>;
    fn can_undo(&self) -> bool;
    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    fn undo(&mut self) -> EngineResult<()>;

    fn redo_description(&self) -> Option<String>;
    fn can_redo(&self) -> bool;
    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    fn redo(&mut self) -> EngineResult<()>;
}

pub trait UndoOperation {
    fn get_description(&self) -> String;

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    fn undo(&mut self, edit_state: &mut EditState) -> EngineResult<()>;
    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    fn redo(&mut self, edit_state: &mut EditState) -> EngineResult<()>;
}
