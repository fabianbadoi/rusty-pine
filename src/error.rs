use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug)]
pub struct ParseError {
    message: String,
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

impl Display for ParseError {
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        write!(formatter, "{}", self.message)
    }
}

impl From<SyntaxError> for ParseError {
    fn from(error: SyntaxError) -> ParseError {
        let message = match error {
            SyntaxError::Detailed(message) => message,
            SyntaxError::Positioned {
                message,
                position,
                input,
            } => format!("{}\n{}\n{}", input, position, message),
        };

        ParseError { message }
    }
}
