use axum::extract::Path;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};

// TODO: What should these routes look like?

pub fn get_routes() -> Router {
    Router::new()
        .route("/external/github", post(webhook))
}

async fn webhook() {}

