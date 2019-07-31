extern crate rusty_pine_lib;

use rusty_pine_lib::read_config;
use rusty_pine_lib::{connect_transpiler, Transpiler};

fn main() {
    let transpiler = connect_transpiler(&read_config(), "penneo").unwrap();

    match transpiler.transpile("users 1 | folders | folderCaseFileMap | caseFiles | customers") {
        Ok(query) => println!("{}", query),
        Err(error) => println!("/*\n{}\n*/", error),
    }
}
