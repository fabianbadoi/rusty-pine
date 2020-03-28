use crate::common::{BinaryFilterType, UnaryFilterType};

#[derive(Debug)]
pub enum Renderable {
    Query(Query),
}

#[cfg(test)]
impl Renderable {
    /// This was introduced to help pre-existing tests not change much during a rewrite
    pub fn query(self) -> Query {
        match self {
            Renderable::Query(query) => query,
        }
    }
}

#[derive(Debug)]
pub struct Query {
    pub selections: Vec<Operand>,
    pub unselections: Vec<Operand>,
    pub from: TableName,
    pub joins: Vec<TableName>,
    pub filters: Vec<Filter>,
    pub group_by: Vec<Operand>,
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
pub enum Operand {
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
