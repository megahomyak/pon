use std::{cell::RefCell, rc::Rc};

use crate::parser;

pub trait Object: downcast_rs::Downcast + ToString {}
downcast_rs::impl_downcast!(Object);
pub type SharedObject = Rc<RefCell<dyn Object>>;
pub fn make_object(o: impl Object) -> SharedObject {
    Rc::new(RefCell::new(o))
}

pub struct Error {
    pub text: &'static str,
    pub position: parser::parser_input::Index,
}
impl ToString for Error {
    fn to_string(&self) -> String {
        self.text.into()
    }
}
impl Object for Error {}

pub struct Callable(
    pub Box<dyn FnMut(&parser::pon_input::Input, &mut Option<Scope>) -> SharedObject>,
);
impl ToString for Callable {
    fn to_string(&self) -> String {
        "Callable".into()
    }
}
impl Object for Callable {}

pub struct RuntimeString {
    pub content: String,
}
impl ToString for RuntimeString {
    fn to_string(&self) -> String {
        self.content.clone()
    }
}
impl Object for RuntimeString {}

pub struct Scope {
    parent: Option<Box<Scope>>,
    name: parser::name::Name,
    value: SharedObject,
}

pub struct Interpreter {
    pub scope: Option<Scope>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self { scope: None }
    }

    fn get(&self, name: &parser::name::Name) -> Option<SharedObject> {
        let scope = &self.scope;
        while let Some(scope) = scope {
            if scope.name.words == name.words {
                return Some(scope.value.clone());
            }
        }
        None
    }

    pub fn set(&mut self, name: parser::name::Name, value: impl Object) {
        let parent = self.scope.take().map(|scope| Box::new(scope));
        self.scope = Some(Scope {
            name,
            value: make_object(value),
            parent,
        });
    }

    pub fn execute(&mut self, command: &parser::command::Named) -> SharedObject {
        let Some(mut obj) = self.get(&command.name.entity) else {
            return make_object(Error {
                text: "undefined name",
                position: command.name.position,
            });
        };
        for input in &command.inputs {
            if let Some(callable) = obj.clone().borrow_mut().downcast_mut::<Callable>() {
                obj = callable.0(&input.entity, &mut self.scope);
            } else {
                return make_object(Error {
                    text: "the called object is not a function",
                    position: input.position,
                });
            }
        }
        obj
    }
}
