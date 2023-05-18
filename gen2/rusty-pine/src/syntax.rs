mod stage1;
mod stage2;
mod stage3;
mod stage4;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum OptionalInput<T> {
    Implicit,
    Specified(T),
}

#[derive(Clone, Copy)]
pub struct TableInput<'a> {
    pub database: OptionalInput<SqlIdentifierInput<'a>>,
    pub table: SqlIdentifierInput<'a>,
    pub position: Position,
}

#[derive(Clone, Copy, Debug)]
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
