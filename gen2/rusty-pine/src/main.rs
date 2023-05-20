mod error;
mod query_builder;
mod rendering;
mod syntax;

use crate::query_builder::build_query;
use crate::rendering::render_query;
use crate::syntax::parse_to_stage4;
pub use error::Error;

fn main() {
    println!(
        "{}",
        render_query(build_query(parse_to_stage4("table").unwrap()))
    );
}
