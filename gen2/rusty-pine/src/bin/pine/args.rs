use clap::{Parser, Subcommand};
use rusty_pine::analyze::ServerParams;
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
    /// Creates a context
    CreateContext(ContextParams),
    UseContext {
        name: String,
    },
}

#[derive(clap::Args, Debug)]
pub struct ContextParams {
    /// You can reuse your context by referencing this name
    name: String,

    /// Hostname or ip address of the MySQL server (without the port number)
    #[arg(long = "host")]
    hostname_or_ip: String,
    /// Port number of the database server
    #[arg(short, long)]
    port: u16,
    /// Username
    #[arg(short, long)]
    username: String,
    /// Default database from the server
    #[arg(short, long)]
    default_database: String,
    /// Use the new context
    #[arg(long = "use")]
    pub use_it: bool,
}

impl From<ContextParams> for Context {
    fn from(value: ContextParams) -> Self {
        Context {
            name: value.name.into(),
            server_params: ServerParams {
                hostname: value.hostname_or_ip,
                port: value.port,
                user: value.username,
            },
            default_database: value.default_database,
        }
    }
}
