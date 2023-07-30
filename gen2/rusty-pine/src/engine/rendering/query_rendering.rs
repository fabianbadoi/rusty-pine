use crate::engine::query_builder::{DatabaseName, Query, Sourced, Table, TableName};
use std::fmt::{Display, Formatter};

pub fn render_query(query: Query) -> String {
    format!("{}", query)
}

impl Display for Query {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "SELECT")?;
        writeln!(f, "FROM {}", self.from)?;
        writeln!(f, "LIMIT")?;

        Ok(())
    }
}

impl Display for Table {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(database) = &self.db {
            write!(f, "{}.", database)?;
        }

        write!(f, "{}", self.name)
    }
}

impl Display for DatabaseName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for TableName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T> Display for Sourced<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.it)
    }
}
