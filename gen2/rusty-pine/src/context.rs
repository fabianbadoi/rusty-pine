use crate::analyze::ServerParams;
use crate::cache;
use crate::cache::Cacheable;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize)]
pub struct Context {
    pub name: ContextName,
    pub server_params: ServerParams,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ContextName(String);

impl ContextName {
    pub fn current() -> Result<ContextName, crate::Error> {
        // All context names use the cache key, because that's how we save the current context.
        // Reading a context named "any" will just get us the current context.
        cache::read(&ContextName("any".to_string()).cache_key())
    }
}

impl From<String> for ContextName {
    fn from(value: String) -> Self {
        ContextName(value)
    }
}

impl From<&str> for ContextName {
    fn from(value: &str) -> Self {
        ContextName(value.to_string())
    }
}

impl From<ContextName> for String {
    fn from(value: ContextName) -> Self {
        value.0
    }
}

impl Display for ContextName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
