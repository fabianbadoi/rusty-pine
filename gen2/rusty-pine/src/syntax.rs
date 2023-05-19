use std::ops::Range;

mod stage1;
mod stage2;
mod stage3;
mod stage4;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum OptionalInput<T> {
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

impl<T> Default for OptionalInput<T> {
    fn default() -> Self {
        return OptionalInput::Implicit;
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
