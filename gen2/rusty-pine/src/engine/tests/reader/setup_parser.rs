//! We can insert create table queries at the beginning of our .sql tests and these will be used
//! to create the test database structure.

// TODO find a way to print the line number on failures here.

use crate::analyze::{Database, Server, ServerParams, Table};
use crate::engine::sql::querying::TableDescription;
use crate::engine::sql::DbStructureParsingContext as Context;
use crate::engine::sql::{DbStructureParseError, InputWindow};
use crate::error::ErrorKind;
use crate::Error;
use std::collections::HashMap;
use std::fs::File;
use std::io::Error as IOError;
use std::io::{BufReader, Lines};
use std::iter::{Enumerate, Peekable};

pub fn read_mock_server(
    context: Context,
    lines: &mut Peekable<Enumerate<Lines<BufReader<File>>>>,
) -> Result<Server, crate::Error> {
    let tables = read_create_table_statements(context, lines)?;

    let databases = HashMap::from([(
        "default".into(),
        Database {
            name: "default".into(),
            tables: tables.into_iter().map(|t| (t.name.clone(), t)).collect(),
        },
    )]);

    Ok(Server {
        // these don't matter
        params: ServerParams {
            hostname: "".to_string(),
            port: 0,
            user: "".to_string(),
        },
        databases,
    })
}

fn read_create_table_statements(
    context: Context,
    lines: &mut Peekable<Enumerate<Lines<BufReader<File>>>>,
) -> Result<Vec<Table>, crate::Error> {
    let mut tables = Vec::new();

    while let Some(next_table) = next_table(&context, lines)? {
        tables.push(next_table);
    }

    return Ok(tables);
}

fn next_table(
    context: &Context,
    lines: &mut Peekable<Enumerate<Lines<BufReader<File>>>>,
) -> Result<Option<Table>, crate::Error> {
    if advance_until_next_create(lines)?.is_none() {
        return Ok(None);
    }

    if lines.peek().is_none() {
        // End of input.
        return Ok(None);
    }

    let (start_line, _) = *lines.peek().expect("lines.peek() is checked above");

    let statement = read_entire_create_statement(context, lines)?;
    let table = Table::from_sql_string(context, &TableDescription::new_for_tests(statement))
        .map_err(|err| {
            let kind = err.into_inner();
            if let ErrorKind::DbStructureParseError(ddl_error) = kind {
                ErrorKind::DbStructureParseError(ddl_error.move_window(start_line))
            } else {
                kind
            }
        })?;

    Ok(Some(table))
}

fn read_entire_create_statement(
    context: &Context,
    lines: &mut Peekable<Enumerate<Lines<BufReader<File>>>>,
) -> Result<String, DbStructureParseError> {
    let start_line = match lines.peek() {
        None => {
            return Ok("".to_string());
        }
        Some((line_nr, _)) => *line_nr,
    };
    let mut input_window = InputWindow {
        start_line,
        context: context.clone(),
        content: String::new(),
    };

    while let Some((line_number, next_item)) = lines.next() {
        let in_buffer_line_nr = line_number - start_line;
        let line = valid_line(in_buffer_line_nr, next_item, &input_window)?;

        input_window.content.push_str(line.as_str());
        input_window.content.push('\n');

        if line.contains(';') && !line.trim().ends_with(';') {
            return Err(DbStructureParseError {
                line_number: in_buffer_line_nr,
                message: "';' should only appear at the very end of ceate table statements"
                    .to_string(),
                input: input_window.with_line(line),
            });
        }

        if line.trim().ends_with(';') {
            break;
        }
    }

    Ok(input_window.content)
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

fn advance_until_next_create(
    lines: &mut Peekable<Enumerate<Lines<BufReader<File>>>>,
) -> Result<Option<()>, crate::Error> {
    while let Some((_, next_item)) = lines.peek() {
        if next_item.is_err() {
            return Err(lines
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

        lines.next();
    }

    Ok(Some(()))
}
