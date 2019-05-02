pub mod ast;
mod pest;
mod pest_tree_translation;

pub use pest_tree_translation::PestPineParser;
use ast::PineNode;

use crate::ParseError;

pub trait PineParser {
    fn parse(self, input: &str) -> Result<PineNode, PineError>;
}

#[derive(Debug)]
pub struct PineError {
    pub message: String,
    pub position: Position,
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

impl From<PineError> for ParseError {
    fn from(other: PineError) -> ParseError {
        ParseError {
            message: format!("{}", other.message),
        }
    }
}
