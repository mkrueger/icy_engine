#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use std::mem;

use i18n_embed_fl::fl;

use crate::{AttributedChar, EngineResult, Layer, Position, Rectangle, Selection, TextPane};

use super::{EditState, EditorError};

fn get_area(sel: Option<Selection>, layer: Rectangle) -> Rectangle {
    if let Some(selection) = sel {
        let rect = selection.as_rectangle();
        rect.intersect(&layer) - layer.start
    } else {
        layer - layer.start
    }
}
impl EditState {
    pub fn justify_left(&mut self) -> EngineResult<()> {
        let _undo = self.begin_atomic_undo(fl!(crate::LANGUAGE_LOADER, "undo-justify-left"));
        let sel = self.get_selection();
        if let Some(layer) = self.get_cur_layer_mut() {
            let area = get_area(sel, layer.get_rectangle());
            let old_layer = Layer::from_layer(layer, area);
            for y in area.y_range() {
                let mut removed_chars = 0;
                let len = area.get_width();
                while removed_chars < len {
                    let ch = layer.get_char((area.left() + removed_chars, y));
                    if ch.is_visible() && !ch.is_transparent() {
                        break;
                    }
                    removed_chars += 1;
                }
                if len <= removed_chars {
                    continue;
                }
                for x in area.x_range() {
                    let ch = if x + removed_chars < area.right() {
                        layer.get_char((x + removed_chars, y))
                    } else {
                        AttributedChar::invisible()
                    };
                    layer.set_char(Position::new(x, y), ch);
                }
            }
            let new_layer = Layer::from_layer(layer, area);
            let op = super::undo_operations::UndoLayerChange::new(
                self.get_current_layer(),
                area.start,
                old_layer,
                new_layer,
            );
            self.redo_stack.clear();
            self.undo_stack.lock().unwrap().push(Box::new(op));
            Ok(())
        } else {
            Err(super::EditorError::CurrentLayerInvalid.into())
        }
    }

    pub fn center(&mut self) -> EngineResult<()> {
        let _undo = self.begin_atomic_undo(fl!(crate::LANGUAGE_LOADER, "undo-center"));
        let sel = self.get_selection();
        self.justify_left()?;
        if let Some(layer) = self.get_cur_layer_mut() {
            let area = get_area(sel, layer.get_rectangle());
            let old_layer = Layer::from_layer(layer, area);

            for y in area.y_range() {
                let mut removed_chars = 0;
                let len = area.get_width();
                while removed_chars < len {
                    let ch = layer.get_char((area.right() - removed_chars - 1, y));
                    if ch.is_visible() && !ch.is_transparent() {
                        break;
                    }
                    removed_chars += 1;
                }
                if len == removed_chars {
                    continue;
                }
                removed_chars = (removed_chars as f32 / 2.0).ceil() as i32;
                for x in area.x_range() {
                    let ch = if area.right() - x - removed_chars >= area.left() {
                        layer.get_char((area.right() - x - removed_chars, y))
                    } else {
                        AttributedChar::invisible()
                    };

                    layer.set_char((area.right() - x - 1, y), ch);
                }
            }
            let new_layer = Layer::from_layer(layer, area);
            let op = super::undo_operations::UndoLayerChange::new(
                self.get_current_layer(),
                area.start,
                old_layer,
                new_layer,
            );
            self.redo_stack.clear();
            self.undo_stack.lock().unwrap().push(Box::new(op));
            Ok(())
        } else {
            Err(EditorError::CurrentLayerInvalid.into())
        }
    }

    pub fn justify_right(&mut self) -> EngineResult<()> {
        let _undo = self.begin_atomic_undo(fl!(crate::LANGUAGE_LOADER, "undo-justify-right"));
        let sel = self.get_selection();
        if let Some(layer) = self.get_cur_layer_mut() {
            let area = get_area(sel, layer.get_rectangle());
            let old_layer = Layer::from_layer(layer, area);

            for y in area.y_range() {
                let mut removed_chars = 0;
                let len = area.get_width();
                while removed_chars < len {
                    let ch = layer.get_char((area.right() - removed_chars - 1, y));
                    if ch.is_visible() && !ch.is_transparent() {
                        break;
                    }
                    removed_chars += 1;
                }
                if len == removed_chars {
                    continue;
                }
                for x in area.x_range() {
                    let ch = if area.right() - x - removed_chars > area.left() {
                        layer.get_char((area.right() - x - removed_chars - 1, y))
                    } else {
                        AttributedChar::invisible()
                    };

                    layer.set_char((area.right() - x - 1, y), ch);
                }
            }
            let new_layer = Layer::from_layer(layer, area);
            let op = super::undo_operations::UndoLayerChange::new(
                self.get_current_layer(),
                area.start,
                old_layer,
                new_layer,
            );
            self.redo_stack.clear();
            self.undo_stack.lock().unwrap().push(Box::new(op));
            Ok(())
        } else {
            Err(EditorError::CurrentLayerInvalid.into())
        }
    }

