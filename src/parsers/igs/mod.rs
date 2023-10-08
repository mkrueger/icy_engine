use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use super::BufferParser;
use crate::{AttributedChar, Buffer, CallbackAction, Caret, EngineResult, Size};

mod cmd;
use cmd::IgsCommands;
mod paint;
pub use paint::*;

#[cfg(test)]
mod tests;
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

pub trait CommandExecutor: Send {
    fn get_resolution(&self) -> Size;
    fn get_texture_data(&self) -> &[u8];

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
        }
    }

    fn run_loop(&mut self, buf: &mut Buffer, caret: &mut Caret, from: i32, to: i32, step: i32, delay: i32, command: char) -> EngineResult<()> {
        let cmd = IgsCommands::from_char(command)?;
        let mut i = from;
        // println!("run loop: {from} {to} {step} {delay} {command}");
        while if from < to { i < to } else { i > to } {
            let cur_parameter = ((i - from) as usize) % self.loop_parameters.len();
            let mut parameters = Vec::new();
            for p in &self.loop_parameters[cur_parameter] {
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

                let x = (i).abs();
                let y = (to - 1 - i).abs();
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
            self.command_executor
                .lock()
                .unwrap()
                .execute_command(buf, caret, cmd, &parameters, &self.parsed_string)?;
            // todo: correct delay?
            std::thread::sleep(Duration::from_millis(200 * delay as u64));
            if from < to {
                i += step;
            } else {
                i -= step;
            }
        }
        Ok(())
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
        //println!("{} {:?} - numbers:{:?}", ch, self.state, self.parsed_numbers);
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
                        println!(">{}<", self.parsed_string);
                        self.parsed_string.clear();
                        return res;
                    }
                    self.parsed_string.push(ch);
                    if ch == '\n' || ch == '\r' {
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
                                    self.run_loop(
                                        buf,
                                        caret,
                                        self.parsed_numbers[0],
                                        self.parsed_numbers[1],
                                        self.parsed_numbers[2],
                                        self.parsed_numbers[3],
                                        self.loop_cmd,
                                    )?;
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
                                    self.run_loop(
                                        buf,
                                        caret,
                                        self.parsed_numbers[0],
                                        self.parsed_numbers[1],
                                        self.parsed_numbers[2],
                                        self.parsed_numbers[3],
                                        self.loop_cmd,
                                    )?;
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
                    ' ' | '>' => { /* ignore */ }
                    '_' => {
                        self.got_double_colon = false;
                    }
                    '\n' | '\r' => {
                        if self.got_double_colon {
                            self.got_double_colon = false;
                            self.state = State::Default;
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
                    '\n' | '\r' => {
                        self.state = State::Default;
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
