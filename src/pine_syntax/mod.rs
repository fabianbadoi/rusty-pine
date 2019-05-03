pub mod ast;
mod pest;
mod pest_tree_translation;

pub use pest_tree_translation::PestPineParser;
use ast::PineNode;

use crate::error::SyntaxError;

pub trait PineParser {
    fn parse(self, input: &str) -> Result<PineNode, SyntaxError>;
}
