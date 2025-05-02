mod args;
mod commands;

use crate::args::{Command, ContextParams};
use args::Args;
use clap::Parser;
use rusty_pine::analyze::{DBType, Server};
use rusty_pine::context::{Context, ContextName};
use rusty_pine::{cache, InternalError};

fn main() {
    env_logger::init();

    let args = Args::parse();

    match args.command {
        Command::CreateContext(context) => create_context(context).unwrap(),
        Command::UseContext { name } => use_context(name).unwrap(),
        Command::ListContexts => list_contexts().unwrap(),
        Command::Analyze => commands::analyze::analyze().unwrap(),
        Command::PineServer => commands::pine_server::run(),
        Command::Translate { input } => commands::translate_one(input),
        Command::ListTables => list_tables().unwrap(),
    }
}

fn create_context(params: ContextParams) -> Result<(), rusty_pine::Error> {
    let use_it = params.use_it;
    let new_context: Context = params.into();

    validate_new_context(&new_context)?;

    cache::write(&new_context)?;

    println!("Create new context \x1b[1m{}\x1b[0m.", new_context.name);

    if use_it {
        use_context(new_context.name.into())?;
    } else {
        println!(
            "Switch to it by running \x1b[1mpine use-context {}\x1b[0m.",
            new_context.name
        );
    }

    Ok(())
}

fn validate_new_context(context: &Context) -> Result<(), rusty_pine::Error> {
    if context.server_params.db_type == DBType::PostgresSQL
        && context.server_params.default_schema.is_none()
    {
        Err(InternalError(
            "You must specify the default schema when using postgres. \
                See --help for more info"
                .to_string(),
        ))?;
    }

    Ok(())
}

fn use_context(name: String) -> Result<(), rusty_pine::Error> {
    let context_name: ContextName = name.into();

    cache::write(&context_name)?;

    println!("Switched to context \x1b[1m{}\x1b[0m.", context_name);

    Ok(())
}

fn list_tables() -> Result<(), rusty_pine::Error> {
    use colored::Colorize;

    let current_context = ContextName::current()
        .expect("Can't find current context, create one using `pine create-context`");
    let context: Context = cache::read(&current_context).expect(
        "Your current context is corrupted. The data is in ~/.cache/rusty-pine/. Good luck!",
    );
    let server: Server =
        cache::read(&context.server_params).expect("You must first run `pine analyze`");

    println!("Database: {}", server.params.hostname.bold());

    let padding = {
        server
            .databases
            .iter()
            .flat_map(|(_, db)| db.tables.iter())
            .map(|(_, table)| table.name.as_str().len())
            .max()
            .unwrap_or(20)
            + 2
    };

    for (_, db) in server.databases {
        for (_, table) in db.tables.iter() {
            println!(
                "{}.  {:<padding$} [{}]",
                db.name,
                table.name.as_str().italic().bold(),
                table
                    .foreign_keys
                    .iter()
                    .map(|fk| fk.to.table.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
    }

    Ok(())
}

fn list_contexts() -> Result<(), rusty_pine::Error> {
    use colored::Colorize;

    let current_context = ContextName::current()?;
    let known_contexts: Vec<Context> = cache::read_all()?;

    println!("Available contexts:");
    for context in &known_contexts {
        println!(
            "{}{}: {} ({})",
            if current_context == context.name {
                " * ".bold()
            } else {
                "   ".into()
            },
            context.name.to_string().bold(),
            context.server_params.hostname,
            context.server_params.database
        )
    }

    Ok(())
}
