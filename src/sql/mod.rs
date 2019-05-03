mod renderer;
#[cfg(test)]
mod shorthand;

pub use self::renderer::StringRenderer;

pub trait Renderer<Q, O> {
    fn render(self, query: &Q) -> O;
}
