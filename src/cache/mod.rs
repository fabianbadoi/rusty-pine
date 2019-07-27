mod file;

pub trait Cache<T> {
    fn get(&self, tag: &str) -> Option<T>;
    fn set(&mut self, tag: &str, data: &T);
}

#[cfg(test)]
mod memory;
#[cfg(test)]
pub use memory::MemoryCache;
