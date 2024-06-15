pub struct TrackedString<'a> {
    content: &'a str,
    column: usize,
    row: usize,
}

impl<'a> TrackedString<'a> {
    pub fn column(&self) -> usize {
        self.column
    }
    pub fn row(&self) -> usize {
        self.row
    }
}
