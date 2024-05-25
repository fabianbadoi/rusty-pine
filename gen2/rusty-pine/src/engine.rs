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
use crate::engine::syntax::{parse_to_stage4, Position};

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

impl Source {
    // TODO delete?
    pub fn holding<T: Debug + Clone>(self, it: T) -> Sourced<T> {
        Sourced { it, source: self }
    }
}

#[derive(Debug, Clone)]
pub enum Limit {
    Implicit(),
    RowCountLimit(usize),
    RangeLimit(Range<usize>),
}
