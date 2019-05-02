mod ast;
mod pest;
mod pest_tree_translation;
mod pine_to_query;
mod query_parser;

use crate::sql::Query;
pub use query_parser::{Parser, QueryParser};
use std::result::Result as StdResult;
use crate::ParseError;

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

pub type Result = StdResult<Query, PineError>;


impl From<PineError> for ParseError {
    fn from(other: PineError) -> ParseError {
        ParseError { message: format!("{}", other.message) }
    }
}
