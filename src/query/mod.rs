mod naive_builder;
mod query;

use crate::error::ParseError;
use crate::pine_syntax::ast::PineNode;

pub use naive_builder::NaiveBuilder;
pub use query::*;

pub type BuildResult = Result<Query, ParseError>;

pub trait QueryBuilder {
    fn build(self, pine: &PineNode) -> BuildResult;
}
