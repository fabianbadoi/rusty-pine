use crate::analyze::{ColumnName, Database, ForeignKey, KeyReference, Server, TableName};
use crate::engine::Comparison;

use crate::engine::query_builder::{
    BinaryCondition, Computation, Condition, QueryBuildError, SelectedColumn, Sourced,
};
use crate::engine::syntax::{OptionalInput, SqlIdentifierInput, TableInput};

type Result<T> = std::result::Result<T, QueryBuildError>;

pub trait Introspective {
    fn join_conditions(&self, from: TableInput, to: TableInput) -> Result<Vec<Sourced<Condition>>>;
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
}

impl Server {
    fn find_join(&self, from: TableInput, to: TableInput) -> Result<FoundForeignKey> {
        use FoundForeignKey as FK;
        if let Some(direct_join) = self.find_direct_join(from, to)? {
            return Ok(FK::direct(direct_join));
        }

        if let Some(inverse_join) = self.find_direct_join(to, from)? {
            // Invert the "join" so the "to" and "from" tables match.
            return Ok(FK::inverse(inverse_join));
        }

        Err(QueryBuildError::JoinNotFound {
            from: from.table.it.into(),
            to: to.table.it.into(),
        })
    }

    fn find_direct_join(&self, from: TableInput, to: TableInput) -> Result<Option<&ForeignKey>> {
        let database = self.database_or_default(from.database)?;

        let mut matching_keys = database
            .tables
            .iter()
            .filter(|(name, _)| **name == from.table.it)
            .flat_map(|(_, table)| table.foreign_keys.as_slice())
            .filter(|fk| fk.to.table == to.table.it);

        // auto joins get the first possible way to join, even if multiple are available
        Ok(matching_keys.next())
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
            OptionalInput::Specified(db_name) => {
                let table = TableName(db_name.it.name.to_string());

                self.databases
                    .get(&table)
                    .ok_or_else(|| QueryBuildError::DatabaseNotFound(self.params.clone(), table))
            }
        }
    }
}

struct FoundForeignKey<'a> {
    from: &'a KeyReference,
    to: &'a KeyReference,
}

impl<'a> FoundForeignKey<'a> {
    fn direct(fk: &ForeignKey) -> FoundForeignKey {
        FoundForeignKey {
            from: &fk.from,
            to: &fk.to,
        }
    }

    fn inverse(fk: &ForeignKey) -> FoundForeignKey {
        FoundForeignKey {
            from: &fk.to,
            to: &fk.from,
        }
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

impl PartialEq<SqlIdentifierInput<'_>> for TableName {
    fn eq(&self, other: &SqlIdentifierInput<'_>) -> bool {
        self.0 == other.name
    }
}
