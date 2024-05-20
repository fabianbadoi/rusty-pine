//! Structures used to represent the structure of the database. Used for using foreign keys to
//! augment our Pines.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

/// Each server config will be cached to disk to responding to queries way snappier.
///
/// This structure represents the info we gather for an entire analyzed DB server.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Server {
    pub params: ServerParams,
    pub databases: HashMap<TableName, Database>,
}

/// Parameters used to connect to a server
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ServerParams {
    pub hostname: String,
    pub port: u16,
    // Because the different users may have access to different databases and different tables,
    pub user: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    pub name: TableName,
    pub tables: HashMap<TableName, Table>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub name: ColumnName,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub name: TableName,
    pub primary_key: Key,
    pub columns: Vec<Column>,
    pub foreign_keys: Vec<ForeignKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Key {
    pub columns: Vec<ColumnName>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForeignKey {
    pub from: KeyReference,
    pub to: KeyReference,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyReference {
    pub table: TableName,
    pub key: Key,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnName(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TableName(pub String);

impl Table {
    pub fn get_foreign_key(&self, to_table: &str) -> Option<&ForeignKey> {
        self.foreign_keys
            .iter()
            .find(|foreign_key| foreign_key.to.table == to_table)
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

impl<T: Into<String>> From<T> for ColumnName {
    fn from(name: T) -> ColumnName {
        ColumnName(name.into())
    }
}

impl<T: Into<String>> From<T> for TableName {
    fn from(name: T) -> TableName {
        TableName(name.into())
    }
}

impl<T: Into<String>> From<T> for Column {
    fn from(name: T) -> Column {
        Column {
            name: name.into().into(),
        }
    }
}

impl<'a> From<&'a TableName> for &'a str {
    fn from(name: &'a TableName) -> &'a str {
        name.0.as_str()
    }
}

impl Display for ServerParams {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.port == 3306 {
            // If we're using the default port, we can just omit it.
            write!(f, "{}@{}", self.user, self.hostname)?
        } else {
            write!(f, "{}@{}:{}", self.user, self.hostname, self.port)?
        }

        Ok(())
    }
}
