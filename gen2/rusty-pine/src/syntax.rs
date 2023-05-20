use std::ops::Range;

mod stage1;
mod stage2;
mod stage3;
mod stage4;

use crate::syntax::stage1::parse_stage1;
use crate::syntax::stage2::Stage2Rep;
use crate::syntax::stage3::Stage3Rep;
pub use stage1::Rule;
pub use stage4::{Stage4ColumnInput, Stage4Rep};

pub fn parse_to_stage4(input: &str) -> Result<Stage4Rep, crate::Error> {
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

// TODO impl display and debug
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

pub struct Positioned<T> {
    node: T,
    position: Position,
}
