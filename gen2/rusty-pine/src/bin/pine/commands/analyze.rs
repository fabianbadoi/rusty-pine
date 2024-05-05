use colored::Colorize;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{MultiSelect, Password};
use mysql::{Opts, OptsBuilder, Pool, PooledConn};
use rusty_pine::analyze::{
    describe_table, list_databases, list_tables, Database, SchemaObjectName, Server, ServerParams,
    Table, TableName,
};
use rusty_pine::context::{Context, ContextName};
use rusty_pine::{cache, Error};

pub fn analyze() -> Result<(), Error> {
    let current_context = ContextName::current()?;
    let context: Context = cache::read(&current_context)?;

    let password = ask_for_password(&context)?;
    let opts: Opts = opts(&context.server_params, password.as_ref());

    let pool = Pool::new::<Opts, _>(opts)?;
    let mut connection = pool.get_conn()?;

    let databases = list_databases(&mut connection)?;

    println!(
        "Use arrow keys (⬆⬇) to navigate, {} to select, and {} to confirm.",
        "<space>".bold(),
        "<enter>".bold(),
    );

    let selection = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select databases")
        .items(&databases)
        .interact()?;

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
        params: context.server_params,
        databases: selected_databases
            .iter()
            .map(|db_name| {
                let database = analyze_db(&mut connection, db_name)?;

                Ok((TableName(db_name.as_str().to_string()), database))
            })
            .collect::<Result<_, Error>>()?,
    };

    cache::write(&server)?;

    println!("Database analyzed and cached");

    Ok(())
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

/// Ask the user for a password.
///
/// I don't want to store passwords, it's too complicated to do safely.
fn ask_for_password(context: &Context) -> Result<String, Error> {
    // the password is the only part that we have to read interactively
    // we use the dialoguer library for this bec
    println!("Using context {}", context.name.to_string().bold().green());
    println!(
        "Please provide the password for {}",
        context.server_params.to_string().bold().green()
    );
    Ok(Password::with_theme(&ColorfulTheme::default())
        .with_prompt("Password: ")
        .interact()?)
}

fn opts(value: &ServerParams, password: &str) -> Opts {
    OptsBuilder::new()
        .ip_or_hostname(Some(value.hostname.clone()))
        .tcp_port(value.port)
        .user(Some(&value.user))
        .pass(Some(password))
        .into()
}
