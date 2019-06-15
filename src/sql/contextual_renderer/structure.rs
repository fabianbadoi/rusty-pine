#[derive(Debug)]
pub struct Column {
    pub name: String,
}

#[derive(Debug)]
pub struct Table {
    pub name: String,
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
pub struct ColumnName(String);
#[derive(Debug, PartialEq, Eq)]
pub struct TableName(String);

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
