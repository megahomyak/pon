use std::sync::{Arc, Mutex};

use crate::interpreter::{
    error,
    fillers::{Filler, Nothing, PonString},
    scope::{NamePart, Scope},
    Output,
};

mod interpreter;
mod parser;

fn main() {
    let mut args = std::env::args();
    args.next().unwrap();
    let file_name = args.next().expect("pass the file name");
    let mut built_in_names = Scope::new(None);
    fn word(content: &str) -> NamePart {
        NamePart::Word(content.to_owned())
    }
    use NamePart::Gap;
    built_in_names.register_magic_action(
        vec![word("name"), Gap, word("as"), Gap],
        |scope, args| {
            let mut args = args.into_iter();
            let value = args.next().unwrap();
            let name = args.next().unwrap();
            let name = name.content_to_string();
            let Ok(name) = parser::parse((&name[..]).into()).map_err(|_| ()).and_then(|program| {
                let mut names = program.names.into_iter();
                let name = names.next().ok_or(())?;
                let mut words = vec![];
                for part in name.parts {
                    match part {
                        parser::NamePart::Word(word) => words.push(NamePart::Word(word.to_lowercase())),
                        _ => return Err(()),
                    }
                }
                match names.next() {
                    Some(_next_name) => return Err(()),
                    None => return Ok(words),
                }
            }) else {
                return Output::Thrown(error(format!("invalid filler name {{{}}}", name)))
            };
            scope.register_filler(name, value);
            Output::Returned(Arc::new(Nothing {}))
        },
    );
    built_in_names.register_magic_action(vec![word("print"), Gap], |_scope, args| {
        println!("{}", args[0].content_to_string());
        Output::Returned(Arc::new(Nothing {}))
    });
    let mut scope = Scope::new(Some(&mut built_in_names));
    let program = parser::parse(
        (&std::fs::read_to_string(file_name).expect("couldn't read your shit")[..]).into(),
    )
    .expect("couldn't parse your shit");
    println!("{}", interpreter::execute(&mut scope, &program).to_string());
}
