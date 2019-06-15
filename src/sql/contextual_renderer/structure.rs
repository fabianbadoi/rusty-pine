#[derive(Debug)]
pub struct Column {
    pub name: String,
}

#[derive(Debug)]
pub struct Table {
    pub name: String, // TODO: TableName?
    pub columns: Vec<Column>,
    pub foreign_keys: Vec<ForeignKey>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ForeignKey {
    pub from_column: ColumnName,
    pub to_table: TableName,
    pub to_column: ColumnName,
}

// TODO use these everywhere
#[derive(Debug, PartialEq, Eq)]
pub struct ColumnName(pub String);
#[derive(Debug, PartialEq, Eq)]
pub struct TableName(pub String);

impl PartialEq<&str> for ColumnName {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<&str> for TableName {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl From<&str> for ColumnName {
    fn from(name: &str) -> ColumnName {
        ColumnName(name.to_string())
    }
}

impl From<&str> for TableName {
    fn from(name: &str) -> TableName {
        TableName(name.to_string())
    }
}

impl From<&str> for Column {
    fn from(name: &str) -> Column {
        Column { name: name.into() }
    }
}

impl From<&(&str, (&str, &str))> for ForeignKey {
    fn from(spec: &(&str, (&str, &str))) -> ForeignKey {
        ForeignKey {
            from_column: spec.0.into(),
            to_table: (spec.1).0.into(),
            to_column: (spec.1).1.into(),
        }
    }
}
