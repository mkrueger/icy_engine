use crate::{Buffer, CallbackAction, Caret, EngineResult, Size};

use super::{cmd::IgsCommands, CommandExecutor};

struct TestExecutor {
    pub commands: Arc<Mutex<Vec<(IgsCommands, Vec<i32>)>>>,
}

impl CommandExecutor for TestExecutor {
    fn get_resolution(&self) -> Size {
        todo!();
    }
    fn get_texture_data(&self) -> &[u8] {
        todo!();
    }

    fn execute_command(&mut self, _buf: &mut Buffer, _caret: &mut Caret, command: IgsCommands, parameters: &[i32]) -> EngineResult<CallbackAction> {
        self.commands.lock().unwrap().push((command, parameters.to_vec()));
        Ok(CallbackAction::NoUpdate)
    }
}

use std::sync::{Arc, Mutex};

use crate::{
    ascii,
    igs::Parser,
    parsers::{create_buffer, update_buffer_force},
    TextPane,
};

fn create_parser() -> (Arc<Mutex<Vec<(IgsCommands, Vec<i32>)>>>, Parser) {
    let commands = Arc::new(Mutex::new(Vec::new()));
    let command_executor: Arc<Mutex<Box<dyn CommandExecutor>>> = Arc::new(Mutex::new(Box::new(TestExecutor { commands: commands.clone() })));
    let igs_parser = Parser::new(Box::<ascii::Parser>::default(), command_executor);
    (commands, igs_parser)
}

#[test]
pub fn test_line_breaks() {
    let (commands, mut igs_parser) = create_parser();
    create_buffer(&mut igs_parser, b"G#?>0:\nG#?>0:");
    assert_eq!(2, commands.lock().unwrap().len());
}

#[test]
pub fn test_igs_version() {
    let (commands, mut igs_parser) = create_parser();
    create_buffer(&mut igs_parser, b"G#?>0:");
    assert_eq!(1, commands.lock().unwrap().len());
    assert_eq!(IgsCommands::AskIG, commands.lock().unwrap()[0].0);
    assert_eq!(vec![0], commands.lock().unwrap()[0].1);
}

#[test]
pub fn parse_two_commands() {
    let (commands, mut igs_parser) = create_parser();
    create_buffer(&mut igs_parser, b"G#?>0:?>0:");
    assert_eq!(2, commands.lock().unwrap().len());
}

#[test]
pub fn test_eol_marker() {
    let (commands, mut igs_parser) = create_parser();
    create_buffer(&mut igs_parser, b"G#?>_\n\r0:?>_\n\r0:");
    assert_eq!(2, commands.lock().unwrap().len());
}

#[test]
pub fn test_text_break_bug() {
    let (_, mut igs_parser) = create_parser();
    let (buf, _) = create_buffer(&mut igs_parser, b"G#W>20,50,Chain@L 0,0,300,190:W>253,_\n140,IG SUPPORT BOARD@");

    assert_eq!(' ', buf.get_char((0, 0)).ch);
}

#[test]
pub fn test_loop_parsing() {
    let (_, mut igs_parser) = create_parser();
    let (mut buf, mut caret) = create_buffer(&mut igs_parser, b"");
    update_buffer_force(&mut buf, &mut caret, &mut igs_parser, b"G#&>0,320,4,0,L,8,0,100,x,0:0,100,x,199:");
    assert_eq!(' ', buf.get_char((0, 0)).ch);
}