    pub fn flip_x(&mut self) -> EngineResult<()> {
        let _undo = self.begin_atomic_undo(fl!(crate::LANGUAGE_LOADER, "undo-flip-x"));
        let sel = self.get_selection();
        if let Some(layer) = self.get_cur_layer_mut() {
            let area = get_area(sel, layer.get_rectangle());
            let old_layer = Layer::from_layer(layer, area);
            let max = area.get_width() / 2;

            for y in area.y_range() {
                for x in 0..=max {
                    let pos1 = Position::new(area.left() + x, y);
                    let pos2 = Position::new(area.right() - x - 1, y);
                    layer.swap_char(pos1, pos2);
                }
            }
            let new_layer = Layer::from_layer(layer, area);
            let op = super::undo_operations::UndoLayerChange::new(
                self.get_current_layer(),
                area.start,
                old_layer,
                new_layer,
            );
            self.redo_stack.clear();
            self.undo_stack.lock().unwrap().push(Box::new(op));
            Ok(())
        } else {
            Err(EditorError::CurrentLayerInvalid.into())
        }
    }

    pub fn flip_y(&mut self) -> EngineResult<()> {
        let _undo = self.begin_atomic_undo(fl!(crate::LANGUAGE_LOADER, "undo-flip-x"));
        let sel = self.get_selection();
        if let Some(layer) = self.get_cur_layer_mut() {
            let area = get_area(sel, layer.get_rectangle());
            let old_layer = Layer::from_layer(layer, area);
            let max = area.get_height() / 2;

            for x in area.x_range() {
                for y in 0..=max {
                    let pos1 = Position::new(x, area.top() + y);
                    let pos2 = Position::new(x, area.bottom() - 1 - y);
                    layer.swap_char(pos1, pos2);
                }
            }
            let new_layer = Layer::from_layer(layer, area);
            let op = super::undo_operations::UndoLayerChange::new(
                self.get_current_layer(),
                area.start,
                old_layer,
                new_layer,
            );
            self.redo_stack.clear();
            self.undo_stack.lock().unwrap().push(Box::new(op));
            Ok(())
        } else {
            Err(EditorError::CurrentLayerInvalid.into())
        }
    }

    pub fn crop(&mut self) -> EngineResult<()> {
        let sel = self.get_selection().unwrap().as_rectangle();
        self.crop_rect(sel)
    }

    pub fn crop_rect(&mut self, rect: Rectangle) -> EngineResult<()> {
        let old_size = self.get_buffer().get_size();
        let mut old_layers = Vec::new();
        mem::swap(&mut self.get_buffer_mut().layers, &mut old_layers);

        self.get_buffer_mut().set_size(rect.size);
        self.get_buffer_mut().layers.clear();

        for old_layer in &old_layers {
            let mut new_layer = old_layer.clone();
            new_layer.lines.clear();
            let new_rectangle = old_layer.get_rectangle().intersect(&rect);
            if new_rectangle.is_empty() {
                continue;
            }

            new_layer.set_offset(new_rectangle.start - rect.start);
            new_layer.set_size(new_rectangle.size);

            for y in 0..new_rectangle.get_height() {
                for x in 0..new_rectangle.get_width() {
                    let ch =
                        old_layer.get_char((x + new_rectangle.left(), y + new_rectangle.top()));
                    new_layer.set_char((x, y), ch);
                }
            }
            self.get_buffer_mut().layers.push(new_layer);
        }
        let op = super::undo_operations::Crop::new(old_size, rect.get_size(), old_layers);
        self.redo_stack.clear();
        self.undo_stack.lock().unwrap().push(Box::new(op));

        Ok(())
    }

