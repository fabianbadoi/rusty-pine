use crate::engine::query_builder::{
    ColumnName, Computation, DatabaseName, ExplicitJoin, FunctionCall, Query, SelectedColumn,
    Table, TableName,
};
use crate::engine::syntax::JoinType;
use crate::engine::{Limit, Sourced};
use std::fmt::{Display, Formatter};

pub fn render_query(query: Query) -> String {
    format!("{};", query)
}

impl Display for Query {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "SELECT {}", RenderableSelect(self.select.as_slice()))?;
        writeln!(f, "FROM {}", self.from)?;

        for join in &self.joins {
            writeln!(f, "{}", join)?;
        }

        write!(f, "LIMIT {}", self.limit)?;

        Ok(())
    }
}

struct RenderableSelect<'a>(&'a [Sourced<Computation>]);

impl Display for RenderableSelect<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            return write!(f, "*");
        }

        if let Some((last, first)) = self.0.split_last() {
            for select in first {
                write!(f, "{}, ", select)?;
            }

            write!(f, "{}", last)?;
        }

        Ok(())
    }
}

impl Display for ExplicitJoin {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Self {
            join_type,
            target_table,
            source_arg,
            target_arg,
        } = self;

        write!(
            f,
            "{join_type} {target_table} ON {target_arg} = {source_arg}"
        )
    }
}

impl Display for Computation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Computation::SelectedColumn(column) => write!(f, "{}", column),
            Computation::FunctionCall(fn_call) => write!(f, "{}", fn_call),
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

impl Display for FunctionCall {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(", self.fn_name)?;

        let nr_params_with_comma_after = match self.params.len() {
            0 | 1 => 0,
            n => n - 1,
        };

        for param in self.params.iter().take(nr_params_with_comma_after) {
            // all params except the last one have a comma (,) after them
            write!(f, "{}, ", param)?;
        }

        // this is optional because some fn calls could take 0 params
        if let Some(param) = self.params.last() {
            // the last param must not have a comma after it
            write!(f, "{}", param)?;
        }

        write!(f, ")")
    }
}

impl Display for Limit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Limit::Implicit() => write!(f, "10"), // default
            Limit::RowCount(max_rows) => write!(f, "{}", max_rows),
            Limit::Range(range) => write!(f, "{}, {}", range.start, range.end),
        }
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

impl Display for JoinType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            JoinType::Left => write!(f, "LEFT JOIN"),
        }
    }
}

impl<T> Display for Sourced<T>
where
    T: Display + Clone,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.it)
    }
}
