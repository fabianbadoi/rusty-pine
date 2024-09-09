use crate::analyze::{
    Column, ColumnName, DBType, Database, DatabaseName, ForeignKey, Key, KeyReference, Server,
    Table, TableName,
};
use crate::engine::query_builder::{
    BinaryCondition, Computation, Condition, QueryBuildError, SelectedColumn, Sourced,
};
use crate::engine::syntax::{OptionalInput, SqlIdentifierInput, TableInput};
use crate::engine::Comparison;
use log::info;
use std::collections::HashSet;
use std::fmt::Debug;

type Result<T> = std::result::Result<T, QueryBuildError>;

pub trait Introspective {
    fn join_conditions(
        &self,
        from: Sourced<TableInput>,
        to: Sourced<TableInput>,
    ) -> Result<Vec<Sourced<Condition>>>;
    fn columns(&self, table: Sourced<TableInput>) -> Result<&[Column]>;
    fn neighbors(&self, table: Sourced<TableInput>) -> Result<Vec<ForeignKey>>;
    fn primary_key(&self, table: Sourced<TableInput>) -> Result<&Key>;
}

impl Introspective for Server {
    fn join_conditions(
        &self,
        from: Sourced<TableInput>,
        to: Sourced<TableInput>,
    ) -> Result<Vec<Sourced<Condition>>> {
        let join = self.find_join(from, to)?;

        if join.from.key.columns.len() != join.to.key.columns.len() {
            // This should never happen.
            return Err(QueryBuildError::InvalidForeignKey {
                from: from.it.table.into(),
                to: to.it.table.into(),
            });
        }

        let left_columns = join.from.key.columns.iter();
        let right_columns = join.to.key.columns.iter();

        let column_pairs = left_columns.zip(right_columns);

        let conditions = column_pairs
            .map(|(from_column, to_column)| BinaryCondition {
                left: selected_column(from.it, from_column),
                comparison: Sourced::from_introspection(Comparison::Equals),
                right: selected_column(to.it, to_column),
            })
            .map(|cond| Condition::Binary(Sourced::from_introspection(cond)))
            .map(Sourced::from_introspection)
            .collect();

        Ok(conditions)
    }

    fn columns(&self, table: Sourced<TableInput>) -> Result<&[Column]> {
        let table = self.table(table)?;

        Ok(table.columns.as_slice())
    }

    fn neighbors(&self, table: Sourced<TableInput>) -> Result<Vec<ForeignKey>> {
        info!("searching for direct joins");
        let direct_joins = self.table(table)?.foreign_keys.iter().cloned();
        info!("searching for reverse joins");
        let reverse_joins = self
            .database_or_default(table.it.database)?
            .tables
            .iter()
            .flat_map(|(_, other)| {
                other
                    .foreign_keys
                    .iter()
                    .find(|other_table| other_table.to.table == table.it.table.it)
            })
            .map(|fk| fk.invert());

        let mut all_joins: Vec<_> = direct_joins.chain(reverse_joins).collect();

        info!("deduplicating joins");
        all_joins.dedup_by(|a, b| a == b);

        Ok(all_joins)
    }

    fn primary_key(&self, table: Sourced<TableInput>) -> Result<&Key> {
        let table = self.table(table)?;

        Ok(&table.primary_key)
    }
}

impl Server {
    fn find_join(&self, from: Sourced<TableInput>, to: Sourced<TableInput>) -> Result<ForeignKey> {
        if let Some(direct_join) = self.find_direct_join(from, to)? {
            return Ok(direct_join.clone());
        }

        if let Some(inverse_join) = self.find_direct_join(to, from)? {
            // Invert the "join" so the "to" and "from" tables match.
            return Ok(inverse_join.invert());
        }

        if let Some(incidental_join) = self.find_incidental_join(from, to)? {
            // The tables happen to have a common FK to another table.
            // For example, both tables could have a "friendId" table.
            return Ok(incidental_join);
        }

        Err(QueryBuildError::JoinNotFound {
            from: from.it.table.into(),
            to: to.it.table.into(),
        })
    }

    fn find_direct_join(
        &self,
        from: Sourced<TableInput>,
        to: Sourced<TableInput>,
    ) -> Result<Option<&ForeignKey>> {
        let mut matching_keys = self
            .table(from)?
            .foreign_keys
            .iter()
            .filter(|fk| fk.to.table == to.it.table.it);

        // auto joins get the first possible way to join, even if multiple are available
        Ok(matching_keys.next())
    }

