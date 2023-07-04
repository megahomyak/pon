use std::sync::Arc;

use crate::parser;

pub struct Executor<FillerGenerator> {
    filler_generator: FillerGenerator,
}

#[must_use]
enum Output<Filler> {
    Returned(Filler),
    Thrown(Filler),
    LastValue(Filler),
}

pub trait FillerGenerator<Filler> {
    fn nothing() -> Filler;
    fn string(content: String) -> Filler;
}

pub enum NamePart {
    Gap,
    Word(String),
}

impl Executor {
    fn execute_name<Scope>(&self, scope: &Arc<Scope>, name: &parser::Name) -> Output {
        let mut name_key = vec![];
        let mut args = vec![];
        for part in name.parts {
            match part {
                parser::NamePart::Word(word) => {
                    name_key.push(NamePart::Word(word.clone()));
                }
                parser::NamePart::String(string) => {

                }
            }
        }
    }

    pub fn execute<Scope, Filler>(
        &self,
        scope: &Arc<Scope>,
        program: &parser::Program,
    ) -> Output<Filler> {
        let mut last_value = Filler::nothing();
        for name in &program.names {
            match self.execute_name(scope, name) {
                output @ Output::Thrown(_) => return output,
                Output::LastValue(filler) | Output::Returned(filler) => last_value = filler,
            }
        }
        Output::LastValue(last_value)
    }
}
