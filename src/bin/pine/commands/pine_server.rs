use axum::routing::get;
use axum::Router;
use rusty_pine::Error;
use tokio::runtime::Builder;

pub fn run() -> Result<(), Error> {
    // TODO explain why building instead of using the attr
    let tokio = Builder::new_current_thread().enable_all().build().unwrap(); // TODO

    let app = Router::new().route("/api/v1/connection", get(|| async { "Hello, World!" }));

    tokio.block_on(async {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:33333")
            .await
            .unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    todo!()
}
