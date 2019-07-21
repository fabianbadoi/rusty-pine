mod contextual_renderer;
mod mysql_reflect;
mod renderer;
#[cfg(test)]
mod shorthand;

pub use self::renderer::DumbRenderer;
use crate::error::PineError;

pub trait Renderer<Q, O> {
    fn render(self, query: &Q) -> Result<O, PineError>;
}
