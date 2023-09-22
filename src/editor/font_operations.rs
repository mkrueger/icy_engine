#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use crate::{BitFont, EngineResult, IceMode, Layer, Palette, PaletteMode, DOS_DEFAULT_PALETTE};

use super::EditState;

impl EditState {
    pub fn switch_to_font_page(&mut self, page: usize) -> EngineResult<()> {
        let op = super::undo_operations::SwitchToFontPage::new(self.caret.get_font_page(), page);
        self.push_undo(Box::new(op))
    }
    pub fn add_ansi_font(&mut self, page: usize) -> EngineResult<()> {
        match self.get_buffer().font_mode {
            crate::FontMode::Unlimited => {
                let new_font = BitFont::from_ansi_font_page(page)?;
                let op = super::undo_operations::AddFont::new(
                    self.caret.get_font_page(),
                    page,
                    new_font,
                );
                self.push_undo(Box::new(op))
            }
            crate::FontMode::Sauce | crate::FontMode::Single | crate::FontMode::FixedSize => {
                Err(anyhow::anyhow!("Not supported for this buffer type."))
            }
        }
    }

    pub fn set_ansi_font(&mut self, page: usize) -> EngineResult<()> {
        match self.get_buffer().font_mode {
            crate::FontMode::Sauce => Err(anyhow::anyhow!("Not supported for sauce buffers.")),
            crate::FontMode::Single => {
                let new_font = BitFont::from_ansi_font_page(page)?;
                let op = super::undo_operations::SetFont::new(
                    0,
                    self.get_buffer().get_font(0).unwrap().clone(),
                    new_font,
                );
                self.push_undo(Box::new(op))
            }
            crate::FontMode::Unlimited | crate::FontMode::FixedSize => {
                let new_font = BitFont::from_ansi_font_page(page)?;
                let op = super::undo_operations::SetFont::new(
                    self.caret.get_font_page(),
                    self.get_buffer().get_font(0).unwrap().clone(),
                    new_font,
                );
                self.push_undo(Box::new(op))
            }
        }
    }

    pub fn set_sauce_font(&mut self, name: &str) -> EngineResult<()> {
        match self.get_buffer().font_mode {
            crate::FontMode::Sauce | crate::FontMode::Single => {
                let new_font = BitFont::from_sauce_name(name)?;
                let op = super::undo_operations::SetFont::new(
                    0,
                    self.get_buffer().get_font(0).unwrap().clone(),
                    new_font,
                );
                self.push_undo(Box::new(op))
            }
            crate::FontMode::Unlimited | crate::FontMode::FixedSize => {
                let new_font = BitFont::from_sauce_name(name)?;
                let op = super::undo_operations::SetFont::new(
                    self.caret.get_font_page(),
                    self.get_buffer().get_font(0).unwrap().clone(),
                    new_font,
                );
                self.push_undo(Box::new(op))
            }
        }
    }

    pub fn add_font(&mut self, new_font: BitFont) -> EngineResult<()> {
        match self.get_buffer().font_mode {
            crate::FontMode::Unlimited => {
                let mut page = 100;
                for i in 100.. {
                    if !self.get_buffer().has_font(i) {
                        page = i;
                        break;
                    }
                }

                let op = super::undo_operations::AddFont::new(
                    self.caret.get_font_page(),
                    page,
                    new_font,
                );
                self.push_undo(Box::new(op))
            }
            crate::FontMode::Sauce | crate::FontMode::Single | crate::FontMode::FixedSize => {
                Err(anyhow::anyhow!("Not supported for this buffer type."))
            }
        }
    }

    pub fn set_font(&mut self, new_font: BitFont) -> EngineResult<()> {
        match self.get_buffer().font_mode {
            crate::FontMode::Sauce => Err(anyhow::anyhow!("Not supported for sauce buffers.")),
            crate::FontMode::Single => {
                let op = super::undo_operations::SetFont::new(
                    0,
                    self.get_buffer().get_font(0).unwrap().clone(),
                    new_font,
                );
                self.push_undo(Box::new(op))
            }
            crate::FontMode::Unlimited | crate::FontMode::FixedSize => {
                let op = super::undo_operations::SetFont::new(
                    self.caret.get_font_page(),
                    self.get_buffer().get_font(0).unwrap().clone(),
                    new_font,
                );
                self.push_undo(Box::new(op))
            }
        }
    }

