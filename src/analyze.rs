extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate mysql;
extern crate regex;

mod cache;
mod error;
mod pine_syntax;
mod pine_transpiler;
mod query;
mod sql;

use sql::{LiveConnection, MySqlReflector, Reflector};

fn main() {
    let connection = LiveConnection::new("root", "development", "localhost", 3306).unwrap();
    let reflector = MySqlReflector::for_connection(connection);

    println!("{:#?}", reflector.analyze());
    println!("Hello, world!");
}
