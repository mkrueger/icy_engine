#![allow(clippy::missing_errors_doc)]

use crate::EngineResult;

use super::EditState;

impl EditState {
    fn get_area(&self) -> (i32, i32, i32, i32) {
        if let Some(selection) = &self.get_selection() {
            let min = selection.min();
            let max = selection.max();
            (min.x, min.y, max.x, max.y)
        } else {
            let size = self.buffer.get_buffer_size();
            (0, 0, size.width - 1, size.height - 1)
        }
    }

    pub fn justify_left(&mut self) -> EngineResult<()> {
        /*
            let _undo = self.begin_atomic_undo("Justify left");
            let (x1, y1, x2, y2) = self.get_blockaction_rectangle();
            let is_bg_layer =
                self.get_cur_layer() == self.buffer_view.lock().get_buffer().layers.len() - 1;
            {
                let lock = &mut self.buffer_view.lock();
                let cur_layer = self.get_cur_layer();
                let layer = &mut lock.get_buffer_mut().layers[cur_layer];
                for y in y1..=y2 {
                    let mut removed_chars = 0;
                    let len = x2 - x1 + 1;
                    while removed_chars < len {
                        let ch = layer.get_char(Position::new(x1 + removed_chars, y));
                        if ch.is_visible() && !ch.is_transparent() {
                            break;
                        }
                        removed_chars += 1;
                    }
                    if len == removed_chars {
                        continue;
                    }
                    for x in x1..=x2 {
                        let ch = if x + removed_chars <= x2 {
                            layer.get_char(Position::new(x + removed_chars, y))
                        } else if is_bg_layer {
                            AttributedChar::default()
                        } else {
                            AttributedChar::invisible()
                        };

                        let pos = Position::new(x, y);
                        self.buffer_view
                            .lock()
                            .get_edit_state_mut()
                            .set_char(pos, ch)
                            .unwrap();
                    }
                }
            }

        */
        Ok(())
    }

    pub fn center(&mut self) -> EngineResult<()> {
        /*   let _undo = self.begin_atomic_undo("Justify center");
            self.justify_left();

            let (x1, y1, x2, y2) = self.get_blockaction_rectangle();
            let is_bg_layer =
                self.get_cur_layer() == self.buffer_view.lock().get_buffer().layers.len() - 1;
            {
                let mut lock: eframe::epaint::mutex::MutexGuard<'_, BufferView> =
                    self.buffer_view.lock();
                let cur_layer = self.get_cur_layer();
                let layer = &mut lock.get_buffer_mut().layers[cur_layer];
                for y in y1..=y2 {
                    let mut removed_chars = 0;
                    let len = x2 - x1 + 1;
                    while removed_chars < len {
                        let ch = layer.get_char(Position::new(x2 - removed_chars, y));
                        if ch.is_visible() && !ch.is_transparent() {
                            break;
                        }
                        removed_chars += 1;
                    }

                    if len == removed_chars {
                        continue;
                    }
                    removed_chars /= 2;
                    for x in 0..len {
                        let ch = if x2 - x - removed_chars >= x1 {
                            layer.get_char(Position::new(x2 - x - removed_chars, y))
                        } else if is_bg_layer {
                            AttributedChar::default()
                        } else {
                            AttributedChar::invisible()
                        };

                        let pos = Position::new(x2 - x, y);
                        let _ = self
                            .buffer_view
                            .lock()
                            .get_edit_state_mut()
                            .set_char(pos, ch);
                    }
                }
            }
        */
        Ok(())
    }

