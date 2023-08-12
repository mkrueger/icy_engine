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
    use_ice: bool,
    baud_rate: u32
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
        Self {
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
            baud_rate: 0
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
