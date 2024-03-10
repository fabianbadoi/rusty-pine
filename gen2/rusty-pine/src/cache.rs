//! Our caching system is used to make sure we don't have to re-analyze entire databases
//! for each call.
//!
//! To use the cache system, implement the Cacheable and CacheKey traits, then you can
//! use the read() and write() functions.
use crate::analyze::{Server, ServerParams};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

/// You need a cache key in order to read something for cache. Why not just use a string
/// as a cache key? Find out in part 2...
pub trait CacheKey {
    fn as_path(&self) -> String;
}

/// Anything that can be cached needs to implement this trait.
///
/// Part 2: Cacheable has an associated type so that can always pair up a struct to be cached
/// with its cache key. The benefit of doing things this way is some compile time protection:
/// ```
/// use rusty_pine::analyze::{Server, ServerParams};
/// use rusty_pine::cache::read;
///
/// let data: Server = read(
/// //        ^^^^^^ -- since Server::CacheKey == ServerParams, trying to use a different key type
/// //                  will cause a compile error; trying to read() from ServerParams into
/// //                  any struct that does not have it as a CacheKey will likewise fail
/// //                  to compile. So we the compiler now makes sure we read and write to our
/// //                  cache in a type safe manner.
///     &ServerParams { hostname: "".to_string(), port: 0, user: "".to_string()}
/// )?;
/// ```
pub trait Cacheable {
    // Binding the cache key type to the type that it will point to helps with type safety.
    type CacheKey;

    // Using CacheKey instead of a String means we use the exact same thing in both
    // the read() and write() functions.
    fn cache_key(&self) -> &Self::CacheKey;
}

pub fn read<D, K>(cache_key: &K) -> Result<D, crate::Error>
where
    // adding the Cacheable trait constraint is optional, but makes sure we can only read() to
    // structs that are actually meant to be read to from that key type.
    // So you can't do `let a: StructA = read(cache_key_that_is_used_for_StructB)`.
    D: Cacheable<CacheKey = K> + DeserializeOwned,
    K: CacheKey,
{
    let file_location = get_cache_path(cache_key.as_path().as_str())?;

    let data = serde_json::from_reader(fs::File::open(file_location)?)?;

    Ok(data)
}

pub fn write<D, K>(data: &D) -> Result<(), crate::Error>
where
    D: Cacheable<CacheKey = K> + Serialize,
    K: CacheKey,
{
    let file_location = get_cache_path(data.cache_key().as_path().as_str())?;

    let data = serde_json::to_string(&data)?;

    fs::write(file_location, data)?;

    Ok(())
}

fn get_cache_path(cache_key: &str) -> Result<PathBuf, crate::Error> {
    let mut location = require_cache_folder()?;

    location.push(cache_key);

    Ok(location)
}

fn require_cache_folder() -> Result<PathBuf, crate::Error> {
    let home = std::env::var("HOME")?;

    let mut path = PathBuf::from(home);
    path.push(".cache");
    path.push("rusty-pine");
    path.push("cache");
    path.push("v2");

    // we have to make sure it exists, right?
    fs::create_dir_all(&path)?;

    Ok(path)
}

// Please dump all impls here so we keep the rest of the code base clean.

impl Cacheable for Server {
    type CacheKey = ServerParams;

    fn cache_key(&self) -> &Self::CacheKey {
        &self.params
    }
}

impl CacheKey for ServerParams {
    fn as_path(&self) -> String {
        format!("server-{}-{}-{}", self.hostname, self.port, self.user)
    }
}
