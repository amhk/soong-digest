#[derive(Debug, PartialOrd, PartialEq, Ord, Eq)]
pub enum ItemType {
    Error,
    Warning,
}

#[derive(Debug, PartialOrd, Ord)]
pub struct Item {
    pub path: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub subject: String,
    pub body: Option<String>,
    pub type_: ItemType,
}

impl PartialEq for Item {
    fn eq(&self, other: &Item) -> bool {
        self.path == other.path
            && self.line == other.line
            && self.column == other.column
            && self.subject == other.subject
            /* ignore body */
            && self.type_ == other.type_
    }
}

impl Eq for Item {}
