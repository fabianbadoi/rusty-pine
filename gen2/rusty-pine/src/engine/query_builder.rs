use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

use crate::analyze::Server;
use crate::engine::syntax::{Stage4ComputationInput, Stage4Rep};
use crate::engine::{JoinType, Limit, Sourced};

mod stage5;

pub fn build_query(input: Stage4Rep<'_>, server: &Server) -> Result<Query, crate::Error> {
    let builder = stage5::Stage5Builder::new(input, server);

    Ok(builder.try_build()?)
}

#[derive(Error, Debug)]
pub struct QueryBuildError {}

#[derive(Debug)]
pub struct Query {
    pub input: String,
    pub from: Sourced<Table>,
    pub joins: Vec<Sourced<ExplicitJoin>>,
    pub select: Vec<Sourced<Computation>>,
    pub limit: Sourced<Limit>,
}

#[derive(Debug, Clone)]
pub struct Table {
    pub name: Sourced<TableName>,
    pub db: Option<Sourced<DatabaseName>>,
}

#[derive(Debug, Clone)]
pub enum Computation {
    SelectedColumn(Sourced<SelectedColumn>),
    FunctionCall(Sourced<FunctionCall>),
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub fn_name: Sourced<String>,
    pub params: Vec<Sourced<Computation>>,
}

#[derive(Debug, Clone)]
pub struct SelectedColumn {
    pub table: Option<Sourced<Table>>,
    pub column: Sourced<ColumnName>,
}

#[derive(Debug, Clone)]
pub struct ExplicitJoin {
    pub join_type: Sourced<JoinType>,
    /// The table to join to.
    pub target_table: Sourced<Table>,
    /// The "source" of the join's ON query.
    ///
    /// All column names will default to referring to the previous table.
    pub source_arg: Sourced<Computation>,
    /// The "target" of the join's ON query.
    ///
    /// All column names will default to referring to the target table.
    pub target_arg: Sourced<Computation>,
}

#[derive(Debug, Clone)]
pub struct ColumnName(pub String);

#[derive(Debug, Clone)]
pub struct TableName(pub String);

#[derive(Debug, Clone)]
pub struct DatabaseName(pub String);

impl Display for QueryBuildError {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

/// These functions here are special because they *omit the table name*.
///
/// The idea behind "from_singly_selected" is that if there is only one table involved, we can
/// simplify the rendered query to implicitly use the select in the FROM clause.
///
/// If we were to use stage4_computation.into(), we would get fully qualified table names.
impl Computation {
    fn without_table_name(input: Stage4ComputationInput) -> Self {
        match input {
            Stage4ComputationInput::Column(column) => {
                Computation::SelectedColumn(column.map(|column| SelectedColumn {
                    column: column.column.into(),
                    table: None,
                }))
            }
            Stage4ComputationInput::FunctionCall(fn_call) => {
                Computation::FunctionCall(fn_call.map(|fn_call| {
                    FunctionCall {
                        fn_name: fn_call.clone().fn_name.into(),
                        params: fn_call
                            .params
                            .into_iter()
                            .map(|param| param.map(|param| Computation::without_table_name(param)))
                            .collect(),
                    }
                }))
            }
        }
    }
}

impl<T> From<T> for ColumnName
where
    T: AsRef<str>,
{
    fn from(value: T) -> Self {
        ColumnName(value.as_ref().to_string())
    }
}

impl<T> From<T> for TableName
where
    T: AsRef<str>,
{
    fn from(value: T) -> Self {
        TableName(value.as_ref().to_string())
    }
}
