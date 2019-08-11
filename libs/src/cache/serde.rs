use super::Cache;
use ::serde::{Deserialize, Serialize};

pub struct SerializedCache<T> {
    inner: T,
}

impl<T> SerializedCache<T> {
    pub fn wrap(inner: T) -> SerializedCache<T> {
        SerializedCache { inner }
    }
}

impl<T, I> Cache<T> for SerializedCache<I>
where
    I: Cache<Vec<u8>>,
    for<'a> T: Serialize + Deserialize<'a>,
{
    fn get(&self, tag: &str) -> Option<T> {
        let cached = self.inner.get(tag);

        match cached {
            Some(item) => serde_json::from_slice(item.as_slice())
                .map(Some)
                .unwrap_or(None),
            None => None,
        }
    }

    fn set(&mut self, tag: &str, data: &T) {
        let serialized = serde_json::to_vec(data).expect("failed to write data");
        self.inner.set(tag, &serialized)
    }

    fn clear(&mut self) {
        self.inner.clear()
    }

    fn has(&self, tag: &str) -> bool {
        // will return true, even if the value will fail to deserialie
        self.inner.has(tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::MemoryCache;

    fn new() -> SerializedCache<MemoryCache<Vec<u8>>> {
        SerializedCache {
            inner: Default::default(),
        }
    }

    #[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
    struct Point {
        x: i64,
        y: i64,
    }

    #[test]
    fn can_set_and_get() {
        let mut cache = new();
        let x = Point { x: -42, y: 42 };

        cache.set("test", &x);
        let deserialied_x: Option<Point> = cache.get("test");

        assert!(deserialied_x.is_some());
        assert_eq!(x, deserialied_x.unwrap());
    }
}
