use super::Connection;
use crate::analyze::{
    Column, ColumnName, ForeignKey, Key, KeyReference, SchemaObjectName, TableName,
};
use crate::engine::sql::querying::to_id;
use crate::engine::sql::querying::Analyzer;
use crate::Error;
use async_trait::async_trait;
use sqlx::{MySql as MariaDB, Pool};
use std::collections::HashMap;

#[async_trait]
impl Analyzer for Connection<Pool<MariaDB>> {
    async fn list_databases(&self) -> Result<Vec<SchemaObjectName>, Error> {
        let rows: Vec<(String,)> = sqlx::query_as(
            // "SELECT TABLE_NAME\n\
            //  FROM information_schema.TABLES\n\
            //  WHERE TABLE_SCHEMA = ?",
            "SELECT SCHEMA_NAME\n\
            FROM information_schema.SCHEMATA",
        )
        .fetch_all(&self.pool)
        .await?;

        let rows = rows.into_iter().map(|row| to_id(row.0)).collect();

        Ok(rows)
    }

    async fn list_tables(&self, database: &SchemaObjectName) -> Result<Vec<TableName>, Error> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT TABLE_NAME\n\
             FROM information_schema.TABLES\n\
             WHERE TABLE_SCHEMA = ?",
        )
        .bind(database.as_str())
        .fetch_all(&self.pool)
        .await?;

        let rows = rows
            .into_iter()
            .map(|row| TableName::named(row.0))
            .collect();

        Ok(rows)
    }

    async fn table_columns(
        &self,
        database: &SchemaObjectName,
    ) -> Result<HashMap<TableName, Vec<Column>>, Error> {
        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT TABLE_NAME, COLUMN_NAME\n\
             FROM information_schema.COLUMNS\n\
             WHERE TABLE_SCHEMA = ?\n\
             ORDER BY ORDINAL_POSITION\n\
             LIMIT 25000",
        )
        .bind(database.as_str())
        .fetch_all(&self.pool)
        .await?;

        let mut columns = HashMap::new();
        for (table_name, column_name) in rows {
            let table_name = TableName::named(table_name);
            if !columns.contains_key(&table_name) {
                columns.insert(table_name.clone(), Vec::new());
            }

            let table_columns = columns
                .get_mut(&table_name)
                .expect("We made sure there's a val here right above.");
            table_columns.push(Column {
                name: ColumnName(column_name),
            });
        }

        Ok(columns)
    }

    async fn table_foreign_keys(
        &self,
        database: &SchemaObjectName,
    ) -> Result<HashMap<TableName, Vec<ForeignKey>>, Error> {
        let rows: Vec<(String, String, String, String, String, String)> = sqlx::query_as(
        "SELECT CONSTRAINT_NAME, TABLE_NAME, COLUMN_NAME, REFERENCED_TABLE_SCHEMA, REFERENCED_TABLE_NAME, REFERENCED_COLUMN_NAME\n\
            FROM information_schema.KEY_COLUMN_USAGE\n\
            WHERE REFERENCED_TABLE_NAME is not null\n\
                AND TABLE_SCHEMA = ?\n\
	        ORDER BY TABLE_NAME, CONSTRAINT_NAME, ORDINAL_POSITION\n\
            LIMIT 25000",
        )
        .bind(database.as_str())
        .fetch_all(&self.pool)
        .await?;

        let mut foreign_keys: HashMap<TableName, HashMap<String, ForeignKey>> = HashMap::new();

        for row in rows {
            let (
                constraint_name,
                table_name,
                column_name,
                _referenced_db_name,
                referenced_table_name,
                referenced_column_name,
            ) = row;

            let table_name = TableName::named(table_name);
            let table_fks = foreign_keys.entry(table_name.clone(/* :'( */)).or_default();
            let fk = table_fks
                .entry(constraint_name)
                .or_insert_with(|| ForeignKey {
                    from: KeyReference {
                        table: table_name,
                        key: Key { columns: vec![] },
                    },
                    to: KeyReference {
                        table: TableName::named(referenced_table_name),
                        key: Key { columns: vec![] },
                    },
                });

            fk.from.key.columns.push(ColumnName(column_name));
            fk.to.key.columns.push(ColumnName(referenced_column_name));
        }

        let foreign_keys = foreign_keys
            .into_iter()
            .map(|(table, fks)| (table, fks.into_values().collect::<Vec<_>>()))
            .collect();

        Ok(foreign_keys)
    }

    async fn table_primary_keys(
        &self,
        database: &SchemaObjectName,
    ) -> Result<HashMap<TableName, Key>, Error> {
        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT TABLE_NAME, COLUMN_NAME\n\
            FROM information_schema.KEY_COLUMN_USAGE\n\
            WHERE REFERENCED_TABLE_NAME is null\n\
                AND TABLE_SCHEMA = ?\n\
	        ORDER BY TABLE_NAME, ORDINAL_POSITION\n\
            LIMIT 25000",
        )
        .bind(database.as_str())
        .fetch_all(&self.pool)
        .await?;

        let mut pks = HashMap::new();
        for (table, column) in rows {
            let table = TableName::named(table);
            let pk = pks.entry(table).or_insert_with(|| Key {
                columns: Vec::new(),
            });

            pk.columns.push(ColumnName(column));
        }

        Ok(pks)
    }
}
