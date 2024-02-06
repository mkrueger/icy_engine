use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use super::BufferParser;
use crate::{Buffer, CallbackAction, Caret, EngineResult, Size};

mod cmd;
use cmd::IgsCommands;
mod paint;
pub use paint::*;

#[cfg(test)]
mod tests;
const IGS_VERSION: &str = "2.19";

#[derive(Default, Debug)]
enum State {
    #[default]
    Default,
    GotIgsStart,
    ReadCommandStart,
    SkipNewLine,
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

pub trait CommandExecutor: Send + Sync {
    fn get_resolution(&self) -> Size;
    fn get_picture_data(&mut self) -> Option<(Size, Vec<u8>)> {
        None
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    fn execute_command(
        &mut self,
        buf: &mut Buffer,
        caret: &mut Caret,
        command: IgsCommands,
        parameters: &[i32],
        string_parameter: &str,
    ) -> EngineResult<CallbackAction>;
}

pub struct Parser {
    fallback_parser: Box<dyn BufferParser>,
    state: State,
    parsed_numbers: Vec<i32>,
    parsed_string: String,
    loop_state: LoopState,
    loop_cmd: char,
    loop_parameters: Vec<Vec<String>>,
    command_executor: Arc<Mutex<Box<dyn CommandExecutor>>>,
    got_double_colon: bool,

    cur_loop: Option<Loop>,
}
struct Loop {
    i: i32,
    from: i32,
    to: i32,
    step: i32,
    delay: i32,
    command: IgsCommands,
    parsed_string: String,
    parameters: Vec<Vec<String>>,
}

impl Loop {
    fn new(from: i32, to: i32, step: i32, delay: i32, command: char, parsed_string: String, loop_parameters: Vec<Vec<String>>) -> EngineResult<Self> {
        let command = IgsCommands::from_char(command)?;
        Ok(Self {
            i: from,
            from,
            to,
            step,
            delay,
            command,
            parsed_string,
            parameters: loop_parameters,
        })
    }

    fn next_step(&mut self, exe: &Arc<Mutex<Box<dyn CommandExecutor>>>, buf: &mut Buffer, caret: &mut Caret) -> Option<EngineResult<CallbackAction>> {
        let is_running = if self.from < self.to { self.i < self.to } else { self.i > self.to };
        if !is_running {
            return None;
        }
        let cur_parameter = ((self.i - self.from) as usize) % self.parameters.len();
        let mut parameters = Vec::new();
        for p in &self.parameters[cur_parameter] {
            let mut p = p.clone();
            let mut add_step_value = false;
            let mut subtract_const_value = false;
            let mut subtract_x_step = false;

            if p.starts_with('+') {
                add_step_value = true;
                p.remove(0);
            } else if p.starts_with('-') {
                subtract_const_value = true;
                p.remove(0);
            } else if p.starts_with('!') {
                subtract_x_step = true;
                p.remove(0);
            }

            let x = (self.i).abs();
            let y = (self.to - 1 - self.i).abs();
            let mut value = if p == "x" {
                x
            } else if p == "y" {
                y
            } else {
                match p.parse::<i32>() {
                    Err(_) => {
                        println!("error parsing parameter: {p}");
                        continue;
                    }
                    Ok(i) => i,
                }
            };

            if add_step_value {
                value += x;
            }
            if subtract_const_value {
                value = x - value;
            }
            if subtract_x_step {
                value -= x;
            }
            parameters.push(value);
        }
        // println!("step: {:?} => {:?}", self.loop_parameters[cur_parameter], parameters);
        let res = exe.lock().unwrap().execute_command(buf, caret, self.command, &parameters, &self.parsed_string);
        // todo: correct delay?
        std::thread::sleep(Duration::from_millis(200 * self.delay as u64));
        if self.from < self.to {
            self.i += self.step;
        } else {
            self.i -= self.step;
        }

        match res {
            Ok(r) => Some(Ok(r)),
            Err(err) => Some(Err(err)),
        }
    }
}

impl Parser {
    pub fn new(fallback_parser: Box<dyn BufferParser>, command_executor: Arc<Mutex<Box<dyn CommandExecutor>>>) -> Self {
        Self {
            fallback_parser,
            state: State::Default,
            parsed_numbers: Vec::new(),
            command_executor,
            parsed_string: String::new(),
            loop_state: LoopState::Start,
            loop_parameters: Vec::new(),
            loop_cmd: ' ',
            got_double_colon: false,
            cur_loop: None,
        }
    }
}

impl BufferParser for Parser {
    fn get_next_action(&mut self, buffer: &mut Buffer, caret: &mut Caret, _current_layer: usize) -> Option<CallbackAction> {
        if let Some(l) = &mut self.cur_loop {
            if let Some(x) = l.next_step(&self.command_executor, buffer, caret) {
                if let Ok(act) = x {
                    return Some(act);
                }
                return None;
            }
            self.cur_loop = None;
        }
        None
    }

