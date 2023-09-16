use crate::engine::syntax::{Position, Stage4ComputationInput, Stage4Rep};
use std::ops::Range;

mod stage5;

pub fn build_query(input: Stage4Rep<'_>) -> Query {
    stage5::Stage5Builder {}.try_build(input).unwrap()
}

#[derive(Debug)]
pub struct Query {
    pub input: String,
    pub from: Sourced<Table>,
    pub select: Vec<Sourced<Computation>>,
    pub limit: Sourced<Limit>,
}

#[derive(Debug)]
pub struct Table {
    pub name: Sourced<TableName>,
    pub db: Option<Sourced<DatabaseName>>,
}

#[derive(Debug)]
pub enum Computation {
    SelectedColumn(SelectedColumn),
    FunctionCall(FunctionCall),
}

#[derive(Debug)]
pub struct FunctionCall {
    pub fn_name: Sourced<String>,
    pub params: Vec<Sourced<Computation>>,
}

#[derive(Debug)]
pub struct SelectedColumn {
    pub table: Option<Sourced<Table>>,
    pub column: Sourced<ColumnName>,
}

#[derive(Debug)]
pub enum Limit {
    Implicit(),
    RowCountLimit(usize),
    RangeLimit(Range<usize>),
}

#[derive(Debug)]
pub struct ColumnName(pub String);

#[derive(Debug)]
pub struct TableName(pub String);

#[derive(Debug)]
pub struct DatabaseName(pub String);

/// These functions here are special because they *omit the table name*.
///
/// The idea behind "from_singly_selected" is that if there is only one table involved, we can
/// simplify the rendered query to implicitly use the select in the FROM clause.
///
/// If we were to use stage4_computation.into(), we would get fully qualified table names.
impl Computation {
    fn without_table_name(input: &Stage4ComputationInput) -> Sourced<Self> {
        match input {
            Stage4ComputationInput::Column(column) => Sourced {
                it: Computation::SelectedColumn(SelectedColumn {
                    column: column.column.to_sourced(),
                    table: None,
                }),
                source: (&column.position).into(),
            },
            Stage4ComputationInput::FunctionCall(fn_call) => Sourced {
                it: Computation::FunctionCall(FunctionCall {
                    fn_name: fn_call.fn_name.to_sourced(),
                    params: fn_call
                        .params
                        .iter()
                        .map(Computation::without_table_name)
                        .collect(),
                }),
                source: (&fn_call.position).into(),
            },
        }
    }
}

#[derive(Debug)]
pub enum Source {
    Implicit,
    Input(Position),
}

#[derive(Debug)]
pub struct Sourced<T: Sized> {
    pub it: T,
    pub source: Source,
}

trait ToSource<D> {
    fn as_it(&self) -> D;
    fn as_source(&self) -> Source;

    fn to_sourced(self) -> Sourced<D>
    where
        Self: Sized,
    {
        let it = self.as_it();
        let source = self.as_source();

        Sourced { it, source }
    }
}

impl<T, D> ToSource<D> for T
where
    for<'a> &'a T: Into<D>,
    for<'a> &'a T: Into<Position>,
{
    fn as_it(&self) -> D {
        self.into()
    }

    fn as_source(&self) -> Source {
        Source::Input(self.into())
    }
}

impl From<&Position> for Source {
    fn from(value: &Position) -> Self {
        Source::Input(*value)
    }
}

impl<T> From<&T> for ColumnName
where
    for<'a> &'a T: Into<String>,
{
    fn from(value: &T) -> Self {
        Self(value.into())
    }
}

impl<T> From<&T> for TableName
where
    for<'a> &'a T: Into<String>,
{
    fn from(value: &T) -> Self {
        Self(value.into())
    }
}

impl<T> From<&T> for DatabaseName
where
    for<'a> &'a T: Into<String>,
{
    fn from(value: &T) -> Self {
        Self(value.into())
    }
}
