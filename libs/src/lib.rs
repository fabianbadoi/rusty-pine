#![allow(clippy::let_and_return)]

extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate strsim;

mod cache;
mod config;
mod error;
mod pine_syntax;
mod pine_transpiler;
mod query;
mod sql;
mod analyzer;

#[cfg(test)]
mod integration_tests;

pub use analyzer::Analyzer;
pub use config::{read as read_config, Config};
pub use pine_transpiler::{connect as connect_transpiler, MySqlTranspiler, Transpiler};
