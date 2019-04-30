mod ast;
mod pest;
mod pest_tree_translation;
mod pine_to_query;

use crate::sql::Query;
pub use ast::Position;
use std::result::Result as StdResult;

#[derive(Debug)]
pub struct PineParseError {
    message: String,
    position: Position,
}

pub type Result = StdResult<Query, PineParseError>;
