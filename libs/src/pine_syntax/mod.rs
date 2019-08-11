pub mod ast;
mod pest;
mod pest_tree_translation;

use ast::{Node, Pine};
pub use pest_tree_translation::PestPineParser;

use crate::error::SyntaxError;

pub trait PineParser {
    fn parse(self, input: &str) -> Result<Node<Pine>, SyntaxError>;
}
