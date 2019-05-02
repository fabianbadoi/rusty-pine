extern crate pest;
#[macro_use]
extern crate pest_derive;

mod pine_syntax;
mod pine_parser;
mod query;
mod sql;

#[derive(Debug)]
pub struct ParseError {
    message: String,
}

fn main() {
    use pine_parser::{Parser, GenericParser};

    let parser = GenericParser::default();

    println!("{}", parser.parse("from: users | where: id = 3 | select: id").unwrap());
    println!("{}", parser.parse("from: users | filter: id = 3 | select: id").unwrap_err().message);
}
