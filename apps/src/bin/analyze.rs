extern crate rusty_pine_lib;

use rusty_pine_lib::read_config;
use rusty_pine_lib::Analyzer;

fn main() {
    let analyezer = Analyzer::connect_fresh(&read_config()).unwrap();

    println!("{:#?}", analyezer.save());
    println!("Hello, world!");
}