    /// Returns the delete selection of this [`EditState`].
    ///
    /// # Panics
    ///
    /// Panics if .
    pub fn erase_selection(&mut self) -> EngineResult<()> {
        if !self.is_something_selected() {
            return Ok(());
        }
        let _undo = self.begin_atomic_undo(fl!(crate::LANGUAGE_LOADER, "undo-delete-selection"));
        let layer_idx = self.get_current_layer();
        let (area, old_layer) = if let Some(layer) = self.buffer.layers.get_mut(layer_idx) {
            (layer.get_rectangle(), layer.clone())
        } else {
            return Err(EditorError::CurrentLayerInvalid.into());
        };

        for y in 0..area.get_height() {
            for x in 0..area.get_width() {
                let pos = Position::new(x, y);
                if self.get_is_selected(pos + area.start) {
                    self.buffer
                        .layers
                        .get_mut(layer_idx)
                        .unwrap()
                        .set_char(pos, AttributedChar::invisible());
                }
            }
        }
        let new_layer = self.buffer.layers.get_mut(layer_idx).unwrap().clone();
        let op = super::undo_operations::UndoLayerChange::new(
            self.get_current_layer(),
            area.start,
            old_layer,
            new_layer,
        );
        self.redo_stack.clear();
        self.undo_stack.lock().unwrap().push(Box::new(op));
        self.clear_selection()
    }

    pub fn scroll_area_up(&mut self) -> EngineResult<()> {
        let _undo = self.begin_atomic_undo(fl!(crate::LANGUAGE_LOADER, "undo-justify-left"));
        let sel = self.get_selection();
        if let Some(layer) = self.get_cur_layer_mut() {
            let area = get_area(sel, layer.get_rectangle());
            if area.is_empty() {
                return Ok(());
            }
            if area.get_width() >= layer.get_width() {
                let op =
                    super::undo_operations::UndoScrollWholeLayerUp::new(self.get_current_layer());
                return self.push_undo(Box::new(op));
            }

            let old_layer = Layer::from_layer(layer, area);

            let mut saved_line = Vec::new();

            for y in area.y_range() {
                let line = &mut layer.lines[y as usize];
                if line.chars.len() < area.right() as usize {
                    line.chars
                        .resize(area.right() as usize, AttributedChar::invisible());
                }
                if y == area.top() {
                    saved_line.extend(
                        line.chars
                            .drain(area.left() as usize..area.right() as usize),
                    );
                    continue;
                }
                if y == area.bottom() - 1 {
                    line.chars.splice(
                        area.right() as usize..area.right() as usize,
                        saved_line.iter().copied(),
                    );
                }
                let chars = line
                    .chars
                    .drain(area.left() as usize..area.right() as usize)
                    .collect::<Vec<AttributedChar>>();
                let line_above = &mut layer.lines[y as usize - 1];
                line_above
                    .chars
                    .splice(area.left() as usize..area.left() as usize, chars);
            }
            let new_layer = Layer::from_layer(layer, area);
            let op = super::undo_operations::UndoLayerChange::new(
                self.get_current_layer(),
                area.start,
                old_layer,
                new_layer,
            );
            self.redo_stack.clear();
            self.undo_stack.lock().unwrap().push(Box::new(op));
            Ok(())
        } else {
            Err(super::EditorError::CurrentLayerInvalid.into())
        }
    }

