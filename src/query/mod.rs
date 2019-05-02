mod naive_builder;
mod query;

use crate::pine_syntax::ast::PineNode;
use crate::ParseError;

pub use query::*;
pub use naive_builder::NaiveBuilder;

pub type BuildResult = Result<Query, ParseError>;

pub trait QueryBuilder {
    fn build(self, pine: &PineNode) -> BuildResult;
}
