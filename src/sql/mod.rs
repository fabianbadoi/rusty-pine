#[derive(Debug, Default)]
pub struct Query<'a> {
    pub selections: Vec<QualifiedColumnIdentifier<'a>>,
    pub from: Option<ColumnName<'a>>,
    pub filters: Vec<Filter<'a>>,
}

#[derive(Debug)]
pub struct QualifiedColumnIdentifier<'a> {
    pub table: TableName<'a>,
    pub column: ColumnName<'a>,
}

#[derive(Debug)]
pub struct Filter<'a> {
    pub column: QualifiedColumnIdentifier<'a>,
    pub condition: Condition<'a>
}

#[derive(Debug, Eq, PartialEq)]
pub enum Condition<'a> {
    Equals(Value<'a>)
}

pub type TableName<'a> = &'a str;
pub type ColumnName<'a> = &'a str;
pub type Value<'a> = &'a str;

