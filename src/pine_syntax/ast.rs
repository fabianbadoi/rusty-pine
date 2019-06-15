use crate::error::Position;

pub type PineNode<'a> = Node<Pine<'a>>;
pub type OperationNode<'a> = Node<Operation<'a>>;
pub type FilterNode<'a> = Node<Filter<'a>>;
pub type ConditionNode<'a> = Node<Condition<'a>>;
pub type TableNameNode<'a> = Node<TableName<'a>>;
pub type ColumnNameNode<'a> = Node<ColumnName<'a>>;
pub type ValueNode<'a> = Node<Value<'a>>;

#[derive(Debug)]
pub struct Pine<'a> {
    pub operations: Vec<OperationNode<'a>>,
    pub pine_string: InputType<'a>,
}

#[derive(Debug)]
pub enum Operation<'a> {
    From(TableNameNode<'a>),
    Join(TableNameNode<'a>),
    Select(Vec<ColumnNameNode<'a>>),
    Filter(Vec<FilterNode<'a>>),
}

impl<'a> Operation<'a> {
    #[cfg(test)]
    pub fn get_name(&self) -> &str {
        use Operation::*;

        match self {
            From(_) => "from",
            Join(_) => "join",
            Select(_) => "select",
            Filter(_) => "filter",
        }
    }
}

#[derive(Debug)]
pub struct Filter<'a> {
    pub column: ColumnNameNode<'a>,
    pub condition: ConditionNode<'a>,
}

#[derive(Debug)]
pub enum Condition<'a> {
    Equals(ValueNode<'a>),
}

pub type Identifier<'a> = InputType<'a>;
pub type TableName<'a> = Identifier<'a>;
pub type ColumnName<'a> = Identifier<'a>;
pub type Value<'a> = InputType<'a>;
pub type InputType<'a> = &'a str;

#[derive(Debug, Default)]
pub struct Node<T> {
    pub position: Position,
    pub inner: T,
}

impl<'a> IntoIterator for &'a PineNode<'a> {
    type Item = &'a OperationNode<'a>;
    type IntoIter = std::slice::Iter<'a, OperationNode<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.operations.iter()
    }
}
