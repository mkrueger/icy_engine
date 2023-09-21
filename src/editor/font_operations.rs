#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use crate::{BitFont, EngineResult, IceMode, Palette, PaletteMode, DOS_DEFAULT_PALETTE};

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
            crate::FontMode::Sauce | crate::FontMode::Single | crate::FontMode::Dual => {
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
            crate::FontMode::Unlimited | crate::FontMode::Dual => {
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
            crate::FontMode::Unlimited | crate::FontMode::Dual => {
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
            crate::FontMode::Sauce | crate::FontMode::Single | crate::FontMode::Dual => {
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
            crate::FontMode::Unlimited | crate::FontMode::Dual => {
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

        let new_palette = match mode {
            PaletteMode::RGB => old_palette.clone(),
            PaletteMode::Fixed16 => Palette::from_colors(DOS_DEFAULT_PALETTE.to_vec()),
            PaletteMode::Free8 => {
                let mut new = old_palette.clone();
                new.resize(8);
                new
            }
            PaletteMode::Free16 => {
                let mut new = old_palette.clone();
                new.resize(16);
                new
            }
        };
        let op =
            super::undo_operations::SwitchPalette::new(old_mode, old_palette, mode, new_palette);
        self.push_undo(Box::new(op))
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
                    self.caret.attribute.set_background(self.caret.attribute.get_background() - 8);
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
                        self.caret.attribute.set_background(self.caret.attribute.get_background() + 8);
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
