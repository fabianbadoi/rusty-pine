mod query_builder;
mod rendering;
/// Provides helpful data from the database
pub mod sql;
mod syntax;

#[cfg(test)]
mod tests;

use crate::analyze::Server;
pub use syntax::Rule;

use crate::engine::query_builder::build_query;
use crate::engine::rendering::render_query;
use crate::engine::syntax::parse_to_stage4;

pub use query_builder::QueryBuildError;
use std::fmt::Debug;
use std::ops::Range;

pub fn render(input: &str, server: &Server) -> Result<String, crate::error::Error> {
    let pine = parse_to_stage4(input)?;
    let query = build_query(pine, server);

    Ok(render_query(query?))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Source {
    Implicit,
    Input(Position),
}

/// Holds a reference to where we got something from.
///
/// I use this to help print better error messages.
/// ```text
/// humans | friends]
///                 ^-- Sourced<':', &input pos 15>
///                 \- I can point to the invalid character because of Sourced<>
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Sourced<T: Sized + Clone> {
    pub it: T,
    pub source: Source,
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

impl<T> Copy for Sourced<T> where T: Copy {}

impl From<&Position> for Source {
    fn from(value: &Position) -> Self {
        Source::Input(*value)
    }
}

#[derive(Debug, Clone)]
pub struct ExplicitJoinHolder<T, C>
where
    T: Clone,
    C: Clone,
{
    pub join_type: Sourced<JoinType>,
    /// The table to join to.
    pub target_table: Sourced<T>,
    /// The "source" of the join's ON query.
    ///
    /// All column names will default to referring to the previous table.
    pub source_arg: Sourced<C>,
    /// The "target" of the join's ON query.
    ///
    /// All column names will default to referring to the target table.
    pub target_arg: Sourced<C>,
}

#[derive(Debug, Clone)]
pub enum Limit {
    Implicit(),
    RowCount(usize),
    Range(Range<usize>),
}

#[derive(Debug, Copy, Clone)]
pub enum JoinType {
    Left,
    // TODO
    // Right,
    // Inner,
}

/// A literal value like 1 or "kitten".
#[derive(Debug, Clone)]
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

impl From<Range<usize>> for Position {
    fn from(range: Range<usize>) -> Self {
        Position {
            start: range.start,
            end: range.end,
        }
    }
}

impl<T> Copy for LiteralValueHolder<T> where T: Copy {}

#[cfg(test)]
impl PartialEq<Source> for Position {
    fn eq(&self, other: &Source) -> bool {
        match other {
            Source::Implicit => false,
            Source::Input(position) => position == self,
        }
    }
}

#[cfg(test)]
impl PartialEq<Position> for Range<usize> {
    fn eq(&self, other: &Position) -> bool {
        self.start == other.start && self.end == other.end
    }
}
