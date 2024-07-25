mod interpreter;
mod non_empty;
mod parser;

fn read_line() -> Option<String> {
    use std::io::Write;
    let mut buffer = String::new();
    std::io::stdout().write(b">>> ").unwrap();
    std::io::stdout().flush().unwrap();
    if std::io::stdin().read_line(&mut buffer).unwrap() == 0 {
        None
    } else {
        Some(buffer)
    }
}

fn main() {
    let mut lines_buffer = Vec::new();
    while let Some(line) = read_line() {
        let mut full_program = String::new();
        for line in lines_buffer.iter().chain(std::iter::once(&line)) {
            full_program.push_str(line);
        }
        let (command, after_command) = parser::command(&mut (&full_program[..]).into());
        match after_command {
            parser::AfterCommand::CommandSeparator() => {

            }
            parser::AfterCommand::ParserInputEnd() => {
                lines_buffer.clear();
                println!("{:?}", command);
            }
            parser::AfterCommand::EscapeAtEndOfInput() => unreachable!(),
            parser::AfterCommand::MissingInputTerminator() => {
                lines_buffer.push(line);
            }
        }
    }
}
