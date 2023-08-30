use std::cmp::max;

use crate::{ansi::BaudEmulation, Buffer, Caret, Size};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TerminalScrolling {
    Smooth,
    Fast,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OriginMode {
    UpperLeftCorner,
    WithinMargins,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AutoWrapMode {
    NoWrap,
    AutoWrap,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontSelectionState {
    NoRequest,
    Success,
    Failure,
}

#[derive(Debug)]
pub struct TerminalState {
    size: Size,

    pub origin_mode: OriginMode,
    pub scroll_state: TerminalScrolling,
    pub auto_wrap_mode: AutoWrapMode,
    margins_top_bottom: Option<(i32, i32)>,
    margins_left_right: Option<(i32, i32)>,
    pub mouse_mode: MouseMode,
    pub dec_margin_mode_left_right: bool,

    pub font_selection_state: FontSelectionState,

    pub normal_attribute_font_slot: usize,
    pub high_intensity_attribute_font_slot: usize,
    pub blink_attribute_font_slot: usize,
    pub high_intensity_blink_attribute_font_slot: usize,

    tab_stops: Vec<i32>,
    baud_rate: BaudEmulation,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseMode {
    // no mouse reporting
    Default,

    /// X10 compatibility mode (9)
    X10,
    /// VT200 mode (1000)
    VT200,
    /// VT200 highlight mode (1001)
    #[allow(non_camel_case_types)]
    VT200_Highlight,

    ButtonEvents,
    AnyEvents,
    FocusEvent,
    AlternateScroll,
    ExtendedMode,
    SGRExtendedMode,
    URXVTExtendedMode,
    PixelPosition,
}

impl TerminalState {
    pub fn from(size: impl Into<Size>) -> Self {
        let mut ret = Self {
            size: size.into(),
            scroll_state: TerminalScrolling::Smooth,
            origin_mode: OriginMode::UpperLeftCorner,
            auto_wrap_mode: AutoWrapMode::AutoWrap,
            mouse_mode: MouseMode::Default,
            margins_top_bottom: None,
            margins_left_right: None,
            dec_margin_mode_left_right: false,
            baud_rate: BaudEmulation::Off,
            tab_stops: vec![],
            font_selection_state: FontSelectionState::NoRequest,
            normal_attribute_font_slot: 0,
            high_intensity_attribute_font_slot: 0,
            blink_attribute_font_slot: 0,
            high_intensity_blink_attribute_font_slot: 0,
        };
        ret.reset_tabs();
        ret
    }

    pub fn get_width(&self) -> usize {
        self.size.width
    }

    pub fn set_width(&mut self, width: usize) {
        self.size.width = width;
        self.reset_tabs();
    }

    pub fn get_height(&self) -> usize {
        self.size.height
    }

    pub fn set_height(&mut self, height: usize) {
        self.size.height = height;
    }

    pub fn get_size(&self) -> Size {
        self.size
    }
    pub fn set_size(&mut self, size: impl Into<Size>) {
        self.size = size.into();
    }

    pub fn tab_count(&self) -> usize {
        self.tab_stops.len()
    }

    pub fn get_tabs(&self) -> &[i32] {
        &self.tab_stops
    }

    pub fn clear_tab_stops(&mut self) {
        self.tab_stops.clear();
    }

    pub fn remove_tab_stop(&mut self, x: i32) {
        self.tab_stops.retain(|&t| t != x);
    }

    fn reset_tabs(&mut self) {
        let mut i = 0;
        self.tab_stops.clear();
        while i < self.get_width() {
            self.tab_stops.push(i as i32);
            i += 8;
        }
    }

    pub fn next_tab_stop(&mut self, x: i32) -> i32 {
        let mut i = 0;
        while i < self.tab_stops.len() && self.tab_stops[i] <= x {
            i += 1;
        }
        if i < self.tab_stops.len() {
            self.tab_stops[i]
        } else {
            self.get_width() as i32
        }
    }

    pub fn prev_tab_stop(&mut self, x: i32) -> i32 {
        let mut i = self.tab_stops.len() as i32 - 1;
        while i >= 0 && self.tab_stops[i as usize] >= x {
            i -= 1;
        }
        if i >= 0 {
            self.tab_stops[i as usize]
        } else {
            0
        }
    }

    pub fn set_tab_at(&mut self, x: i32) {
        if !self.tab_stops.contains(&x) {
            self.tab_stops.push(x);
            self.tab_stops.sort_unstable();
        }
    }

    pub fn get_baud_emulation(&self) -> BaudEmulation {
        self.baud_rate
    }

    pub fn set_baud_rate(&mut self, baud_rate: BaudEmulation) {
        self.baud_rate = baud_rate;
    }

    pub fn limit_caret_pos(&self, buf: &Buffer, caret: &mut Caret) {
        match self.origin_mode {
            crate::OriginMode::UpperLeftCorner => {
                if buf.is_terminal_buffer {
                    let first = buf.get_first_visible_line() as i32;
                    let n = (first + buf.get_height() as i32 - 1).min(first.max(caret.pos.y));

                    caret.pos.y = n;
                }
                caret.pos.x = caret.pos.x.clamp(0, max(0, self.get_width() as i32 - 1));
            }
            crate::OriginMode::WithinMargins => {
                let first = buf.get_first_editable_line() as i32;
                let height = buf.get_last_editable_line() as i32 - first;
                let n = caret.pos.y.clamp(first, max(first, first + height - 1));
                caret.pos.y = n;
                caret.pos.x = caret.pos.x.clamp(0, max(0, self.get_width() as i32 - 1));
            }
        }
    }

    pub fn get_margins_top_bottom(&self) -> Option<(i32, i32)> {
        self.margins_top_bottom
    }

    pub fn get_margins_left_right(&self) -> Option<(i32, i32)> {
        self.margins_left_right
    }

    pub fn set_margins_top_bottom(&mut self, top: i32, bottom: i32) {
        self.margins_top_bottom = if top > bottom {
            None
        } else {
            Some((top, bottom))
        };
    }

    pub fn set_margins_left_right(&mut self, left: i32, right: i32) {
        self.margins_left_right = if left > right {
            None
        } else {
            Some((left, right))
        };
    }

    pub fn clear_margins_top_bottom(&mut self) {
        self.margins_top_bottom = None;
    }

    pub fn clear_margins_left_right(&mut self) {
        self.margins_left_right = None;
    }
}
