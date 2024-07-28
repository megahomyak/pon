use std::collections::HashMap;

use crate::parser;

pub struct Command(parser::Name, Vec<parser::PonInput>);

pub enum ConversionError {
    NameMissing(parser::Index),
}
pub fn convert(old_command: parser::Command) -> Result<Command, ConversionError> {
    let parser::Command(index, kind) = old_command;
    let (name, pon_inputs) = match kind {
        parser::CommandKind::Named(name, pon_inputs) => (name, pon_inputs),
        parser::CommandKind::Unnamed(..) => return Err(ConversionError::NameMissing(index)),
    };
    Ok(Command(name, pon_inputs))
}

pub trait Object: downcast_rs::Downcast {}
downcast_rs::impl_downcast!(Object);

pub struct Interpreter {
    pub scope: HashMap<parser::Name, Box<dyn Object>>,
}

impl Interpreter {
    pub fn interpret(&mut self, command: Command) -> () {
        let Command(name, input) = command;
    }
}
