use super::Cache;

use std::collections::HashMap;

#[derive(Default)]
pub struct MemoryCache<T> {
    cache: HashMap<String, T>,
}

impl<T> Cache<T> for MemoryCache<T>
where
    T: Clone,
{
    fn get(&self, tag: &str) -> Option<T> {
        self.cache.get(tag).cloned()
    }

    fn set(&mut self, tag: &str, data: &T) {
        let _ = self.cache.insert(tag.to_owned(), data.clone());
    }

    fn clear(&mut self) {
        self.cache.clear();
    }

    fn has(&self, tag: &str) -> bool {
        self.cache.contains_key(tag)
    }
}
