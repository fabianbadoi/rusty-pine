extern crate pretty_env_logger;
extern crate rusty_pine_lib;

use rusty_pine_lib::read_config;
use rusty_pine_lib::{offline, Transpiler};

fn main() {
    pretty_env_logger::init();

    let transpiler = offline(&read_config(), "penneo").unwrap();

    let input = match get_input() {
        Some(string) => string,
        None => return,
    };

    match transpiler.transpile(input.as_ref() as &str) {
        Ok(query) => println!("{}", query),
        Err(error) => println!("/*\n{}\n*/", error),
    }
}

fn get_input() -> Option<String> {
    std::env::args().nth(1)
}
