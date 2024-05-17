mod query_builder;
mod rendering;
/// Provides helpful data from the database
pub mod sql;
mod syntax;

#[cfg(test)]
mod tests;

use crate::analyze::Server;
pub use syntax::Rule;

use crate::engine::query_builder::build_query;
use crate::engine::rendering::render_query;
use crate::engine::syntax::parse_to_stage4;

pub fn render(input: &str, server: &Server) -> Result<String, crate::error::Error> {
    let pine = parse_to_stage4(input)?;
    let query = build_query(pine, server);

    Ok(render_query(query))
}
