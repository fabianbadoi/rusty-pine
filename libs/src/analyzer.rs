use crate::sql::analyzer::connect;
use crate::sql::DefaultReflector;
use crate::sql::Reflector;

use crate::error::PineError;

pub struct Analyzer {
    inner: DefaultReflector,
}

impl Analyzer {
    pub fn connect(
        user: &str,
        password: &str,
        host: &str,
        port: u16,
    ) -> Result<Analyzer, PineError> {
        Ok(Analyzer {
            inner: connect(user, password, host, port)?,
        })
    }

    pub fn save(&self) -> Result<(), PineError> {
        self.inner.analyze().map(|_| ())
    }
}
