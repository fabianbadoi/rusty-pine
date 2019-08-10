extern crate rusty_pine_lib;
extern crate pretty_env_logger;

use rusty_pine_lib::read_config;
use rusty_pine_lib::Analyzer;

fn main() {
    pretty_env_logger::init();

    let analyezer = Analyzer::connect_fresh(&read_config()).unwrap();

    println!("{:#?}", analyezer.save());
    println!("Hello, world!");
}
