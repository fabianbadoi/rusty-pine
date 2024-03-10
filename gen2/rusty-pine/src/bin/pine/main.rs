mod args;

use crate::args::{Command, ContextParams};
use args::Args;
use clap::Parser;
use rusty_pine::cache;
use rusty_pine::context::{Context, ContextName};

fn main() {
    let args = Args::parse();

    match args.command {
        Command::CreateContext(context) => create_context(context).unwrap(),
        Command::UseContext { name } => use_context(name).unwrap(),
    }
}

fn create_context(params: ContextParams) -> Result<(), rusty_pine::Error> {
    let use_it = params.use_it;
    let new_context: Context = params.into();

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

fn use_context(name: String) -> Result<(), rusty_pine::Error> {
    let context_name: ContextName = name.into();

    cache::write(&context_name)?;

    println!("Switched to context \x1b[1m{}\x1b[0m", context_name);

    Ok(())
}
