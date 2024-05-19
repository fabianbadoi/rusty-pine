/// Parses CREATE TABLE queries into Database instances.
mod parsing;
pub mod querying;
/// Structs used to represent database structure.
pub mod structure;

use crate::engine::sql::structure::Database;
use colored::Colorize;
use std::fmt::{Display, Formatter};
use std::ops::Add;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub struct DbStructureParseError {
    pub input: InputWindow,
    pub line_number: usize,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub enum DbStructureParsingContext {
    File(PathBuf),
    Connection {
        database: String,
        table: String,
    },
    #[default]
    None,
}

#[derive(Debug, Clone)]
pub struct InputWindow {
    pub context: DbStructureParsingContext,
    pub start_line: usize,
    pub content: String,
}

impl InputWindow {
    pub fn with_line<T: AsRef<str>>(&self, line: T) -> Self {
        InputWindow {
            content: self.content.clone().add(line.as_ref()),
            context: self.context.clone(),
            ..*self
        }
    }

    fn move_window(self, base_line: usize) -> Self {
        InputWindow {
            start_line: base_line + self.start_line,
            ..self
        }
    }
}

impl DbStructureParseError {
    pub fn move_window(self, base_line: usize) -> DbStructureParseError {
        DbStructureParseError {
            input: self.input.move_window(base_line),
            ..self
        }
    }
}

impl Display for DbStructureParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Error parsing DDL statement:")?;
        writeln!(f, "{}", self.message.bold())?;
        writeln!(f, "{}", self.input.context)?;

        for line in self.input.content.lines().enumerate() {
            let (nr, text) = line;

            writeln!(f, "{:>3} | {}", nr + self.input.start_line + 1, text)?;

            if nr == self.line_number {
                writeln!(f, "    | {}", "^".repeat(text.len()).red())?;
            }
        }

        Ok(())
    }
}

impl Display for DbStructureParsingContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DbStructureParsingContext::File(path) => {
                write!(
                    f,
                    "In file: {}",
                    path.to_str().expect("You better fucking work")
                )?;
            }
            DbStructureParsingContext::Connection { .. } => todo!(),
            DbStructureParsingContext::None => {}
        };

        Ok(())
    }
}
