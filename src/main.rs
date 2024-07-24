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
        let (program, after_program) = parser::program(&full_program);
        match after_program {
            parser::AfterProgram::ParserInputEnd() => {
                lines_buffer.clear();
                println!("{:?}", program);
            }
            parser::AfterProgram::EscapeAtEndOfInput() => unreachable!(),
            parser::AfterProgram::MissingInputTerminator() => {
                lines_buffer.push(line);
            }
        }
    }
}
