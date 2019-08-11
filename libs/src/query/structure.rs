#[derive(Debug)]
pub struct Query {
    pub selections: Vec<QualifiedColumnIdentifier>,
    pub from: TableName,
    pub joins: Vec<TableName>,
    pub filters: Vec<Filter>,
    pub limit: usize,
}

impl Default for Query {
    fn default() -> Query {
        Query {
            selections: Default::default(),
            from: Default::default(),
            joins: Default::default(),
            filters: Default::default(),
            limit: 10,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct QualifiedColumnIdentifier {
    pub table: TableName,
    pub column: ColumnName,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Filter {
    Equals(Operand, Operand),
}

#[derive(Debug, Eq, PartialEq)]
pub enum Operand {
    Value(Value),
    Column(QualifiedColumnIdentifier),
}

pub type TableName = String;
pub type ColumnName = String;
pub type Value = String;
