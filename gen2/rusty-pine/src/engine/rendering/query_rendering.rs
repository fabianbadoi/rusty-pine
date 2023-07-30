use crate::engine::query_builder::{
    ColumnName, DatabaseName, Query, Select, SelectedColumn, Sourced, Table, TableName,
};
use std::fmt::{write, Display, Formatter};

pub fn render_query(query: Query) -> String {
    format!("{}", query)
}

impl Display for Query {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "SELECT {}", RenderableSelect(self.select.as_slice()))?;
        writeln!(f, "FROM {}", self.from)?;
        writeln!(f, "LIMIT")?;

        Ok(())
    }
}

struct RenderableSelect<'a>(&'a [Sourced<Select>]);

impl Display for RenderableSelect<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some((last, first)) = self.0.split_last() {
            for select in first {
                write!(f, "{}, ", select)?;
            }

            write!(f, "{}", last)?;
        }

        Ok(())
    }
}

impl Display for Select {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Select::SelectedColumn(column) => write!(f, "{}", column),
        }
    }
}

impl Display for SelectedColumn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(table) = &self.table {
            write!(f, "{}.", table)?;
        }

        write!(f, "{}", self.column)
    }
}

impl Display for ColumnName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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
