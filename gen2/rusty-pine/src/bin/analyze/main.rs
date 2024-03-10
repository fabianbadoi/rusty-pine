mod args;

use args::MySqlConnectionArgs;
use dialoguer::theme::ColorfulTheme;
use dialoguer::MultiSelect;
use mysql::{Opts, Pool, PooledConn};
use rusty_pine::analyze::{
    describe_table, list_databases, list_tables, Database, SchemaObjectName, Server, ServerParams,
    Table, TableName,
};
use rusty_pine::{cache, Error};

fn main() {
    let cli_args = MySqlConnectionArgs::cli_interactive();

    let pool = Pool::new::<Opts, _>((&cli_args).into()).expect("Could not connect to database");
    let mut connection = pool.get_conn().expect("Could not connect to database :(");

    let databases = list_databases(&mut connection).expect("Could not read databases");

    let selection = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select databases")
        .items(&databases)
        .interact()
        // It's OK to unwrap here, because we will see the error message from the dialoguer crate.
        // The errors from there will be stuff like "Must be run in a terminal", which is not stuff I
        // want to concern myself with.
        .unwrap();

    // You'd normally expect to see something like selection.map(|i| databases.get(i)).
    // The approach tries to move data into this closure -------/********************.
    // That is not possible. So we take all the database names, and iterate over that instead.
    let selected_databases: Vec<_> = databases
        .into_iter()
        .enumerate()
        .filter(|(index, _)| selection.contains(index))
        .map(|(_, item)| item)
        .collect();

    let server = Server {
        params: ServerParams {
            hostname: cli_args.hostname_or_ip,
            port: cli_args.port,
            user: cli_args.username,
        },
        databases: selected_databases
            .iter()
            .map(|db_name| {
                let database = analyze_db(&mut connection, db_name).unwrap(); // TODO

                (TableName(db_name.as_str().to_string()), database)
            })
            .collect(),
    };

    cache::write(&server).unwrap();

    println!("Database analyzed and cached");
}

fn analyze_db(connection: &mut PooledConn, db_name: &SchemaObjectName) -> Result<Database, Error> {
    let tables = list_tables(connection, db_name)?;

    let tables: Result<_, Error> = tables
        .into_iter()
        .map(|table_name| {
            let create = describe_table(connection, db_name, &table_name)?;
            let table_name = TableName(table_name.to_string());

            Ok((table_name, Table::from_sql_string(&create)?))
        })
        .collect();

    Ok(Database {
        name: TableName(db_name.as_str().to_string()),
        tables: tables?,
    })
}
