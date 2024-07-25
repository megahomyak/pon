use std::collections::HashMap;

use crate::parser;

pub struct Command(parser::Name, Vec<parser::PonInput>);

pub enum ConversionError {
    NameMissing(parser::Index),
}
pub fn convert(
    old_command: parser::Positioned<parser::Command>,
) -> Result<Command, ConversionError> {
    let parser::Positioned(old_command, index) = old_command;
    let parser::Command(old_name, old_inputs) = old_command;
    let Some(old_name) = old_name else {
        return Err(ConversionError::NameMissing(index));
    };
    Ok(Command(old_name, old_inputs))
}

pub trait Object: downcast_rs::Downcast {}
downcast_rs::impl_downcast!(Object);

pub struct Interpreter {
    pub scope: HashMap<parser::Name, Box<dyn Object>>,
}

impl Interpreter {
    pub fn interpret(command: Command) -> () {
        let Command(name, input) = command;
    }
}
