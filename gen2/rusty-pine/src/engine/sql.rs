/// Parses CREATE TABLE queries into Database instances.
mod parsing;
pub mod querying;
/// Structs used to represent database structure.
pub mod structure;

use crate::engine::sql::structure::Database;
use colored::Colorize;
use std::fmt::{Display, Formatter};
use std::ops::Add;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) struct DbStructureParseError {
    pub input: InputWindow,
    pub line_number: usize,
    pub message: String,
}

#[derive(Debug, Clone)]
pub(crate) struct InputWindow {
    pub start_line: usize,
    pub content: String,
}

struct DatabaseInfo {
    /// The original create table queries.
    ///
    /// The way this struct works is by keeping the create table queries in memory, and only making
    /// certain views into the data available. The lsp idea is that any return type that can be
    /// read from this struct, will only contain references to the "inner" data.
    /// If we had really large create table queries, this would mean we avoid duplicating/cloning
    /// some strings. I suspect that in practice this "optimization" is worthless, but it was more
    /// fun to write.
    create_table_queries: Vec<String>,

    /// Structure of the database.
    ///
    /// Only contains &str's from the create table queries.
    database: Database,
}

impl InputWindow {
    pub fn with_line<'a, T: AsRef<str>>(&self, line: T) -> Self {
        InputWindow {
            content: self.content.clone().add(line.as_ref()),
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
