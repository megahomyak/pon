use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use downcast_rs::DowncastSync;

mod parser;
mod scope;
mod fillers;
mod executor;

type SharedFiller = Arc<dyn Filler>;

struct Action {
    handler: Arc<dyn Send + Sync + Fn(Vec<SharedFiller>, Arc<Scope>) -> Output>,
}

// This name SUCKS, it is not descriptive!
enum Entity {
    Action(Action),
    Filler(SharedFiller),
}

impl std::fmt::Debug for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            Self::Filler(filler) => format!("{{Filler: {}}}", filler.to_string()),
            Self::Action(_) => format!("{{Action}}"),
        })
    }
}

#[derive(Debug)]
struct Scope {
    values: Mutex<HashMap<Vec<NamePart>, Entity>>,
    outer: Option<Arc<Scope>>,
}

struct PonString {
    context: Arc<Scope>,
    content: String,
}

impl Filler for PonString {
    fn to_string(&self) -> String {
        format!("{{{}}}", self.content)
    }

    fn content_to_string(&self) -> String {
        self.content.clone()
    }
}

struct Nothing {}

impl Filler for Nothing {
    fn to_string(&self) -> String {
        format!("(nothing)")
    }

    fn content_to_string(&self) -> String {
        format!("nothing")
    }
}

#[must_use]
enum Output {
    Returned(SharedFiller),
    Thrown(SharedFiller),
    LastValue(SharedFiller),
}

impl Output {
    pub fn to_string(&self) -> String {
        match self {
            Self::Thrown(filler) => format!("Thrown: {}", filler.to_string()),
            Self::Returned(filler) => format!("Returned: {}", filler.to_string()),
            Self::LastValue(filler) => format!("Last value: {}", filler.to_string()),
        }
    }
}

struct Error {
    content: String,
}

fn name_to_string(parts: &Vec<NamePart>) -> String {
    let mut name = String::new();
    for part in parts {
        if !name.is_empty() {
            name.push(' ');
        }
        name.push_str(match part {
            NamePart::Gap => "{}",
            NamePart::Word(word) => &word,
        });
    }
    name
}

impl Filler for Error {
    fn content_to_string(&self) -> String {
        self.content.clone()
    }
}

fn execute(scope: &Arc<Scope>, program: &parser::Program) -> Output {
    let mut last_value: SharedFiller = Arc::new(Nothing {});
    for name in &program.names {
        let mut args: Vec<SharedFiller> = Vec::new();
        let mut key = Vec::new();
        for part in &name.parts {
            match part {
                parser::NamePart::Word(word) => key.push(NamePart::Word(word.content.clone())),
                parser::NamePart::String(string) => {
                    key.push(NamePart::Gap);
                    args.push(Arc::new(PonString {
                        content: string.content.clone(),
                        context: Arc::clone(scope),
                    }))
                }
                parser::NamePart::Filler(filler) => {
                    let value = match execute(scope, &filler.content) {
                        output @ (Output::Thrown(_) | Output::Returned(_)) => return output,
                        Output::LastValue(filler) => filler,
                    };
                    args.push(value);
                    key.push(NamePart::Gap);
                }
            }
        }
        let mut scope = scope;
        loop {
            let values = scope.values.lock().unwrap();
            if let Some(entity) = values.get(&key) {
                match entity {
                    Entity::Filler(filler) => last_value = Arc::clone(filler),
                    Entity::Action(action) => {
                        let handler = Arc::clone(&action.handler);
                        drop(values);
                        match handler(args, Arc::clone(scope)) {
                            Output::Returned(filler) | Output::LastValue(filler) => {
                                last_value = filler
                            }
                            output @ Output::Thrown(_) => return output,
                        };
                    }
                }
                break;
            }
            scope = match &scope.outer {
                None => {
                    return Output::Thrown(Arc::new(Error {
                        content: format!("name {{{}}} not found", name_to_string(&key)),
                    }))
                }
                Some(outer) => outer,
            }
        }
    }
    Output::LastValue(last_value)
}

fn main() {
    let mut args = std::env::args();
    args.next().unwrap();
    let file_name = args.next().expect("pass the file name");
    let mut built_in_names = HashMap::new();
    fn word(content: &str) -> NamePart {
        NamePart::Word(content.to_owned())
    }
    use NamePart::Gap;
    fn action(
        action: impl 'static + Send + Sync + Fn(Vec<SharedFiller>, Arc<Scope>) -> Output,
    ) -> Entity {
        Entity::Action(Action {
            handler: Arc::new(action),
        })
    }
    built_in_names.insert(
        vec![word("name"), Gap, word("as"), Gap],
        action(|args, scope| {
            let mut args = args.into_iter();
            let value = args.next().unwrap();
            let name = args.next().unwrap();
            let name = name.content_to_string();
            if name.chars().any(|c| "{}()\n".contains(c)) {
                return Output::Thrown(Arc::new(Error {
                    content: format!("invalid filler name {{{}}}", name),
                }));
            }
            scope.values.lock().unwrap().insert(
                name.split_whitespace()
                    .map(|word| NamePart::Word(word.to_owned()))
                    .collect(),
                Entity::Filler(value),
            );
            Output::Returned(Arc::new(Nothing {}))
        }),
    );
    built_in_names.insert(
        vec![word("print"), Gap],
        action(|args, _stack| {
            println!("{}", args[0].content_to_string());
            Output::Returned(Arc::new(Nothing {}))
        }),
    );
    let scope = Arc::new(Scope {
        values: Mutex::new(built_in_names),
        outer: None,
    });
    let program = parser::parse(
        (&std::fs::read_to_string(file_name).expect("couldn't read your shit")[..]).into(),
    )
    .expect("couldn't parse your shit");
    println!("{}", execute(&scope, &program).to_string());
}
