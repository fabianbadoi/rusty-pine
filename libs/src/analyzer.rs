use crate::sql::analyzer::{connect, connect_fresh};
use crate::sql::DefaultReflector;
use crate::sql::Reflector;
use crate::error::PineError;
use crate::config::Config;

pub struct Analyzer {
    inner: DefaultReflector,
}

impl Analyzer {
    pub fn connect(config: &Config) -> Result<Analyzer, PineError> {
        Ok(Analyzer {
            inner: connect(&config.user, &config.password, &config.host, config.port)?,
        })
    }

    pub fn connect_fresh(config: &Config) -> Result<Analyzer, PineError> {
        Ok(Analyzer {
            inner: connect_fresh(&config.user, &config.password, &config.host, config.port)?,
        })
    }

    pub fn save(&self) -> Result<(), PineError> {
        self.inner.analyze().map(|_| ())
    }
}