    pub fn scroll_area_down(&mut self) -> EngineResult<()> {
        let _undo = self.begin_atomic_undo(fl!(crate::LANGUAGE_LOADER, "undo-justify-left"));
        let sel = self.get_selection();
        if let Some(layer) = self.get_cur_layer_mut() {
            let area = get_area(sel, layer.get_rectangle());
            if area.is_empty() {
                return Ok(());
            }
            if area.get_width() >= layer.get_width() {
                let op =
                    super::undo_operations::UndoScrollWholeLayerDown::new(self.get_current_layer());
                return self.push_undo(Box::new(op));
            }
            let old_layer = Layer::from_layer(layer, area);

            let mut saved_line = Vec::new();

            for y in area.y_range().rev() {
                let line = &mut layer.lines[y as usize];
                if line.chars.len() < area.right() as usize {
                    line.chars
                        .resize(area.right() as usize, AttributedChar::invisible());
                }
                if y == area.bottom() - 1 {
                    saved_line.extend(
                        line.chars
                            .drain(area.left() as usize..area.right() as usize),
                    );
                    continue;
                }
                if y == area.top() {
                    line.chars.splice(
                        area.right() as usize..area.right() as usize,
                        saved_line.iter().copied(),
                    );
                }
                let chars = line
                    .chars
                    .drain(area.left() as usize..area.right() as usize)
                    .collect::<Vec<AttributedChar>>();
                let line_below = &mut layer.lines[y as usize + 1];
                line_below
                    .chars
                    .splice(area.left() as usize..area.left() as usize, chars);
            }
            let new_layer = Layer::from_layer(layer, area);
            let op = super::undo_operations::UndoLayerChange::new(
                self.get_current_layer(),
                area.start,
                old_layer,
                new_layer,
            );
            self.redo_stack.clear();
            self.undo_stack.lock().unwrap().push(Box::new(op));
            Ok(())
        } else {
            Err(super::EditorError::CurrentLayerInvalid.into())
        }
    }

    pub fn scroll_area_left(&mut self) -> EngineResult<()> {
        let _undo = self.begin_atomic_undo(fl!(crate::LANGUAGE_LOADER, "undo-justify-left"));
        let sel = self.get_selection();
        if let Some(layer) = self.get_cur_layer_mut() {
            let area = get_area(sel, layer.get_rectangle());
            if area.is_empty() {
                return Ok(());
            }
            let old_layer = Layer::from_layer(layer, area);
            for y in area.y_range() {
                let line = &mut layer.lines[y as usize];
                if line.chars.len() < area.right() as usize {
                    line.chars
                        .resize(area.right() as usize, AttributedChar::invisible());
                }
                let ch = line.chars.remove(area.left() as usize);
                line.chars.insert(area.right() as usize - 1, ch);
            }
            let new_layer = Layer::from_layer(layer, area);
            let op = super::undo_operations::UndoLayerChange::new(
                self.get_current_layer(),
                area.start,
                old_layer,
                new_layer,
            );
            self.redo_stack.clear();
            self.undo_stack.lock().unwrap().push(Box::new(op));
            Ok(())
        } else {
            Err(super::EditorError::CurrentLayerInvalid.into())
        }
    }

