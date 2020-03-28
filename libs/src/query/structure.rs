use crate::common::{BinaryFilterType, UnaryFilterType};

#[derive(Debug)]
pub struct Query {
    pub selections: Vec<ResultColumn>,
    pub unselections: Vec<ResultColumn>,
    pub from: TableName,
    pub joins: Vec<TableName>,
    pub filters: Vec<Filter>,
    pub group_by: Vec<ResultColumn>,
    pub order: Vec<Order>,
    pub limit: usize,
}

impl Default for Query {
    fn default() -> Query {
        Query {
            selections: Default::default(),
            unselections: Default::default(),
            from: Default::default(),
            joins: Default::default(),
            filters: Default::default(),
            group_by: Default::default(),
            order: Default::default(),
            limit: 10,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ResultColumn {
    Value(Value),
    Column(QualifiedColumnIdentifier),
    FunctionCall(FunctionName, QualifiedColumnIdentifier),
}

#[derive(Debug, PartialEq, Eq)]
pub struct QualifiedColumnIdentifier {
    pub table: TableName,
    pub column: ColumnName,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Filter {
    Unary(ResultColumn, UnaryFilterType),
    Binary(ResultColumn, ResultColumn, BinaryFilterType),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Order {
    Ascending(ResultColumn),
    Descending(ResultColumn),
}

pub type TableName = String;
pub type ColumnName = String;
pub type Value = String;
pub type FunctionName = String;
