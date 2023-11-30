mod configuration;
mod features;

use anyhow::Result;
use axum::http::{StatusCode, Uri};
use axum::routing::get;
use axum::Router;
use opendal::services::Fs;
use opendal::Operator;
use tokio::signal;

use crate::configuration::get_configuration;
use crate::features::deployment;

// State is global within the router
// The state passed to this method will be used for all requests this router receives.
// That means it is not suitable for holding state derived from a request, such as
// authorization data extracted in a middleware.
// Use Extension instead for such data.

// TODO: read about into_make_service_with_connect_info and ConnectInfo

// The application state
// here we can add configuration, database connection pools, or whatever
// state you need.
// Your top level state type must implement `Clone` to be extractable with State
// see axum substates docs
// I'm thinking that I want to follow a composition root here and a good
// reference might be how .NET core does this via their WebApplication builder.
// They the concept of host configuration, app settings, services, etc
// I think this could work well with how details are passed to axum handlers via state
#[derive(Clone)]
struct AppState {}

#[tokio::main]
async fn main() {
    // TODO: setup configuration
    // TODO: setup logging
    // TODO: setup tracing
    // TODO: setup routes
    // TODO: graceful shutdown
    // TODO: not found handler(s`)
    // TODO: setup composition root including...
    // db connection factory / pool
    // application services
    // will probably use redis for rate limiting

    get_configuration();

    let storage = get_storage().expect("unable to create storage");

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/health", get(health))
        .merge(deployment::get_routes(storage))
        //.with_state(storage)
        .fallback(not_found);

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

// TODO: swap for configuration built storage that returns appropriate storage provider
fn get_storage() -> Result<Operator> {
    let mut builder = Fs::default();
    // Set the root for fs, all operations will happen under this root.
    //
    // NOTE: the root must be absolute path.
    builder.root("/Users/seancarroll");

    // `Accessor` provides the low level APIs, we will use `Operator` normally.
    let op: Operator = Operator::new(builder)?.finish();

    Ok(op)
}

// TODO: implement with actual checks
async fn health() -> StatusCode {
    StatusCode::OK
}

async fn not_found(uri: Uri) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, format!("No route for {}", uri))
}


async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
        let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}