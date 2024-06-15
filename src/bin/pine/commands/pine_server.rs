//! Runs a local server to interact with https://try.pine-lang.org/
//!
//! https://try.pine-lang.org/ is just a UI for a local server. All requests are
//! sent to localhost:33333
use axum::http::{HeaderValue, Method, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use rusty_pine::context::ContextName;
use serde::Serialize;
use tokio::runtime::Builder;
use tower_http::cors::CorsLayer;

static TRY_PINE_LANG_ORG_LOCAL_SERVER: &str = "0.0.0.0:33333";

pub fn run() {
    // Tokio is our async runtime. In rust you need to have one of these handle your
    // async code.
    // We're using single threaded runtime because that's enough for a server which
    // I expect to be called never times per second.
    let tokio = Builder::new_current_thread()
        .enable_io()
        .build()
        .expect("Cannot build tokio runtime");

    let app = Router::new()
        .route("/api/v1/connection", get(get_current_context))
        .layer(
            CorsLayer::new()
                .allow_origin("https://try.pine-lang.org".parse::<HeaderValue>().unwrap())
                .allow_headers(["*".parse().unwrap()])
                .allow_methods([Method::GET]),
        );

    tokio.block_on(async {
        let listener = tokio::net::TcpListener::bind(TRY_PINE_LANG_ORG_LOCAL_SERVER)
            .await
            .expect("Cannot start pine server: network bind failed.");
        axum::serve(listener, app)
            .await
            .expect("Cannot start pine server: cannot run app");
    });
}

async fn get_current_context() -> impl IntoResponse {
    // todo unwraps
    let current_context = ContextName::current().unwrap();
    // let context: Context = cache::read(&current_context).unwrap();

    (
        StatusCode::OK,
        Json(ContextResult {
            result: InnerResult {
                connection_id: current_context.as_ref().to_string(),
                metadata: vec![],
            },
        }),
    )
}

#[derive(Serialize)]
struct ContextResult {
    result: InnerResult,
}

#[derive(Serialize)]
struct InnerResult {
    #[serde(rename = "connection-id")]
    connection_id: String,
    metadata: Vec<usize>,
}
