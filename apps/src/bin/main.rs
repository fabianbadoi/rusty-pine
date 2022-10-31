extern crate pretty_env_logger;
extern crate rusty_pine_lib;

use rusty_pine_lib::read_config;
use rusty_pine_lib::{offline, Transpiler};
use std::process::ExitCode;

fn main() -> ExitCode {
    pretty_env_logger::init();

    let database = match get_database() {
        Some(string) => string,
        None => {
            eprintln!("/*\nFirst argument must be a database name\n*/");
            return ExitCode::FAILURE;
        }
    };

    let transpiler = offline(&read_config(), database.as_str()).unwrap();

    let input = match get_input_pine() {
        Some(string) => string,
        None => {
            eprintln!("/*\nSecond argument must be a database Pine\n*/");
            return ExitCode::FAILURE;
        }
    };

    match transpiler.transpile(input.as_ref() as &str) {
        Ok(query) => {
            println!("{}", query);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("/*\n{}\n*/", error);
            ExitCode::FAILURE
        }
    }
}

fn get_database() -> Option<String> {
    std::env::args().nth(1)
}

fn get_input_pine() -> Option<String> {
    std::env::args().nth(2)
}
