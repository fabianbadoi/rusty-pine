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

mod engine;
mod error;

use crate::engine::render;

fn main() {
    println!("{}", render("table").unwrap());
}
