extern crate pest;
#[macro_use]
extern crate pest_derive;

mod pine_syntax;
mod pine_translator;
mod sql;

#[derive(Debug)]
pub struct PineError {
    message: String,
    position: Position,
}

#[derive(Copy, Clone, Debug)]
pub struct Position {
    pub start: usize,
    pub end: usize,
}

impl Default for Position {
    fn default() -> Self {
        Position { start: 0, end: 0 }
    }
}

fn main() {}
