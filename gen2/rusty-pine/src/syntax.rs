mod stage1;
mod stage2;
mod stage3;
mod stage4;

#[derive(Clone, Copy)]
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
