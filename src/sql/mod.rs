mod renderer;
#[cfg(test)]
mod shorthand;
mod contextual_renderer;

pub use self::renderer::StringRenderer;

pub trait Renderer<Q, O> {
    fn render(self, query: &Q) -> O;
}
