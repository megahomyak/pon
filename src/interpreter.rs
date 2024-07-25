use crate::non_empty::NonEmpty;
use crate::parser;

pub struct Command(parser::Name, Vec<parser::PonInput>);
pub struct Program(Vec<Command>);

pub enum ConversionError {
    NameMissing(),
}
pub fn convert(old_program: parser::Program) -> Result<Program, ConversionError> {
    let mut new_program = Vec::with_capacity(old_program.0.len());
    for old_command in old_program.0 {
        let parser::Command(old_name, old_inputs) = old_command;
        let Some(old_name) = old_name else {
            return Err(ConversionError::NameMissing());
        };
        new_program.push(Command(old_name, old_inputs));
    }
    new_program.shrink_to_fit();
    Ok(Program(new_program))
}

pub fn interpret(program: &Program) {}
