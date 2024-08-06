use crate::parser::parser_input;

#[derive(Debug)]
pub struct Input<C> {
    pub content: Vec<C>,
}
pub(super) enum After {
    PonInputTerminator(),
    ParserInputEnd(),
}
pub(super) fn parse<P, C>(mut parser_input: impl parser_input::Input<P, C>) -> (Input<C>, After) {
    let mut content = Vec::new();
    let mut nesting_level = 0;
    let after = loop {
        match parser_input.next() {
            None => break After::ParserInputEnd(),
            Some(mut part) => {
                match part.kind {
                    parser_input::part::Kind::PonInputCloser() => {
                        if nesting_level == 0 {
                            break After::PonInputTerminator();
                        }
                        nesting_level -= 1;
                    }
                    parser_input::part::Kind::PonInputOpener() => nesting_level += 1,
                    parser_input::part::Kind::NextCharacterEscaper() => match parser_input.next() {
                        None => break After::EscapeAtEndOfInput(),
                        Some(escaped_part) => {
                            content.push(part.content);
                            part = escaped_part;
                        }
                    },
                    _ => (),
                }
                content.push(part.content);
            }
        }
    };
    content.shrink_to_fit();
    (Input { content }, after)
}
