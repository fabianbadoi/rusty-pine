mod stage1;
mod stage2;
mod stage3;

struct SqlIdentifier<'a> {
    pub name: &'a str,
    pub position: Position,
}

// TODO impl display and debug
#[derive(PartialEq, Eq, Debug)]
pub struct Position {
    // pub input: &'a str,
    pub start: usize,
    pub end: usize,
}
