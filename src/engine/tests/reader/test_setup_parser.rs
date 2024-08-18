//! We can insert create table queries at the beginning of our .sql tests and these will be used
//! to create the test database structure.
//!
//! Because I'm lazy, the create table statements have to be written in just the right way.

use crate::analyze::{DBType, Database, Server, ServerParams, Table};
use crate::engine::sql::DbStructureParsingContext as Context;
use crate::engine::sql::{DbStructureParseError, InputWindow};
use crate::engine::tests::reader::TestLineIterator;
use crate::error::ErrorKind;
use std::collections::HashMap;
use std::io::Error as IOError;
use std::path::PathBuf;

pub fn read_mock_server(
    file: &PathBuf,
    lines: &mut TestLineIterator,
) -> Result<Server, crate::Error> {
    let tables = read_create_table_statements(file, lines)?;

    let databases = HashMap::from([(
        "default".into(),
        Database {
            name: "default".into(),
            tables: tables
                .into_iter()
                .map(|t| (t.name.clone(), t))
                .collect::<HashMap<_, _>>()
                .into(),
        },
    )]);

    Ok(Server {
        // these don't matter
        params: ServerParams {
            db_type: DBType::MariaDB,
            hostname: "".to_string(),
            port: 0,
            user: "".to_string(),
            database: "default".into(),
        },
        databases,
    })
}

fn read_create_table_statements(
    file: &PathBuf,
    lines: &mut TestLineIterator,
) -> Result<Vec<Table>, crate::Error> {
    let table_reader = TableParser::new(file, lines);

    Ok(table_reader
        .into_iter()
        .collect::<Result<Vec<Table>, crate::Error>>()?)
}

struct TableParser<'a> {
    lines: &'a mut TestLineIterator,
    context: Context,
}

impl Iterator for TableParser<'_> {
    type Item = Result<Table, crate::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_table() {
            Ok(table_or_none) => table_or_none.map(|table| Ok(table)),
            Err(err) => Some(Err(err)),
        }
    }
}

impl<'a> TableParser<'a> {
    fn new(file: &'a PathBuf, lines: &'a mut TestLineIterator) -> Self {
        TableParser {
            context: Context::File(file.clone()),
            lines,
        }
    }

    fn next_table(&mut self) -> Result<Option<Table>, crate::Error> {
        if self.advance_until_next_create()?.is_none() {
            return Ok(None);
        }

        if self.lines.peek().is_none() {
            // End of input.
            return Ok(None);
        }

        let (start_line, _) = *self.lines.peek().expect("lines.peek() is checked above");

        let statement = self.read_entire_create_statement()?;
        let table = Table::from_sql_string(&self.context, statement.as_str())
            .map_err(|err| move_start_line(err, start_line))?;

        Ok(Some(table))
    }

    fn advance_until_next_create(&mut self) -> Result<Option<()>, crate::Error> {
        while let Some((_, next_item)) = self.lines.peek() {
            if next_item.is_err() {
                return Err(self
                    .lines
                    .next()
                    .expect("lines.next() should be a Some because we already checked")
                    .1
                    .expect_err("lines.next().1 should be an Err because we checked"))?;
            }

            let line = next_item
                .as_ref()
                .expect("next_item was checked right before")
                .trim();

            if line.starts_with("-- Test: ") {
                // You can put create table statements at the begging of a test.sql file. Any create
                // table statements AFTER the first test will be ignored.
                return Ok(None);
            }

            if line.to_lowercase().starts_with("create table ") {
                // We found it, the next lines is a create table statement
                break;
            }

            self.lines.next();
        }

        Ok(Some(()))
    }

    fn read_entire_create_statement(&mut self) -> Result<String, DbStructureParseError> {
        let reader = match SingleCreateTableStatementReader::new(&self.context, self.lines) {
            Some(reader) => reader,
            None => {
                return Ok("".to_string());
            }
        };

        reader.read_statement()
    }
}

struct SingleCreateTableStatementReader<'a> {
    input: InputWindow,
    lines: &'a mut TestLineIterator,
}

impl<'a> SingleCreateTableStatementReader<'a> {
    fn new(context: &'a Context, lines: &'a mut TestLineIterator) -> Option<Self> {
        match lines.peek() {
            None => None,
            Some((start_line, _)) => Some(Self {
                input: InputWindow {
                    start_line: *start_line,
                    context: context.clone(),
                    content: String::new(),
                },
                lines,
            }),
        }
    }

    fn read_statement(mut self) -> Result<String, DbStructureParseError> {
        while let Some((line_number, next_item)) = self.lines.next() {
            let in_buffer_line_nr = line_number - self.input.start_line;

            let line = valid_line(in_buffer_line_nr, next_item, &self.input)?;

            self.input.content.push_str(line.as_str());
            self.input.content.push('\n');

            if line.contains(';') && !line.trim().ends_with(';') {
                return Err(DbStructureParseError {
                    line_number: in_buffer_line_nr,
                    message: "';' should only appear at the very end of ceate table statements"
                        .to_string(),
                    input: self.input.with_line(line),
                });
            }

            if line.trim().ends_with(';') {
                break;
            }
        }

        Ok(self.input.content)
    }
}

fn valid_line(
    line_number: usize,
    next_item: Result<String, IOError>,
    input_window: &InputWindow,
) -> Result<String, DbStructureParseError> {
    if next_item.is_err() {
        return Err(DbStructureParseError {
            line_number,
            message: "Cannot parse line as UTF-8".to_string(),
            input: input_window.clone(),
        });
    }

    let line = next_item.expect("next_item was checked right before");
    let trimmed_lines = line.trim();

    if trimmed_lines.starts_with("-- Test:") {
        return Err(DbStructureParseError {
            line_number,
            message: "Found unexpected '-- Test:' in .sql cerate table section".to_string(),
            input: input_window.with_line(line),
        });
    }

    if trimmed_lines.is_empty() {
        return Err(DbStructureParseError {
            line_number,
            message: "Empty lines not supported in .sql create table statements".to_string(),
            input: input_window.with_line(line),
        });
    }

    if trimmed_lines.starts_with("--") {
        return Err(DbStructureParseError {
            line_number,
            message: "Comments (--) not supported in .sql create table statements".to_string(),
            input: input_window.with_line(line),
        });
    }

    Ok(line)
}

fn move_start_line(err: crate::Error, mode_by: usize) -> ErrorKind {
    use ErrorKind::DbStructureParseError as ParseError;

    let kind = err.into_inner();

    match kind {
        ParseError(ddl_error) => ParseError(ddl_error.move_window(mode_by)),
        _ => kind,
    }
}
