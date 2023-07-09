use std::{sync::{Arc, Mutex}, collections::HashMap};

use self::{fillers::SharedFiller, scope::Scope};

mod fillers;
mod scope;

#[must_use]
enum Output {
    Returned(SharedFiller),
    Thrown(SharedFiller),
    LastValue(SharedFiller),
}

pub fn execute(scope: &Arc<Scope>, program: &parser::Program) -> Output {
    let mut last_value: SharedFiller = Arc::new(Nothing {});
    for name in &program.names {
        let mut args: Vec<SharedFiller> = Vec::new();
        let mut key = Vec::new();
        for part in &name.parts {
            match part {
                parser::NamePart::Word(word) => key.push(NamePart::Word(word.clone())),
                parser::NamePart::String(string) => {
                    key.push(NamePart::Gap);
                    args.push(Arc::new(PonString {
                        content: string.clone(),
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
