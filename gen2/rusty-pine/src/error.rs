use mysql::Error as MySqlError;
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
pub enum ErrorKind {
    /// Errors originating from the Pest library
    #[error("Invalid syntax, failed to parse:\n{0}")]
    SyntaxError(#[from] PestError),
    /// Errors originating from the MySQL library
    #[error("Error trying to query database:\n{0}")]
    MySqlError(#[from] MySqlError),

    #[error("Internal error:\n{0}")]
    InternalError(#[from] InternalError),
}

pub type PestError = pest::error::Error<crate::engine::Rule>;

#[derive(Error, Debug)]
pub struct InternalError(pub String);

impl Display for InternalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
