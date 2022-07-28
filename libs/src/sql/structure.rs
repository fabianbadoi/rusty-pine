pub use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    pub name: String, // TODO: DatabaseName?
    pub tables: Vec<Table>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub name: String, // TODO: TableName?
    pub primary_key: Column,
    pub columns: Vec<Column>,
    pub foreign_keys: Vec<ForeignKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForeignKey {
    pub from_column: ColumnName,
    pub to_table: TableName,
    pub to_column: ColumnName,
}

// TODO use these everywhere
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnName(pub String);
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableName(pub String);

impl Table {
    pub fn get_foreign_key(&self, to_table: &str) -> Option<&ForeignKey> {
        self.foreign_keys
            .iter()
            .find(|foreign_key| foreign_key.to_table == to_table)
    }
}

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

impl<'a> From<&'a TableName> for &'a str {
    fn from(name: &'a TableName) -> &'a str {
        &name.0
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
