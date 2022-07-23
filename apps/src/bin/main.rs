extern crate pretty_env_logger;
extern crate rusty_pine_lib;

use rusty_pine_lib::read_config;
use rusty_pine_lib::{offline, Transpiler};

fn main() {
    pretty_env_logger::init();

    let database = match get_database() {
        Some(string) => string,
        None => return,
    };

    let transpiler = offline(&read_config(), database.as_str()).unwrap();

    let input = match get_input_pine() {
        Some(string) => string,
        None => return,
    };

    match transpiler.transpile(input.as_ref() as &str) {
        Ok(query) => println!("{}", query),
        Err(error) => println!("/*\n{}\n*/", error),
    }
}

fn get_database() -> Option<String> {
    std::env::args().nth(1)
}

fn get_input_pine() -> Option<String> {
    std::env::args().nth(2)
}
