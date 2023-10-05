use super::BufferParser;
use crate::{AttributedChar, Buffer, CallbackAction, Caret, EngineResult, Size};

mod cmd;
use cmd::IgsCommands;

const IGS_VERSION: &str = "1.8";

#[derive(Default)]
enum State {
    #[default]
    Default,
    GotIgsStart,
    ReadCommandStart,
    ReadCommand(IgsCommands),
}

pub enum TerminalResolution {
    /// 320x200
    Low,
    /// 640x200
    Medium,
    /// 640x400  
    High,
}

impl TerminalResolution {
    pub fn resolution_id(&self) -> String {
        match self {
            TerminalResolution::Low => "0".to_string(),
            TerminalResolution::Medium => "1".to_string(),
            TerminalResolution::High => "2".to_string(),
        }
    }

    pub fn get_resolution(&self) -> Size {
        match self {
            TerminalResolution::Low => Size { width: 320, height: 200 },
            TerminalResolution::Medium => Size { width: 640, height: 200 },
            TerminalResolution::High => Size { width: 640, height: 400 },
        }
    }
}

pub struct Parser {
    fallback_parser: Box<dyn BufferParser>,
    state: State,
    parsed_numbers: Vec<i32>,
    parsed_string: String,
    terminal_resolution: TerminalResolution,

    igs_texture: Vec<u8>,
}

impl Parser {
    pub fn new(fallback_parser: Box<dyn BufferParser>) -> Self {
        Self {
            fallback_parser,
            state: State::Default,
            parsed_numbers: Vec::new(),
            terminal_resolution: TerminalResolution::Medium,
            igs_texture: Vec::new(),
            parsed_string: String::new()
        }
    }
    pub fn clear(&mut self) {
        // clear viewport
    }

    pub fn set_resolution(&mut self) {
        let res = self.terminal_resolution.get_resolution();
        self.igs_texture = vec![0; res.width as usize * 2 * 4 * res.height as usize * 2];
    }

    pub fn set_palette(&mut self) {
        // TODO
    }

    pub fn reset_attributes(&mut self) {
        // TODO
    }

    fn execute_command(&mut self, command: IgsCommands) -> EngineResult<CallbackAction> {
        match command {
            IgsCommands::Initialize => {
                if self.parsed_numbers.len() != 1 {
                    return Err(anyhow::anyhow!("Initialize command requires 1 argument"));
                }
                match self.parsed_numbers.pop().unwrap() {
                    0 => {
                        self.set_resolution();
                        self.set_palette();
                        self.reset_attributes();
                    }
                    1 => {
                        self.set_resolution();
                        self.set_palette();
                    }
                    2 => {
                        self.reset_attributes();
                    }
                    3 => {
                        self.set_resolution();
                    }
                    x => return Err(anyhow::anyhow!("Initialize unknown/unsupported argument: {x}")),
                }

                Ok(CallbackAction::Update)
            }
            IgsCommands::AskIG => {
                if self.parsed_numbers.len() != 1 {
                    return Err(anyhow::anyhow!("AskIG command requires 1 argument"));
                }
                match self.parsed_numbers.pop().unwrap() {
                    0 => Ok(CallbackAction::SendString(IGS_VERSION.to_string())),
                    3 => Ok(CallbackAction::SendString(self.terminal_resolution.resolution_id())),
                    x => Err(anyhow::anyhow!("AskIG unknown/unsupported argument: {x}")),
                }
            }
            _ => Err(anyhow::anyhow!("Unimplemented IGS command: {command:?}")),
        }
    }
}

impl BufferParser for Parser {
    fn convert_from_unicode(&self, ch: char, font_page: usize) -> char {
        self.fallback_parser.convert_from_unicode(ch, font_page)
    }

    fn convert_to_unicode(&self, ch: AttributedChar) -> char {
        self.fallback_parser.convert_to_unicode(ch)
    }

