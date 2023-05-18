mod stage1;
mod stage2;

struct SqlIdentifier<'a> {
    pub name: &'a str,
    pub position: Position<'a>,
}

// TODO impl display and debug
struct Position<'a> {
    // pub input: &'a str,
    pub start: usize,
    pub end: usize,
}
