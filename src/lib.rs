// I don't really care, and it's not important for this project
#![allow(clippy::result_large_err)]

pub mod cache;
pub mod context;
mod engine;
mod error;

pub use engine::render;

pub mod analyze {
    pub use crate::engine::sql::querying::{
        mariadb, postgres, Analyzer, Connection, MariaDBConnection, SchemaObjectName,
    };
    pub use crate::engine::sql::structure::*;
    pub use crate::engine::sql::DbStructureParsingContext;
}

pub use error::{Error, InternalError};
