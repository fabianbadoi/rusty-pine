use crate::error::{Error, InternalError};
use mysql::prelude::Queryable;
use mysql::PooledConn;
use std::fmt::{Display, Formatter};

/// Holds the name of a database or table.
///
/// This struct can only be constructed in this module. Wrapping table and database names like means
/// that the compiler will guarantee that we don't call functions with db and table names that don't
/// exit.
///
/// Note: it can still happen if the table/database is dropped right after it's first listed, but before
/// we analyze it.
#[derive(Debug)]
pub struct SchemaObjectName(String);

/// Holds the description of a table, as returned from SHOW CREATE TABLE queries.
///
/// This struct can only be constructed in this module. Wrapping the results of SHOW CREATE TABLE queries
/// like this means the compile kind of guarantees parsing them will always succeed - because we know 100%
/// there are no syntax errors, because it was generated by MySQL itself.
///
/// Note: this is not strictly true, because there may be features we don't support.
#[derive(Debug)]
pub struct TableDescription(String);

pub fn list_databases(connection: &mut PooledConn) -> Result<Vec<SchemaObjectName>, Error> {
    let databases = connection.query(r"SHOW DATABASES")?;

    Ok(databases.into_iter().map(to_id).collect())
}

pub fn list_tables(
    connection: &mut PooledConn,
    db_name: &SchemaObjectName,
) -> Result<Vec<SchemaObjectName>, Error> {
    let tables = connection.query(format!("SHOW TABLES IN {}", db_name))?;

    Ok(tables.into_iter().map(to_id).collect())
}

pub fn describe_table(
    connection: &mut PooledConn,
    database: &SchemaObjectName,
    table: &SchemaObjectName,
) -> Result<TableDescription, Error> {
    let result: Option<(String, String)> =
        connection.query_first(format!("SHOW CREATE TABLE `{}`.`{}`", database, table))?;

    match result {
        Some((_, create_table_query)) => Ok(TableDescription(create_table_query)),
        None => Err(InternalError(format!("Table disappeared `{}`.`{}`", database, table)).into()),
    }
}

// We do not export this function because we want all instances of the struct to be created inside this
// module.
fn to_id(db_identifier: String) -> SchemaObjectName {
    SchemaObjectName(db_identifier)
}

// This makes the most convenient way of using the struct also be SQL injection safe.
impl Display for SchemaObjectName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Even though we fully control the providence of SchemaObjectNames, they can still
        // contain weird stuff.
        let sql_injection_safe = self.0.replace('`', "``");

        write!(f, "{}", sql_injection_safe)
    }
}

impl TableDescription {
    /// Convenience method to enable constructing this in tests
    ///
    /// Since we use #[cfg(test)], there is no danger of someone constructing stupid queries.
    #[cfg(test)]
    pub(crate) fn new_for_tests<T>(input: T) -> Self
    where
        T: Into<String>,
    {
        TableDescription(input.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl SchemaObjectName {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}
