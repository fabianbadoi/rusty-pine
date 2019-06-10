#[derive(Debug)]
pub struct Column {
    pub name: String,
}

#[derive(Debug)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
}