    fn print_char(&mut self, buf: &mut Buffer, current_layer: usize, caret: &mut Caret, ch: char) -> EngineResult<CallbackAction> {
        // println!("{} {:?} - numbers:{:?}", ch as u32, self.state, self.parsed_numbers);
        match &self.state {
            State::ReadCommand(command) => {
                if *command == IgsCommands::WriteText && self.parsed_numbers.len() >= 3 {
                    if ch == '@' {
                        let parameters: Vec<_> = self.parsed_numbers.drain(..).collect();
                        let res = self
                            .command_executor
                            .lock()
                            .unwrap()
                            .execute_command(buf, caret, *command, &parameters, &self.parsed_string);
                        self.state = State::ReadCommandStart;
                        self.parsed_string.clear();
                        return res;
                    }
                    self.parsed_string.push(ch);
                    if ch == '\n' {
                        self.parsed_string.clear();
                        self.state = State::ReadCommandStart;
                        return Ok(CallbackAction::NoUpdate);
                    }
                    return Ok(CallbackAction::NoUpdate);
                }
                if *command == IgsCommands::LoopCommand && self.parsed_numbers.len() >= 4 {
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
                            } else {
                                self.loop_cmd = ch;
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
                                self.got_double_colon = false;
                                self.loop_state = LoopState::ReadParameter;
                            }
                            _ => {
                                self.state = State::Default;
                            }
                        },
                        LoopState::ReadParameter => match ch {
                            '_' | '\n' | '\r' => { /* ignore */ }
                            ',' => {
                                if self.parsed_numbers[4]
                                    <= self.loop_parameters.iter().fold(0, |mut x, p| {
                                        x += p.len() as i32;
                                        x
                                    })
                                {
                                    self.state = State::ReadCommandStart;

                                    let mut l = Loop::new(
                                        self.parsed_numbers[0],
                                        self.parsed_numbers[1],
                                        self.parsed_numbers[2],
                                        self.parsed_numbers[3],
                                        self.loop_cmd,
                                        self.parsed_string.clone(),
                                        self.loop_parameters.clone(),
                                    )?;

                                    if let Some(x) = l.next_step(&self.command_executor, buf, caret) {
                                        self.cur_loop = Some(l);
                                        return x;
                                    }
                                    return Ok(CallbackAction::Update);
                                }
                                self.loop_parameters.last_mut().unwrap().push(String::new());
                            }
                            ':' => {
                                //println!("{:?} : {}", self.parsed_numbers, self.loop_parameters.iter().fold(0, |mut x, p| {x += p.len() as i32; x }) );
                                if self.parsed_numbers[4]
                                    <= self.loop_parameters.iter().fold(0, |mut x, p| {
                                        x += p.len() as i32;
                                        x
                                    })
                                {
                                    self.state = State::ReadCommandStart;
                                    let mut l = Loop::new(
                                        self.parsed_numbers[0],
                                        self.parsed_numbers[1],
                                        self.parsed_numbers[2],
                                        self.parsed_numbers[3],
                                        self.loop_cmd,
                                        self.parsed_string.clone(),
                                        self.loop_parameters.clone(),
                                    )?;

                                    if let Some(x) = l.next_step(&self.command_executor, buf, caret) {
                                        self.cur_loop = Some(l);
                                        return x;
                                    }
                                    return Ok(CallbackAction::Update);
                                }
                                self.loop_parameters.push(vec![String::new()]);
                            }
                            _ => {
                                self.loop_parameters.last_mut().unwrap().last_mut().unwrap().push(ch);
                            }
                        },
                    }
                    return Ok(CallbackAction::NoUpdate);
                }
                match ch {
                    ' ' | '>' | '\r' => { /* ignore */ }
                    '_' => {
                        self.got_double_colon = false;
                    }
                    '\n' => {
                        if self.got_double_colon {
                            self.got_double_colon = false;
                            self.state = State::SkipNewLine;
                        }
                    }
                    '0'..='9' => {
                        self.got_double_colon = false;
                        let d = match self.parsed_numbers.pop() {
                            Some(number) => number,
                            _ => 0,
                        };
                        self.parsed_numbers.push(parse_next_number(d, ch as u8));
                    }
                    ',' => {
                        self.got_double_colon = false;
                        self.parsed_numbers.push(0);
                    }
                    ':' => {
                        self.got_double_colon = true;
                        let parameters: Vec<_> = self.parsed_numbers.drain(..).collect();
                        let res = self
                            .command_executor
                            .lock()
                            .unwrap()
                            .execute_command(buf, caret, *command, &parameters, &self.parsed_string);
                        self.state = State::ReadCommandStart;
                        return res;
                    }
                    _ => {
                        self.got_double_colon = false;
                        self.state = State::Default;
                    }
                }
                Ok(CallbackAction::NoUpdate)
            }
            State::ReadCommandStart => {
                self.parsed_numbers.clear();
                match ch {
                    '\r' => Ok(CallbackAction::NoUpdate),
                    '\n' => {
                        self.state = State::SkipNewLine;
                        Ok(CallbackAction::NoUpdate)
                    }

                    '&' => {
                        self.state = State::ReadCommand(IgsCommands::LoopCommand);
                        self.loop_state = LoopState::Start;
                        Ok(CallbackAction::NoUpdate)
                    }

                    _ => match IgsCommands::from_char(ch) {
                        Ok(cmd) => {
                            self.state = State::ReadCommand(cmd);
                            Ok(CallbackAction::NoUpdate)
                        }
                        Err(err) => {
                            self.state = State::Default;
                            Err(anyhow::anyhow!("{err}"))
                        }
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
            State::SkipNewLine => {
                self.state = State::Default;
                if ch == '\r' {
                    return Ok(CallbackAction::NoUpdate);
                }
                if ch == 'G' {
                    self.state = State::GotIgsStart;
                    return Ok(CallbackAction::NoUpdate);
                }
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

    fn get_picture_data(&mut self) -> Option<(Size, Vec<u8>)> {
        self.command_executor.lock().unwrap().get_picture_data()
    }
}

pub fn parse_next_number(x: i32, ch: u8) -> i32 {
    x.saturating_mul(10).saturating_add(ch as i32).saturating_sub(b'0' as i32)
}

const HOLLOW_PATTERN: [u16; 1] = [0x0000];
const SOLID_PATTERN: [u16; 1] = [0xFFFF];
const TYPE_PATTERN: [[u16; 8]; 24] = [
    // intensity level 2
    [0x0000, 0x4444, 0x0000, 0x1111, 0x0000, 0x4444, 0x0000, 0x1111],
    // intensity level 4
    [0x0000, 0x5555, 0x0000, 0x5555, 0x0000, 0x5555, 0x0000, 0x5555],
    // intensity level 6
    [0x8888, 0x5555, 0x2222, 0x5555, 0x8888, 0x5555, 0x2222, 0x5555],
    // intensity level 8
    [0xAAAA, 0x5555, 0xAAAA, 0x5555, 0xAAAA, 0x5555, 0xAAAA, 0x5555],
    // intensity level 10
    [0xAAAA, 0xDDDD, 0xAAAA, 0x7777, 0xAAAA, 0xDDDD, 0xAAAA, 0x7777],
    // intensity level 12
    [0xAAAA, 0xFFFF, 0xAAAA, 0xFFFF, 0xAAAA, 0xFFFF, 0xAAAA, 0xFFFF],
    // intensity level 14
    [0xEEEE, 0xFFFF, 0xBBBB, 0xFFFF, 0xEEEE, 0xFFFF, 0xBBBB, 0xFFFF],
    // intensity level 16
    [0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF],
    // Brick
    [0xFFFF, 0x8080, 0x8080, 0x8080, 0xFFFF, 0x0808, 0x0808, 0x0808],
    // Diagonal Bricks
    [0x2020, 0x4040, 0x8080, 0x4141, 0x2222, 0x1414, 0x0808, 0x1010],
    // Grass
    [0x0000, 0x0000, 0x1010, 0x2828, 0x0000, 0x0000, 0x0101, 0x8282],
    // Trees
    [0x0202, 0x0202, 0xAAAA, 0x5050, 0x2020, 0x2020, 0xAAAA, 0x0505],
    // Dashed x's
    [0x4040, 0x8080, 0x0000, 0x0808, 0x0404, 0x0202, 0x0000, 0x2020],
    // Cobble Stones
    [0x6606, 0xC6C6, 0xD8D8, 0x1818, 0x8181, 0x8DB1, 0x0C33, 0x6000],
    // Sand
    [0x0000, 0x0000, 0x0400, 0x0000, 0x0010, 0x0000, 0x8000, 0x0000],
    // Rough Weave
    [0xF8F8, 0x6C6C, 0xC6C6, 0x8F8F, 0x1F1F, 0x3636, 0x6363, 0xF1F1],
    // Quilt
    [0xAAAA, 0x0000, 0x8888, 0x1414, 0x2222, 0x4141, 0x8888, 0x0000],
    // Patterned Cross
    [0x0808, 0x0000, 0xAAAA, 0x0000, 0x0808, 0x0000, 0x8888, 0x0000],
    // Balls
    [0x7777, 0x9898, 0xF8F8, 0xF8F8, 0x7777, 0x8989, 0x8F8F, 0x8F8F],
    // Vertical Scales
    [0x8080, 0x8080, 0x4141, 0x3E3E, 0x0808, 0x0808, 0x1414, 0xE3E3],
    // Diagonal scales
    [0x8181, 0x4242, 0x2424, 0x1818, 0x0606, 0x0101, 0x8080, 0x8080],
    // Checker Board
    [0xF0F0, 0xF0F0, 0xF0F0, 0xF0F0, 0x0F0F, 0x0F0F, 0x0F0F, 0x0F0F],
    // Filled Diamond
    [0x0808, 0x1C1C, 0x3E3E, 0x7F7F, 0xFFFF, 0x7F7F, 0x3E3E, 0x1C1C],
    // Herringbone
    [0x1111, 0x2222, 0x4444, 0xFFFF, 0x8888, 0x4444, 0x2222, 0xFFFF],
];

const HATCH_PATTERN: [[u16; 8]; 6] = [
    // narrow spaced + 45
    [0x0101, 0x0202, 0x0404, 0x0808, 0x1010, 0x2020, 0x4040, 0x8080],
    // medium spaced thick 45 deg
    [0x6060, 0xC0C0, 0x8181, 0x0303, 0x0606, 0x0C0C, 0x1818, 0x3030],
    // medium +-45 deg
    [0x4242, 0x8181, 0x8181, 0x4242, 0x2424, 0x1818, 0x1818, 0x2424],
    // medium spaced vertical
    [0x8080, 0x8080, 0x8080, 0x8080, 0x8080, 0x8080, 0x8080, 0x8080],
    // medium spaced horizontal
    [0xFFFF, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
    // medium spaced cross
    [0xFFFF, 0x8080, 0x8080, 0x8080, 0x8080, 0x8080, 0x8080, 0x8080],
];

const HATCH_WIDE_PATTERN: [[u16; 16]; 6] = [
    // wide +45 deg
    [
        0x0001, 0x0002, 0x0004, 0x0008, 0x0010, 0x0020, 0x0040, 0x0080, 0x0100, 0x0200, 0x0400, 0x0800, 0x1000, 0x2000, 0x4000, 0x8000,
    ],
    // widely spaced thick 45 deg
    [
        0x8003, 0x0007, 0x000E, 0x001C, 0x0038, 0x0070, 0x00E0, 0x01C0, 0x0380, 0x0700, 0x0E00, 0x1C00, 0x3800, 0x7000, 0x0E000, 0x0C001,
    ],
    // widely +- 45 deg
    [
        0x8001, 0x4002, 0x2004, 0x1008, 0x0810, 0x0420, 0x0240, 0x0180, 0x0180, 0x0240, 0x0420, 0x0810, 0x1008, 0x2004, 0x4002, 0x8001,
    ],
    // widely spaced vertical
    [
        0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000,
    ],
    // widely spaced horizontal
    [
        0xFFFF, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    ],
    // widely spaced horizontal/vert cross
    [
        0xFFFF, 0x8080, 0x8080, 0x8080, 0x8080, 0x8080, 0x8080, 0x8080, 0xFFFF, 0x8080, 0x8080, 0x8080, 0x8080, 0x8080, 0x8080, 0x8080,
    ],
];

const LINE_STYLE: [u16; 6] = [
    // Solid
    0xFFFF, // Long Dash
    0xFFF0, // Dotted
    0xC0C0, // Dash Dot
    0xFF18, // Dashed
    0xFF00, // DASH Dot Dot
    0xF191,
];
