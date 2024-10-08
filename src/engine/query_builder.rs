use crate::analyze;
use log::info;
use std::fmt::Debug;
use thiserror::Error;

use crate::analyze::{
    Column, ColumnName, DatabaseName, ForeignKey, Server, ServerParams, TableName,
};
use crate::engine::syntax::{Stage4ComputationInput, Stage4Query, TableInput};
use crate::engine::{
    BinaryConditionHolder, ConditionHolder, JoinType, LimitHolder, LiteralValueHolder, OrderHolder,
    SelectableHolder, Sourced, UnaryConditionHolder,
};
use sql_introspection::Introspective;

mod sql_introspection;
mod stage5;

pub fn build_query(input: Stage4Query<'_>, server: &Server) -> Result<Query, QueryBuildError> {
    info!("creating stage 5 builder");
    let builder = stage5::Stage5Builder::new(input, server);

    info!("starting stage 5 build");
    builder.try_build()
}

pub fn get_neighbors(
    for_table: Sourced<TableInput>,
    server: &Server,
) -> Result<Vec<ForeignKey>, QueryBuildError> {
    info!("showing neighbors for '{}'", for_table.it.table.it.name);
    let neighboring_tables = server.neighbors(for_table)?;

    Ok(neighboring_tables)
}

pub fn get_columns<'a>(
    for_table: Sourced<TableInput>,
    server: &'a Server,
) -> Result<&'a [Column], QueryBuildError> {
    server.columns(for_table)
}

#[derive(Error, Debug, Clone)]
pub enum QueryBuildError {
    InvalidPostgresConfig,
    DefaultDatabaseNotFound(ServerParams),
    DatabaseNotFound(Sourced<DatabaseName>),
    TableNotFound(Sourced<analyze::TableName>),
    InvalidForeignKey {
        from: Sourced<analyze::TableName>,
        to: Sourced<analyze::TableName>,
    },
    JoinNotFound {
        from: Sourced<analyze::TableName>,
        to: Sourced<analyze::TableName>,
    },
    InvalidImplicitIdCondition(
        Sourced<analyze::TableName>,
        analyze::Key,
        Sourced<LiteralValue>,
    ),
}

#[derive(Debug)]
pub struct Query {
    pub from: Sourced<Table>,
    pub filters: Vec<Sourced<Condition>>,
    pub joins: Vec<Sourced<ExplicitJoin>>,
    pub select: Vec<Sourced<Selectable>>,
    pub orders: Vec<Sourced<OrderHolder<Selectable>>>,
    pub group_by: Vec<Sourced<Selectable>>,
    pub limit: Sourced<LimitHolder<LiteralValue>>,
}

pub type Selectable = SelectableHolder<Condition, Computation>;
pub type Condition = ConditionHolder<Computation>;
pub type BinaryCondition = BinaryConditionHolder<Computation>;
pub type UnaryCondition = UnaryConditionHolder<Computation>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Table {
    pub name: Sourced<TableName>,
    pub db: Option<Sourced<DatabaseName>>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Computation {
    SelectedColumn(Sourced<SelectedColumn>),
    FunctionCall(Sourced<FunctionCall>),
    Value(Sourced<LiteralValue>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FunctionCall {
    pub fn_name: Sourced<String>,
    pub params: Vec<Sourced<Computation>>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SelectedColumn {
    pub table: Option<Sourced<Table>>,
    pub column: Sourced<ColumnName>,
}

#[derive(Debug, Clone)]
pub struct ExplicitJoin {
    pub join_type: Sourced<JoinType>,
    /// The table to join to.
    pub target_table: Sourced<Table>,
    pub conditions: Vec<Sourced<Condition>>,
}

pub type LiteralValue = LiteralValueHolder<String>;

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
            Stage4ComputationInput::Value(value) => Computation::Value(value.into()),
        }
    }
}
