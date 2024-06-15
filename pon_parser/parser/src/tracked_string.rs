use crate::trimmable::{Trimmable, TrimmingResult};

#[derive(Clone, Copy)]
pub struct TrackedString<'a> {
    content: &'a str,
    column: usize,
    row: usize,
}

impl<'a> From<&'a str> for TrackedString<'a> {
    fn from(content: &'a str) -> Self {
        TrackedString {
            content,
            column: 0,
            row: 1,
        }
    }
}

impl<'a> TrackedString<'a> {
    pub fn column(&self) -> usize {
        self.column
    }

    pub fn row(&self) -> usize {
        self.row
    }
}

impl<'a> Trimmable for TrackedString<'a> {
    type Part = char;

    fn trim(&self) -> TrimmingResult<Self::Part, Self> {
        let mut chars = self.content.chars();
        match chars.next() {
            None => TrimmingResult::NothingToTrim,
            Some(c) => TrimmingResult::Trimmed {
                part: c,
                rest: match c {
                    '\n' => TrackedString {
                        column: 0,
                        row: self.row + 1,
                        ..*self
                    },
                    _ => TrackedString {
                        column: self.column + 1,
                        ..*self
                    },
                },
            },
        }
    }
}
