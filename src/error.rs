mod pest;

use crate::engine::sql::DbStructureParseError;
use crate::engine::RenderingError;
use crate::error::pest::WrappedPestError;
use colored::Colorize;
use mysql::Error as MySqlError;
use std::env::VarError;
use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Debug, Error)]
#[error(transparent)]
pub struct Error(Box<ErrorKind>);

impl<E> From<E> for Error
where
    ErrorKind: From<E>,
{
    fn from(value: E) -> Self {
        Error(Box::new(value.into()))
    }
}

#[derive(Debug, Error)]
#[allow(clippy::enum_variant_names)]
pub enum ErrorKind {
    /// Errors originating from the Pest library
    #[error("{}\n{0}", "Invalid syntax, failed to parse".bold())]
    SyntaxError(#[from] WrappedPestError),
    /// Errors originating from the MySQL library
    #[error("{}\n{0}", "Error trying to query database".bold())]
    MySqlError(#[from] MySqlError),
    #[error("{}\n{0}", "Internal error".bold())]
    InternalError(#[from] InternalError),
    #[error("{}\n{0}", "Error parsing database structure".bold())]
    DbStructureParseError(#[from] DbStructureParseError),
    #[error("{}\n{0}", "Error rendering query".bold())]
    QueryBuildingError(#[from] RenderingError),
    #[error("{}\n{0}", "Could not find environment variable".bold())]
    EnvVarError(#[from] VarError),
    #[error("{}\n{0}", "IO error".bold())]
    IoError(#[from] std::io::Error),
    #[error("{}\n{0}", "JSON error".bold())]
    JsonError(#[from] serde_json::Error),
    #[error("{}\n{0}", "Error reading data from stdin".bold())]
    DialogueError(#[from] dialoguer::Error),
}

#[derive(Error, Debug)]
pub struct InternalError(pub String);

impl Display for InternalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error {
    pub fn into_inner(self) -> ErrorKind {
        *self.0
    }
}
