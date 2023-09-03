use crate::{CallbackAction, EngineResult};

use super::EditState;

impl EditState {
    pub fn print_char_unsafe(&mut self, c: char) -> EngineResult<CallbackAction> {
        self.parser
            .print_char(&mut self.buffer, 0, &mut self.caret, c)
    }
}
