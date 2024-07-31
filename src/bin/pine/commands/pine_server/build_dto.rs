use rusty_pine::analyze::Column;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct BuildResult {
    #[serde(rename = "connection-id")]
    pub connection_id: String,
    pub version: &'static str,
    pub query: String,
    pub state: State,
}

#[derive(Serialize, Deserialize)]
pub struct State {
    pub hints: Hints,
    #[serde(rename = "table-count")]
    pub table_count: usize,
    #[serde(rename = "where")]
    pub conditions: Vec<()>, // TODO
    pub limit: Option<String>,
    #[serde(rename = "pending-count")]
    pub pending_count: i64,
    pub columns: Vec<()>, // TODO
    // pub operation: Operation, // TODO wtf is this?
    pub joins: Joins,
    pub tables: Vec<Table>,
    /// the last table's alias
    pub context: String,
    #[serde(rename = "connection-id")]
    pub connection_id: String,
    // pub aliases: Aliases,
}

#[derive(Serialize, Deserialize)]
pub struct Hints {
    pub table: Vec<TableHint>,
}

#[derive(Serialize, Deserialize)]
pub struct TableHint {
    pub schema: String,
    pub table: String,
    #[serde(rename = "column")]
    pub join_using_column: String,
}

pub type Joins = HashMap<
    // the source table
    String,
    JoinSpec,
>;
pub type JoinSpec = HashMap<
    // the target table
    String,
    // "from", "from_key", "=", "to", "to_key"
    [String; 5],
>;

#[derive(Serialize, Deserialize)]
pub struct Table {
    #[serde(rename = "table")]
    pub name: String,
    pub alias: String,
}

#[derive(Deserialize)]
pub struct BuildRequest {
    pub expression: String,
}
