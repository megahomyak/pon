mod interpreter;
mod non_empty;
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

struct ProgramText(String);

enum ProgramReadingResult {
    TerminationRequested(),
    Success(parser::Program, ProgramText),
}

fn read_program() -> ProgramReadingResult {
    let mut lines_buffer = Vec::new();
    loop {
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
            parser::AfterProgram::ParserInputEnd() => {
                return ProgramReadingResult::Success(program, ProgramText(full_program))
            }
            parser::AfterProgram::EscapeAtEndOfInput() => unreachable!(),
            parser::AfterProgram::MissingInputTerminator { opener_index: _ } => {
                lines_buffer.push(line);
                continue;
            }
        }
    }
}

struct ErrorMessage<'a>(&'a str);

fn print_error(program: &ProgramText, position: parser::Index, message: ErrorMessage) {
    let mut total_index = 0;
    let mut line_num = 1;
    let mut line_text = &program.0[..];
    let column = 'column: loop {
        for new_line_excerpt in program.0.split('\n') {
            line_text = new_line_excerpt;
            line_num += 1;
            if total_index == 0 {
                total_index += line_text.len();
            } else {
                total_index += 1 + line_text.len();
            }
            if total_index > position.0 {
                break 'column total_index - position.0;
            }
        }
        break line_text.len();
    };
    eprintln!("An error occured at line {}, column {}: {}", line_num, column, message.0);
    eprintln!("");
    eprintln!("  {line_text}");
    eprint!("  ");
    for _ in 1..column {
        eprint!(" ");
    }
    eprintln!("^");
}

fn main() {
    let mut interpreter = interpreter::Interpreter::new();
    loop {
        let (program, program_text) = match read_program() {
            ProgramReadingResult::Success(program, program_text) => (program, program_text),
            ProgramReadingResult::TerminationRequested() => {
                println!("Terminating as requested.");
                return;
            }
        };
        let mut any_errors = false;
        for command in program.0 {
            match interpreter::convert(command) {
                Ok(program) => {
                    if !any_errors {
                        interpreter.interpret(program);
                    }
                }
                Err(interpreter::ConversionError::NameMissing(position)) => {
                    print_error(&program_text, position, ErrorMessage("name missing"));
                    any_errors = true;
                },
            }
        }
    }
}
