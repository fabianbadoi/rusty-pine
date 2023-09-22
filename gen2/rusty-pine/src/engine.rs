mod query_builder;
mod rendering;
/// Provides helpful data from the database
mod sql;
mod syntax;

#[cfg(test)]
mod tests;

pub use syntax::Rule;

use crate::engine::query_builder::build_query;
use crate::engine::rendering::render_query;
use crate::engine::syntax::parse_to_stage4;

pub fn render(input: &str) -> Result<String, crate::error::Error> {
    let pine = parse_to_stage4(input)?;
    let query = build_query(pine);

    Ok(render_query(query))
}
