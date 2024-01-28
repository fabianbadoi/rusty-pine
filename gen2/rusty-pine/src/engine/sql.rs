/// Parses CREATE TABLE queries into Database instances.
mod parsing;
pub mod querying;
/// Structs used to represent database structure.
pub mod structure;

use crate::engine::sql::structure::Database;

struct DatabaseInfo<'a> {
    /// The original create table queries.
    ///
    /// The way this struct works is by keeping the create table queries in memory, and only making
    /// certain views into the data available. The lsp idea is that any return type that can be
    /// read from this struct, will only contain references to the "inner" data.
    /// If we had really large create table queries, this would mean we avoid duplicating/cloning
    /// some strings. I suspect that in practice this "optimization" is worthless, but it was more
    /// fun to write.
    create_table_queries: Vec<String>,

    /// Structure of the database.
    ///
    /// Only contains &str's from the create table queries.
    database: Database<'a>,
}
