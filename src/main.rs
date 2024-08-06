mod non_empty_vec;
mod parser;

struct Line(String);
enum LineReadingResult {
    TerminationRequested(),
    Success(Line),
}

struct IsAppending(bool);

fn read_line(is_appending: IsAppending) -> LineReadingResult {
    use std::io::Write;
    let mut buffer = String::new();
    std::io::stdout()
        .write(if is_appending.0 { b"... " } else { b">>> " })
        .unwrap();
    std::io::stdout().flush().unwrap();
    if std::io::stdin().read_line(&mut buffer).unwrap() == 0 {
        LineReadingResult::TerminationRequested()
    } else {
        LineReadingResult::Success(Line(buffer))
    }
}

struct ProgramText {
    content: String,
}

enum ProgramReadingResult {
    TerminationRequested(),
    Success(parser::program::Program, ProgramText),
}

fn read_program() -> ProgramReadingResult {
    let mut lines_buffer = Vec::new();
    loop {
        let line = match read_line(IsAppending(!lines_buffer.is_empty())) {
            LineReadingResult::Success(Line(line)) => line,
            LineReadingResult::TerminationRequested() => {
                return ProgramReadingResult::TerminationRequested()
            }
        };
        let mut full_program = String::new();
        for line in lines_buffer.iter().chain(std::iter::once(&line)) {
            full_program.push_str(line);
        }
        let (program, after_program) =
            parser::program::parse(&mut parser::parser_input::Input::new(&full_program));
        match after_program {
            parser::program::After::ParserInputEnd() => {
                return ProgramReadingResult::Success(
                    program,
                    ProgramText {
                        content: full_program,
                    },
                )
            }
            parser::program::After::EscapeAtEndOfInput() => unreachable!(),
            parser::program::After::MissingInputTerminator { opener_index: _ } => {
                lines_buffer.push(line);
                continue;
            }
        }
    }
}

struct ErrorPrinter {
    was_activated: bool,
}

impl ErrorPrinter {
    fn print_error(
        &mut self,
        program: &ProgramText,
        position: parser::parser_input::Index,
        message: &str,
    ) {
        self.was_activated = true;
        let mut line = 1;
        let mut column = 1;
        let mut line_content = &program.content[..];
        for part in parser::parser_input::Input::new(&program.content) {
            if part.position == position {
                break;
            }
            if part.character == '\n' {
                line_content = &program.content[part.position.0 + part.character.len_utf8()..];
                column = 1;
                line += 1;
            } else {
                column += 1;
            }
        }
        let line_content = line_content.split('\n').next().unwrap_or("");
        eprintln!(
            "[!] An error occured at line {}, column {}: {}",
            line, column, message
        );
        eprintln!("");
        eprintln!("  {line_content}");
        eprint!("  ");
        for _ in 1..column {
            eprint!(" ");
        }
        eprintln!("^");
    }
}

fn main() {
    loop {
        let (program, program_text) = match read_program() {
            ProgramReadingResult::Success(program, program_text) => (program, program_text),
            ProgramReadingResult::TerminationRequested() => {
                println!("\nTerminating as requested.");
                return;
            }
        };
        let mut error_printer = ErrorPrinter {
            was_activated: false,
        };
        for command in program.0 {
            match command {
                parser::command::Command::Named(command) => {
                    if !error_printer.was_activated {
                        let object = interpreter.execute(&command);
                        if let Some(error) = object
                            .clone()
                            .borrow_mut()
                            .downcast_mut::<interpreter::Error>()
                        {
                            error_printer.print_error(&program_text, error.position, error.text);
                        } else {
                            println!("{}", object.borrow().to_string())
                        }
                    }
                }
                parser::command::Command::Unnamed(command) => {
                    error_printer.print_error(
                        &program_text,
                        command.inputs.first.position,
                        "name missing",
                    );
                }
            }
        }
    }
}
