#[derive(Debug)]
pub enum ItemType {
    Error,
    Warning,
}

#[derive(Debug)]
pub struct Item {
    pub path: String,
    pub line: usize,
    pub column: Option<usize>,
    pub subject: String,
    pub body: String,
    pub type_: ItemType,
}
