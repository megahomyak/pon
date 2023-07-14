use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

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
    Magic(Arc<dyn Fn(&mut Arc<Mutex<Scope>>, Vec<Arc<dyn Filler>>) -> Output>),
    Returning(Arc<dyn Fn(Vec<Arc<dyn Filler>>) -> Output>),
}

enum Entity {
    Action(Action),
    Filler(Arc<dyn Filler>),
}

struct Scope {
    values: HashMap<Vec<NamePart>, Entity>,
    outer: Option<Arc<Mutex<Scope>>>,
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

fn execute(scope: &mut Arc<Mutex<Scope>>, program: parser::Program) -> Output {
    let mut last_value: Arc<dyn Filler> = Arc::new(Nothing {});
    for name in program.names {
        let mut name_key = vec![];
        let mut args: Vec<Arc<dyn Filler>> = vec![];
        for part in name.parts {
            name_key.push(match part {
                parser::NamePart::Word(word) => NamePart::Word(word.to_lowercase()),
                parser::NamePart::Filler(filler) => {
                    let mut scope = Arc::new(Mutex::new(Scope {
                        values: HashMap::new(),
                        outer: Some(Arc::clone(&scope)),
                    }));
                    args.push(match execute(&mut scope, filler.content) {
                        output @ Output::Thrown(_) => return output,
                        Output::Returned(filler) | Output::LastValue(filler) => filler,
                    });
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
        let mut scope = Arc::clone(scope);
        last_value = loop {
            let scope_guard = scope.lock().unwrap();
            if let Some(entity) = scope_guard.values.get(&name_key) {
                break match &entity {
                    Entity::Action(Action::Magic(action)) => {
                        let action = Arc::clone(action);
                        drop(scope_guard);
                        match action(&mut scope, args) {
                            output @ Output::Thrown(_) => return output,
                            Output::Returned(filler) | Output::LastValue(filler) => filler,
                        }
                    }
                    Entity::Action(Action::Returning(action)) => match action(args) {
                        output @ Output::Thrown(_) => return output,
                        Output::Returned(filler) | Output::LastValue(filler) => filler,
                    },
                    Entity::Filler(filler) => Arc::clone(filler),
                };
            } else {
                match &scope_guard.outer {
                    None => return error(format!("name not found")),
                    Some(outer) => {
                        let outer = Arc::clone(outer);
                        drop(scope_guard);
                        scope = outer
                    }
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
    let builtins = Arc::new(Mutex::new(Scope {
        outer: None,
        values: actions::builtins(),
    }));
    match execute(
        &mut Arc::new(Mutex::new(Scope {
            outer: Some(builtins),
            values: HashMap::new(),
        })),
        program,
    ) {
        Output::LastValue(filler) => println!("Last value: {}", filler),
        Output::Returned(filler) => println!("Returned: {}", filler),
        Output::Thrown(filler) => println!("Thrown: {}", filler),
    }
}
