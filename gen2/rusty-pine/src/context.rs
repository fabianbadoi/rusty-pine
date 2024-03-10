use crate::analyze::ServerParams;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize)]
pub struct Context {
    pub name: ContextName,
    pub server_params: ServerParams,
    pub default_database: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ContextName(String);

impl From<String> for ContextName {
    fn from(value: String) -> Self {
        ContextName(value)
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
