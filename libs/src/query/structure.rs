use crate::common::{BinaryFilterType, UnaryFilterType};

#[derive(Debug, Eq, PartialEq)]
pub enum Renderable {
    Query(Query),
    Meta(RenderableMetaOperation),
}

#[derive(Debug, Eq, PartialEq)]
pub enum RenderableMetaOperation {
    ShowNeighbours(TableName),
    ShowColumns(TableName),
}

#[cfg(test)]
impl Renderable {
    /// This was introduced to help pre-existing tests not change much during a rewrite
    pub fn query(self) -> Query {
        match self {
            Renderable::Query(query) => query,
            Renderable::Meta(_) => panic!("incorrect call of .query() on meta operation"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Query {
    pub selections: Vec<Operand>,
    pub unselections: Vec<Operand>,
    pub from: TableName,
    pub joins: Vec<Join>,
    pub filters: Vec<Filter>,
    pub group_by: Vec<Operand>,
    pub order: Vec<Order>,
    pub limit: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Join {
    Auto(TableName),
    Explicit(JoinSpec),
}

#[derive(Debug, PartialEq, Eq)]
pub struct JoinSpec {
    pub from: TableName,
    pub from_foreign_key: ColumnName,
    pub to: TableName,
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
pub enum Operand {
    Value(Value),
    Column(QualifiedColumnIdentifier),
    FunctionCall(FunctionName, FunctionOperand),
}

#[derive(Debug, PartialEq, Eq)]
pub enum FunctionOperand {
    Column(QualifiedColumnIdentifier),
    Constant(Value),
}

#[derive(Debug, PartialEq, Eq)]
pub struct QualifiedColumnIdentifier {
    pub table: TableName,
    pub column: ColumnName,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Filter {
    PrimaryKey{table: TableName, value: Operand},
    Unary(Operand, UnaryFilterType),
    Binary(Operand, Operand, BinaryFilterType),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Order {
    Ascending(Operand),
    Descending(Operand),
}

pub type TableName = String;
pub type ColumnName = String;
pub type Value = String;
pub type FunctionName = String;
