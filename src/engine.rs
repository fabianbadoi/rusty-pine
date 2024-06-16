mod query_builder;
mod rendering;
/// Provides helpful data from the database
pub mod sql;
mod syntax;

#[cfg(test)]
mod tests;

use crate::analyze::Server;
pub use syntax::Rule;

use crate::engine::query_builder::{build_query, get_columns, get_neighbors};
use crate::engine::rendering::{render_columns, render_neighbors, render_query};
use crate::engine::syntax::{parse_to_stage4, Stage4Rep};

pub use query_builder::QueryBuildError;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Range;
use thiserror::Error;

pub fn render(input: &str, server: &Server) -> Result<String, crate::error::Error> {
    let pine = parse_to_stage4(input)?;

    match pine {
        Stage4Rep::Query(query) => {
            let query = map_err(input, build_query(query, server))?;

            Ok(render_query(query))
        }
        Stage4Rep::ShowNeighbors(for_table) => {
            let neighbors = map_err(input, get_neighbors(for_table, server))?;

            Ok(render_neighbors(neighbors))
        }
        Stage4Rep::ShowColumns(for_table) => {
            let columns = map_err(input, get_columns(for_table, server))?;

            Ok(render_columns(for_table.it, columns))
        }
    }
}

#[derive(Debug, Error)]
pub struct RenderingError {
    input: String,
    build_error: QueryBuildError,
}

// TODO move
impl Display for RenderingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{input}\n{error}",
            input = self.input,
            error = self.build_error,
        )
    }
}

fn map_err<T>(input: &str, result: Result<T, QueryBuildError>) -> Result<T, RenderingError> {
    result.map_err(|build_error| RenderingError {
        input: input.to_string(),
        build_error,
    })
}

#[derive(Debug, Clone, Copy, Eq)]
pub enum Source {
    /// Things like default values are implicit.
    Implicit,
    /// These are things that we deduced by analyzing the database structure.
    Introspection,
    /// We found this in the input provided by the user.
    Input(Position),
}

impl PartialEq for Source {
    fn eq(&self, _: &Self) -> bool {
        // Doing this makes comparing things deduplicating items much easier.
        true
    }
}

/// Holds a reference to where we got something from.
///
/// I use this to help print better error messages.
/// ```text
/// humans | friends]
///                 ^-- Sourced<':', &input pos 15>
///                 \- I can point to the invalid character because of Sourced<>
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sourced<T: Sized + Clone> {
    pub it: T,
    pub source: Source,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SelectableHolder<Cond, Comp>
where
    Cond: Clone,
    Comp: Clone,
{
    Condition(Sourced<Cond>),
    Computation(Sourced<Comp>),
}

#[derive(Debug, Clone)]
pub struct JoinHolder<T, C>
where
    T: Clone,
    C: Clone,
{
    pub join_type: Sourced<JoinType>,
    /// The table to join to.
    pub target_table: Sourced<T>,
    pub conditions: JoinConditions<C>,
}

#[derive(Debug, Clone)]
pub enum JoinConditions<T>
where
    T: Clone,
{
    Auto,
    Explicit(Vec<Sourced<T>>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ConditionHolder<T>
where
    T: Clone + Debug,
{
    Unary(Sourced<UnaryConditionHolder<T>>),
    Binary(Sourced<BinaryConditionHolder<T>>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BinaryConditionHolder<T>
where
    T: Clone + Debug,
{
    pub left: Sourced<T>,
    pub comparison: Sourced<Comparison>,
    pub right: Sourced<T>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum UnaryConditionHolder<T>
where
    T: Clone + Debug,
{
    IsNull(Sourced<T>),
    IsNotNull(Sourced<T>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Comparison {
    Equals,
    NotEquals,
    GreaterThan,
    GreaterOrEqual,
    LesserThan,
    LesserOrEqual,
}

#[derive(Debug, Clone)]
pub enum LimitHolder<T>
where
    T: Debug + Clone,
{
    Implicit(),
    RowCount(Sourced<T>),
    Range {
        start: Sourced<T>,
        count: Sourced<T>,
    },
}

#[derive(Debug, Clone)]
pub struct OrderHolder<T>
where
    T: Debug + Clone,
{
    selectable: Sourced<T>,
    direction: Sourced<OrderDirection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Copy, Clone)]
pub enum JoinType {
    Left,
    // TODO
    // Right,
    // Inner,
}

/// A literal value like 1 or "kitten".
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LiteralValueHolder<T> {
    Number(T),
    String(T),
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Position {
    // pub input: &'a str,
    pub start: usize,
    pub end: usize,
}

impl<T: Sized + Clone> Sourced<T> {
    /// Something from the source input the user provided.
    pub fn from_input<P>(position: P, it: T) -> Self
    where
        P: Into<Position>,
    {
        Sourced {
            it,
            source: Source::Input(position.into()),
        }
    }

    /// Something that is implicit, this could be default values, for example.
    pub fn implicit(it: T) -> Self {
        Sourced {
            it,
            source: Source::Implicit,
        }
    }

    pub fn from_source(source: Source, it: T) -> Sourced<T> {
        Sourced { it, source }
    }

    pub fn from_introspection(it: T) -> Sourced<T> {
        Sourced {
            it,
            source: Source::Introspection,
        }
    }

    pub fn into<D>(self) -> Sourced<D>
    where
        D: Clone + Debug + From<T>,
    {
        let source = self.source;
        let it = self.it.into();

        Sourced { it, source }
    }

    pub fn map<D, F>(self, mapper: F) -> Sourced<D>
    where
        F: FnOnce(T) -> D,
        D: Sized + Clone,
    {
        Sourced {
            it: mapper(self.it),
            source: self.source,
        }
    }

    pub fn map_ref<D, F>(&self, mapper: F) -> Sourced<D>
    where
        F: FnOnce(&T) -> D,
        D: Sized + Clone,
    {
        Sourced {
            it: mapper(&self.it),
            source: self.source,
        }
    }
}

impl<T: Sized + Clone, E: Clone> Sourced<Result<T, E>> {
    pub fn unwrap_result(self) -> Result<Sourced<T>, E> {
        let Sourced { it, source } = self;

        match it {
            Ok(it) => Ok(Sourced::from_source(source, it)),
            Err(error) => Err(error),
        }
    }
}

impl<T> Copy for Sourced<T> where T: Copy {}

impl From<&Position> for Source {
    fn from(value: &Position) -> Self {
        Source::Input(*value)
    }
}

impl From<Range<usize>> for Position {
    fn from(range: Range<usize>) -> Self {
        Position {
            start: range.start,
            end: range.end,
        }
    }
}

impl<T> Copy for LiteralValueHolder<T> where T: Copy {}

impl<T> LiteralValueHolder<T> {
    fn into<D>(self) -> LiteralValueHolder<D>
    where
        D: From<T>,
    {
        match self {
            LiteralValueHolder::Number(nr) => LiteralValueHolder::Number(nr.into()),
            LiteralValueHolder::String(str) => LiteralValueHolder::String(str.into()),
        }
    }
}

#[cfg(test)]
impl PartialEq<Source> for Position {
    fn eq(&self, other: &Source) -> bool {
        match other {
            Source::Input(position) => position == self,
            _ => false,
        }
    }
}

#[cfg(test)]
impl PartialEq<Position> for Range<usize> {
    fn eq(&self, other: &Position) -> bool {
        self.start == other.start && self.end == other.end
    }
}
