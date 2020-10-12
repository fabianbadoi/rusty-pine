use crate::config::Config;
use crate::error::PineError;
use crate::sql::analyzer::{connect_fresh, offline as inner_offline};
use crate::sql::structure::Database;
use crate::sql::CacheBuilder;
use crate::sql::OfflineReflector;
use crate::sql::Reflector;

pub struct Analyzer<T> {
    inner: T,
}

pub fn connect(config: &Config) -> Result<Analyzer<CacheBuilder>, PineError> {
    Ok(Analyzer {
        inner: connect_fresh(&config.user, &config.password, &config.host, config.port)?,
    })
}

pub fn offline(config: &Config) -> Result<Analyzer<OfflineReflector>, PineError> {
    Ok(Analyzer {
        inner: inner_offline(&config.user, &config.host, config.port),
    })
}

impl<T> Analyzer<T>
where
    T: Reflector,
{
    pub fn analyze(&self, db_name: &str) -> Result<Database, PineError> {
        let databases = self.inner.analyze()?;
        let all_db_names: Vec<_> = databases.iter().map(|db| db.name.clone()).collect();

        for database in databases {
            if database.name == db_name {
                return Ok(database);
            }
        }

        Err(PineError::from(format!(
            "Could not find database {:?}, try: {:?}",
            db_name, all_db_names
        )))
    }

    pub fn save(&self) -> Result<(), PineError> {
        self.inner.analyze().map(|_| ())
    }
}
