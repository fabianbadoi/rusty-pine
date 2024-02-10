use clap::Parser;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Password;
use mysql::{Opts, OptsBuilder};

// Uses clap::Parser to give some really nice CLI options.
//
// I want to use #[derive], and unfortunately it does not support input validation. This
// is why I also have a MySqlConnectionArgs struct.
// The /// docs are converted into --help text
/// Analyzes a database and stores the result locally. Only pre-analyzed database are available
/// for querying via Pine.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct ProgramArgs {
    /// Hostname to connect to: example.com:port
    pub host: String,
    /// Username
    pub username: String,
}

/// Holder for common MySql connection args. Some of these are read from CLI input,
/// others are read interactively.
#[derive(Default)]
pub struct MySqlConnectionArgs {
    pub hostname_or_ip: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

impl MySqlConnectionArgs {
    pub fn cli_interactive() -> Self {
        // Reading the CLI args first assures that if the app is called with --help we get
        // the desired effect. If we were to ask for the password first, you would only see
        // help text after typing something in.
        let cli_args = Self::read_from_cli();

        // the password is the only part that we have to read interactively
        // we use the dialoguer library for this bec
        let password = Password::with_theme(&ColorfulTheme::default())
            .with_prompt("Password: ")
            .interact()
            .unwrap();

        MySqlConnectionArgs {
            password,
            ..cli_args
        }
    }

    fn read_from_cli() -> Self {
        let ProgramArgs { host, username } = ProgramArgs::parse();
        // Using the same param for both the host and port just because it's more convenient
        // to call the application that way.
        let (hostname_or_ip, port) = split_host(host);

        MySqlConnectionArgs {
            hostname_or_ip,
            port,
            username,
            ..Default::default()
        }
    }
}

impl From<&MySqlConnectionArgs> for Opts {
    fn from(value: &MySqlConnectionArgs) -> Self {
        OptsBuilder::new()
            .ip_or_hostname(Some(value.hostname_or_ip.clone()))
            .tcp_port(value.port)
            .user(Some(value.username.clone()))
            .pass(Some(value.password.clone()))
            .into()
    }
}

fn split_host(host: String) -> (String, u16) {
    let (host, port) = host.split_once(':').unwrap_or((host.as_str(), "3306"));

    let port = {
        let as_u16 = port.parse::<u16>();

        as_u16.expect("Host port invalid")
    };

    (host.to_owned(), port)
}
