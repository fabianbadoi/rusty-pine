pub type Pine = Positioned<Vec<Positioned<Operation>>>;

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
pub type Value = Positioned<String>;

// make type -> typeNode pairs for everything
pub type FilterNode = Positioned<Filter>;
pub type ConditionNode =Positioned<Condition>;

#[derive(Debug)]
pub enum Operation {
    From(TableName),
    Select(Vec<ColumnName>),
    Filter(Vec<FilterNode>),
}

impl Operation {
    pub fn get_name(&self) -> &str {
        use Operation::*;

        match self {
            From(_) => "from",
            Select(_) => "select",
            Filter(_) => "filter",
        }
    }
}

#[derive(Debug)]
pub struct Filter {
    pub column: ColumnName,
    pub condition: ConditionNode,
}

#[derive(Debug)]
pub enum Condition {
    Equals(Value)
}
