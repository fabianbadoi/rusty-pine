mod naive_builder;
mod structure;

use crate::error::PineError;
use crate::pine_syntax::ast::{Node, Pine};

pub use naive_builder::NaiveBuilder;
pub use structure::*;

pub type BuildResult = Result<Renderable, PineError>;

pub trait QueryBuilder {
    fn build(self, pine: &Node<Pine>) -> BuildResult;
}
