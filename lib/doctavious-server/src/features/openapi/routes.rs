use axum::extract::Path;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};

pub fn get_routes() -> Router {
    Router::new().route(
        "/projects/{project_id}/openapi",
        get(get_openapi).post(new_openapi),
    )
}

async fn get_openapi(Path(project_id): Path<u64>) {}

// accept json openapi schema
// validate schema
// store
async fn new_openapi(Path(project_id): Path<u64>) {}
