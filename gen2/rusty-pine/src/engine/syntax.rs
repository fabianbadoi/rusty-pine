//! Pine language input parsing
//!
//! The pine language looks like this:
//! ```
//!     some_table | another_table_to_be_joined | s: column_name count(1) | g: column_name
//! ```
//! `s:` is shorthand for `select:` and `g:` is shorthand for `group:`.
//!
//!
//! Why are there so many stages?
//! -----------------------------
//!
//! There are exactly as many stages as needed.
//! Jokes aside, if you don't split this parsing operation into these multiple stages, then you end
//! up with over-complicated code.
//!
//! Each stage is slightly different, and the nature of the processing varies. Some stages just deal
//! with the straight input, other's have internal history.

/// Uses Pest to parse input strings.
mod stage1;

/// Takes Pest's output and transforms it into something a bit nicer.
mod stage2;

/// Each "pine" has implicit data from the previous ones. This steps injects that data.
mod stage3;

/// Produces a structure that is a little bit easier to use in the future.
mod stage4;

pub use stage1::Rule;
pub use stage4::{Stage4ColumnInput, Stage4LimitInput, Stage4Rep};

use crate::engine::syntax::stage1::parse_stage1;
use crate::engine::syntax::stage2::Stage2Rep;
use crate::engine::syntax::stage3::Stage3Rep;
use std::ops::Range;

pub fn parse_to_stage4(input: &str) -> Result<Stage4Rep, crate::error::Error> {
    let stage1 = parse_stage1(input)?;
    let stage2: Stage2Rep = stage1.into();
    let stage3: Stage3Rep = stage2.into();

    Ok(stage3.into())
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Default)]
pub enum OptionalInput<T> {
    #[default]
    Implicit,
    Specified(T),
}

impl<T> OptionalInput<T> {
    pub fn or<Alt>(self, alternative: Alt) -> T
    where
        Alt: Into<T>,
    {
        match self {
            OptionalInput::Implicit => alternative.into(),
            OptionalInput::Specified(inner) => inner,
        }
    }

    #[cfg(test)]
    pub fn unwrap(&self) -> &T {
        match self {
            OptionalInput::Implicit => panic!("You done fucked up!"),
            OptionalInput::Specified(value) => value,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableInput<'a> {
    pub database: OptionalInput<SqlIdentifierInput<'a>>,
    pub table: SqlIdentifierInput<'a>,
    pub position: Position,
}

#[derive(Clone, Copy, Debug)]
pub struct ColumnInput<'a> {
    pub table: OptionalInput<TableInput<'a>>, // we always know it because of SYNTAX
    pub column: SqlIdentifierInput<'a>,
    pub position: Position,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SqlIdentifierInput<'a> {
    pub name: &'a str,
    pub position: Position,
}

impl From<&SqlIdentifierInput<'_>> for String {
    fn from(value: &SqlIdentifierInput<'_>) -> Self {
        value.name.to_owned()
    }
}

impl From<&SqlIdentifierInput<'_>> for Position {
    fn from(value: &SqlIdentifierInput<'_>) -> Self {
        value.position
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Position {
    // pub input: &'a str,
    pub start: usize,
    pub end: usize,
}

#[cfg(test)]
impl PartialEq<Position> for Range<usize> {
    fn eq(&self, other: &Position) -> bool {
        self.start == other.start && self.end == other.end
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

impl Position {
    pub fn holding<T>(self, node: T) -> Positioned<T> {
        Positioned {
            node,
            position: self,
        }
    }
}

#[derive(Debug)]
pub struct Positioned<T> {
    node: T,
    position: Position,
}
