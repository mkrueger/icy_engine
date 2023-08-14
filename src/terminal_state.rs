use std::cmp::max;

use crate::{Buffer, Caret};

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
    pub width: i32,
    pub height: i32,

    pub origin_mode: OriginMode,
    pub scroll_state: TerminalScrolling,
    pub auto_wrap_mode: AutoWrapMode,
    pub margins_up_down: Option<(i32, i32)>,
    pub margins_left_right: Option<(i32, i32)>,
    pub mouse_mode: MouseMode,
    pub dec_margin_mode_left_right: bool,

    pub font_selection_state: FontSelectionState,

    pub normal_attribute_font_slot: usize,
    pub high_intensity_attribute_font_slot: usize,
    pub blink_attribute_font_slot: usize,
    pub high_intensity_blink_attribute_font_slot: usize,

    tab_stops: Vec<i32>,
    use_ice: bool,
    baud_rate: u32,
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
    pub fn from(width: i32, height: i32) -> Self {
        let mut ret = Self {
            width,
            height,
            scroll_state: TerminalScrolling::Smooth,
            origin_mode: OriginMode::UpperLeftCorner,
            auto_wrap_mode: AutoWrapMode::AutoWrap,
            mouse_mode: MouseMode::Default,
            margins_up_down: None,
            margins_left_right: None,
            use_ice: false,
            dec_margin_mode_left_right: false,
            baud_rate: 0,
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
        while i < self.width {
            self.tab_stops.push(i);
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
            self.width
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

    pub fn get_baud_rate(&self) -> u32 {
        self.baud_rate
    }

    pub fn set_baud_rate(&mut self, baud_rate: u32) {
        self.baud_rate = baud_rate;
    }

    pub fn reset(&mut self) {
        self.margins_up_down = None;
        self.origin_mode = OriginMode::UpperLeftCorner;
        self.scroll_state = TerminalScrolling::Smooth;
        self.auto_wrap_mode = AutoWrapMode::AutoWrap;
        self.reset_tabs();
    }

    pub fn use_ice_colors(&self) -> bool {
        self.use_ice
    }
    pub fn set_use_ice_colors(&mut self, use_ice: bool) {
        self.use_ice = use_ice;
    }

    pub fn limit_caret_pos(&self, buf: &Buffer, caret: &mut Caret) {
        match self.origin_mode {
            crate::OriginMode::UpperLeftCorner => {
                /*      let first = buf.get_first_visible_line();
                let n = min(first + buf.get_buffer_height() - 1, max(first, caret.pos.y));
                if n < caret.pos.y {
                    println!("limit! {}", Backtrace::force_capture());
                }
                caret.pos.y = n;*/
                caret.pos.x = caret.pos.x.clamp(0, max(0, self.width - 1));
            }
            crate::OriginMode::WithinMargins => {
                let first = buf.get_first_editable_line();
                let height = buf.get_last_editable_line() - first;
                let n = caret.pos.y.clamp(first, max(first, first + height - 1));
                caret.pos.y = n;
                caret.pos.x = caret.pos.x.clamp(0, max(0, self.width - 1));
            }
        }
    }
}
