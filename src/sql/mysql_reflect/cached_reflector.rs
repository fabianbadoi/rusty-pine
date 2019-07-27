use crate::sql::Reflector;
use crate::sql::structure::Database;
use crate::error::PineError;
use std::cell::RefCell;
use crate::cache::Cache;

struct CachedReflector<T, U> {
    inner: T,
    cache: RefCell<U>,
    tag: String,
    // ...
}

impl<T, U> Reflector for CachedReflector<T, U>
where T: Reflector, U: Cache<Vec<Database>> {
    fn analyze(&self) -> Result<Vec<Database>, PineError> {
        let from_cache = self.cache.borrow().get(&self.tag);

        match from_cache {
            Some(value) => Ok(value),
            None => {
                let analysis = self.inner.analyze()?;

                self.cache.borrow_mut().set(&self.tag, &analysis);

                Ok(analysis)
            }
        }
    }
}

impl<T, U> CachedReflector<T, U> {
    fn new<V>(inner: T, cache: U, tag: V) -> Self where V: Into<String> {
        let tag = tag.into();
        let cache = RefCell::new(cache);

        CachedReflector { inner, cache, tag }
    }
}

#[cfg(test)] 
mod tests {
    use crate::cache::MemoryCache;
    use super::*;
    use std::cell::{ Cell};
    use std::default::Default;

    #[derive(Default)]
    struct MockReflector {
        was_called: Cell<bool>,
    }

    impl MockReflector {
        fn new() -> MockReflector {
            MockReflector { was_called: Cell::new(false) }
        }
    }

    impl Reflector for MockReflector {
        fn analyze(&self) -> Result<Vec<Database>, PineError> {
            self.was_called.replace(true);

            Ok(Vec::new())
        }
    }

    #[test]
    fn read_from_inner_if_not_cached() {
        let reflector = CachedReflector::new(MockReflector::default(), MemoryCache::<Vec<Database>>::default(), "debug");

        let _ = reflector.analyze();

        assert!(reflector.inner.was_called.get());
    }

    #[test]
    fn return_from_cache_if_available() {
        let mut cache = MemoryCache::<Vec<Database>>::default();
        cache.set("debug", &Vec::new());

        let reflector = CachedReflector::new(MockReflector::default(), cache, "debug");

        let _ = reflector.analyze();

        assert!(!reflector.inner.was_called.get());
    }

    #[test]
    fn values_get_cached() {
        let reflector = CachedReflector::new(MockReflector::default(), MemoryCache::<Vec<Database>>::default(), "debug");

        let _ = reflector.analyze();

        assert!(reflector.inner.was_called.get());
        assert!(reflector.cache.borrow().get("debug").is_some());
    }
}
