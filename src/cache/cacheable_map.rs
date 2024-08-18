//! Some of the types require special serialization because they can't be mapped 1:1 to JSON
//! objects. For example Map<{x: usize, y: usize}, String> can't be converted directly to JSON
//! because its keys are not strings.

use serde::de::{Error as DeError, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeSeq, Serializer};
use serde::{Deserialize, Deserializer};
use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct CacheableMap<K, V> {
    inner: HashMap<K, V>,
}

struct CacheableMapVisitor<K, V> {
    marker: PhantomData<CacheableMap<K, V>>,
}

impl<K, V> CacheableMap<K, V>
where
    K: Eq + Hash,
{
    pub fn get(&self, key: &K) -> Option<&V> {
        self.inner.get(key)
    }

    pub fn iter(&self) -> Iter<K, V> {
        self.inner.iter()
    }
}

impl<K, V> From<HashMap<K, V>> for CacheableMap<K, V> {
    fn from(value: HashMap<K, V>) -> Self {
        CacheableMap { inner: value }
    }
}

impl<K, V> Serialize for CacheableMap<K, V>
where
    K: Serialize + Debug,
    V: Serialize + Debug,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // TODO explain
        let mut list = serializer.serialize_seq(Some(self.inner.len()))?;
        for (k, v) in &self.inner {
            list.serialize_element(&(k, v))?;
        }
        list.end()
    }
}

impl<K, V> CacheableMapVisitor<K, V> {
    fn new() -> Self {
        CacheableMapVisitor {
            marker: PhantomData,
        }
    }
}

impl<'de, K, V> Deserialize<'de> for CacheableMap<K, V>
where
    K: Deserialize<'de> + Eq + Hash,
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(CacheableMapVisitor::new())
    }
}

impl<'de, K, V> Visitor<'de> for CacheableMapVisitor<K, V>
where
    K: Deserialize<'de> + Eq + Hash,
    V: Deserialize<'de>,
{
    type Value = CacheableMap<K, V>;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "a CacheableMap<K, V>")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut map: HashMap<K, V> = HashMap::new();

        while let Some((k, v)) = seq.next_element()? {
            let previous = map.insert(k, v);

            if previous.is_some() {
                return Err(DeError::custom(
                    "duplicate field when trying to deserialize CacheableMap",
                ));
            }
        }

        Ok(CacheableMap { inner: map })
    }
}
