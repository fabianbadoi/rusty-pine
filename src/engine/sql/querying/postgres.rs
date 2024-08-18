use super::Connection;
use crate::analyze::{
    Column, ColumnName, ForeignKey, Key, KeyReference, SchemaObjectName, TableName,
};
use crate::engine::sql::querying::to_id;
use crate::engine::sql::querying::Analyzer;
use crate::Error;
use async_trait::async_trait;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;

#[async_trait]
impl Analyzer for Connection<Pool<Postgres>> {
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
        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT table_schema, table_name\n\
             FROM information_schema.tables\n\
             WHERE TABLE_SCHEMA = $1",
        )
        .bind(database.as_str())
        .fetch_all(&self.pool)
        .await?;

        let rows = rows.into_iter().map(TableName::new).collect();

        Ok(rows)
    }

    async fn table_columns(
        &self,
        database: &SchemaObjectName,
    ) -> Result<HashMap<TableName, Vec<Column>>, Error> {
        let rows: Vec<(String, String, String)> = sqlx::query_as(
            "SELECT table_schema, table_name, column_name\n\
             FROM information_schema.columns\n\
             WHERE TABLE_SCHEMA = $1\n\
             ORDER BY ordinal_position\n\
             LIMIT 25000",
        )
        .bind(database.as_str())
        .fetch_all(&self.pool)
        .await?;

        let mut columns = HashMap::new();
        for (schema_name, table_name, column_name) in rows {
            let table_name = TableName::with(schema_name, table_name);
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
        #[derive(sqlx::FromRow)]
        struct FKRow {
            ct_catalog: String,
            ct_schema: String,
            ct_name: String,
            table_schema: String,
            table_name: String,
            column_name: String,
            foreign_table_schema: String,
            foreign_table_name: String,
            foreign_column_name: String,
        }

        let rows: Vec<FKRow> = sqlx::query_as(
            "SELECT\n\
                    kcu.constraint_catalog AS ct_catalog,\n\
                    kcu.constraint_schema AS ct_schema,\n\
                    kcu.constraint_name AS ct_name,\n\
                    t.table_schema AS table_schema,\n\
                    t.table_name AS table_name,\n\
                    kcu.column_name as column_name,\n\
                    ccu.table_schema AS foreign_table_schema,\n\
                    ccu.table_name AS foreign_table_name,\n\
                    ccu.column_name AS foreign_column_name\n\
                FROM information_schema.table_constraints AS tc\n\
                JOIN information_schema.key_column_usage AS kcu\n\
                    ON tc.constraint_name = kcu.constraint_name\n\
                        AND tc.table_schema = kcu.table_schema\n\
                        and tc.table_catalog = kcu.table_catalog\n\
                JOIN information_schema.constraint_column_usage AS ccu\n\
                    ON ccu.constraint_name = tc.constraint_name\n\
                        and ccu.constraint_catalog = tc.constraint_catalog\n\
                        and ccu.constraint_schema  = tc.constraint_schema\n\
                left join information_schema.tables t\n\
                    on tc.table_catalog = t.table_catalog\n\
                        and tc.table_schema  = t.table_schema\n\
                        and tc.table_name  = t.table_name\n\
                WHERE tc.constraint_type = 'FOREIGN KEY'\n\
                    and t.table_catalog  = $1\n\
                    and ccu.table_catalog = $1\n\
                    and t.table_schema = $2\n\
                    ORDER BY kcu.ordinal_position asc\n\
                    limit 25000",
        )
        // TODO catalogs or dbs?
        .bind(self.pool.connect_options().get_database().expect("TODO"))
        .bind(database.as_str())
        .fetch_all(&self.pool)
        .await?;

        let mut foreign_keys: HashMap<TableName, HashMap<(String, String, String), ForeignKey>> =
            HashMap::new();

        for row in rows {
            let FKRow {
                ct_catalog,
                ct_schema,
                ct_name,
                table_schema,
                table_name,
                column_name,
                foreign_table_schema,
                foreign_table_name,
                foreign_column_name,
            } = row;

            let table_name = TableName::with(table_schema, table_name);
            let table_fks = foreign_keys
                .entry(table_name.clone(/* :'''(*/))
                .or_default();
            let fk = table_fks
                .entry((ct_catalog, ct_schema, ct_name))
                .or_insert_with(|| ForeignKey {
                    from: KeyReference {
                        table: table_name,
                        key: Key { columns: vec![] },
                    },
                    to: KeyReference {
                        table: TableName::with(foreign_table_schema, foreign_table_name),
                        key: Key { columns: vec![] },
                    },
                });

            fk.from.key.columns.push(ColumnName(column_name));
            fk.to.key.columns.push(ColumnName(foreign_column_name));
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
        let rows: Vec<(String, String, String)> = sqlx::query_as(
            "SELECT tab.table_schema,
                    tab.table_name,
                    kcu.column_name
                FROM information_schema.tables tab
                LEFT JOIN information_schema.table_constraints tco
                    ON tco.table_schema = tab.table_schema
                        AND tco.table_name = tab.table_name
                        AND tco.constraint_type = 'PRIMARY KEY'
                LEFT JOIN information_schema.key_column_usage kcu
                    ON kcu.constraint_name = tco.constraint_name
                        AND kcu.constraint_schema = tco.constraint_schema
                        AND kcu.constraint_name = tco.constraint_name
                WHERE tab.table_catalog = $1
                    AND tab.table_schema = $2
                    AND column_name is not null
                    -- AND tab.table_type = 'BASE TABLE'
                ORDER BY tab.table_schema, tab.table_name
                LIMIT 25000",
        )
        .bind(self.pool.connect_options().get_database().expect("TODO"))
        .bind(database.as_str())
        .fetch_all(&self.pool)
        .await?;

        let mut pks = HashMap::new();
        for (schema, table, column) in rows {
            let table = TableName::with(schema, table);
            let pk = pks.entry(table).or_insert_with(|| Key {
                columns: Vec::new(),
            });

            pk.columns.push(ColumnName(column));
        }

        Ok(pks)
    }
}
