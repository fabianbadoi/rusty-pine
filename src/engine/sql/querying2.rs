use crate::analyze::{Column, ForeignKey, Key, SchemaObjectName, ServerParams, TableName};
use crate::engine::sql::querying::TableDescription;
use crate::Error;
use async_trait::async_trait;
use sqlx::{MySql as MariaDB, MySqlPool, Pool};
use std::collections::HashMap;
use std::future::Future;

mod mariadb;

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
    server_params: ServerParams,
}

pub type MariaDBConnection<'a> = Connection<Pool<MariaDB>>;

pub async fn mariadb(
    server_params: ServerParams,
    password: &str,
) -> Result<Connection<Pool<MariaDB>>, Error> {
    let pool = MySqlPool::connect(&format!(
        "mariadb://{user}:{password}@{host}:{port}/{db_name}",
        user = &server_params.user,
        host = &server_params.hostname,
        port = &server_params.port,
        db_name = &server_params.default_database,
    ))
    .await?;

    Ok(Connection {
        pool,
        server_params,
    })
}
