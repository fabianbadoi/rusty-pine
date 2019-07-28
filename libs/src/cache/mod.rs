mod file;
mod serde;

pub use self::serde::SerializedCache;
pub use file::ByteFileCache;

pub type DefaultCache = SerializedCache<ByteFileCache>;

pub trait Cache<T> {
    fn get(&self, tag: &str) -> Option<T>;
    fn set(&mut self, tag: &str, data: &T);
    fn clear(&mut self);
}

pub fn make_cahe(path: &str) -> DefaultCache {
    use std::path::Path;

    let path = Path::new(&std::env::var("HOME").unwrap())
        .join(".cache/rusty-pine")
        .join(path);

    SerializedCache::wrap(ByteFileCache::new(path.into()))
}

#[cfg(test)]
mod memory;
#[cfg(test)]
pub use memory::MemoryCache;
