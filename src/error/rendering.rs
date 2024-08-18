use crate::engine::{Position, QueryBuildError, RenderingError, Source};
use colored::Colorize;
use std::fmt::{Display, Formatter};

impl Display for RenderingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{input}\n\
             {underline} {message}",
            input = self.input,
            underline = self.underline().red().bold(),
            message = self.build_error.message().red().bold(),
        )?;

        write!(f, "\n\n{details}", details = self.build_error)?;

        Ok(())
    }
}

impl RenderingError {
    fn underline(&self) -> String {
        let input_ranges = self.build_error.input_ranges();

        if input_ranges.is_empty() {
            "^".repeat(self.input.len())
        } else {
            let mut buffer = "".to_string();

            for Position { start, end } in input_ranges {
                // This will look like shit or even panic! if the ranges are messed up.
                buffer.push_str(&" ".repeat(start - buffer.len()));
                buffer.push_str(&"^".repeat(end - start));
            }

            buffer
        }
    }
}

impl QueryBuildError {
    fn input_ranges(&self) -> Vec<Position> {
        let sources = match self {
            QueryBuildError::InvalidPostgresConfig
            | QueryBuildError::DefaultDatabaseNotFound(_) => return vec![], // It's not found in the input
            QueryBuildError::DatabaseNotFound(db) => vec![db.source],
            QueryBuildError::TableNotFound(table) => vec![table.source],
            QueryBuildError::InvalidForeignKey { from, to } => vec![from.source, to.source],
            QueryBuildError::JoinNotFound { from, to } => vec![from.source, to.source],
            QueryBuildError::InvalidImplicitIdCondition(table, _, value) => {
                vec![table.source, value.source]
            }
        };

        let mut positions: Vec<_> = sources
            .iter()
            .filter_map(|source| match source {
                Source::Input(position) => Some(position),
                _ => None,
            })
            .cloned()
            .collect();

        // Sometimes the from and to tables are reversed, which would lead to some
        // really weird underlining.
        positions.sort_by_key(|position| position.start);

        positions
    }

    fn message(&self) -> &str {
        match self {
            QueryBuildError::InvalidPostgresConfig => "Postgres context is misconfigured",
            QueryBuildError::DefaultDatabaseNotFound(_) => "Default database not found",
            QueryBuildError::DatabaseNotFound(_) => "Database not found",
            QueryBuildError::TableNotFound(_) => "Table not found",
            QueryBuildError::InvalidForeignKey { .. } => "Invalid foreign key between tables",
            QueryBuildError::JoinNotFound { .. } => "Can't join tables",
            QueryBuildError::InvalidImplicitIdCondition(..) => "Can't use implicit id filtering",
        }
    }
}

impl Display for QueryBuildError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryBuildError::InvalidPostgresConfig => write!(
                f,
                "Your postgres context is corrupted, it is missing the default schema.\n\
                Your only option is to recreate it."
            ),
            QueryBuildError::DefaultDatabaseNotFound(server) => write!(
                f,
                "The {server} server is configured to use {default_db} as a default database \
                 but this database does not exist.\n\
                 Either switch to another context using {switch_context} or try recreating the \
                 current one.",
                server = format!("{}", server).yellow().bold(),
                default_db = format!("{}", server.database).yellow().bold(),
                switch_context = "pine use-context <context name>".green().bold(),
            ),
            QueryBuildError::DatabaseNotFound(database) => write!(
                f,
                "The {database} database is not present in your context. \n\
                 If your context is out of date, re-run {pine_analyze}. \n\
                 You can also try switching to another context using {switch_context}.",
                database = format!("{}", database).yellow().bold(),
                pine_analyze = "pine analyze".green().bold(),
                switch_context = "pine use-context <context name>".green().bold(),
            ),
            QueryBuildError::TableNotFound(table) => write!(
                f,
                "The {table} table is not present in your context. \n\
                 If your context is out of date, re-run {pine_analyze}. \n\
                 You can also try switching to another context using {switch_context}.",
                table = format!("{}", table).yellow().bold(),
                pine_analyze = "pine analyze".green().bold(),
                switch_context = "pine use-context <context name>".green().bold(),
            ),
            QueryBuildError::InvalidForeignKey { from, to } => write!(
                f,
                "The foreign key linking {from} to {to} is not usable.\n\
                 If your context is out of date, re-run {pine_analyze}.",
                from = format!("{}", from).yellow().bold(),
                to = format!("{}", to).yellow().bold(),
                pine_analyze = "pine analyze".green().bold(),
            ),
            QueryBuildError::JoinNotFound { from, to } => write!(
                f,
                "No foreign key linking {from} to {to} found.\n\
                 If your context is out of date, re-run {pine_analyze}.",
                from = format!("{}", from).yellow().bold(),
                to = format!("{}", to).yellow().bold(),
                pine_analyze = "pine analyze".green().bold(),
            ),
            QueryBuildError::InvalidImplicitIdCondition(table, primary_key, value) => write!(
                f,
                "Cannot use implicit id conditions for `{table} because it has a composite primary key.\n\
                 You are trying to filter for `[{columns}] = {value}`, but that does not work.\n\
                 \n\
                 If your context is out of date, re-run {pine_analyze}.",
                table = format!("{}", table).yellow().bold(),
                value = format!("{}", value).yellow().bold(),
                columns = primary_key.columns.iter().map(|c| c.0.as_str())
                    .collect::<Vec<_>>().join(", ").yellow().bold(),
                pine_analyze = "pine analyze".green().bold(),
            ),
        }
    }
}
