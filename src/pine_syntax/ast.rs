pub type PineNode = Node<Pine>;
pub type OperationNode = Node<Operation>;
pub type FilterNode = Node<Filter>;
pub type ConditionNode = Node<Condition>;
pub type TableNameNode = Node<TableName>;
pub type ColumnNameNode = Node<ColumnName>;
pub type ValueNode = Node<Value>;

#[derive(Debug)]
pub struct Pine {
    pub operations: Vec<OperationNode>,
}

#[derive(Debug)]
pub enum Operation {
    From(TableNameNode),
    Select(Vec<ColumnNameNode>),
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
    pub column: ColumnNameNode,
    pub condition: ConditionNode,
}

#[derive(Debug)]
pub enum Condition {
    Equals(ValueNode),
}

pub type Identifier = String;
pub type TableName = Identifier;
pub type ColumnName = Identifier;
pub type Value = String;

#[derive(Copy, Clone, Debug)]
pub struct Position {
    pub start: usize,
    pub end: usize,
}

impl Default for Position {
    fn default() -> Self {
        Position { start: 0, end: 0 }
    }
}

#[derive(Debug, Default)]
pub struct Node<T> {
    pub position: Position,
    pub inner: T,
}
