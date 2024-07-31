use std::rc::Rc;

use crate::parser;

pub struct Command(parser::NamedCommand);

pub enum ConversionError {
    NameMissing(parser::Index),
}
pub fn convert(old_command: parser::Command) -> Result<Command, ConversionError> {
    let (name, pon_inputs) = match old_command.kind {
        parser::CommandKind::Named(command) => (name, pon_inputs),
        parser::CommandKind::Unnamed { inputs: _ } => return Err(ConversionError::NameMissing(old_command.position))
        // parser::CommandKind::Named(name, pon_inputs) => (name, pon_inputs),
        // parser::CommandKind::Unnamed(..) => return Err(ConversionError::NameMissing(index)),
    };
    Ok(Command(name, pon_inputs))
}

pub trait Object: downcast_rs::Downcast {}
downcast_rs::impl_downcast!(Object);
#[derive(Clone)]
pub struct SharedObject(Rc<dyn Object>);
impl SharedObject {
    fn new(o: impl Object) -> Self {
        Self(Rc::new(o))
    }
}

pub struct Error {
    text: &'static str,
    position: parser::Index,
}
impl Object for Error {}

pub struct Callable(pub Box<dyn FnMut() -> SharedObject>);
impl Object for Callable {}

pub struct Scope {
    parent: Option<Box<Scope>>,
    name: parser::Name,
    value: SharedObject,
}

pub struct Interpreter {
    pub scope: Option<Scope>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self { scope: None }
    }

    fn get(&self, name: &parser::Name) -> Option<SharedObject> {
        let scope = &self.scope;
        while let Some(scope) = scope {
            if scope.name.words == name.words {
                return Some(scope.value.clone());
            }
        }
        None
    }

    pub fn set(&mut self, name: parser::Name, value: impl Object) {
        let parent = self.scope.take().map(|scope| Box::new(scope));
        self.scope = Some(Scope {
            name,
            value: SharedObject::new(value),
            parent,
        });
    }

    pub fn execute(&mut self, command: Command) -> SharedObject {
        let Command(name, inputs) = command;
        let Some(obj) = self.get(&name) else {
            return SharedObject::new(Error {
                text: "undefined name",
                position: name.position,
            });
        };
        for input in inputs {

        }
        obj
    }
}
