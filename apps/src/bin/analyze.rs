extern crate pretty_env_logger;
extern crate rusty_pine_lib;

use rusty_pine_lib::connect;
use rusty_pine_lib::read_config;

fn main() {
    pretty_env_logger::init();

    let analyezer = connect(&read_config()).unwrap();

    println!("{:#?}", analyezer.save());
    println!("Hello, world!");
}
