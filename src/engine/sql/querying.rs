mod mariadb;
mod postgres;

use std::fmt::{Display, Formatter};

use crate::analyze::{Column, ForeignKey, Key, ServerParams, TableName};
use crate::Error;
use async_trait::async_trait;
use sqlx::postgres::PgPoolOptions;
use sqlx::{MySql as MariaDB, MySqlPool, Pool, Postgres};
use std::collections::HashMap;

/// Holds the name of a database or table.
///
/// This struct can only be constructed in this module. Wrapping table and database names like means
/// that the compiler will guarantee that we don't call functions with db and table names that don't
/// exit.
///
/// Note: it can still happen if the table/database is dropped right after it's first listed, but before
/// we analyze it.
#[derive(Debug, Clone)]
pub struct SchemaObjectName(String);

// We do not export this function because we want all instances of the struct to be created inside this
// module.
pub fn to_id(db_identifier: String) -> SchemaObjectName {
    SchemaObjectName(db_identifier)
}

#[async_trait]
pub trait Analyzer {
    async fn list_databases(&self) -> Result<Vec<SchemaObjectName>, Error>;

    async fn list_tables(&self, database: &SchemaObjectName) -> Result<Vec<TableName>, Error>;

    async fn table_columns(
        &self,
        database: &SchemaObjectName,
    ) -> Result<HashMap<TableName, Vec<Column>>, Error>;

    async fn table_foreign_keys(
        &self,
        database: &SchemaObjectName,
    ) -> Result<HashMap<TableName, Vec<ForeignKey>>, Error>;

    async fn table_primary_keys(
        &self,
        database: &SchemaObjectName,
    ) -> Result<HashMap<TableName, Key>, Error>;
}

pub struct Connection<T> {
    pool: T,
}

pub type MariaDBConnection<'a> = Connection<Pool<MariaDB>>;

pub async fn postgres(
    server_params: ServerParams,
    password: &str,
) -> Result<Connection<Pool<Postgres>>, Error> {
    let pool = PgPoolOptions::new()
        .connect(&format!(
            "postgres://{user}:{password}@{host}:{port}/{catalog}",
            user = &server_params.user,
            host = &server_params.hostname,
            port = &server_params.port,
            catalog = &server_params.database,
        ))
        .await?;

    Ok(Connection { pool })
}

pub async fn mariadb(
    server_params: ServerParams,
    password: &str,
) -> Result<Connection<Pool<MariaDB>>, Error> {
    let pool = MySqlPool::connect(&format!(
        "mariadb://{user}:{password}@{host}:{port}/{db_name}",
        user = &server_params.user,
        host = &server_params.hostname,
        port = &server_params.port,
        db_name = &server_params.database,
    ))
    .await?;

    Ok(Connection { pool })
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

impl SchemaObjectName {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}
