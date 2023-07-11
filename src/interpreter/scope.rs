use std::{collections::HashMap, sync::{Arc, Mutex}};

use super::{fillers::SharedFiller, Output};

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum NamePart {
    Word(String),
    Gap,
}

pub enum Action {
    Returning(Arc<dyn Fn(Vec<SharedFiller>) -> Output>),
    // Shit name!!!
    Magic(Arc<dyn Fn(&mut Scope, Vec<SharedFiller>) -> Output>),
}

impl std::fmt::Debug for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Magic(_) => "magic",
            Self::Returning(_) => "returning",
        })
    }
}

pub type Name = Vec<NamePart>;

pub enum ActionOrFiller {
    Action(Action),
    Filler(SharedFiller),
}

impl std::fmt::Debug for ActionOrFiller {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            Self::Action(action) => format!("Action {{ {:?} }}", action),
            Self::Filler(filler) => format!("Filler {{ {} }}", filler.to_string()),
        })
    }
}

#[derive(Debug)]
pub struct Scope<'a> {
    values: HashMap<Vec<NamePart>, ActionOrFiller>,
    outer: Option<&'a mut Scope<'a>>,
}

impl<'a> Scope<'a> {
    pub fn new(outer: Option<&'a mut Scope<'a>>) -> Self {
        Self {
            values: HashMap::new(),
            outer,
        }
    }

    pub fn register_filler(&mut self, name: Name, filler: SharedFiller) {
        self.values.insert(name, ActionOrFiller::Filler(filler));
    }

    pub fn find(&self, name: &[NamePart]) -> Option<&ActionOrFiller> {
        let mut scope = self;
        loop {
            if let action_or_filler @ Some(_) = scope.values.get(name) {
                return action_or_filler;
            }
            match &scope.outer {
                None => return None,
                Some(outer) => scope = outer,
            }
        }
    }

    pub fn register_magic_action<F: Fn(&mut Scope, Vec<SharedFiller>) -> Output + 'static>(
        &mut self,
        name: Name,
        action: F,
    ) {
        self.values.insert(
            name,
            ActionOrFiller::Action(Action::Magic(Arc::new(action))),
        );
    }

    pub fn register_returning_action<F: Fn(Vec<SharedFiller>) -> Output + 'static>(
        &mut self,
        name: Name,
        action: F,
    ) {
        self.values.insert(
            name,
            ActionOrFiller::Action(Action::Returning(Arc::new(action))),
        );
    }
}
