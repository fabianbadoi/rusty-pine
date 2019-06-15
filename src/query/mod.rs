mod naive_builder;
mod query;

use crate::error::PineError;
use crate::pine_syntax::ast::PineNode;

pub use naive_builder::NaiveBuilder;
pub use query::*;

pub type BuildResult = Result<Query, PineError>;

pub trait QueryBuilder {
    fn build(self, pine: &PineNode) -> BuildResult;
}
