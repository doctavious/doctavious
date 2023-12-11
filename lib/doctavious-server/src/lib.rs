mod configuration;
mod features;

use std::time::Duration;
use anyhow::Result;
use axum::{extract::Request, routing::get, Router};
use axum::http::{StatusCode, Uri};
use hyper::body::Incoming;
use hyper_util::rt::TokioIo;
use opendal::services::Fs;
use opendal::Operator;
use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::watch;
use tower_service::Service;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing::debug;

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
        .fallback(not_found)
        .layer((
            TraceLayer::new_for_http(),
            // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
            // requests don't hang forever.
            TimeoutLayer::new(Duration::from_secs(60)),
        ));;

    // Supporting graceful shutdown requires a bit of boilerplate. In the future hyper-util will
    // provide convenience helpers but for now we have to use hyper directly.

    // run it with hyper on localhost:3000
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    // Create a watch channel to track tasks that are handling connections and wait for them to
    // complete.
    let (close_tx, close_rx) = watch::channel(());

    // Continuously accept new connections.
    loop {
        let (socket, remote_addr) = tokio::select! {
            // Either accept a new connection...
            result = listener.accept() => {
                result.unwrap()
            }
            // ...or wait to receive a shutdown signal and stop the accept loop.
            _ = shutdown_signal() => {
                debug!("signal received, not accepting new connections");
                break;
            }
        };

        debug!("connection {remote_addr} accepted");

        // We don't need to call `poll_ready` because `Router` is always ready.
        let tower_service = app.clone();

        // Clone the watch receiver and move it into the task.
        let close_rx = close_rx.clone();

        // Spawn a task to handle the connection. That way we can serve multiple connections
        // concurrently.
        tokio::spawn(async move {
            // Hyper has its own `AsyncRead` and `AsyncWrite` traits and doesn't use tokio.
            // `TokioIo` converts between them.
            let socket = TokioIo::new(socket);

            // Hyper also has its own `Service` trait and doesn't use tower. We can use
            // `hyper::service::service_fn` to create a hyper `Service` that calls our app through
            // `tower::Service::call`.
            let hyper_service = hyper::service::service_fn(move |request: Request<Incoming>| {
                // We have to clone `tower_service` because hyper's `Service` uses `&self` whereas
                // tower's `Service` requires `&mut self`.
                //
                // We don't need to call `poll_ready` since `Router` is always ready.
                tower_service.clone().call(request)
            });

            // `hyper_util::server::conn::auto::Builder` supports both http1 and http2 but doesn't
            // support graceful so we have to use hyper directly and unfortunately pick between
            // http1 and http2.
            let conn = hyper::server::conn::http1::Builder::new()
                .serve_connection(socket, hyper_service)
                // `with_upgrades` is required for websockets.
                .with_upgrades();

            // `graceful_shutdown` requires a pinned connection.
            let mut conn = std::pin::pin!(conn);

            loop {
                tokio::select! {
                    // Poll the connection. This completes when the client has closed the
                    // connection, graceful shutdown has completed, or we encounter a TCP error.
                    result = conn.as_mut() => {
                        if let Err(err) = result {
                            debug!("failed to serve connection: {err:#}");
                        }
                        break;
                    }
                    // Start graceful shutdown when we receive a shutdown signal.
                    //
                    // We use a loop to continue polling the connection to allow requests to finish
                    // after starting graceful shutdown. Our `Router` has `TimeoutLayer` so
                    // requests will finish after at most 10 seconds.
                    _ = shutdown_signal() => {
                        debug!("signal received, starting graceful shutdown");
                        conn.as_mut().graceful_shutdown();
                    }
                }
            }

            debug!("connection {remote_addr} closed");

            // Drop the watch receiver to signal to `main` that this task is done.
            drop(close_rx);
        });
    }

    // We only care about the watch receivers that were moved into the tasks so close the residual
    // receiver.
    drop(close_rx);

    // Close the listener to stop accepting new connections.
    drop(listener);

    // Wait for all tasks to complete.
    debug!("waiting for {} tasks to finish", close_tx.receiver_count());
    close_tx.closed().await;

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
