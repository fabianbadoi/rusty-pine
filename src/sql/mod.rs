mod renderer;
#[cfg(test)]
mod shorthand;

pub use self::renderer::{Renderer, StringRenderer};

#[derive(Debug, Default)]
pub struct Query {
    pub selections: Vec<QualifiedColumnIdentifier>,
    pub from: Option<ColumnName>,
    pub filters: Vec<Filter>,
}

#[derive(Debug)]
pub struct QualifiedColumnIdentifier {
    pub table: TableName,
    pub column: ColumnName,
}

#[derive(Debug)]
pub struct Filter {
    pub column: QualifiedColumnIdentifier,
    pub condition: Condition,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Condition {
    Equals(Value),
}

pub type TableName = String;
pub type ColumnName = String;
pub type Value = String;
