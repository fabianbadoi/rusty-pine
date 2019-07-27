extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate mysql;
extern crate regex;
extern crate serde;
extern crate serde_json;

mod cache;
mod error;
mod pine_syntax;
mod pine_transpiler;
mod query;
mod sql;

use sql::analyzer::connect;
use sql::Reflector;

fn main() {
    let reflector = connect("root", "development", "localhost", 3306).unwrap();

    println!("{:#?}", reflector.analyze());
    println!("Hello, world!");
}
