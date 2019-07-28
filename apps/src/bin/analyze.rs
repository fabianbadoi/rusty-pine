extern crate rusty_pine_lib;

use rusty_pine_lib::Analyzer;

fn main() {
    let analyezer = Analyzer::connect_fresh("root", "development", "localhost", 3306).unwrap();

    println!("{:#?}", analyezer.save());
    println!("Hello, world!");
}
