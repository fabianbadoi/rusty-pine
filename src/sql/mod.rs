mod renderer;
#[cfg(test)]
mod shorthand;
mod contextual_renderer;

pub use self::renderer::DumbRenderer;
use crate::error::PineError;

pub trait Renderer<Q, O> {
    fn render(self, query: &Q) -> Result<O, PineError>;
}
