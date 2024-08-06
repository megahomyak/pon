use crate::parser::Positioned;

#[derive(Debug, PartialEq, Eq)]
pub struct NonEmptyVec<T> {
    pub first: T,
    pub rest: Vec<T>,
}

pub struct Incomplete<P, T> {
    pub first: Option<Positioned<P, T>>,
    pub rest: Vec<T>,
}

impl<P, T> Incomplete<P, T> {
    pub fn new() -> Self {
        Self {
            first: None,
            rest: Vec::new(),
        }
    }

    pub fn push(&mut self, t: Positioned<P, T>) {
        match self.first {
            None => self.first = Some(t),
            Some(_) => self.rest.push(t.entity),
        }
    }

    pub fn finalize(mut self) -> Option<Positioned<P, NonEmptyVec<T>>> {
        self.first.map(|first| {
            self.rest.shrink_to_fit();
            Positioned {
                position: first.position,
                entity: NonEmptyVec {
                    first: first.entity,
                    rest: self.rest,
                },
            }
        })
    }
}