    fn print_char(&mut self, buf: &mut Buffer, current_layer: usize, caret: &mut Caret, ch: char) -> EngineResult<CallbackAction> {
        match &self.state {
            State::ReadCommand(command) => {
                if *command == IgsCommands::WriteText && self.parsed_numbers.len() >= 2 {
                    self.parsed_string.push(ch);
                    if ch == '@' {
                        println!("parsed string:{}", self.parsed_string);
                        self.parsed_string.clear();
                        self.state = State::ReadCommandStart;
                        return Ok(CallbackAction::NoUpdate); 
                    }
                }
                match ch {
                    ' ' | '>' | '_' => { /* ignore */ }
                    '0'..='9' => {
                        let d = match self.parsed_numbers.pop() {
                            Some(number) => number,
                            _ => 0,
                        };
                        self.parsed_numbers.push(parse_next_number(d, ch as u8));
                    }
                    ',' => {
                        self.parsed_numbers.push(0);
                    }
                    ':' => {
                        self.execute_command(*command)?;
                        self.state = State::ReadCommandStart;
                    }
                    _ => {
                        self.state = State::Default;
                    }
                }
                Ok(CallbackAction::NoUpdate)
            }
            State::ReadCommandStart => {
                self.parsed_numbers.clear();
                match ch {
                    '\n' | '\r' => Ok(CallbackAction::NoUpdate),
                    'B' => {
                        self.state = State::ReadCommand(IgsCommands::Box);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'C' => {
                        self.state = State::ReadCommand(IgsCommands::ColorSet);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'D' => {
                        self.state = State::ReadCommand(IgsCommands::LineDrawTo);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'E' => {
                        self.state = State::ReadCommand(IgsCommands::TextEffects);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'F' => {
                        self.state = State::ReadCommand(IgsCommands::FloodFill);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'g' => {
                        self.state = State::ReadCommand(IgsCommands::GraphicScaling);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'G' => {
                        self.state = State::ReadCommand(IgsCommands::GrabScreen);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'q' => {
                        self.state = State::ReadCommand(IgsCommands::QuickPause);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'H' => {
                        self.state = State::ReadCommand(IgsCommands::HollowSet);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'I' => {
                        self.state = State::ReadCommand(IgsCommands::Initialize);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'J' => {
                        self.state = State::ReadCommand(IgsCommands::EllipticalArc);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'k' => {
                        self.state = State::ReadCommand(IgsCommands::Cursor);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'K' => {
                        self.state = State::ReadCommand(IgsCommands::Arc);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'L' => {
                        self.state = State::ReadCommand(IgsCommands::DrawLine);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'M' => {
                        self.state = State::ReadCommand(IgsCommands::DrawingMode);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'n' => {
                        self.state = State::ReadCommand(IgsCommands::ChipMusic);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'N' => {
                        self.state = State::ReadCommand(IgsCommands::Noise);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'O' => {
                        self.state = State::ReadCommand(IgsCommands::Circle);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'P' => {
                        self.state = State::ReadCommand(IgsCommands::PolymarkerPlot);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'Q' => {
                        self.state = State::ReadCommand(IgsCommands::Ellipse);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'R' => {
                        self.state = State::ReadCommand(IgsCommands::SetResolution);
                        Ok(CallbackAction::NoUpdate)
                    }
                    's' => {
                        self.state = State::ReadCommand(IgsCommands::ScreenClear);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'S' => {
                        self.state = State::ReadCommand(IgsCommands::SetPenColor);
                        Ok(CallbackAction::NoUpdate)
                    }
                    't' => {
                        self.state = State::ReadCommand(IgsCommands::TimeAPause);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'T' => {
                        self.state = State::ReadCommand(IgsCommands::LineMarkerTypes);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'U' => {
                        self.state = State::ReadCommand(IgsCommands::RoundedRectangles);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'V' => {
                        self.state = State::ReadCommand(IgsCommands::Pieslice);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'W' => {
                        self.state = State::ReadCommand(IgsCommands::WriteText);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'Y' => {
                        self.state = State::ReadCommand(IgsCommands::EllipticalPieslice);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'Z' => {
                        self.state = State::ReadCommand(IgsCommands::FilledRectangle);
                        Ok(CallbackAction::NoUpdate)
                    }
                    '<' => {
                        self.state = State::ReadCommand(IgsCommands::InputCommand);
                        Ok(CallbackAction::NoUpdate)
                    }
                    '?' => {
                        self.state = State::ReadCommand(IgsCommands::AskIG);
                        Ok(CallbackAction::NoUpdate)
                    }
                    '&' => {
                        self.state = State::ReadCommand(IgsCommands::LoopCommand);
                        Ok(CallbackAction::NoUpdate)
                    }

                    // Modified VT-52 Commands
                    'c' => {
                        self.state = State::ReadCommand(IgsCommands::VTColor);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'd' => {
                        self.state = State::ReadCommand(IgsCommands::VTDeleteLine);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'i' => {
                        self.state = State::ReadCommand(IgsCommands::VTLineInsert);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'l' => {
                        self.state = State::ReadCommand(IgsCommands::VTLineClear);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'm' => {
                        self.state = State::ReadCommand(IgsCommands::VTCursorMotion);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'p' => {
                        self.state = State::ReadCommand(IgsCommands::VTPosition);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'r' => {
                        self.state = State::ReadCommand(IgsCommands::VTRemember);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'v' => {
                        self.state = State::ReadCommand(IgsCommands::VTInverseVideo);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'w' => {
                        self.state = State::ReadCommand(IgsCommands::VTLineWrap);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'X' => {
                        self.state = State::ReadCommand(IgsCommands::ExtendedCommands);
                        Ok(CallbackAction::NoUpdate)
                    }
                    _ => {
                        self.state = State::Default;
                        Err(anyhow::anyhow!("Unknown IGS command: {ch}"))
                    },
                }
            }
            State::GotIgsStart => {
                if ch == '#' {
                    self.state = State::ReadCommandStart;
                    return Ok(CallbackAction::NoUpdate);
                }
                self.state = State::Default;
                let _ = self.fallback_parser.print_char(buf, current_layer, caret, 'G');
                self.fallback_parser.print_char(buf, current_layer, caret, ch)
            }
            State::Default => {
                if ch == 'G' {
                    self.state = State::GotIgsStart;
                    return Ok(CallbackAction::NoUpdate);
                }
                self.fallback_parser.print_char(buf, current_layer, caret, ch)
            }
        }
    }
}

pub fn parse_next_number(x: i32, ch: u8) -> i32 {
    x.saturating_mul(10).saturating_add(ch as i32).saturating_sub(b'0' as i32)
}
