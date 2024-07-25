use std::collections::HashMap;

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
    let interpreter = interpreter::Interpreter {
        scope: HashMap::new(),
    };
    loop {
        let mut lines_buffer = Vec::new();
        let program = loop {
            let Some(line) = read_line() else {
                println!("Terminating as requested.");
                return;
            };
            let mut full_program = String::new();
            for line in lines_buffer.iter().chain(std::iter::once(&line)) {
                full_program.push_str(line);
            }
            let (program, after_program) = parser::program(&full_program);
            match after_program {
                parser::AfterProgram::ParserInputEnd() => break program,
                parser::AfterProgram::EscapeAtEndOfInput() => unreachable!(),
                parser::AfterProgram::MissingInputTerminator { opener_index: _ } => {
                    lines_buffer.push(line);
                    continue;
                }
            }
        };
        for command in program.0 {
            let program = interpreter::convert(command);
            interpreter::interpret(program);
        }
    }
}
