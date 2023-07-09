use std::{collections::HashMap, sync::Arc};

use super::fillers::SharedFiller;

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum NamePart {
    Word(String),
    Gap,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Name {
    parts: Vec<NamePart>,
}

enum Action {
    Returning(Arc<dyn Fn(Vec<SharedFiller>)>),
    // Shit name!!!
    Magic(Arc<dyn Fn(Arc<Scope>, Vec<SharedFiller>)>),
}

pub enum ActionOrFiller {
    Action(Action),
    Filler(SharedFiller),
}

#[derive(Debug)]
pub struct Scope {
    values: HashMap<Name, ActionOrFiller>,
    outer: Option<Arc<Scope>>,
}

impl Scope {
    pub fn new(values: HashMap<Name>, outer: Option<Arc<Scope>>) -> Self {
        Self { values, outer }
    }

    pub fn register_filler(&mut self, name: Name, filler: SharedFiller) {
        self.values.insert(name, filler);
    }

    pub fn find(&self, name: &Name) -> Option<&ActionOrFiller> {
        let mut scope = self;
        loop {
            if let action_or_filler @ Some(_) = scope.values.get(name) {
                return action_or_filler;
            }
            match scope.outer {
                None => return None,
                Some(outer) => scope = &outer,
            }
        }
    }

    pub fn register_magic_action<F: Fn(Arc<Scope>, Vec<SharedFiller>)>(&mut self, name: Name, action: F) {
        self.values.insert(
            name,
            ActionOrFiller::Action(Action::Magic(Arc::new(action))),
        )
    }

    pub fn register_returning_action<F: Fn(Vec<SharedFiller>)>(&mut self, name: Name, action: F) {
        self.values.insert(name, ActionOrFiller::Action(Action::Returning(Arc::new(action))))
    }
}
