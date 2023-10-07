use crate::{Buffer, CallbackAction, Caret, EngineResult, Size};

use super::{cmd::IgsCommands, CommandExecutor};

#[derive(Default)]
struct TestExecutor {
    pub commands: Vec<(IgsCommands, Vec<i32>)>,
}

impl CommandExecutor for TestExecutor {
    fn get_resolution(&self) -> Size {
        todo!();
    }
    fn get_texture_data(&self) -> &[u8] {
        todo!();
    }

    fn execute_command(&mut self, _buf: &mut Buffer, _caret: &mut Caret, command: IgsCommands, parameters: &[i32]) -> EngineResult<CallbackAction> {
        self.commands.push((command, parameters.to_vec()));
        Ok(CallbackAction::NoUpdate)
    }
}
/*
use std::sync::{Arc, Mutex};

use crate::{
    ascii,
    parsers::{create_buffer, update_buffer_force},
    TextPane, igs::Parser
};


#[test]
pub fn test_igs_version() {
    let command_executor: Arc<Mutex<Box<TestExecutor>>> = Arc::new(Mutex::new(Box::new(TestExecutor{commands: Vec::new()})));
    let mut igs_parser = Parser::new(Box::<ascii::Parser>::default(), command_executor.clone());
    create_buffer(&mut igs_parser, b"G#?>0:");
    assert_eq!(1, command_executor.lock().unwrap().commands.len());
    assert_eq!(IgsCommands::AskIG, command_executor.lock().unwrap().commands[0].0);
    assert_eq!(vec![0], command_executor.lock().unwrap().commands[0].1);
}

#[test]
pub fn parse_two_commands() {
    let command_executor = Arc::new(Mutex::new(Box::new(TestExecutor{commands: Vec::new()})));
    let mut igs_parser = Parser::new(Box::<ascii::Parser>::default(), command_executor.clone());
    create_buffer(&mut igs_parser, b"G#?>0:?>0:");
    assert_eq!(2, command_executor.lock().unwrap().commands.len());
}

#[test]
pub fn test_eol_marker() {
    let command_executor = Arc::new(Mutex::new(Box::new(TestExecutor{commands: Vec::new()})));
    let mut igs_parser = Parser::new(Box::<ascii::Parser>::default(), command_executor.clone());
    create_buffer(&mut igs_parser, b"G#?>_\n\r0:?>_\n\r0:");
    assert_eq!(2, command_executor.lock().unwrap().commands.len());
}

#[test]
pub fn test_text_break_bug() {
    let command_executor = Arc::new(Mutex::new(Box::new(TestExecutor{commands: Vec::new()})));
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
pub fn test_loop_parsing() {
    let command_executor = Arc::new(Mutex::new(Box::new(TestExecutor{commands: Vec::new()})));
    let (mut buf, mut caret) = create_buffer(&mut igs_parser, b"");
    update_buffer_force(&mut buf, &mut caret, &mut igs_parser, b"G#&>0,320,4,0,L,8,0,100,x,0:0,100,x,199:");
    assert_eq!(' ', buf.get_char((0, 0)).ch);
}
*/
