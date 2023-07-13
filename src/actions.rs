use std::{collections::HashMap, sync::Arc};

use crate::{
    error, ok, parser, Action, Entity, Filler, NamePart, Nothing, Output, PonString, Scope,
};

fn word(w: &str) -> NamePart {
    NamePart::Word(w.to_owned())
}

fn gap() -> NamePart {
    NamePart::Gap
}

fn magic<F: 'static + Fn(&mut Scope, Vec<Arc<dyn Filler>>) -> Output>(f: F) -> Entity {
    Entity::Action(Action::Magic(Box::new(f)))
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
            if let Ok(program) = parser::parse((&name.content[..]).into())
                && let mut names = program.names.into_iter()
                && let Some(name) = names.next()
                && let None = names.next()
                && let Some(parts) = name.parts.into_iter().map(|part| match part {
                        parser::NamePart::Filler(_) | parser::NamePart::String(_) => None,
                        parser::NamePart::Word(word) => Some(NamePart::Word(word)),
                    }).collect()
                {
                    scope.values.insert(parts, Entity::Filler(value));
                    ok()
                } else {
                    error(format!("bad name"))
                }
        }),
    );
    scope
}
