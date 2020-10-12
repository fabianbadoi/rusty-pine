mod contextual_renderer;
mod mysql_reflect;
#[cfg(test)]
mod shorthand;
pub mod structure;

pub use contextual_renderer::SmartRenderer;
pub use mysql_reflect::connection::{Connection, LiveConnection};
pub use mysql_reflect::live_analysis::{MySqlReflector, Reflector};
pub use mysql_reflect::CacheBuilder;
pub use mysql_reflect::OfflineReflector;

use crate::error::PineError;

pub mod analyzer {
    pub use super::mysql_reflect::connect_fresh;
    pub use super::mysql_reflect::offline;
}

pub trait Renderer<Q, O> {
    fn render(self, query: &Q) -> Result<O, PineError>;
}
