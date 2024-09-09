//! Structures used to represent the structure of the database. Used for using foreign keys to
//! augment our Pines.
use crate::analyze::SchemaObjectName;
use crate::cache::CacheableMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

/// Each server config will be cached to disk to responding to queries way snappier.
///
/// This structure represents the info we gather for an entire analyzed DB server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub params: ServerParams,
    pub databases: HashMap<DatabaseName, Database>,
}

/// Parameters used to connect to a server
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerParams {
    pub db_type: DBType,
    pub hostname: String,
    pub port: u16,
    // Because the different users may have access to different databases and different tables,
    pub user: String,
    /// Used for the default db for MariaDB and the database for Postgres.
    pub database: DatabaseName,
    pub default_schema: Option<DatabaseName>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DBType {
    PostgresSQL,
    MariaDB,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Database {
    pub name: DatabaseName,
    pub tables: CacheableMap<TableName, Table>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd, Hash)]
pub struct ColumnName(pub String);

#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub struct TableName {
    pub schema: Option<DatabaseName>, // Option because MariaDB does not have schemas
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DatabaseName(pub String);

impl TableName {
    /// Creates a TableName without a schema. Useful for MariaDB.
    pub fn named(name: String) -> Self {
        Self { schema: None, name }
    }

    pub fn with(schema: String, name: String) -> Self {
        Self {
            schema: Some(DatabaseName(schema)),
            name,
        }
    }

    pub fn new(pair: (String, String)) -> Self {
        Self {
            schema: Some(DatabaseName(pair.0)),
            name: pair.1,
        }
    }
}

impl PartialEq for TableName {
    fn eq(&self, other: &Self) -> bool {
        let schemas_match =
            self.schema.is_none() || other.schema.is_none() || self.schema == other.schema;

        schemas_match && self.name == other.name
    }
}

// I have to implement this manually because the schema should sometimes not play a part in Eq.
// Without this form of Hash, HashMap<TableName, _> would not work.
impl Hash for TableName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // intentionally ignoring schema
        self.name.hash(state)
    }
}

impl Table {
    pub fn get_foreign_key(&self, to_table: &str) -> Option<&ForeignKey> {
        self.foreign_keys
            .iter()
            .find(|foreign_key| foreign_key.to.table == to_table)
    }
}

impl ForeignKey {
    pub fn key_pairs(&self) -> Vec<(&ColumnName, &ColumnName)> {
        self.from
            .key
            .columns
            .iter()
            .zip(&self.to.key.columns)
            .collect()
    }

    pub fn invert(&self) -> ForeignKey {
        ForeignKey {
            from: self.to.clone(),
            to: self.from.clone(),
        }
    }
}

impl PartialEq<&str> for ColumnName {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<&str> for TableName {
    fn eq(&self, other: &&str) -> bool {
        self.name == *other
    }
}

impl<T: Into<String>> From<T> for ColumnName {
    fn from(name: T) -> ColumnName {
        ColumnName(name.into())
    }
}

impl AsRef<str> for ColumnName {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl<T: Into<String>> From<T> for TableName {
    fn from(name: T) -> TableName {
        TableName::named(name.into())
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
        name.name.as_str()
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

impl DatabaseName {
    pub fn new(id: SchemaObjectName) -> Self {
        DatabaseName(id.into_string())
    }
}
