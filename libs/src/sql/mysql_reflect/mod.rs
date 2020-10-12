pub mod cached_reflector;
pub mod connection;
pub mod live_analysis;
pub mod parsing;

use crate::cache::{make_cache, Cache, DefaultCache};
use crate::error::PineError;
use cached_reflector::CachingReflector;
use connection::{LiveConnection, OfflineConnection};
use live_analysis::{MySqlReflector, MySqlTableParser};
use log::info;

pub type CacheBuilder =
    CachingReflector<MySqlReflector<LiveConnection, MySqlTableParser>, DefaultCache>;

pub type OfflineReflector =
    CachingReflector<MySqlReflector<OfflineConnection, MySqlTableParser>, DefaultCache>;

/// Clear cache before connecting
pub fn connect_fresh(
    user: &str,
    password: &str,
    host: &str,
    port: u16,
) -> Result<CacheBuilder, PineError> {
    info!("Setting up uncached connection to {}@{}", user, host);

    let connection = LiveConnection::new(user, password, host, port)?;
    let mut cache = make_reflector_cache();

    // it doesn't matter what cache type this is, it will clear everything anyway
    (&mut cache as &mut dyn Cache<u8>).clear();

    Ok(CacheBuilder::wrap(
        MySqlReflector::for_connection(connection),
        cache,
        format!("{}@{}_{}", user, host, port),
    ))
}

/// Offline reflector
pub fn offline(user: &str, host: &str, port: u16) -> OfflineReflector {
    info!("Setting up offline use of {}@{}", user, host);

    let cache = make_reflector_cache();

    OfflineReflector::wrap(
        MySqlReflector::for_connection(OfflineConnection),
        cache,
        format!("{}@{}_{}", user, host, port),
    )
}

fn make_reflector_cache() -> DefaultCache {
    make_cache("cache/v1")
}
