use std::{cmp::{max, min}, backtrace::Backtrace};

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
    pub margins: Option<(i32, i32)>,
    pub mouse_mode: MouseMode,
    use_ice: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseMode {
    Default,
    Highlight,
    X10,
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
            margins: None,
            use_ice: false,
        }
    }

    pub fn reset(&mut self) {
        self.margins = None;
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
                let first = buf.get_first_visible_line();
         /*       let n = min(first + buf.get_buffer_height() - 1, max(first, caret.pos.y));
                if n < caret.pos.y {
                    println!("limit! {}", Backtrace::force_capture());
                }
                caret.pos.y = n;*/
                caret.pos.x = min(self.width - 1, max(0, caret.pos.x));
            }
            crate::OriginMode::WithinMargins => {
                let first = buf.get_first_editable_line();
                let height = buf.get_last_editable_line() - first;
                let n = min(first + height - 1, max(first, caret.pos.y));
                caret.pos.y = n;
                caret.pos.x = min(self.width - 1, max(0, caret.pos.x));
            }
        }
    }
}
