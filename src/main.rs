extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate regex;

mod pine_syntax;
mod pine_parser;
mod query;
mod sql;
mod error;

fn main() {
    use pine_parser::{NaiveParser, Parser};

    let parser = NaiveParser::default();

    // normal flow
    println!("------------------------------");
    println!("{}", parser.parse("from: users | where: id = 3").unwrap());
    println!("------------------------------");

    // syntax error 1
    println!("------------------------------");
    println!(
        "{}",
        parser
            .parse("from: users | filter: id = 3 | select: id")
            .unwrap_err()
    );
    println!("------------------------------");

    // syntax erro 2
    println!("------------------------------");
    println!(
        "{}",
        parser
            .parse("from: users | where: id  3 3 id | select: id")
            .unwrap_err()
    );
    println!("------------------------------");

    // query builder flow
    println!("------------------------------");
    println!(
        "{}",
        parser.parse("where: id = 3 | select: id").unwrap_err()
    );
    println!("------------------------------");

    println!("------------------------------");
    println!(
        "{}",
        parser.parse("users 3").unwrap()
    );
    println!("------------------------------");
}
