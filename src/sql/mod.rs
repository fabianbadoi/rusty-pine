mod contextual_renderer;
mod mysql_reflect;
mod renderer;
#[cfg(test)]
mod shorthand;
mod structure;

pub use self::renderer::DumbRenderer;
pub use mysql_reflect::connection::{Connection, LiveConnection};
pub use mysql_reflect::live_analysis::{MySqlReflector, Reflector};

use crate::error::PineError;

pub trait Renderer<Q, O> {
    fn render(self, query: &Q) -> Result<O, PineError>;
}
