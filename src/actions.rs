use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    error, ok, parser, Action, Entity, Filler, NamePart, Nothing, Output, PonString, Scope,
};

fn word(w: &str) -> NamePart {
    NamePart::Word(w.to_owned())
}

fn gap() -> NamePart {
    NamePart::Gap
}

fn magic<F: 'static + Fn(&mut Arc<Mutex<Scope>>, Vec<Arc<dyn Filler>>) -> Output>(f: F) -> Entity {
    Entity::Action(Action::Magic(Arc::new(f)))
}

pub(crate) fn builtins() -> HashMap<Vec<NamePart>, Entity> {
    let mut scope = HashMap::new();
    scope.insert(
        vec![word("print"), gap()],
        magic(|_scope, args| {
            println!("{}", args[0]);
            Output::Returned(Arc::new(Nothing {}))
        }),
    );
    scope.insert(
        vec![word("name"), gap(), word("as"), gap()],
        magic(|scope, args| {
            let [value, name] = <[_; 2]>::try_from(args).unwrap_or_else(|_| unreachable!());
            let Some(name) = name.downcast_ref::<PonString>() else {
                return error(format!("the name is not a string"));
            };
            match (|| {
                let mut names = parser::parse((&name.content[..]).into())
                    .ok()?
                    .names
                    .into_iter();
                let (Some(name), None) = (names.next(), names.next()) else { return None; };
                name.parts
                    .into_iter()
                    .map(|part| match part {
                        parser::NamePart::Filler(_) | parser::NamePart::String(_) | parser::NamePart::Comment(_) => None,
                        parser::NamePart::Word(word) => Some(NamePart::Word(word.to_lowercase())),
                    })
                    .collect()
            })() {
                None => error(format!("bad name")),
                Some(parts) => {
                    scope
                        .lock()
                        .unwrap()
                        .values
                        .insert(parts, Entity::Filler(value));
                    ok()
                }
            }
        }),
    );
    scope.insert(
        vec![word("if"), gap(), word(","), word("then"), gap()],
        magic(|scope, args| {
            let [condition, branch] = <[_; 2]>::try_from(args).unwrap_or_else(|_| unreachable!());
            if condition.bool() {
                let Some(branch) = branch.downcast_ref::<PonString>() else {
                return error(format!("the branch is not a string"));
            };
                let mut scope = Arc::new(Mutex::new(Scope {
                    outer: Some(Arc::clone(scope)),
                    values: HashMap::new(),
                }));
                let Ok(program) = parser::parse((&branch.content[..]).into()) else {
                return error(format!("couldn't parse the branch as a program"));
            };
                match crate::execute(&mut scope, program) {
                    output @ Output::Thrown(_) => return output,
                    Output::Returned(_) | Output::LastValue(_) => (),
                }
            }
            ok()
        }),
    );
    scope
}
