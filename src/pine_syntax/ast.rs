#[derive(Debug)]
pub struct Position {
    pub start: usize,
    pub end: usize
}

#[derive(Debug)]
pub struct Positioned<T> {
    pub item: T,
    pub position: Position
}

pub type TableName = Positioned<String>;
pub type ColumnName = Positioned<String>;

#[derive(Debug)]
pub enum Operation {
    From(TableName),
    Select(Vec<ColumnName>)
}

impl Operation {
    pub fn get_name(&self) -> &str {
        use Operation::*;

        match self {
            From(_) => "from",
            Select(_) => "select",
        }
    }
}


pub type Pine = Positioned<Vec<Positioned<Operation>>>;
