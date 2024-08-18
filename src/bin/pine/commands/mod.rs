use colored::Colorize;
use rusty_pine::analyze::Server;
use rusty_pine::context::{Context, ContextName};
use rusty_pine::{cache, render};
use std::process::exit;

pub mod analyze;
pub mod pine_server;

pub fn translate_one(input: String) {
    let current_context = ContextName::current()
        .expect("Can't find current context, create one using `pine create-context`");
    let context: Context = cache::read(&current_context).expect(
        "Your current context is corrupted. The data is in ~/.cache/rusty-pine/. Good luck!",
    );
    let server: Server =
        cache::read(&context.server_params).expect("You must first run `pine analyze`");

    let result = render(input.as_str(), &server);
    match result {
        Ok(output) => println!("{output}"),
        Err(error) => {
            eprintln!("{intro}: {error}", intro = "error".bold().red(),);
            exit(1);
        }
    }
}
