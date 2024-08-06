mod command;

#[derive(Debug)]
pub struct Program(pub Vec<command::Command>);
#[derive(Debug)]
pub enum After<P> {
    EscapeAtEndOfInput(),
    ParserInputEnd(),
    MissingInputTerminator { position: P },
}
pub fn parse(parser_input: impl super::parser_input::Input) -> (Program, After) {
    let mut commands = Vec::new();
    let after = loop {
        let (command, after_command) = command::parse(parser_input);
        if let Some(command) = command {
            commands.push(command);
        }
        match after_command {
            command::After::CommandSeparator() => (),
            command::After::EscapeAtEndOfInput() => break After::EscapeAtEndOfInput(),
            command::After::ParserInputEnd() => break After::ParserInputEnd(),
            command::After::MissingInputTerminator { opener_index } => {
                break After::MissingInputTerminator { opener_index }
            }
        }
    };
    commands.shrink_to_fit();
    (Program(commands), after)
}
