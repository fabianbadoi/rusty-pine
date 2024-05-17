//! We can insert create table queries at the beginning of our .sql tests and these will be used
//! to create the test database structure.

// TODO find a way to print the line number on failures here.

use crate::analyze::{Database, Server, ServerParams, Table};
use crate::engine::sql::querying::TableDescription;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Lines};
use std::iter::{Enumerate, Peekable};

pub fn read_mock_server(
    lines: &mut Peekable<Enumerate<Lines<BufReader<File>>>>,
) -> Result<Server, crate::Error> {
    let tables = read_create_table_statements(lines)?;

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
    lines: &mut Peekable<Enumerate<Lines<BufReader<File>>>>,
) -> Result<Vec<Table>, crate::Error> {
    let mut buffer = String::new();

    while let Some((line_number, next_item)) = lines.peek() {
        if next_item.is_err() {
            break; // malformed UTF-8, we'll deal with it later
        }

        let line = next_item
            .as_ref()
            .expect("next_item was checked right before");

        if line.starts_with("-- Test: ") {
            // You can put create table statements at the begging of a test.sql file. Any create
            // table statements AFTER the first test will be ignored.
            break;
        }

        if !line.is_empty() && !line.starts_with("--") {
            buffer.push_str(line);
            buffer.push_str("\n");
        }

        lines.next();
    }

    Ok(buffer
        .split(';')
        .filter(|s| !s.trim().is_empty())
        .map(&str::trim)
        .map(|statement| Table::from_sql_string(&TableDescription::new_for_tests(statement)))
        .collect::<Result<Vec<_>, _>>()?)
}