    pub fn set_palette_mode(&mut self, mode: PaletteMode) -> EngineResult<()> {
        let old_palette = self.get_buffer().palette.clone();
        let old_mode = self.get_buffer().palette_mode;
        let old_layers = self.get_buffer().layers.clone();
        let new_palette = match mode {
            PaletteMode::RGB => old_palette.clone(),
            PaletteMode::Fixed16 => Palette::from_colors(DOS_DEFAULT_PALETTE.to_vec()),
            PaletteMode::Free8 => get_palette(&old_layers, &old_palette, 8),
            PaletteMode::Free16 => get_palette(&old_layers, &old_palette, 16),
        };
        self.adjust_layer_colors(&old_palette, &new_palette);
        let new_layers = self.get_buffer().layers.clone();

        let op = super::undo_operations::SwitchPalette::new(
            old_mode,
            old_palette,
            old_layers,
            mode,
            new_palette,
            new_layers,
        );
        self.push_undo(Box::new(op))
    }

    fn adjust_layer_colors(&mut self, old_palette: &crate::Palette, new_palette: &crate::Palette) {
        for layer in &mut self.get_buffer_mut().layers {
            for line in &mut layer.lines {
                for ch in &mut line.chars {
                    let fg = ch.attribute.get_foreground();
                    ch.attribute
                        .set_foreground(find_new_color(old_palette, new_palette, fg));
                    let bg = ch.attribute.get_background();
                    ch.attribute
                        .set_background(find_new_color(old_palette, new_palette, bg));
                }
            }
        }
    }

    pub fn set_ice_mode(&mut self, mode: IceMode) -> EngineResult<()> {
        let old_layers = self.get_buffer().layers.clone();
        let old_mode = self.get_buffer().ice_mode;

        let mut new_layers = old_layers.clone();
        match mode {
            IceMode::Unlimited => { /* no conversion needed */ }
            IceMode::Blink => {
                if self.caret.attribute.get_background() > 7 {
                    self.caret.attribute.set_is_blinking(true);
                    self.caret
                        .attribute
                        .set_background(self.caret.attribute.get_background() - 8);
                }

                for layer in &mut new_layers {
                    for line in &mut layer.lines {
                        for ch in &mut line.chars {
                            let bg = ch.attribute.get_background();
                            if bg > 7 && bg < 16 {
                                ch.attribute.set_is_blinking(true);
                                ch.attribute.set_background(bg - 8);
                            }
                        }
                    }
                }
            }
            IceMode::Ice => {
                if self.caret.attribute.is_blinking() {
                    self.caret.attribute.set_is_blinking(false);
                    if self.caret.attribute.get_background() < 8 {
                        self.caret
                            .attribute
                            .set_background(self.caret.attribute.get_background() + 8);
                    }
                }

                for layer in &mut new_layers {
                    for line in &mut layer.lines {
                        for ch in &mut line.chars {
                            if ch.attribute.is_blinking() {
                                ch.attribute.set_is_blinking(false);
                                let bg = ch.attribute.get_background();
                                if bg < 8 {
                                    ch.attribute.set_background(bg + 8);
                                }
                            }
                        }
                    }
                }
            }
        };
        let op = super::undo_operations::SetIceMode::new(old_mode, old_layers, mode, new_layers);
        self.push_undo(Box::new(op))
    }
}

fn get_palette(old_layers: &[Layer], old_palette: &Palette, palette_size: usize) -> Palette {
    let mut color_count = vec![0; old_palette.len()];
    for layer in old_layers {
        for line in &layer.lines {
            for ch in &line.chars {
                let fg = ch.attribute.get_foreground();
                let bg = ch.attribute.get_background();
                color_count[fg as usize] += 1;
                color_count[bg as usize] += 1;
            }
        }
    }
    let mut new_palette = Palette::new();
    new_palette.insert_color(old_palette.get_color(0));
    while new_palette.len() < palette_size {
        let mut max = -1;
        let mut idx = 0;
        (1..old_palette.len()).for_each(|i| {
            if color_count[i] > max {
                max = color_count[i];
                idx = i;
            }
        });
        if max < 0 {
            break;
        }
        color_count[idx] = 0;
        new_palette.insert_color(old_palette.get_color(idx));
    }
    new_palette.resize(palette_size);
    new_palette
}

fn find_new_color(old_palette: &Palette, new_palette: &Palette, color: u32) -> u32 {
    let (o_r, o_g, o_b) = old_palette.get_rgb(color as usize);
    let o_r = o_r as i32;
    let o_g = o_g as i32;
    let o_b = o_b as i32;

    let mut new_color = 0;
    let mut delta = i32::MAX;
    for i in 0..new_palette.len() {
        let (r, g, b) = new_palette.get_rgb(i);
        let r = r as i32;
        let g = g as i32;
        let b = b as i32;

        let new_delta = (o_r - r).abs() + (o_g - g).abs() + (o_b - b).abs();
        if new_delta < delta {
            new_color = i;
            delta = new_delta;
        }
    }
    new_color as u32
}