    pub fn scroll_area_right(&mut self) -> EngineResult<()> {
        let _undo = self.begin_atomic_undo(fl!(crate::LANGUAGE_LOADER, "undo-justify-left"));
        let sel = self.get_selection();
        if let Some(layer) = self.get_cur_layer_mut() {
            let area = get_area(sel, layer.get_rectangle());
            if area.is_empty() {
                return Ok(());
            }
            let old_layer = Layer::from_layer(layer, area);
            for y in area.y_range() {
                let line = &mut layer.lines[y as usize];
                if line.chars.len() < area.right() as usize {
                    line.chars
                        .resize(area.right() as usize, AttributedChar::invisible());
                }
                let ch = line.chars.remove(area.right() as usize - 1);
                line.chars.insert(area.left() as usize, ch);
            }
            let new_layer = Layer::from_layer(layer, area);
            let op = super::undo_operations::UndoLayerChange::new(
                self.get_current_layer(),
                area.start,
                old_layer,
                new_layer,
            );
            self.redo_stack.clear();
            self.undo_stack.lock().unwrap().push(Box::new(op));
            Ok(())
        } else {
            Err(super::EditorError::CurrentLayerInvalid.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        editor::{EditState, UndoState},
        Layer, Position, Rectangle, Size, TextPane,
    };

    #[test]
    fn test_delete_selection() {
        let mut state = EditState::default();
        for y in 0..20 {
            for x in 0..20 {
                state.set_char((x, y), '#'.into()).unwrap();
            }
        }

        let rect = Rectangle::from(5, 5, 10, 10);
        state.set_selection(rect).unwrap();
        state.erase_selection().unwrap();
        for y in 0..20 {
            for x in 0..20 {
                let pos = Position::new(x, y);
                let ch = state.get_buffer().get_char(pos);

                if rect.is_inside(pos) {
                    assert_eq!(ch.ch, ' ');
                } else {
                    assert_eq!(ch.ch, '#');
                }
            }
        }

        state.undo().unwrap();

        for y in 0..20 {
            for x in 0..20 {
                let pos = Position::new(x, y);
                let ch = state.get_buffer().get_char(pos);
                assert_eq!(ch.ch, '#');
            }
        }
    }

    #[test]
    fn test_flip_x() {
        let mut state = EditState::default();
        for y in 0..20 {
            for x in 0..20 {
                state.set_char((x, y), '#'.into()).unwrap();
            }
        }

        state.set_selection(Rectangle::from(0, 0, 10, 10)).unwrap();
        state.erase_selection().unwrap();

        state.set_selection(Rectangle::from(0, 0, 10, 10)).unwrap();
        state.set_char((3, 5), '#'.into()).unwrap();
        state.set_char((0, 9), '#'.into()).unwrap();

        state.flip_x().unwrap();
        for y in 10..20 {
            for x in 10..20 {
                let ch = state.get_buffer().get_char((x, y));
                assert_eq!(ch.ch, '#');
            }
        }

        for y in 0..10 {
            for x in 0..10 {
                let ch = state.get_buffer().get_char((x, y));
                if x == 9 && y == 9 || x == 6 && y == 5 {
                    assert_eq!(ch.ch, '#');
                } else {
                    assert_eq!(ch.ch, ' ');
                }
            }
        }

        state.undo().unwrap();
        for y in 0..10 {
            for x in 0..10 {
                let ch = state.get_buffer().get_char((x, y));

                if x == 3 && y == 5 || x == 0 && y == 9 {
                    assert_eq!(ch.ch, '#');
                } else {
                    assert_eq!(ch.ch, ' ');
                }
            }
        }
    }

    #[test]
    fn test_flip_y() {
        let mut state = EditState::default();
        for y in 0..20 {
            for x in 0..20 {
                state.set_char((x, y), '#'.into()).unwrap();
            }
        }

        state.set_selection(Rectangle::from(0, 0, 10, 10)).unwrap();
        state.erase_selection().unwrap();

        state.set_selection(Rectangle::from(0, 0, 10, 10)).unwrap();
        state.set_char((3, 3), '#'.into()).unwrap();
        state.set_char((9, 9), '#'.into()).unwrap();

        state.flip_y().unwrap();
        for y in 10..20 {
            for x in 10..20 {
                let ch = state.get_buffer().get_char((x, y));
                assert_eq!(ch.ch, '#');
            }
        }

        for y in 0..10 {
            for x in 0..10 {
                let ch = state.get_buffer().get_char((x, y));
                if x == 9 && y == 0 || x == 3 && y == 6 {
                    assert_eq!(ch.ch, '#');
                } else {
                    assert_eq!(ch.ch, ' ');
                }
            }
        }

        state.undo().unwrap();
        for y in 0..10 {
            for x in 0..10 {
                let ch = state.get_buffer().get_char((x, y));

                if x == 3 && y == 3 || x == 9 && y == 9 {
                    assert_eq!(ch.ch, '#');
                } else {
                    assert_eq!(ch.ch, ' ');
                }
            }
        }
    }
    #[test]
    fn test_justify_right() {
        let mut state = EditState::default();
        for y in 0..20 {
            for x in 0..20 {
                state.set_char((x, y), '#'.into()).unwrap();
            }
        }

        state.set_selection(Rectangle::from(0, 0, 10, 10)).unwrap();
        state.erase_selection().unwrap();

        state.set_selection(Rectangle::from(0, 0, 10, 10)).unwrap();
        state.set_char((5, 5), '#'.into()).unwrap();
        state.set_char((0, 9), '#'.into()).unwrap();

        state.justify_right().unwrap();

        for y in 10..20 {
            for x in 10..20 {
                let ch = state.get_buffer().get_char((x, y));
                assert_eq!(ch.ch, '#');
            }
        }

        for y in 0..10 {
            for x in 0..10 {
                let ch = state.get_buffer().get_char((x, y));
                if x == 9 && (y == 5 || y == 9) {
                    assert_eq!(ch.ch, '#');
                } else {
                    assert_eq!(ch.ch, ' ');
                }
            }
        }

        state.undo().unwrap();
        for y in 0..10 {
            for x in 0..10 {
                let ch = state.get_buffer().get_char((x, y));

                if x == 5 && y == 5 || x == 0 && y == 9 {
                    assert_eq!(ch.ch, '#');
                } else {
                    assert_eq!(ch.ch, ' ');
                }
            }
        }
    }

    #[test]
    fn test_center() {
        let mut state = EditState::default();
        for y in 0..20 {
            for x in 0..20 {
                state.set_char((x, y), '#'.into()).unwrap();
            }
        }

        state.set_selection(Rectangle::from(0, 0, 10, 10)).unwrap();
        state.erase_selection().unwrap();

        state.set_selection(Rectangle::from(0, 0, 10, 10)).unwrap();
        state.set_char((0, 5), '#'.into()).unwrap();
        state.set_char((9, 9), '#'.into()).unwrap();

        state.center().unwrap();

        for y in 10..20 {
            for x in 10..20 {
                let ch = state.get_buffer().get_char((x, y));
                assert_eq!(ch.ch, '#');
            }
        }
        for y in 0..10 {
            for x in 0..10 {
                let ch = state.get_buffer().get_char((x, y));
                if x == 4 && (y == 5 || y == 9) {
                    assert_eq!(ch.ch, '#');
                } else {
                    assert_eq!(ch.ch, ' ');
                }
            }
        }
        state.undo().unwrap();
        for y in 0..10 {
            for x in 0..10 {
                let ch = state.get_buffer().get_char((x, y));

                if x == 0 && y == 5 || x == 9 && y == 9 {
                    assert_eq!(ch.ch, '#');
                } else {
                    assert_eq!(ch.ch, ' ');
                }
            }
        }
    }

    #[test]
    fn test_justify_left() {
        let mut state = EditState::default();
        for y in 0..20 {
            for x in 0..20 {
                state.set_char((x, y), '#'.into()).unwrap();
            }
        }

        state.set_selection(Rectangle::from(0, 0, 10, 10)).unwrap();
        state.erase_selection().unwrap();

        state.set_selection(Rectangle::from(0, 0, 10, 10)).unwrap();
        state.set_char((5, 5), '#'.into()).unwrap();
        state.set_char((9, 9), '#'.into()).unwrap();

        state.justify_left().unwrap();

        for y in 10..20 {
            for x in 10..20 {
                let ch = state.get_buffer().get_char((x, y));
                assert_eq!(ch.ch, '#');
            }
        }
        for y in 0..10 {
            for x in 0..10 {
                let ch = state.get_buffer().get_char((x, y));
                if x == 0 && (y == 5 || y == 9) {
                    assert_eq!(ch.ch, '#');
                } else {
                    assert_eq!(ch.ch, ' ');
                }
            }
        }

        state.undo().unwrap();
        for y in 0..10 {
            for x in 0..10 {
                let ch = state.get_buffer().get_char((x, y));

                if x == 5 && y == 5 || x == 9 && y == 9 {
                    assert_eq!(ch.ch, '#');
                } else {
                    assert_eq!(ch.ch, ' ');
                }
            }
        }
    }

    #[test]
    fn test_crop() {
        let mut state = EditState::default();

        let mut layer = Layer::new("1", Size::new(100, 100));
        layer.set_offset((-5, -5));
        state.get_buffer_mut().layers.push(layer);

        let mut layer = Layer::new("2", Size::new(2, 2));
        layer.set_offset((7, 6));
        state.get_buffer_mut().layers.push(layer);

        state.set_selection(Rectangle::from(5, 5, 5, 4)).unwrap();

        state.crop().unwrap();

        assert_eq!(state.get_buffer().get_width(), 5);
        assert_eq!(state.get_buffer().get_height(), 4);
        assert_eq!(state.get_buffer().layers[1].get_size(), Size::new(5, 4));
        assert_eq!(state.get_buffer().layers[2].get_size(), Size::new(2, 2));

        state.undo().unwrap();

        assert_eq!(state.get_buffer().get_width(), 80);
        assert_eq!(state.get_buffer().get_height(), 25);
        assert_eq!(state.get_buffer().layers[1].get_size(), Size::new(100, 100));
        assert_eq!(state.get_buffer().layers[2].get_size(), Size::new(2, 2));
    }
}