    /// Finds joins that incidentally happen to be usable.
    ///
    /// For tables that don't have direct foreign keys between them, we can
    /// try to find if they share any other foreign key that they share.
    ///
    /// For example, the userSettings and userLogs tables might both have a
    /// foreign key to the users table. We can then join on userId.
    fn find_incidental_join(
        &self,
        from: Sourced<TableInput>,
        to: Sourced<TableInput>,
    ) -> Result<Option<ForeignKey>> {
        let from = self.table(from)?;
        let to = self.table(to)?;

        let mut first_common = from
            .foreign_keys
            .iter()
            .filter_map(|from_fk| {
                // O(n^2) is not the best, but the numbers should be low.
                to.foreign_keys
                    .iter()
                    .find(|to_fk| {
                        if from_fk.to.table != to_fk.to.table {
                            return false;
                        }

                        // We use a hash for comparison here because in this case the order of
                        // keys is not relevant.
                        let from_keys: HashSet<_> = from_fk.to.key.columns.iter().collect();
                        let to_keys: HashSet<_> = to_fk.to.key.columns.iter().collect();

                        from_keys == to_keys
                    })
                    .map(|to_fk| (from_fk, to_fk))
            })
            .map(|(from_fk, to_fk)| {
                // Our foreign key could look something like this:
                // key A:   col_a1    col_a2
                //             |         |
                // other:   col_o1    col_o2
                //                \  /
                //                 \/
                //                 /\
                //                /  \
                // key B:   col_b1    col_b2
                // That is to say, the column order might not be the same for both keys:
                // key A:  (col_a1 -> col_o1) (col_a2 -> col_b2)
                // key B:  (col_b1 -> col_12) (col_b2 -> col_o1)
                //
                // We first have to order the pairs in the right order, and then produce:
                // "key": (col_a1 -> col_b2) (col_a2 -> col_b1)
                // I'm using quotes here because this is not a real foreign key on the server,
                // but we can use it like one.

                // It really doesn't matter how we order the key pairs, as long as it's consistent
                // and it's done by comparing the "to" fk columns.
                fn sort_by_to_col<'a>(
                    (_, to_col): &(&ColumnName, &'a ColumnName),
                ) -> &'a ColumnName {
                    to_col
                }

                let from_key_columns = get_from_columns_sorted_by(from_fk, sort_by_to_col);

                let to_key_columns = get_from_columns_sorted_by(to_fk, sort_by_to_col);

                // Yes, this is a fake foreign key.
                ForeignKey {
                    from: KeyReference {
                        table: from.name.clone(),
                        key: Key {
                            columns: from_key_columns,
                        },
                    },
                    to: KeyReference {
                        table: to.name.clone(),
                        key: Key {
                            columns: to_key_columns,
                        },
                    },
                }
            });

        // auto joins get the first possible way to join, even if multiple are available
        Ok(first_common.next())
    }

    fn table(&self, name: Sourced<TableInput>) -> Result<&Table> {
        let database = match name.it.database {
            OptionalInput::Implicit => self.default_database()?,
            OptionalInput::Specified(name) => self.database(name)?,
        };

        let table_name = TableName::new(name.it.table.it.name.to_string());
        let table = database
            .tables
            .get(&table_name)
            .ok_or_else(|| QueryBuildError::TableNotFound(name.map(|_| table_name)))?;

        Ok(table)
    }

    fn default_database(&self) -> Result<&Database> {
        let db_or_schema = match self.params.db_type {
            DBType::PostgresSQL => self
                .params
                .default_schema
                .as_ref()
                .ok_or(QueryBuildError::InvalidPostgresConfig)?,
            DBType::MariaDB => &self.params.database,
        };

        self.databases
            .get(db_or_schema)
            .ok_or_else(|| QueryBuildError::DefaultDatabaseNotFound(self.params.clone()))
    }

    fn database_or_default(
        &self,
        db_or_none: OptionalInput<Sourced<SqlIdentifierInput>>,
    ) -> Result<&Database> {
        match db_or_none {
            OptionalInput::Implicit => self.default_database(),
            OptionalInput::Specified(db_name) => self.database(db_name),
        }
    }

    fn database<T: AsRef<str> + Clone + Debug>(&self, name: Sourced<T>) -> Result<&Database> {
        let db_name = DatabaseName(name.it.as_ref().to_string());

        self.databases
            .get(&db_name)
            .ok_or_else(|| QueryBuildError::DatabaseNotFound(name.map(|_| db_name)))
    }
}

fn selected_column(from: TableInput, from_column: &ColumnName) -> Sourced<Computation> {
    Sourced::from_introspection(Computation::SelectedColumn(Sourced::from_introspection(
        SelectedColumn {
            table: Some(Sourced::from_introspection(from.into())),
            column: Sourced::from_introspection(from_column.clone()),
        },
    )))
}

fn get_from_columns_sorted_by<F>(key: &ForeignKey, sort_key: F) -> Vec<ColumnName>
where
    for<'a> F: Fn(&(&'a ColumnName, &'a ColumnName)) -> &'a ColumnName,
{
    let mut from_pairs = key.key_pairs();

    from_pairs.sort_by_key(sort_key);

    from_pairs
        .iter()
        .map(|(from_column, _)| from_column)
        .map(|col| (*col).clone())
        .collect()
}
