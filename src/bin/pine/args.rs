use clap::{Parser, Subcommand, ValueEnum};
use rusty_pine::analyze::{DBType as AnalyzeDBType, DatabaseName, ServerParams};
use rusty_pine::context::Context;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Creates a context.
    ///
    /// Contexts allow the pine command to switch between different database connections.
    CreateContext(ContextParams),
    /// Selects an existing context.
    UseContext { name: String },
    /// List available contexts.
    ListContexts,
    /// Analyzes the database of the current context, updating the known structure used when
    /// analyzing pines.
    Analyze,
    /// Runs a pine server that can be used with https://try.pine-lang.org/
    PineServer,
    /// Translates a single pine to SQL using the current context.
    Translate { input: String },
}

#[derive(clap::Args, Debug)]
pub struct ContextParams {
    /// You can reuse your context by referencing this name
    name: String,

    ///. Database type: PostgresSQL, MariaDB.
    #[arg(long = "type")]
    db_type: DBType,
    /// Hostname or ip address of the MySQL server (without the port number)
    #[arg(value_enum, long = "host")]
    hostname_or_ip: String,
    /// Port number of the database server
    #[arg(short, long)]
    port: u16,
    /// Username
    #[arg(short, long)]
    username: String,
    /// Database. Will be used for the database to scan or the default database for MariaDB.
    #[arg(short, long)]
    database: String,
    /// When using Postgres, this is the schema used when the user does not specify one.
    #[arg(short = 's', long)]
    default_schema: Option<String>,
    /// Use the new context
    #[arg(long = "use")]
    pub use_it: bool,
}

#[derive(Debug, ValueEnum, Clone)]
pub enum DBType {
    MariaDB,
    PostgresSQL,
}

impl From<ContextParams> for Context {
    fn from(value: ContextParams) -> Self {
        Context {
            name: value.name.into(),
            server_params: ServerParams {
                db_type: value.db_type.into(),
                hostname: value.hostname_or_ip,
                port: value.port,
                user: value.username,
                database: value.database.into(),
                default_schema: value.default_schema.map(DatabaseName),
            },
        }
    }
}

impl From<DBType> for AnalyzeDBType {
    fn from(value: DBType) -> Self {
        match value {
            DBType::MariaDB => Self::MariaDB,
            DBType::PostgresSQL => Self::PostgresSQL,
        }
    }
}
