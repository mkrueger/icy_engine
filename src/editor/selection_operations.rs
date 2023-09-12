#![allow(clippy::missing_errors_doc)]
use super::{undo_operations, EditState};
use crate::{AttributedChar, EngineResult, Position, Rectangle, Selection, TextPane};

impl EditState {
    pub fn get_selection(&self) -> Option<Selection> {
        self.selection_opt
    }

    pub fn set_selection(&mut self, sel: impl Into<Selection>) -> EngineResult<()> {
        let sel = sel.into();
        let selection = Some(sel);
        if self.selection_opt == selection {
            Ok(())
        } else {
            self.push_undo(Box::new(undo_operations::SetSelection::new(
                self.selection_opt,
                selection,
            )))
        }
    }

    pub fn clear_selection(&mut self) -> EngineResult<()> {
        if self.is_something_selected() {
            let sel = self.selection_opt.take();
            let mask = self.selection_mask.clone();
            self.push_undo(Box::new(undo_operations::SelectNothing::new(sel, mask)))
        } else {
            Ok(())
        }
    }

    pub fn deselect(&mut self) -> EngineResult<()> {
        if let Some(sel) = self.selection_opt.take() {
            self.push_undo(Box::new(undo_operations::Deselect::new(sel)))
        } else {
            Ok(())
        }
    }

    fn is_something_selected(&self) -> bool {
        self.selection_opt.is_some() || !self.selection_mask.is_empty()
    }

    pub fn get_is_selected(&self, pos: impl Into<Position>) -> bool {
        let pos = pos.into();
        if let Some(sel) = self.selection_opt {
            if sel.is_inside(pos) {
                return !sel.is_negative_selection;
            }
        }

        self.selection_mask.get_is_selected(pos)
    }

    pub fn add_selection_to_mask(&mut self) -> EngineResult<()> {
        if let Some(selection) = self.selection_opt {
            self.push_undo(Box::new(undo_operations::AddSelectionToMask::new(
                self.selection_mask.clone(),
                selection,
            )))
        } else {
            Ok(())
        }
    }

    pub fn get_selected_rectangle(&self) -> Rectangle {
        let mut rect = self.selection_mask.get_rectangle();
        if let Some(sel) = self.selection_opt {
            rect = rect.union(&sel.as_rectangle());
        }
        rect
    }

    /// .
    ///
    /// # Panics
    ///
    /// Panics if .
    pub fn enumerate_selections<F>(&mut self, f: F)
    where
        F: Fn(Position, AttributedChar, bool) -> Option<bool>,
    {
        let old_mask = self.selection_mask.clone();
        for y in 0..self.buffer.get_height() {
            for x in 0..self.buffer.get_width() {
                let pos = Position::new(x, y);
                let is_selected = self.selection_mask.get_is_selected(pos);
                if let Some(res) = f(pos, self.buffer.get_char(pos), is_selected) {
                    self.selection_mask.set_is_selected(pos, res);
                }
            }
        }

        if old_mask != self.selection_mask {
            self.redo_stack.clear();
            self.undo_stack
                .lock()
                .unwrap()
                .push(Box::new(undo_operations::SetSelectionMask::new(
                    old_mask,
                    self.selection_mask.clone(),
                )));
        }
    }
}
