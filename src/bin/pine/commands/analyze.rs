use colored::Colorize;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{MultiSelect, Password};
use rusty_pine::analyze::{
    mariadb, postgres, Analyzer, DBType, Database, DatabaseName, SchemaObjectName, Server, Table,
};
use rusty_pine::context::{Context, ContextName};
use rusty_pine::{cache, Error, InternalError};
use std::collections::HashMap;
use tokio::runtime::Builder;

pub fn analyze() -> Result<(), Error> {
    // we need tokio here because sqlx is exclusively async
    let tokio = Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .expect("Cannot build tokio runtime");

    tokio.block_on(async { run_analyze().await })
}

async fn run_analyze() -> Result<(), Error> {
    let current_context = ContextName::current()?;
    let context: Context = cache::read(&current_context)?;

    let password = ask_for_password(&context)?;

    let db_connection: Box<dyn Analyzer> = match context.server_params.db_type {
        DBType::PostgresSQL => Box::new(
            postgres(context.server_params.clone(), &password)
                .await
                .expect("Could not connect to the PostgresSQL instance"),
        ),
        DBType::MariaDB => Box::new(
            mariadb(context.server_params.clone(), &password)
                .await
                .expect("Could not connect to the MariaDB instance"),
        ),
    };

    let databases = db_connection.list_databases().await?;

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

    let mut databases = HashMap::new();

    for db_name in selected_databases {
        let database = analyze_db(db_connection.as_ref(), db_name.clone()).await?;
        let db_name = DatabaseName(db_name.as_str().to_string());

        databases.insert(db_name, database);
    }

    let server = Server {
        params: context.server_params,
        databases,
    };

    cache::write(&server)?;

    println!("Database analyzed and cached");
    Ok(())
}

async fn analyze_db(
    connection: &dyn Analyzer,
    db_name: SchemaObjectName,
) -> Result<Database, Error> {
    let table_names = connection.list_tables(&db_name).await?;
    let mut all_columns = connection.table_columns(&db_name).await?;
    let mut all_fks = connection.table_foreign_keys(&db_name).await?;
    let mut all_pks = connection.table_primary_keys(&db_name).await?;

    let mut tables = HashMap::new();
    for table_name in table_names {
        let columns = all_columns.remove(&table_name).unwrap_or_default();
        let foreign_keys = all_fks.remove(&table_name).unwrap_or_default();
        let primary_key = all_pks.remove(&table_name).ok_or(InternalError(
            "Encountered tables without primary keys".to_string(),
        ))?;

        tables.insert(
            table_name.clone(),
            Table {
                name: table_name,
                columns,
                foreign_keys,
                primary_key,
            },
        );
    }

    Ok(Database {
        name: DatabaseName::new(db_name),
        tables: tables.into(),
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
