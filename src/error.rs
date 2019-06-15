use std::fmt::{Display, Formatter, Result as FmtResult};
use std::error::Error;

#[derive(Debug)]
pub struct PineError {
    message: String,
    cause: Option<Box<dyn Error>>,
}

#[derive(Debug)]
pub enum SyntaxError {
    Positioned {
        message: String,
        position: Position,
        input: String,
    },
    Detailed(String),
}

#[derive(Copy, Clone, Debug)]
pub struct Position {
    pub start: usize,
    pub end: usize,
}

impl Default for Position {
    fn default() -> Self {
        Position { start: 0, end: 0 }
    }
}

impl SyntaxError {
    fn message(&self) -> &str {
        match self {
            SyntaxError::Detailed(message) => message,
            SyntaxError::Positioned {
                message,
                ..
            } => message,
        }
    }

    fn to_string(&self) -> String {
        match self {
            SyntaxError::Detailed(message) => message.to_string(),
            SyntaxError::Positioned {
                message,
                position,
                input,
            } => format!("{}\n{}\n{}", input, position, message),
        }
    }
}

impl Display for Position {
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        let underline = if self.end > self.start + 1 {
            "-".repeat(self.end - self.start)
        } else {
            "".to_string()
        };

        write!(formatter, "{}^{}", " ".repeat(self.start), underline)
    }
}

impl Display for PineError {
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        write!(formatter, "{}", self.message)
    }
}

impl Error for PineError {
    fn description(&self) -> &str {
        &self.message
    }

    fn cause(&self) -> Option<&Error> {
        self.cause.as_ref().map(|boxed| &**boxed)
    }
}

impl Display for SyntaxError {
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        write!(formatter, "{}", self.to_string())
    }
}

impl Error for SyntaxError {
    fn description(&self) -> &str {
        self.message()
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

impl From<SyntaxError> for PineError {
    fn from(error: SyntaxError) -> PineError {
        let message = error.to_string();

        let cause: Box<dyn Error> =  Box::new(error);
        let cause = Some(cause);

        PineError { message, cause }
    }
}
