use std::{collections::HashMap, sync::Arc};

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum NamePart {
    Word(String),
    Gap,
}

// This name SUCKS, it is not descriptive! (Also don't forget to change the names of variables)
pub enum Entity<Action, Filler> {
    Action(Action),
    Filler(Filler),
}

pub struct Scope<Action, Filler> {
    contents: HashMap<Vec<NamePart>, Entity<Action, Filler>>,
    parent: Option<Arc<Scope<Action, Filler>>>,
}

impl<Action, Filler> Scope<Action, Filler> {
    pub fn new(parent: Option<Arc<Scope<Action, Filler>>>) -> Self {
        Self {
            contents: Default::default(),
            parent,
        }
    }

    pub fn register(&mut self, name: Vec<NamePart>, entity: Entity<Action, Filler>) {
        self.contents.insert(name, entity);
    }

    pub fn find(&self, name: &[NamePart]) -> Option<&Entity<Action, Filler>> {
        let mut scope = self;
        loop {
            if let Some(entity) = self.contents.get(name) {
                return Some(entity);
            }
            scope = match &scope.parent {
                Some(parent) => parent,
                None => return None,
            }
        }
    }
}
