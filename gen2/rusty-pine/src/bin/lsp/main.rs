//! Helps run SQL queries by writing in a shorter syntax.
//!
//! The language we're going to write everything in is called "Pine". I have no idea why, I just got
//! inspired by [a friend's really cool project] that does the same thing.
//!
//! The original Pine project is written in Ahmad's favorite language: clojure. It's beloved by many,
//! but sadly I can't stand it. After playing around with the original code base, trying to add a
//! new feature, I decided it was for not for me.
//!
//! I then did the typical thing and rewrote the project in rust.
//!
//! [a friend's really cool project]: https://github.com/pine-lang/pine

use log::{debug, LevelFilter};
use rusty_pine::lsp::Backend;
use std::fs::File;
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    let target = Box::new(File::create("/tmp/rusty.log").expect("Can't create file"));

    env_logger::builder()
        .target(env_logger::Target::Pipe(target))
        .filter_module("rusty_pine", LevelFilter::Debug)
        .try_init()
        .unwrap();

    let read = tokio::io::stdin();
    let write = tokio::io::stdout();

    debug!("starting");

    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(read, write, socket).serve(service).await;
}
