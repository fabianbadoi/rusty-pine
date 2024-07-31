//! Runs a local server to interact with https://try.pine-lang.org/
//!
//! https://try.pine-lang.org/ is just a UI for a local server. All requests are
//! sent to localhost:33333
use axum::http::{HeaderValue, Method, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use build_dto::BuildRequest;
use log::info;
use rusty_pine::analyze::Server;
use rusty_pine::context::{Context, ContextName};
use rusty_pine::{cache, render};
use serde::{Deserialize, Serialize};
use tokio::runtime::Builder;
use tower_http::cors::CorsLayer;

mod build;
mod build_dto;

static TRY_PINE_LANG_ORG_LOCAL_SERVER: &str = "127.0.0.1:33333";

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
        .route("/api/v1/build", post(build::build))
        .layer(
            CorsLayer::new()
                .allow_origin("https://try.pine-lang.org".parse::<HeaderValue>().unwrap())
                .allow_headers(["*".parse().unwrap()])
                .allow_methods([Method::GET]),
        );

    info!("Listening on {TRY_PINE_LANG_ORG_LOCAL_SERVER}");

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

    (
        StatusCode::OK,
        Json(ContextResult {
            result: InnerResult {
                connection_id: current_context.as_ref().to_string(),
                version: "1.1.0-rusty",
                metadata: vec![],
            },
        }),
    )
}

/*
{
  "connection-id": "host.docker.internal",
  "version": "0.5.4",
  "query": "\nSELECT t_1.* FROM \"test\" AS \"t_0\" JOIN \"test2\" AS \"t_1\" ON \"t_1\".\"test_id\" = \"t_0\".\"id\" LIMIT 250;\n",
  "state": {
    "hints": {
      "table": [
        {
          "schema": "public",
          "table": "test2",
          "column": "test_id"
        }
      ]
    },
    "table-count": 2,
    "where": [],
    "limit": null,
    "pending-count": 0,
    "columns": [],
    "operation": {
      "type": "table",
      "value": {
        "table": "test2"
      }
    },
    "joins": {
      "t_0": {
        "t_1": [
          "t_1",
          "test_id",
          "=",
          "t_0",
          "id"
        ]
      }
    },
    "tables": [
      {
        "table": "test",
        "alias": "t_0"
      },
      {
        "table": "test2",
        "alias": "t_1"
      }
    ],
    "context": "t_1",
    "connection-id": "default",
    "aliases": {
      "t_0": {
        "table": "test",
        "schema": null
      },
      "t_1": {
        "table": "test2",
        "schema": null
      }
    }
  },
}

 */

#[derive(Serialize)]
struct ContextResult {
    result: InnerResult,
}

#[derive(Serialize)]
struct InnerResult {
    #[serde(rename = "connection-id")]
    connection_id: String,
    version: &'static str,
    metadata: Vec<usize>,
}

/*
{
  "result": {
    "connection-id": "host.docker.internal",
    "version": "0.5.4",
    "metadata": {
      "db/references": {
        "table": {
          "test2": {
            "refers-to": {
              "test": {
                "via": {
                  "test_id": [
                    [
                      "public",
                      "test2",
                      "test_id",
                      "refers-to",
                      "public",
                      "test",
                      "id"
                    ]
                  ]
                }
              }
            },
            "in": {
              "public": {
                "refers-to": {
                  "test": {
                    "in": {
                      "public": {
                        "via": {
                          "test_id": [
                            "public",
                            "test2",
                            "test_id",
                            "refers-to",
                            "public",
                            "test",
                            "id"
                          ]
                        }
                      }
                    }
                  }
                }
              }
            }
          },
          "test": {
            "referred-by": {
              "test2": {
                "via": {
                  "test_id": [
                    [
                      "public",
                      "test2",
                      "test_id",
                      "referred-by",
                      "public",
                      "test",
                      "id"
                    ]
                  ]
                }
              }
            },
            "in": {
              "public": {
                "referred-by": {
                  "test2": {
                    "in": {
                      "public": {
                        "via": {
                          "test_id": [
                            "public",
                            "test2",
                            "test_id",
                            "referred-by",
                            "public",
                            "test",
                            "id"
                          ]
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        },
        "schema": {
          "public": {
            "contains": {
              "test2": true,
              "test": true
            }
          }
        }
      }
    }
  }
}

 */
