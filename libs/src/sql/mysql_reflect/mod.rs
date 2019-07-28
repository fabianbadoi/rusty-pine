pub mod cached_reflector;
pub mod connection;
pub mod live_analysis;
pub mod parsing;

use crate::cache::{make_cahe, DefaultCache};
use crate::error::PineError;
use cached_reflector::CachedReflector;
use connection::LiveConnection;
use live_analysis::{MySqlReflector, MySqlTableParser};

pub type DefaultReflector =
    CachedReflector<MySqlReflector<LiveConnection, MySqlTableParser>, DefaultCache>;

pub fn connect(
    user: &str,
    password: &str,
    host: &str,
    port: u16,
) -> Result<DefaultReflector, PineError> {
    let connection = LiveConnection::new(user, password, host, port)?;

    Ok(DefaultReflector::wrap(
        MySqlReflector::for_connection(connection),
        make_cahe(".cache/rusty-pine/cache/v1"),
        format!("{}@{}_{}", user, host, port),
    ))
}
