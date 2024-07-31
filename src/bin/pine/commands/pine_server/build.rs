use crate::commands::pine_server::build_dto::{
    BuildRequest, BuildResult, Hints, State, Table, TableHint,
};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use dialoguer::Input;
use log::info;
use rusty_pine::analyze::Server;
use rusty_pine::context::{Context, ContextName};
use rusty_pine::{build_query, cache, render, Introspective, LimitHolder};

pub async fn build(Json(payload): Json<BuildRequest>) -> impl IntoResponse {
    info!(target: "build", "Building expression: {}", payload.expression);

    // TODO bad code
    let input = payload.expression.trim().trim_end_matches('|');

    let current_context = ContextName::current().unwrap();
    let context: Context = cache::read(&current_context).unwrap();
    let server: Server = cache::read(&context.server_params).unwrap();

    let result = build_query(input, &server);
    let query = result.unwrap();

    (
        StatusCode::OK,
        Json(BuildResult {
            connection_id: current_context.to_string(),
            version: "1.1.0-rusty",
            query: query.to_string(),
            state: State {
                hints: Hints {
                    table: server
                        .neighbors((&query.from).as_holder())
                        .unwrap()
                        .into_iter()
                        .map(|fk| TableHint {
                            schema: "default".to_string(),
                            table: fk.to.table.0,
                            join_using_column: fk
                                .to
                                .key
                                .columns
                                .iter()
                                .map(ToString::to_string)
                                .collect::<Vec<_>>()
                                .join(", "),
                        })
                        .collect(),
                },
                table_count: 1 + query.joins.len(),
                conditions: vec![],
                limit: match query.limit.it {
                    LimitHolder::Implicit() => None,
                    _ => Some(format!("{}", query.limit)),
                },
                pending_count: 0,
                columns: vec![],
                joins: Default::default(),
                tables: server
                    .default_database()
                    .unwrap()
                    .tables
                    .keys()
                    .map(|table_name| Table {
                        name: table_name.to_string(),
                        alias: table_name.to_string(),
                    })
                    .collect(),
                context: current_context.to_string(),
                connection_id: current_context.to_string(),
            },
        }),
    )
}
