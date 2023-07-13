use std::{collections::HashMap, sync::Arc};

use fillers::{Error, Filler, Nothing, PonString};

mod actions;
mod fillers;
mod parser;

#[derive(PartialEq, Eq, Hash)]
enum NamePart {
    Gap,
    Word(String),
}

enum Action {
    Magic(Box<dyn Fn(&mut Scope, Vec<Arc<dyn Filler>>) -> Output>),
    Returning(Box<dyn Fn(Vec<Arc<dyn Filler>>) -> Output>),
}

enum Entity {
    Action(Action),
    Filler(Arc<dyn Filler>),
}

struct Scope<'a> {
    values: HashMap<Vec<NamePart>, Entity>,
    outer: Option<&'a mut Scope<'a>>,
}

enum Output {
    Returned(Arc<dyn Filler>),
    LastValue(Arc<dyn Filler>),
    Thrown(Arc<dyn Filler>),
}

fn error(text: String) -> Output {
    Output::Thrown(Arc::new(Error { text }))
}

fn ok() -> Output {
    Output::Returned(Arc::new(Nothing {}))
}

fn execute(scope: Scope, program: &parser::Program) -> Output {
    let mut last_value: Arc<dyn Filler> = Arc::new(Nothing {});
    for name in &program.names {
        let mut name_key = vec![];
        let mut args: Vec<Arc<dyn Filler>> = vec![];
        for part in &name.parts {
            name_key.push(match part {
                parser::NamePart::Word(word) => NamePart::Word(word.to_owned()),
                parser::NamePart::Filler(filler) => {
                    let scope = Scope {
                        values: HashMap::new(),
                        outer: Some(&mut scope),
                    };
                    args.push(match execute(scope, &filler.content) {
                        output @ Output::Thrown(_) => return output,
                        Output::Returned(filler) | Output::LastValue(filler) => filler,
                    });
                    drop(scope);
                    NamePart::Gap
                }
                parser::NamePart::String(content) => {
                    args.push(Arc::new(PonString {
                        content: content.to_owned(),
                    }));
                    NamePart::Gap
                }
            })
        }
        let mut scope = &mut scope;
        last_value = loop {
            if let Some(entity) = scope.values.get(&name_key) {
                let last_value = match &entity {
                    Entity::Action(Action::Magic(action)) => match action(scope, args) {
                        output @ Output::Thrown(_) => return output,
                        Output::Returned(filler) | Output::LastValue(filler) => filler,
                    },
                    Entity::Action(Action::Returning(action)) => match action(args) {
                        output @ Output::Thrown(_) => return output,
                        Output::Returned(filler) | Output::LastValue(filler) => filler,
                    },
                    Entity::Filler(filler) => Arc::clone(filler),
                };
                break last_value;
            } else {
                match &mut scope.outer {
                    None => return error(format!("name not found")),
                    Some(outer) => scope = outer,
                }
            }
        }
    }
    Output::LastValue(last_value)
}

fn main() {
    let file_name = std::env::args().nth(1).expect("pass the file name");
    let file_contents = std::fs::read_to_string(file_name).expect("couldn't read the program");
    let program = parser::parse((&file_contents[..]).into()).expect("couldn't parse the program");
    let mut builtins = Scope {
        outer: None,
        values: actions::builtins(),
    };
    match execute(
        Scope {
            outer: Some(&mut builtins),
            values: HashMap::new(),
        },
        &program,
    ) {
        Output::LastValue(filler) => println!("Last value: {}", filler),
        Output::Returned(filler) => println!("Returned: {}", filler),
        Output::Thrown(filler) => println!("Thrown: {}", filler),
    }
}
