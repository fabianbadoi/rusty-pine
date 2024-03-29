use crate::common::{BinaryFilterType, UnaryFilterType};
use crate::error::Position;

#[derive(Debug)]
pub struct Pine<'a> {
    pub operations: Vec<Node<Operation<'a>>>,
    pub pine_string: InputType<'a>,
}

#[derive(Debug)]
pub enum Operation<'a> {
    From(Node<TableName<'a>>),
    Join(Node<TableName<'a>>),
    ExplicitJoin(Node<TableName<'a>>, Node<ColumnName<'a>>),
    Select(Vec<Node<Operand<'a>>>),
    Unselect(Vec<Node<Operand<'a>>>),
    Filter(Vec<Node<Filter<'a>>>),
    GroupBy(Vec<Node<Operand<'a>>>),
    Order(Vec<Node<Order<'a>>>),
    Limit(Node<Value<'a>>),
    Meta(MetaOperation),
}

impl<'a> Operation<'a> {
    #[cfg(test)]
    pub fn get_name(&self) -> &str {
        use Operation::*;

        match self {
            From(_) => "from",
            Join(_) | ExplicitJoin(_, _) => "join",
            Select(_) => "select",
            Unselect(_) => "unselect",
            Filter(_) => "filter",
            GroupBy(_) => "group by",
            Order(_) => "order",
            Limit(_) => "limit",
            Meta(meta_operation) => meta_operation.get_name(),
        }
    }
}

#[derive(Debug)]
pub enum MetaOperation {
    ShowNeighbours(Position),
    ShowColumns(Position),
}

impl MetaOperation {
    #[cfg(test)]
    pub fn get_name(&self) -> &'static str {
        match self {
            MetaOperation::ShowNeighbours(_) => "show neighbours",
            MetaOperation::ShowColumns(_) => "show columns",
        }
    }
}

#[derive(Debug)]
pub enum Operand<'a> {
    Value(Node<Value<'a>>),
    Column(Node<ColumnIdentifier<'a>>),
    FunctionCall(Node<FunctionName<'a>>, Node<FunctionOperand<'a>>),
}

#[derive(Debug)]
pub enum Filter<'a> {
    PrimaryKey(Node<Operand<'a>>),
    Unary(Node<Operand<'a>>, UnaryFilterType),
    Binary(Node<Operand<'a>>, Node<Operand<'a>>, BinaryFilterType),
}

#[derive(Debug)]
pub enum Order<'a> {
    Ascending(Node<Operand<'a>>),
    Descending(Node<Operand<'a>>),
}

#[derive(Debug)]
pub enum Value<'a> {
    Numeric(InputType<'a>),
    String(InputType<'a>),
}

#[derive(Debug)]
pub enum ColumnIdentifier<'a> {
    Implicit(Node<ColumnName<'a>>),
    Explicit(Node<TableName<'a>>, Node<ColumnName<'a>>),
}

#[derive(Debug)]
pub enum FunctionOperand<'a> {
    Identifier(Node<ColumnIdentifier<'a>>),
    Constant(Node<Constant<'a>>),
}

pub type Identifier<'a> = InputType<'a>;
pub type TableName<'a> = Identifier<'a>;
pub type ColumnName<'a> = Identifier<'a>;
pub type FunctionName<'a> = Identifier<'a>;
pub type Constant<'a> = InputType<'a>;
pub type InputType<'a> = &'a str;

#[derive(Debug, Default)]
pub struct Node<T> {
    pub position: Position,
    pub inner: T,
}

impl<'a> IntoIterator for &'a Node<Pine<'a>> {
    type Item = &'a Node<Operation<'a>>;
    type IntoIter = std::slice::Iter<'a, Node<Operation<'a>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.operations.iter()
    }
}

impl<'a> Node<Pine<'a>> {
    pub fn last_operation(&self) -> Option<&Node<Operation<'a>>> {
        self.inner.operations.last()
    }
}

impl Value<'_> {
    pub fn to_string(&self) -> String {
        match self {
            Value::Numeric(value) => value.to_string(),
            Value::String(value) => format!("{}", value),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Value::Numeric(value) | Value::String(value) => value,
        }
    }
}

impl Node<&'_ str> {
    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }
}
