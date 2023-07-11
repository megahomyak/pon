use std::sync::Arc;

use crate::parser;

use self::{
    fillers::{Filler, Nothing, PonString, SharedFiller},
    scope::{Action, ActionOrFiller, NamePart, Scope},
};

pub mod fillers;
pub mod scope;

#[must_use]
pub enum Output {
    Returned(SharedFiller),
    Thrown(SharedFiller),
    LastValue(SharedFiller),
}

impl Filler for Output {
    fn content_to_string(&self) -> String {
        match self {
            Self::Thrown(filler) => format!("Thrown: {}", filler.to_string()),
            Self::LastValue(filler) => format!("LastValue: {}", filler.to_string()),
            Self::Returned(filler) => format!("Returned: {}", filler.to_string()),
        }
    }
}

pub fn error(content: String) -> Arc<dyn Filler> {
    Arc::new(PonString { content })
}

pub fn execute(scope: &mut Scope, program: &parser::Program) -> Output {
    let mut last_value: SharedFiller = Arc::new(Nothing {});
    for name in &program.names {
        let mut args: Vec<SharedFiller> = Vec::new();
        let mut key = Vec::new();
        for part in &name.parts {
            match part {
                parser::NamePart::Word(word) => key.push(NamePart::Word(word.to_lowercase())),
                parser::NamePart::String(string) => {
                    key.push(NamePart::Gap);
                    args.push(Arc::new(PonString {
                        content: string.clone(),
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
        match scope.find(&key) {
            None => return Output::Thrown(error(format!("name {:?} not found", key))),
            Some(action_or_filler) => match action_or_filler {
                ActionOrFiller::Filler(filler) => last_value = Arc::clone(filler),
                ActionOrFiller::Action(action) => match action {
                    Action::Magic(action) => {
                        let handler = Arc::clone(&action);
                        drop(action_or_filler);
                        match handler(scope, args) {
                            Output::Returned(filler) | Output::LastValue(filler) => {
                                last_value = filler
                            }
                            output @ Output::Thrown(_) => return output,
                        };
                    }
                    Action::Returning(action) => {
                        let handler = Arc::clone(&action);
                        drop(action_or_filler);
                        match handler(args) {
                            Output::Returned(filler) | Output::LastValue(filler) => {
                                last_value = filler
                            }
                            output @ Output::Thrown(_) => return output,
                        };
                    }
                },
            },
        }
    }
    Output::LastValue(last_value)
}
