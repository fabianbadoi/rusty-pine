pub mod cached_reflector;
pub mod connection;
pub mod live_analysis;
pub mod parsing;

use crate::cache::{make_cache, Cache, DefaultCache};
use crate::error::PineError;
use cached_reflector::CachedReflector;
use connection::LiveConnection;
use live_analysis::{MySqlReflector, MySqlTableParser};
use log::info;

pub type DefaultReflector =
    CachedReflector<MySqlReflector<LiveConnection, MySqlTableParser>, DefaultCache>;

/// Connects and uses cache where possible
pub fn connect(
    user: &str,
    password: &str,
    host: &str,
    port: u16,
) -> Result<DefaultReflector, PineError> {
    let connection = LiveConnection::new(user, password, host, port)?;

    Ok(DefaultReflector::wrap(
        MySqlReflector::for_connection(connection),
        make_reflector_cache(),
        format!("{}@{}_{}", user, host, port),
    ))
}

/// Clear cache before connecting
pub fn connect_fresh(
    user: &str,
    password: &str,
    host: &str,
    port: u16,
) -> Result<DefaultReflector, PineError> {
    info!("Setting up uncached connection to {}@{}", user, host);

    let connection = LiveConnection::new(user, password, host, port)?;
    let mut cache = make_reflector_cache();

    // it doesn't matter what cache type this is, it will clear everything anyway
    (&mut cache as &mut Cache<u8>).clear();

    Ok(DefaultReflector::wrap(
        MySqlReflector::for_connection(connection),
        cache,
        format!("{}@{}_{}", user, host, port),
    ))
}

fn make_reflector_cache() -> DefaultCache {
    make_cache("cache/v1")
}
