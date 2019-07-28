use super::Cache;

use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Default)]
pub struct MemoryCache<T> {
    cache: RefCell<HashMap<String, T>>,
}

impl<T> Cache<T> for MemoryCache<T>
where
    T: Clone,
{
    fn get(&self, tag: &str) -> Option<T> {
        self.cache.borrow().get(tag).cloned()
    }

    fn set(&mut self, tag: &str, data: &T) {
        let _ = self.cache.borrow_mut().insert(tag.to_owned(), data.clone());
    }

    fn clear(&mut self) {
        self.cache.borrow_mut().clear();
    }
}
