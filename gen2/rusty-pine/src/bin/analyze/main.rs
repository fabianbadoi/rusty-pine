use clap::Parser;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{MultiSelect, Password};
use mysql::{Opts, OptsBuilder, Pool};
use rusty_pine::analyze::{
    describe_table, list_databases, list_tables, Database, Server, Table, TableName,
};

// Uses clap::Parser to give some really nice CLI options.
//
// I want to use #[derive], and unfortunately it does not support input validation. This
// is why I also have a MySqlConnectionArgs struct.
// The /// docs are converted into --help text
/// Analyzes a database and stores the result locally. Only pre-analyzed database are available
/// for querying via Pine.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct ProgramArgs {
    /// Hostname to connect to: example.com:port
    host: String,
    /// Username
    username: String,
}

/// Holder for common MySql connection args. Some of these are read from CLI input,
/// others are read interactively.
#[derive(Default)]
struct MySqlConnectionArgs {
    hostname_or_ip: String,
    port: u16,
    username: String,
    password: String,
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

impl From<MySqlConnectionArgs> for Opts {
    fn from(value: MySqlConnectionArgs) -> Self {
        OptsBuilder::new()
            .ip_or_hostname(Some(value.hostname_or_ip))
            .tcp_port(value.port)
            .user(Some(value.username))
            .pass(Some(value.password))
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

fn main() {
    let pool = Pool::new::<Opts, _>(MySqlConnectionArgs::cli_interactive().into())
        .expect("Could not connect to database");
    let mut connection = pool.get_conn().unwrap(); // TODO expect

    let databases = list_databases(&mut connection).expect("Could not read databases");

    let selection = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select databases")
        .items(&databases)
        .interact()
        .unwrap(); // TODO expect

    // Iterating over selection would have taken fewer iterations, but we cannot move data
    // out of databases like that. We would have had to clone the strings.
    // This way, we take the entire databases vector, and then we're left with just the databases
    // we want.
    let selected_databases: Vec<_> = databases
        .into_iter()
        .enumerate()
        .filter_map(|(index, item)| {
            if selection.contains(&index) {
                Some(item)
            } else {
                None
            }
        })
        .collect();

    let server = Server {
        hostname: "fdsa".to_string(),
        port: 3306,
        user: "fabi".to_string(),
        databases: selected_databases
            .iter()
            .map(|db_name| {
                let tables = list_tables(&mut connection, db_name).expect("Cannot read table"); // TODO

                let tables = tables
                    .into_iter()
                    .map(|table_name| {
                        let create = describe_table(&mut connection, db_name, &table_name)
                            .expect("Cannot read table description"); // TODO

                        Table::from_sql_string(&create)
                    })
                    .filter(Result::is_ok) // todo
                    .map(Result::unwrap) // TODO
                    .collect();

                Database {
                    name: TableName(db_name.as_str().to_string()),
                    tables,
                }
            })
            .collect(),
    };

    println!("{:#?}", server);
}
