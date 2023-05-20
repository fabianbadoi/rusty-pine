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
    #[error("Invalid syntax, failed to parse:\n{0}")]
    SyntaxError(#[from] PestError),
}

pub type PestError = pest::error::Error<crate::syntax::Rule>;
