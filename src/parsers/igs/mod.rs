use super::BufferParser;
use crate::{AttributedChar, Buffer, CallbackAction, Caret, EngineResult, Size};

mod cmd;
use cmd::IgsCommands;

const IGS_VERSION: &str = "1.8";

#[derive(Default, Debug)]
enum State {
    #[default]
    Default,
    GotIgsStart,
    ReadCommandStart,
    ReadCommand(IgsCommands),
}

#[derive(Default, Debug)]
enum LoopState {
    #[default]
    Start,
    ReadCommand,
    ReadCount,
    ReadParameter,
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
    loop_state: LoopState,
    igs_texture: Vec<u8>,
    loop_parameters: Vec<Vec<String>>,
}

impl Parser {
    pub fn new(fallback_parser: Box<dyn BufferParser>) -> Self {
        Self {
            fallback_parser,
            state: State::Default,
            parsed_numbers: Vec::new(),
            terminal_resolution: TerminalResolution::Medium,
            igs_texture: Vec::new(),
            parsed_string: String::new(),
            loop_state: LoopState::Start,
            loop_parameters: Vec::new(),
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
        // println!("{} {:?} - numbers:{:?}", ch, self.state, self.parsed_numbers);
        match &self.state {
            State::ReadCommand(command) => {
                if *command == IgsCommands::WriteText && self.parsed_numbers.len() >= 3 {
                    self.parsed_string.push(ch);
                    if ch == '@' || ch == '\n' || ch == '\r' {
                        self.parsed_string.clear();
                        self.state = State::ReadCommandStart;
                        return Ok(CallbackAction::NoUpdate);
                    }
                    return Ok(CallbackAction::NoUpdate);
                }
                if *command == IgsCommands::LoopCommand && self.parsed_numbers.len() >= 4 {
                    self.parsed_string.push(ch);
                    match self.loop_state {
                        LoopState::Start => {
                            if ch == ',' {
                                self.loop_state = LoopState::ReadCommand;
                            }
                        }
                        LoopState::ReadCommand => {
                            if ch == '@' || ch == '|' || ch == ',' {
                                self.loop_state = LoopState::ReadCount;
                                self.parsed_numbers.push(0);

                                self.parsed_string.clear();
                            }
                        }
                        LoopState::ReadCount => match ch {
                            '0'..='9' => {
                                let d = match self.parsed_numbers.pop() {
                                    Some(number) => number,
                                    _ => 0,
                                };
                                self.parsed_numbers.push(parse_next_number(d, ch as u8));
                            }
                            ',' => {
                                self.loop_parameters.clear();
                                self.loop_parameters.push(vec![String::new()]);

                                self.loop_state = LoopState::ReadParameter;
                            }
                            _ => {
                                self.state = State::Default;
                            }
                        },
                        LoopState::ReadParameter => match ch {
                            ',' => {
                                self.loop_parameters.last_mut().unwrap().push(String::new());
                                if self.parsed_numbers[4] < self.loop_parameters.len() as i32 {
                                    self.state = State::ReadCommandStart;
                                }
                            }

                            ':' => {
                                self.loop_parameters.push(vec![String::new()]);

                                if self.parsed_numbers[4] < self.loop_parameters.len() as i32 {
                                    self.state = State::ReadCommandStart;
                                }
                            }
                            _ => {
                                self.loop_parameters.last_mut().unwrap().last_mut().unwrap().push(ch);
                            }
                        },
                    }
                    return Ok(CallbackAction::NoUpdate);
                }
                match ch {
                    ' ' | '>' | '_' | '\n' | '\r' => { /* ignore */ }

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
                        let res = self.execute_command(*command);
                        self.state = State::ReadCommandStart;
                        return res;
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
                    'A' => {
                        self.state = State::ReadCommand(IgsCommands::AttributeForFills);
                        Ok(CallbackAction::NoUpdate)
                    }
                    'b' => {
                        self.state = State::ReadCommand(IgsCommands::BellsAndWhistles);
                        Ok(CallbackAction::NoUpdate)
                    }
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
                        self.loop_state = LoopState::Start;
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
                    }
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

#[cfg(test)]
mod tests {
    use crate::{
        ascii,
        parsers::{create_buffer, get_simple_action, update_buffer_force},
        CallbackAction, TextPane,
    };

    #[test]
    pub fn test_igs_version() {
        let mut igs_parser = super::Parser::new(Box::<ascii::Parser>::default());
        let action = get_simple_action(&mut igs_parser, b"G#?>0:");
        if let CallbackAction::SendString(version) = action {
            assert_eq!(version, super::IGS_VERSION);
        } else {
            panic!("Expected SendString action was :{action:?}");
        }
    }

    #[test]
    pub fn parse_two_commands() {
        let mut igs_parser = super::Parser::new(Box::<ascii::Parser>::default());
        let action = get_simple_action(&mut igs_parser, b"G#?>0:?>0:");
        if let CallbackAction::SendString(version) = action {
            assert_eq!(version, super::IGS_VERSION);
        } else {
            panic!("Expected SendString action was :{action:?}");
        }
    }

    #[test]
    pub fn test_eol_marker() {
        let mut igs_parser = super::Parser::new(Box::<ascii::Parser>::default());
        let action = get_simple_action(&mut igs_parser, b"G#?>_\n\r0:?>_\n\r0:");
        if let CallbackAction::SendString(version) = action {
            assert_eq!(version, super::IGS_VERSION);
        } else {
            panic!("Expected SendString action was :{action:?}");
        }
    }

    #[test]
    pub fn test_text_break_bug() {
        let mut igs_parser = super::Parser::new(Box::<ascii::Parser>::default());
        let (mut buf, mut caret) = create_buffer(&mut igs_parser, b"");
        update_buffer_force(
            &mut buf,
            &mut caret,
            &mut igs_parser,
            b"G#W>20,50,Chain@L 0,0,300,190:W>253,_\n140,IG SUPPORT BOARD@",
        );
        assert_eq!(' ', buf.get_char((0, 0)).ch);
    }

    #[test]
    pub fn test_loop() {
        let mut igs_parser = super::Parser::new(Box::<ascii::Parser>::default());
        let (mut buf, mut caret) = create_buffer(&mut igs_parser, b"");
        update_buffer_force(&mut buf, &mut caret, &mut igs_parser, b"G#&>0,320,4,0,L,8,0,100,x,0:0,100,x,199:");
        assert_eq!(' ', buf.get_char((0, 0)).ch);
    }
}
