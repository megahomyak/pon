use std::collections::HashMap;

mod non_empty;
mod interpreter;
mod parser;

struct Line(String);
enum LineReadingResult {
    TerminationRequested(),
    Success(Line),
}

fn read_line() -> LineReadingResult {
    use std::io::Write;
    let mut buffer = String::new();
    std::io::stdout().write(b">>> ").unwrap();
    std::io::stdout().flush().unwrap();
    if std::io::stdin().read_line(&mut buffer).unwrap() == 0 {
        LineReadingResult::TerminationRequested()
    } else {
        LineReadingResult::Success(Line(buffer))
    }
}

enum ProgramReadingResult {
    TerminationRequested(),
    Success(parser::Program),
}

fn read_program() -> ProgramReadingResult {
    let mut lines_buffer = Vec::new();
    let program = loop {
        let line = match read_line() {
            LineReadingResult::Success(Line(line)) => line,
            LineReadingResult::TerminationRequested() => {
                return ProgramReadingResult::TerminationRequested()
            }
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
    ProgramReadingResult::Success(program)
}

fn main() {
    let interpreter = interpreter::Interpreter {
        scope: HashMap::new(),
    };
    loop {
        let program = match read_program() {
            ProgramReadingResult::Success(program) => program,
            ProgramReadingResult::TerminationRequested() => {
                println!("Terminating as requested.");
                return;
            }
        };
        for command in program.0 {
            let program = interpreter::convert(command);
            interpreter.interpret(program);
        }
    }
}
