pub trait Cache<T> {
    fn get(&self, tag: &str) -> Option<T>;
    fn set(&mut self, tag: &str, data: &T);
}


#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::cell::RefCell;

#[cfg(test)]
#[derive(Default)]
pub struct MemoryCache<T> {
    cache: RefCell<HashMap<String, T>>,
}

#[cfg(test)]
impl<T> Cache<T> for MemoryCache<T> {
    fn get(&self, tag: &str) -> Option<T> {
        self.cache.borrow_mut().remove(tag)
    }

    fn set(&mut self, tag: &str, data: &T) {
        let _ = self.cache.borrow_mut().insert(tag.to_owned(), unsafe { std::mem::transmute_copy(data) });
    }
}
