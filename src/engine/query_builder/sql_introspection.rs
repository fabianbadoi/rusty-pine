use crate::analyze::{
    Column, ColumnName, Database, ForeignKey, Key, KeyReference, Server, Table, TableName,
};
use crate::engine::Comparison;
use std::collections::HashSet;

use crate::engine::query_builder::{
    BinaryCondition, Computation, Condition, QueryBuildError, SelectedColumn, Sourced,
};
use crate::engine::syntax::{OptionalInput, SqlIdentifierInput, TableInput};

type Result<T> = std::result::Result<T, QueryBuildError>;

pub trait Introspective {
    fn join_conditions(&self, from: TableInput, to: TableInput) -> Result<Vec<Sourced<Condition>>>;
    fn columns(&self, table: TableInput) -> Result<&[Column]>;
    fn neighbors(&self, table: TableInput) -> Result<Vec<ForeignKey>>;
}

impl Introspective for Server {
    fn join_conditions(&self, from: TableInput, to: TableInput) -> Result<Vec<Sourced<Condition>>> {
        let join = self.find_join(from, to)?;

        if join.from.key.columns.len() != join.to.key.columns.len() {
            // This should never happen.
            return Err(QueryBuildError::InvalidForeignKey {
                from: join.from.clone(),
                to: join.to.clone(),
            });
        }

        let left_columns = join.from.key.columns.iter();
        let right_columns = join.to.key.columns.iter();

        let column_pairs = left_columns.zip(right_columns);

        let conditions = column_pairs
            .map(|(from_column, to_column)| BinaryCondition {
                left: selected_column(from, from_column),
                comparison: Sourced::from_introspection(Comparison::Equals),
                right: selected_column(to, to_column),
            })
            .map(|cond| Condition::Binary(Sourced::from_introspection(cond)))
            .map(Sourced::from_introspection)
            .collect();

        Ok(conditions)
    }

    fn columns(&self, table: TableInput) -> Result<&[Column]> {
        let table = self.table(table)?;

        Ok(table.columns.as_slice())
    }

    fn neighbors(&self, table: TableInput) -> Result<Vec<ForeignKey>> {
        let direct_joins = self.table(table)?.foreign_keys.iter().cloned();
        let reverse_joins = self
            .database_or_default(table.database)?
            .tables
            .iter()
            .flat_map(|(_, other)| {
                other
                    .foreign_keys
                    .iter()
                    .find(|other_table| other_table.to.table == table.table.it)
            })
            .map(|fk| fk.invert());

        let mut all_joins: Vec<_> = direct_joins.chain(reverse_joins).collect();

        all_joins.dedup_by(|a, b| a == b);

        Ok(all_joins)
    }
}

impl Server {
    fn find_join(&self, from: TableInput, to: TableInput) -> Result<ForeignKey> {
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
            from: from.table.it.into(),
            to: to.table.it.into(),
        })
    }

    fn find_direct_join(&self, from: TableInput, to: TableInput) -> Result<Option<&ForeignKey>> {
        let mut matching_keys = self
            .table(from)?
            .foreign_keys
            .iter()
            .filter(|fk| fk.to.table == to.table.it);

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
    fn find_incidental_join(&self, from: TableInput, to: TableInput) -> Result<Option<ForeignKey>> {
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

    fn table(&self, table: TableInput) -> Result<&Table> {
        let database = match table.database {
            OptionalInput::Implicit => self.default_database()?,
            OptionalInput::Specified(name) => self.database(name.it)?,
        };

        let table_name = TableName(table.table.it.name.to_string());
        let table = database
            .tables
            .get(&table_name)
            .ok_or_else(|| QueryBuildError::TableNotFound(self.params.clone(), table_name))?;

        Ok(table)
    }

    fn default_database(&self) -> Result<&Database> {
        self.databases
            .get(&self.params.default_database)
            .ok_or_else(|| {
                QueryBuildError::DefaultDatabaseNotFound(
                    self.params.clone(),
                    self.params.default_database.clone(),
                )
            })
    }

    fn database_or_default(
        &self,
        db_or_none: OptionalInput<Sourced<SqlIdentifierInput>>,
    ) -> Result<&Database> {
        match db_or_none {
            OptionalInput::Implicit => self.default_database(),
            OptionalInput::Specified(db_name) => self.database(db_name.it),
        }
    }

    fn database<T: AsRef<str>>(&self, name: T) -> Result<&Database> {
        let table = TableName(name.as_ref().to_string());

        self.databases
            .get(&table)
            .ok_or_else(|| QueryBuildError::DatabaseNotFound(self.params.clone(), table))
    }
}

fn selected_column(from: TableInput, from_column: &ColumnName) -> Sourced<Computation> {
    Sourced::from_introspection(Computation::SelectedColumn(Sourced::from_introspection(
        SelectedColumn {
            table: Some(Sourced::from_introspection(from.into())),
            column: Sourced::from_introspection(from_column.clone().into()),
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

impl PartialEq<SqlIdentifierInput<'_>> for TableName {
    fn eq(&self, other: &SqlIdentifierInput<'_>) -> bool {
        self.0 == other.name
    }
}
