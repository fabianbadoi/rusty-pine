use crate::cache::Cache;
use crate::error::PineError;
use crate::sql::structure::Database;
use crate::sql::Reflector;
use log::info;
use std::cell::RefCell;

/// This reflector writes to cache
pub struct CachingReflector<T, U> {
    inner: T,
    cache: RefCell<U>,
    tag: String,
}

impl<T, U> Reflector for CachingReflector<T, U>
where
    T: Reflector,
    U: Cache<Vec<Database>>,
{
    fn analyze(&self) -> Result<Vec<Database>, PineError> {
        info!("Starting analysis");

        let from_cache = self.cache.borrow().get(&self.tag);

        match from_cache {
            Some(value) => {
                info!("Using cahe");
                Ok(value)
            }
            None => {
                info!("Using live data");

                let analysis = self.inner.analyze()?;

                self.cache.borrow_mut().set(&self.tag, &analysis);

                Ok(analysis)
            }
        }
    }
}

impl<T, U> CachingReflector<T, U> {
    pub fn wrap<V>(inner: T, cache: U, tag: V) -> Self
    where
        V: Into<String>,
    {
        let tag = tag.into();
        let cache = RefCell::new(cache);

        CachingReflector { inner, cache, tag }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::MemoryCache;
    use std::cell::Cell;
    use std::default::Default;

    #[derive(Default)]
    struct MockReflector {
        was_called: Cell<bool>,
    }

    impl Reflector for MockReflector {
        fn analyze(&self) -> Result<Vec<Database>, PineError> {
            self.was_called.replace(true);

            Ok(Vec::new())
        }
    }

    #[test]
    fn read_from_inner_if_not_cached() {
        let reflector = CachingReflector::wrap(
            MockReflector::default(),
            MemoryCache::<Vec<Database>>::default(),
            "debug",
        );

        let _ = reflector.analyze();

        assert!(reflector.inner.was_called.get());
    }

    #[test]
    fn return_from_cache_if_available() {
        let mut cache = MemoryCache::<Vec<Database>>::default();
        cache.set("debug", &Vec::new());

        let reflector = CachingReflector::wrap(MockReflector::default(), cache, "debug");

        let _ = reflector.analyze();

        assert!(!reflector.inner.was_called.get());
    }

    #[test]
    fn values_get_cached() {
        let reflector = CachingReflector::wrap(
            MockReflector::default(),
            MemoryCache::<Vec<Database>>::default(),
            "debug",
        );

        let _ = reflector.analyze();

        assert!(reflector.inner.was_called.get());
        assert!(reflector.cache.borrow().get("debug").is_some());
    }
}
