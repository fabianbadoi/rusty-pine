use crate::engine::query_builder::{
    ColumnName, Computation, DatabaseName, ExplicitJoin, FunctionCall, Query, Selectable,
    SelectedColumn, Table, TableName,
};
use crate::engine::rendering::OptionalClause;
use crate::engine::{
    BinaryConditionHolder, Comparison, ConditionHolder, JoinType, LiteralValueHolder,
    OrderDirection, OrderHolder, UnaryConditionHolder,
};
use crate::engine::{LimitHolder, Sourced};
use std::fmt::{Debug, Display, Formatter};

pub fn render_query(query: Query) -> String {
    format!("{};", query)
}

impl Display for Query {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "SELECT {}", RenderableSelect(self.select.as_slice()))?;
        writeln!(f, "FROM {}", self.from)?;

        // We reverse the order of joins so that the most recent pine is the first JOIN.
        // This is just something I find more natural.
        for join in self.joins.iter().rev() {
            writeln!(f, "{}", join)?;
        }

        write!(f, "{}", OptionalClause::filter(self.filters.as_slice()))?;
        write!(f, "{}", OptionalClause::order_by(self.orders.as_slice()))?;
        write!(f, "{}", OptionalClause::group_by(self.group_by.as_slice()))?;
        write!(f, "LIMIT {}", self.limit)?;

        Ok(())
    }
}

struct RenderableSelect<'a>(&'a [Sourced<Selectable>]);

impl Display for RenderableSelect<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some((last, first)) = self.0.split_last() {
            for select in first {
                write!(f, "{}, ", select)?;
            }

            write!(f, "{}", last)?;
        } else {
            write!(f, "*")?;
        }

        Ok(())
    }
}

impl<T> Display for OrderHolder<T>
where
    T: Display + Clone + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.selectable)?;

        if self.direction.it == OrderDirection::Descending {
            write!(f, " DESC")?;
        }

        Ok(())
    }
}

impl Display for ExplicitJoin {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Self {
            join_type,
            target_table,
            conditions,
        } = self;

        write!(f, "{join_type} {target_table} ON ")?;

        let mut condition_iterator = conditions.iter();

        if let Some(condition) = condition_iterator.next() {
            write!(f, "{condition}")?;
        }

        for condition in condition_iterator {
            write!(f, " AND {condition}")?;
        }

        Ok(())
    }
}

impl Display for Selectable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Selectable::Condition(condition) => write!(f, "{}", condition),
            Selectable::Computation(computation) => write!(f, "{}", computation),
        }
    }
}

impl Display for Computation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Computation::SelectedColumn(column) => write!(f, "{}", column),
            Computation::FunctionCall(fn_call) => write!(f, "{}", fn_call),
            Computation::Value(value) => write!(f, "{}", value),
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

impl<T> Display for ConditionHolder<T>
where
    T: Display + Debug + Clone,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConditionHolder::Unary(condition) => {
                write!(f, "{}", condition)
            }
            ConditionHolder::Binary(condition) => {
                write!(f, "{}", condition)
            }
        }
    }
}

impl<T> Display for BinaryConditionHolder<T>
where
    T: Display + Clone + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Self {
            left,
            comparison,
            right,
        } = self;

        write!(f, "{left} {comparison} {right}")
    }
}

impl<T> Display for UnaryConditionHolder<T>
where
    T: Display + Clone + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryConditionHolder::IsNull(computation) => {
                write!(f, "{computation} IS NULL")
            }
            UnaryConditionHolder::IsNotNull(computation) => {
                write!(f, "{computation} IS NOT NULL")
            }
        }
    }
}

impl<T> Display for LiteralValueHolder<T>
where
    T: AsRef<str>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            // We support numbers like this 1_000, but MySQL doesn't -> strip _ out
            LiteralValueHolder::Number(number) => write!(f, "{}", number.as_ref().replace('_', "")),
            LiteralValueHolder::String(string) => write!(f, "{}", string.as_ref()),
        }
    }
}

impl Display for Comparison {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let symbol = match self {
            Comparison::Equals => "=",
            Comparison::NotEquals => "!=",
            Comparison::GreaterThan => ">",
            Comparison::GreaterOrEqual => ">=",
            Comparison::LesserThan => "<",
            Comparison::LesserOrEqual => "<=",
        };

        write!(f, "{symbol}")
    }
}

impl<T> Display for LimitHolder<T>
where
    T: Display + Debug + Clone,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LimitHolder::Implicit() => write!(f, "10"), // default
            LimitHolder::RowCount(max_rows) => write!(f, "{}", max_rows),
            LimitHolder::Range { start, count } => write!(f, "{}, {}", start, count),
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