    pub fn justify_right(&mut self) -> EngineResult<()> {
        /*
            let _undo = self.begin_atomic_undo("Justify right");
            let (x1, y1, x2, y2) = self.get_blockaction_rectangle();
            let is_bg_layer =
                self.get_cur_layer() == self.buffer_view.lock().get_buffer().layers.len() - 1;
            {
                let mut lock: eframe::epaint::mutex::MutexGuard<'_, BufferView> =
                    self.buffer_view.lock();
                let cur_layer = self.get_cur_layer();
                let layer = &mut lock.get_buffer_mut().layers[cur_layer];
                for y in y1..=y2 {
                    let mut removed_chars = 0;
                    let len = x2 - x1 + 1;
                    while removed_chars < len {
                        let ch = layer.get_char(Position::new(x2 - removed_chars, y));
                        if ch.is_visible() && !ch.is_transparent() {
                            break;
                        }
                        removed_chars += 1;
                    }

                    if len == removed_chars {
                        continue;
                    }
                    for x in 0..len {
                        let ch = if x2 - x - removed_chars >= x1 {
                            layer.get_char(Position::new(x2 - x - removed_chars, y))
                        } else if is_bg_layer {
                            AttributedChar::default()
                        } else {
                            AttributedChar::invisible()
                        };

                        let pos = Position::new(x2 - x, y);
                        let _ = self
                            .buffer_view
                            .lock()
                            .get_edit_state_mut()
                            .set_char(pos, ch);
                    }
                }
            }
        */
        Ok(())
    }

    pub fn flip_x(&mut self) -> EngineResult<()> {
        /*    let _undo = self.begin_atomic_undo("Flip X");
        let (x1, y1, x2, y2) = self.get_blockaction_rectangle();
        {
            for y in y1..=y2 {
                for x in 0..=(x2 - x1) / 2 {
                    let pos1 = Position::new(x1 + x, y);
                    let pos2 = Position::new(x2 - x, y);
                    let _ = self
                        .buffer_view
                        .lock()
                        .get_edit_state_mut()
                        .swap_char(pos1, pos2);
                }
            }
        }*/
        Ok(())
    }

    pub fn flip_y(&mut self) -> EngineResult<()> {
        /*   let _undo = self.begin_atomic_undo("Flip Y");
        let (x1, y1, x2, y2) = self.get_blockaction_rectangle();
        {
            for x in x1..=x2 {
                for y in 0..=(y2 - y1) / 2 {
                    let pos1 = Position::new(x, y1 + y);
                    let pos2 = Position::new(x, y2 - y);
                    let _ = self
                        .buffer_view
                        .lock()
                        .get_edit_state_mut()
                        .swap_char(pos1, pos2);
                }
            }
        }*/
        Ok(())
    }

    pub fn crop(&mut self) -> EngineResult<()> {
        /*
        let _undo = self.begin_atomic_undo("Crop");
        let (x1, y1, x2, y2) = self.get_blockaction_rectangle();

        let new_height = y2 - y1;
        let new_width = x2 - x1;

        if new_height == self.buffer_view.lock().get_buffer().get_line_count()
            && new_width == self.buffer_view.lock().get_buffer().get_width()
        {
            return;
        }

        let mut new_layers = Vec::new();
        let max = self.buffer_view.lock().get_buffer().layers.len();
        for l in 0..max {
            let lock = &self.buffer_view.lock();
            let old_layer = &lock.get_buffer().layers[l];
            let mut new_layer = Layer::default();
            new_layer.title = old_layer.title.clone();
            new_layer.is_visible = old_layer.is_visible;
            new_layer.set_offset(Position::new(0, 0));
            new_layer.lines = Vec::new();
            for y in y1..=y2 {
                for x in x1..=x2 {
                    new_layer.set_char(
                        Position::new(x - x1, y - y1),
                        old_layer.get_char(Position::new(x, y)),
                    );
                }
            }

            new_layer.is_locked = old_layer.is_locked;
            new_layer.is_position_locked = old_layer.is_position_locked;
            new_layers.push(new_layer);
        }

        /* TODO
        self.undo_stack.push(Box::new(super::UndoReplaceLayers {

            old_layer: self.buffer_view.lock().get_buffer().layers.clone(),
            new_layer: new_layers.clone(),
            old_size: Size::new(
                self.buffer_view.lock().get_buffer().get_width(),
                self.buffer_view.lock().get_buffer().get_line_count(),
            ),c
            new_size: Size::new(new_width, new_height),
        })); */

        self.buffer_view.lock().get_buffer_mut().layers = new_layers;
        self.buffer_view
            .lock()
            .get_buffer_mut()
            .set_buffer_width(new_width);
        self.buffer_view
            .lock()
            .get_buffer_mut()
            .set_buffer_height(new_height);*/
        Ok(())
    }
}
