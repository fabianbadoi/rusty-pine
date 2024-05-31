//! Pine language input parsing
//!
//! The pine language looks like this:
//! ```ignore
//!     some_table | another_table_to_be_joined | s: column_name count(1) | g: column_name
//! ```
//! `s:` is shorthand for `select:` and `g:` is shorthand for `group:`.
//!
//!
//! Why are there so many stages?
//! -----------------------------
//!
//! There are exactly as many stages as needed.
//! Jokes aside, if you don't split this parsing operation into these multiple stages, then you end
//! up with over-complicated code.
//!
//! Each stage is slightly different, and the nature of the processing varies. Some stages just deal
//! with the straight input, other's have internal history.

/// Uses Pest to parse input strings.
mod stage1;

/// Takes Pest's output and transforms it into something a bit nicer.
mod stage2;

/// Each "pine" has implicit data from the previous ones. This steps injects that data.
mod stage3;

/// Produces a structure that is a little bit easier to use in the future.
mod stage4;

pub use stage1::Rule;
pub use stage3::Stage3ExplicitJoin;
pub use stage4::{
    Stage4ColumnInput, Stage4ComputationInput, Stage4FunctionCall, Stage4LiteralValue, Stage4Rep,
};

use crate::engine::syntax::stage1::parse_stage1;
use crate::engine::syntax::stage2::Stage2Rep;
use crate::engine::syntax::stage3::Stage3Rep;
use crate::engine::Sourced;

pub fn parse_to_stage4(input: &str) -> Result<Stage4Rep, crate::error::Error> {
    let stage1 = parse_stage1(input)?;
    let stage2: Stage2Rep = stage1.into();
    let stage3: Stage3Rep = stage2.into();

    Ok(stage3.into())
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum OptionalInput<T> {
    #[default]
    Implicit,
    Specified(T),
}

impl<T> OptionalInput<T> {
    pub fn or<Alt>(self, alternative: Alt) -> T
    where
        Alt: Into<T>,
    {
        match self {
            OptionalInput::Implicit => alternative.into(),
            OptionalInput::Specified(inner) => inner,
        }
    }

    #[cfg(test)]
    pub fn unwrap(&self) -> &T {
        match self {
            OptionalInput::Implicit => panic!("You done fucked up!"),
            OptionalInput::Specified(value) => value,
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub struct TableInput<'a> {
    pub database: OptionalInput<Sourced<SqlIdentifierInput<'a>>>,
    pub table: Sourced<SqlIdentifierInput<'a>>,
}

pub type Stage2LiteralValue<'a> = Stage4LiteralValue<'a>;

#[derive(Clone, Debug)]
pub enum Computation<'a> {
    Column(Sourced<ColumnInput<'a>>),
    FunctionCall(Sourced<FunctionCall<'a>>),
    Value(Sourced<Stage2LiteralValue<'a>>),
}

#[derive(Clone, Debug)]
pub struct ColumnInput<'a> {
    pub table: OptionalInput<Sourced<TableInput<'a>>>, // we always know it because of SYNTAX
    pub column: Sourced<SqlIdentifierInput<'a>>,
}

#[derive(Clone, Debug)]
pub struct FunctionCall<'a> {
    pub fn_name: Sourced<SqlIdentifierInput<'a>>,
    /// Params for the function call.
    ///
    /// A Computation can contain a FunctionCall can contain multiple Computations. This means
    /// we're dealing with a recursive data type.
    /// Such types cannot be build without Vec<> or Box<>. Because they contain themselves, their
    /// memory size would be infinite (Computation.computation.computation.computation...).
    /// Of course, in reality the function calls stop, and it's not really infinite. But because of
    /// this limitation, aka not knowing how deep the structure goes, we have to allocate the params
    /// on the heap.
    ///
    /// If we only supported one param, it would need to be Box<>ed, which might look weird to
    /// someone unfamiliar with the problem.
    /// We support multiple params, so we already need to use Vec, which is fortunate.
    pub params: Vec<Sourced<Computation<'a>>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SqlIdentifierInput<'a> {
    pub name: &'a str,
}

impl AsRef<str> for SqlIdentifierInput<'_> {
    fn as_ref(&self) -> &str {
        self.name
    }
}

impl From<SqlIdentifierInput<'_>> for String {
    fn from(value: SqlIdentifierInput<'_>) -> Self {
        value.name.to_string()
    }
}

#[cfg(test)]
impl PartialEq<str> for SqlIdentifierInput<'_> {
    fn eq(&self, other: &str) -> bool {
        self.name == other
    }
}
